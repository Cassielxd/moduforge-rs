use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use tracing::{info, warn, error};

use super::state::State;
use moduforge_transform::draft::Draft;
use moduforge_model::node::Node;
use moduforge_model::node_pool::NodePool;
use moduforge_transform::attr_step::AttrStep;
use moduforge_transform::node_step::AddNodeStep;
use moduforge_transform::transform::{Transform, TransformError};
use moduforge_transform::PatchStep;
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
        let schema = state.schema();
        Transaction {
            meta: im::HashMap::new(),
            id: get_transaction_id(),
            transform: Transform {
                doc: node.clone(),
                draft: Draft::new(node),
                steps: im::Vector::new(),
                patches: im::Vector::new(),
                schema,
            },
        }
    }
    pub fn merge(
        &mut self,
        other: &mut Self,
    ) {
        self.steps.extend(other.steps.iter().cloned());
        self.patches.extend(other.patches.iter().cloned());
        self.doc = other.doc.clone();
    }
    /// 获取当前文档状态
    pub fn doc(&self) -> Arc<NodePool> {
        self.doc.clone()
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
    ) -> Option<&Arc<T>>
    where
        K: Into<String>,
    {
        let key_str = key.into();
        self.meta.get(&key_str)?.downcast_ref::<Arc<T>>()
    }
}
