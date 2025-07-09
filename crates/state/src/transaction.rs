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
#[derive(Debug, Clone)]
pub struct Transaction {
    /// 存储元数据的哈希表，支持任意类型数据
    pub meta: im::HashMap<String, Arc<dyn std::any::Any>>,
    /// 事务的时间戳
    pub id: u64,
    transform: Transform,
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
            meta: im::HashMap::new(),
            id: get_transaction_id(),
            transform: Transform::new(node, schema),
        }
    }
    pub fn merge(
        &mut self,
        other: &mut Self,
    ) -> TransformResult<()> {
        // 创建事务合并的快照，以支持原子性操作
        let original_steps_count = self.steps.len();
        let original_meta = self.meta.clone();
        
        // 原子性地获取other的步骤，避免并发修改
        let steps_to_apply: Vec<_> = std::mem::take(&mut other.steps);
        
        // 合并元数据，避免数据丢失
        for (key, value) in other.meta.iter() {
            self.meta.insert(key.clone(), value.clone());
        }
        
        // 尝试批量应用步骤
        match self.apply_steps_batch(steps_to_apply.clone()) {
            Ok(_) => {
                // 成功：清空other的元数据，因为已经被合并
                other.meta.clear();
                Ok(())
            },
            Err(e) => {
                // 失败：回滚所有变更，确保事务的原子性
                tracing::warn!("事务合并失败，正在回滚: {}", e);
                
                // 回滚步骤
                self.steps.truncate(original_steps_count);
                
                // 回滚元数据
                self.meta = original_meta;
                
                // 恢复other的步骤
                other.steps = steps_to_apply;
                
                Err(e)
            }
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
        values: im::HashMap<String, Value>,
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
    pub fn set_meta<K, T: std::any::Any>(
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
    pub fn get_meta<T: 'static>(
        &self,
        key: &str,
    ) -> Option<&Arc<T>> {
        self.meta.get(key)?.downcast_ref::<Arc<T>>()
    }
}
