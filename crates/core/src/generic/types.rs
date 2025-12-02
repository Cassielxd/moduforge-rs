//! 泛型类型定义
//!
//! 此模块包含运行时系统所需的泛型类型定义。

use std::sync::Arc;
use std::time::SystemTime;

use mf_model::traits::{DataContainer, SchemaDefinition};
use mf_state::{
    state::{StateGeneric, TransactionResultGeneric},
    transaction::TransactionGeneric,
};

/// 带元信息的历史记录项（混合方案：同时存储事务和状态）
#[derive(Debug, Clone)]
pub struct HistoryEntryWithMetaGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 事务列表（支持批量操作、协作编辑、操作追踪）
    pub transactions: Vec<Arc<TransactionGeneric<C, S>>>,

    /// 状态快照（用于快速撤销/重做，每个历史点都保存）
    pub state: Arc<StateGeneric<C, S>>,

    /// 操作描述
    pub description: String,

    /// 时间戳
    pub timestamp: SystemTime,

    /// 元数据
    pub meta: serde_json::Value,
}

impl<C, S> HistoryEntryWithMetaGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 创建单事务历史条目
    pub fn new(
        transaction: Arc<TransactionGeneric<C, S>>,
        state: Arc<StateGeneric<C, S>>,
        description: String,
        meta: serde_json::Value,
    ) -> Self {
        Self {
            transactions: vec![transaction],
            state,
            description,
            timestamp: SystemTime::now(),
            meta,
        }
    }

    /// 创建批量事务历史条目
    pub fn new_batch(
        transactions: Vec<Arc<TransactionGeneric<C, S>>>,
        state: Arc<StateGeneric<C, S>>,
        description: String,
        meta: serde_json::Value,
    ) -> Self {
        Self {
            transactions,
            state,
            description,
            timestamp: SystemTime::now(),
            meta,
        }
    }
}

/// 事务处理状态
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
    Rolled,
    NotFound,
}

/// 处理器结果（泛型版本）
#[derive(Debug, Clone)]
pub struct ProcessorResultGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub status: TransactionStatus,
    pub error: Option<String>,
    pub result: Option<TransactionResultGeneric<C, S>>,
}

/// 任务参数类型（泛型版本）
pub type TaskParamsGeneric<C, S> = (Arc<StateGeneric<C, S>>, TransactionGeneric<C, S>);
