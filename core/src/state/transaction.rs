use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use tracing::{info, warn, error};

use super::state::State;
use crate::model::node::Node;
use crate::model::node_pool::{Draft, NodePool};
use crate::model::patch::Patch;
use crate::model::schema::Schema;
use crate::transform::attr_step::AttrStep;
use crate::transform::node_step::AddNodeStep;
use crate::transform::step::{Step, StepResult};
use crate::transform::transform::{Transform, TransformError};
use crate::transform::{ConcreteStep, PatchStep};
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
    ) -> Result<(), TransformError>;
    fn name(&self) -> String;
}

/// 事务结构体，用于管理文档的修改操作
#[derive(Debug, Clone)]
pub struct Transaction {
    /// 存储元数据的哈希表，支持任意类型数据
    pub meta: im::HashMap<String, Arc<dyn std::any::Any>>,
    /// 事务的时间戳
    pub id: u64,
    /// 存储所有操作步骤
    pub steps: im::Vector<Arc<dyn Step>>,
    /// 存储每个步骤对应的补丁列表
    pub patches: im::Vector<Vec<Patch>>,
    /// 当前文档状态
    pub doc: Arc<NodePool>,
    /// 文档的草稿状态，用于临时修改
    pub draft: Draft,
    /// 文档的模式定义
    pub schema: Arc<Schema>,
}
unsafe impl Send for Transaction {}
unsafe impl Sync for Transaction {}

impl Transform for Transaction {
    /// 执行一个修改步骤
    /// step: 要执行的步骤
    /// 返回: Result，表示执行成功或失败
    fn step(
        &mut self,
        step: Arc<dyn Step>,
    ) -> Result<(), TransformError> {
        let result = step.apply(&mut self.draft, self.schema.clone())?;
        match result.failed {
            Some(message) => Err(TransformError::new(message)),
            None => {
                self.add_step(step, result);
                Ok(())
            },
        }
    }
    /// 检查文档是否被修改
    fn doc_changed(&self) -> bool {
        !self.steps.is_empty()
    }
    /// 添加一个步骤及其结果到事务中
    fn add_step(
        &mut self,
        step: Arc<dyn Step>,
        result: StepResult,
    ) {
        self.steps.push_back(step);
        self.patches.push_back(result.patches);
        self.doc = result.doc.unwrap();
    }
}
impl Transaction {
    /// 执行一个事务操作
    /// call_back: 要执行的命令
    pub async fn transaction(
        &mut self,
        call_back: Arc<dyn Command>,
    ) {
        info!("开始执行事务: {}", call_back.name());
        self.draft.begin = true;
        let result = call_back.execute(self).await;
        self.draft.begin = false;
        match result {
            Ok(_) => {
                info!("事务执行成功，正在提交更改");
                let result = self.draft.commit();
                self.add_step(
                    Arc::new(PatchStep { patches: result.patches.clone() }),
                    result,
                );
            },
            Err(e) => {
                error!("事务执行失败: {}", e);
                warn!("事务回滚");
            },
        }
    }
    /// 创建新的事务实例
    /// state: 当前状态对象
    /// 返回: Transaction 实例
    pub fn new(state: &State) -> Self {
        let node = state.doc();
        Transaction {
            meta: im::HashMap::new(),
            id: get_transaction_id(),
            steps: im::Vector::new(),
            doc: node,
            schema: state.schema(),
            draft: Draft::new(state.doc()),
            patches: im::Vector::new(),
        }
    }
    /// 获取当前文档状态
    pub fn doc(&self) -> Arc<NodePool> {
        self.doc.clone()
    }
    /// 将步骤转换为具体步骤
    pub fn as_concrete(step: &Arc<dyn Step>) -> ConcreteStep {
        step.to_concrete()
    }
    /// 设置节点属性
    /// id: 节点ID
    /// values: 属性键值对
    pub fn set_node_attribute(
        &mut self,
        id: String,
        values: im::HashMap<String, Value>,
    ) {
        let _ = self.step(Arc::new(AttrStep::new(id, values)));
    }
    /// 添加新节点
    /// parent_id: 父节点ID
    /// node: 要添加的节点
    pub fn add_node(
        &mut self,
        parent_id: String,
        nodes: Vec<Node>,
    ) {
        let _ = self.step(Arc::new(AddNodeStep::new(parent_id, nodes)));
    }
    /// 设置事务时间戳
    pub fn set_time(
        &mut self,
        id: u64,
    ) -> &mut Self {
        self.id = id;
        self
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
    pub fn get_meta<T: 'static, K>(
        &self,
        key: K,
    ) -> Option<&T>
    where
        K: Into<String>,
    {
        let key_str = key.into();
        self.meta.get(&key_str)?.downcast_ref::<T>()
    }
}
