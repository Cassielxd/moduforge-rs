use std::any::Any;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use async_trait::async_trait;
use mf_model::mark::Mark;
use mf_model::node_definition::NodeTree;
use mf_model::types::NodeId;
use mf_model::traits::{DataContainer, SchemaDefinition};
use mf_transform::TransformResult;
use serde_json::Value;

use super::state::State;
use mf_model::node_pool::NodePool;
use mf_model::schema::Schema;
use mf_transform::attr_step::AttrStep;
use mf_transform::node_step::{AddNodeStep, RemoveNodeStep};
use mf_transform::mark_step::{AddMarkStep, RemoveMarkStep};
use mf_transform::transform::{Transform, TransformGeneric};
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use mf_model::rpds::{HashTrieMapSync};

/// 定义可执行的命令接口 (泛型版本)
/// 要求实现 Send + Sync 以支持并发操作，并实现 Debug 以支持调试
#[async_trait]
pub trait CommandGeneric<C, S>: Send + Sync + Debug
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    async fn execute(
        &self,
        tr: &mut TransactionGeneric<C, S>,
    ) -> TransformResult<()>;
    fn name(&self) -> String;
}

/// 默认的 Command trait (NodePool + Schema)
///
/// 这是一个便利别名，自动实现了 CommandGeneric<NodePool, Schema>
/// 现有代码可以继续使用 Command trait 而无需修改
pub trait Command: CommandGeneric<NodePool, Schema> {}

/// 为所有实现了 CommandGeneric<NodePool, Schema> 的类型自动实现 Command
impl<T> Command for T where T: CommandGeneric<NodePool, Schema> {}

static VERSION: AtomicU64 = AtomicU64::new(1);
pub fn get_tr_id() -> u64 {
    //生成 全局自增的版本号，用于兼容性
    VERSION.fetch_add(1, Ordering::SeqCst)
}

/// 事务结构体，用于管理文档的修改操作 (泛型版本)
#[derive(Clone)]
pub struct TransactionGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 存储元数据的哈希表，支持任意类型数据
    pub meta: HashTrieMapSync<String, Arc<dyn Any + Send + Sync>>,
    pub id: u64,
    transform: TransformGeneric<C, S>,
}

impl<C, S> Debug for TransactionGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "Transaction {{ id: {}}}", self.id)
    }
}

impl<C, S> Deref for TransactionGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    type Target = TransformGeneric<C, S>;

    fn deref(&self) -> &Self::Target {
        &self.transform
    }
}

impl<C, S> DerefMut for TransactionGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transform
    }
}

impl<C, S> TransactionGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 创建新的事务实例 (泛型版本)
    pub fn new_generic(
        node: Arc<C>,
        schema: Arc<S>,
    ) -> Self {
        TransactionGeneric {
            meta: HashTrieMapSync::new_sync(),
            id: get_tr_id(),
            transform: TransformGeneric::new(node, schema),
        }
    }

    /// 获取当前文档状态
    pub fn doc(&self) -> Arc<C> {
        self.transform.doc()
    }

    /// 设置元数据
    /// key: 键
    /// value: 值（支持任意类型）
    pub fn set_meta<K, T: Send + Sync + 'static>(
        &mut self,
        key: K,
        value: T,
    ) -> &mut Self
    where
        K: Into<String>,
    {
        let key_str = key.into();
        self.meta.insert_mut(key_str, Arc::new(value));
        self
    }

    /// 获取元数据
    /// key: 键
    /// 返回: Option<&T>，如果存在且类型匹配则返回Some，否则返回None
    pub fn get_meta<T: Clone + 'static>(
        &self,
        key: &str,
    ) -> Option<T> {
        let value = self.meta.get(key)?;

        value.downcast_ref::<T>().cloned()
    }
}

// ========================================
// NodePool 特化实现
// ========================================

/// 默认的 Transaction 实现（NodePool + Schema）
pub type Transaction = TransactionGeneric<NodePool, Schema>;

