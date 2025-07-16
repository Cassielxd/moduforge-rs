use std::any::Any;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use mf_model::mark::Mark;
use mf_model::node_type::NodeEnum;
use mf_model::types::NodeId;
use mf_transform::TransformResult;
use serde_json::Value;

use super::state::State;
use mf_model::node_pool::NodePool;
use mf_transform::attr_step::AttrStep;
use mf_transform::node_step::{AddNodeStep, RemoveNodeStep};
use mf_transform::mark_step::{AddMarkStep, RemoveMarkStep};
use mf_transform::transform::{Transform};
use std::fmt::Debug;

static IDS: AtomicU64 = AtomicU64::new(1);
pub fn get_transaction_id() -> u64 {
    //生成 全局自增的版本号，用于兼容性
    IDS.fetch_add(1, Ordering::SeqCst)
}

/// 定义可执行的命令接口
/// 要求实现 Send + Sync 以支持并发操作，并实现 Debug 以支持调试
#[async_trait]
pub trait Command: Send + Sync + Debug {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()>;
    fn name(&self) -> String;
}
/// 事务结构体，用于管理文档的修改操作
#[derive(Clone)]
pub struct Transaction {
    /// 存储元数据的哈希表，支持任意类型数据
    pub meta: imbl::HashMap<String, Arc<dyn Any>>,
    /// 事务的时间戳
    pub id: u64,
    transform: Transform,
}
impl Debug for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Transaction {{ id: {}}}", self.id)
    }
}

unsafe impl Send for Transaction {}
unsafe impl Sync for Transaction {}
impl Deref for Transaction {
    type Target = Transform;

    fn deref(&self) -> &Self::Target {
        &self.transform
    }
}

impl DerefMut for Transaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transform
    }
}

impl Transaction {
    /// 创建新的事务实例
    /// state: 当前状态对象
    /// 返回: Transaction 实例
    pub fn new(state: &State) -> Self {
        let node = state.doc();
        let schema = state.schema();
        Transaction {
            meta: imbl::HashMap::new(),
            id: get_transaction_id(),
            transform: Transform::new(node, schema),
        }
    }
    pub fn merge(
        &mut self,
        other: &mut Self,
    ) {
        // 使用批量应用来优化性能
        let steps_to_apply: Vec<_> = other.steps.iter().cloned().collect();
        if let Err(e) = self.apply_steps_batch(steps_to_apply) {
            eprintln!("批量应用步骤失败: {}", e);
        }
    }
    /// 获取当前文档状态
    pub fn doc(&self) -> Arc<NodePool> {
        self.transform.doc()
    }
    /// 设置节点属性
    /// id: 节点ID
    /// values: 属性键值对
    pub fn set_node_attribute(
        &mut self,
        id: String,
        values: imbl::HashMap<String, Value>,
    ) -> TransformResult<()> {
        self.step(Arc::new(AttrStep::new(id, values)))?;
        Ok(())
    }
    /// 添加新节点
    /// parent_id: 父节点ID
    /// node: 要添加的节点
    pub fn add_node(
        &mut self,
        parent_id: NodeId,
        nodes: Vec<NodeEnum>,
    ) -> TransformResult<()> {
        self.step(Arc::new(AddNodeStep::new(parent_id, nodes)))?;
        Ok(())
    }
    /// 删除节点
    /// id: 节点ID
    /// nodes: 要删除的节点
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
    pub fn remove_mark(
        &mut self,
        id: NodeId,
        mark_types: Vec<String>,
    ) -> TransformResult<()> {
        self.step(Arc::new(RemoveMarkStep::new(id, mark_types)))?;
        Ok(())
    }
    /// 设置元数据
    /// key: 键
    /// value: 值（支持任意类型）
    pub fn set_meta<K, T: 'static>(
        &mut self,
        key: K,
        value: T,
    ) -> &mut Self
    where
        K: Into<String>,
    {
        let key_str = key.into();
        self.meta.insert(key_str, Arc::new(value));
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
        let value_any = value.downcast_ref::<T>().cloned();
        value_any
    }
}
