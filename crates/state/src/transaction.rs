use std::any::Any;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use mf_model::mark::Mark;
use mf_model::node_type::NodeEnum;
use mf_model::types::NodeId;
use mf_model::{Schema, Tree};
use mf_transform::step::{Step, StepResult};
use mf_transform::TransformResult;
use serde_json::Value;

use super::state::State;
use mf_model::node_pool::NodePool;
use mf_transform::attr_step::AttrStep;
use mf_transform::node_step::{AddNodeStep, RemoveNodeStep};
use mf_transform::mark_step::{AddMarkStep, RemoveMarkStep};
use mf_transform::transform::{LazyDoc, Transform};
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
    /// 原始文档状态
    pub base_doc: Arc<NodePool>,
    /// 文档的模式定义
    pub schema: Arc<Schema>,
    /// 存储元数据的哈希表，支持任意类型数据
    pub meta: imbl::HashMap<String, Arc<dyn Any + Send + Sync>>,
    /// 事务的时间戳
    pub id: u64,
    /// 存储所有操作步骤
    pub steps: imbl::Vector<Arc<dyn Step>>,
    /// 存储所有反向操作步骤
    pub invert_steps: imbl::Vector<Arc<dyn Step>>,
    transform: Transform,
}
impl Debug for Transaction {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "Transaction {{ id: {}}}", self.id)
    }
}


impl Transaction {
    pub fn get_draft(&self) -> Arc<RwLock<Tree>> {
        self.transform.get_draft()
    }
    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }
    pub fn new_shared(&self) -> Self {
        Transaction {
            base_doc: self.doc(),
            schema: self.schema.clone(),
            steps: imbl::Vector::new(),
            invert_steps: imbl::Vector::new(),
            meta: imbl::HashMap::new(),
            id: get_transaction_id(),
            transform: Transform::new_shared(self.transform.get_draft()),
        }
    }
    /// 创建新的事务实例
    /// state: 当前状态对象
    /// 返回: Transaction 实例
    pub fn new(state: &State) -> Self {
        let node = state.doc();
        let schema = state.schema();
        Transaction {
            base_doc: node.clone(),
            schema: schema.clone(),
            steps: imbl::Vector::new(),
            invert_steps: imbl::Vector::new(),
            meta: imbl::HashMap::new(),
            id: get_transaction_id(),
            transform: Transform::new(node.get_inner().as_ref().clone()),
        }
    }

    /// 获取当前文档状态，使用延迟计算
    pub fn doc(&self) -> Arc<NodePool> {
        if self.steps.is_empty() {
            return self.base_doc.clone();
        }
        // 只有在真正需要时才进行计算
        NodePool::new(Arc::new(self.transform.get_draft().read().unwrap().clone()))
    }
 
    /// 检查文档是否被修改
    pub fn doc_changed(&self) -> bool {
        !self.steps.is_empty()
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

    pub fn step(
        &mut self,
        step: Arc<dyn Step>,
    ) -> TransformResult<()> {
        let schema = self.schema.clone();
        let draft = self.transform.get_draft();
        let mut draft = draft.write().unwrap();
        let result: StepResult = step.apply(&mut draft, schema)?;

        match result.failed {
            Some(message) => Err(anyhow::anyhow!(message)),
            None => {
                self.add_step(step);
                Ok(())
            },
        }
    }
    /// 添加一个步骤及其结果到事务中
    fn add_step(
        &mut self,
        step: Arc<dyn Step>,
    ) {
        // 生成反向步骤
        if let Some(invert_step) = step.invert(&self.base_doc.get_inner()) {
            self.invert_steps.push_back(invert_step);
        }

        self.steps.push_back(step.clone());
    }

    /// 批量应用步骤（优化版本）
    pub fn apply_steps_batch(
        &mut self,
        steps: Vec<Arc<dyn Step>>,
    ) -> TransformResult<()> {
        let schema = self.schema.clone();
        let base_doc_inner = self.base_doc.get_inner().clone();

        // 收集反向步骤
        let mut new_invert_steps = Vec::new();
        for step in &steps {
            if let Some(invert_step) = step.invert(&base_doc_inner) {
                new_invert_steps.push(invert_step);
            }
        }
        let draft = self.transform.get_draft();
        let mut draft = draft.write().unwrap();

        // 批量应用，减少中间状态创建
        for step in &steps {
            let result = step.apply(&mut draft, schema.clone())?;
            if let Some(message) = result.failed {
                return Err(anyhow::anyhow!(message));
            }
        }

        // 更新步骤列表
        for step in steps {
            self.steps.push_back(step);
        }
        for invert_step in new_invert_steps {
            self.invert_steps.push_back(invert_step);
        }
        Ok(())
    }
    /// 设置节点属性
    /// id: 节点ID
    /// values: 属性键值对
    pub fn set_node_attribute(
        &mut self,
        id: NodeId,
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
    pub fn set_meta<K, T: Send + Sync + 'static>(
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
    /// 清除历史记录（释放内存）
    pub fn clear_history(&mut self) {
        self.steps = imbl::Vector::new();
        self.invert_steps = imbl::Vector::new();
    }

    /// 获取历史记录大小
    pub fn history_size(&self) -> usize {
        self.steps.len()
    }
}