impl Transaction {
    /// 创建新的事务实例
    /// state: 当前状态对象
    /// 返回: Transaction 实例
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(state), fields(
        crate_name = "state",
        state_version = state.version,
        doc_size = state.node_pool.size()
    )))]
    pub fn new(state: &State) -> Self {
        let node = state.doc();
        let schema = state.schema();
        let tr = Transaction {
            meta: HashTrieMapSync::new_sync(),
            id: get_tr_id(), // ✅ 使用 UUID v4 生成唯一标识
            transform: Transform::new(node, schema),
        };
        #[cfg(feature = "dev-tracing")]
        tracing::debug!(tr_id = %tr.id, "事务创建成功");
        tr
    }
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, other), fields(
        crate_name = "state",
        self_tr_id = %self.id,
        other_tr_id = %other.id,
        self_steps = self.steps.len(),
        other_steps = other.steps.len()
    )))]
    pub fn merge(
        &mut self,
        other: &mut Self,
    ) {
        // 使用批量应用来优化性能
        let steps_to_apply: Vec<_> = other.steps.iter().cloned().collect();
        if let Err(e) = self.apply_steps_batch(steps_to_apply) {
            #[cfg(feature = "dev-tracing")]
            tracing::error!(error = %e, "批量应用步骤失败");
            eprintln!("批量应用步骤失败: {e}");
        } else {
            #[cfg(feature = "dev-tracing")]
            tracing::debug!(total_steps = self.steps.len(), "事务合并成功");
        }
    }

    /// 设置节点属性
    /// id: 节点ID
    /// values: 属性键值对
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, values), fields(
        crate_name = "state",
        tr_id = %self.id,
        node_id = %id,
        attr_count = values.len()
    )))]
    pub fn set_node_attribute(
        &mut self,
        id: NodeId,
        values: HashTrieMapSync<String, Value>,
    ) -> TransformResult<()> {
        self.step(Arc::new(AttrStep::new(id, values)))?;
        Ok(())
    }
    /// 添加新节点
    /// parent_id: 父节点ID
    /// node: 要添加的节点
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, nodes), fields(
        crate_name = "state",
        tr_id = %self.id,
        parent_id = %parent_id,
        node_count = nodes.len()
    )))]
    pub fn add_node(
        &mut self,
        parent_id: NodeId,
        nodes: Vec<NodeTree>,
    ) -> TransformResult<()> {
        self.step(Arc::new(AddNodeStep::new(parent_id, nodes)))?;
        Ok(())
    }
    /// 删除节点
    /// id: 节点ID
    /// nodes: 要删除的节点
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, node_ids), fields(
        crate_name = "state",
        tr_id = %self.id,
        parent_id = %parent_id,
        remove_count = node_ids.len()
    )))]
    pub fn remove_node(
        &mut self,
        parent_id: NodeId,
        node_ids: Vec<NodeId>,
    ) -> TransformResult<()> {
        self.step(Arc::new(RemoveNodeStep::new(parent_id, node_ids)))?;
        Ok(())
    }
    /// 添加标记
    /// id: 节点ID
    /// marks: 要添加的标记
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, marks), fields(
        crate_name = "state",
        tr_id = %self.id,
        node_id = %id,
        mark_count = marks.len()
    )))]
    pub fn add_mark(
        &mut self,
        id: NodeId,
        marks: Vec<Mark>,
    ) -> TransformResult<()> {
        self.step(Arc::new(AddMarkStep::new(id, marks)))?;
        Ok(())
    }
    /// 删除标记
    /// id: 节点ID
    /// marks: 要删除的标记
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, mark_types), fields(
        crate_name = "state",
        tr_id = %self.id,
        node_id = %id,
        mark_type_count = mark_types.len()
    )))]
    pub fn remove_mark(
        &mut self,
        id: NodeId,
        mark_types: Vec<String>,
    ) -> TransformResult<()> {
        self.step(Arc::new(RemoveMarkStep::new(id, mark_types)))?;
        Ok(())
    }
}
