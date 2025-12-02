//! Actor 消息系统的泛型类型
//!
//! 定义了所有 Actor 消息的泛型版本，支持任意 DataContainer 和 SchemaDefinition 组合。

use std::sync::Arc;
use tokio::sync::oneshot;

use mf_model::traits::{DataContainer, SchemaDefinition};
use mf_state::{
    state::StateGeneric,
    transaction::TransactionGeneric,
};

use crate::{
    config::{EventConfig, ForgeConfig},
    error::ForgeResult,
    event::{EventHandler, HandlerId},
};

use super::event::EventGeneric;

// ==================== State Actor Messages ====================

/// 状态管理 Actor 消息（泛型版本）
#[derive(Debug)]
pub enum StateMessageGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 获取当前状态
    GetState { reply: oneshot::Sender<Arc<StateGeneric<C, S>>> },
    /// 应用事务（包含元信息）
    ApplyTransaction {
        transaction: TransactionGeneric<C, S>,
        description: String,
        meta: serde_json::Value,
        reply: oneshot::Sender<ForgeResult<Arc<StateGeneric<C, S>>>>,
    },
    /// 批量应用事务
    ApplyTransactionBatch {
        transactions: Vec<TransactionGeneric<C, S>>,
        description: String,
        meta: serde_json::Value,
        reply: oneshot::Sender<ForgeResult<Arc<StateGeneric<C, S>>>>,
    },
    /// 撤销操作
    Undo { reply: oneshot::Sender<ForgeResult<Arc<StateGeneric<C, S>>>> },
    /// 重做操作
    Redo { reply: oneshot::Sender<ForgeResult<Arc<StateGeneric<C, S>>>> },
    /// 跳转到指定历史位置
    Jump { steps: isize, reply: oneshot::Sender<ForgeResult<Arc<StateGeneric<C, S>>>> },
    /// 获取历史记录信息
    GetHistoryInfo { reply: oneshot::Sender<HistoryInfo> },
    /// 创建状态快照
    CreateSnapshot { reply: oneshot::Sender<StateSnapshotGeneric<C, S>> },
    /// 记录已应用的事务到历史（不实际应用事务）
    RecordTransactions {
        state: Arc<StateGeneric<C, S>>,
        transactions: Vec<Arc<TransactionGeneric<C, S>>>,
        description: String,
        meta: serde_json::Value,
        reply: oneshot::Sender<ForgeResult<()>>,
    },
}

/// 历史记录信息
#[derive(Debug, Clone)]
pub struct HistoryInfo {
    pub current_index: usize,
    pub total_entries: usize,
    pub can_undo: bool,
    pub can_redo: bool,
}

/// 状态快照（泛型版本）
#[derive(Debug, Clone)]
pub struct StateSnapshotGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub state: Arc<StateGeneric<C, S>>,
    pub timestamp: std::time::SystemTime,
    pub version: u64,
}

// ==================== Transaction Processor Messages ====================

/// 事务处理 Actor 消息（泛型版本）
#[derive(Debug)]
pub enum TransactionMessageGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 处理事务（保持与原始dispatch_with_meta完全相同的逻辑）
    ProcessTransaction {
        transaction: TransactionGeneric<C, S>,
        description: String,
        meta: serde_json::Value,
        reply: oneshot::Sender<ForgeResult<()>>,
    },
    /// 获取处理统计信息
    GetStats { reply: oneshot::Sender<TransactionStats> },
    /// 更新配置
    UpdateConfig {
        config: ForgeConfig,
        reply: oneshot::Sender<ForgeResult<()>>,
    },
}

/// 事务处理统计信息
#[derive(Debug, Clone)]
pub struct TransactionStats {
    pub transactions_processed: u64,
    pub transaction_failures: u64,
    pub avg_processing_time_ms: u64,
    pub middleware_timeouts: u64,
}

// ==================== Event Bus Messages ====================

/// 事件总线 Actor 消息（泛型版本）
#[derive(Debug)]
pub enum EventBusMessageGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 发布事件
    PublishEvent { event: EventGeneric<C, S> },
    /// 添加事件处理器
    AddHandler {
        handler: Arc<dyn EventHandler<EventGeneric<C, S>> + Send + Sync>,
        reply: oneshot::Sender<HandlerId>,
    },
    /// 移除事件处理器
    RemoveHandler {
        handler_id: HandlerId,
        reply: oneshot::Sender<ForgeResult<()>>,
    },
    /// 获取事件总线统计信息
    GetStats { reply: oneshot::Sender<EventBusStats> },
    /// 更新配置
    UpdateConfig {
        config: EventConfig,
        reply: oneshot::Sender<ForgeResult<()>>,
    },
}

/// 事件总线统计信息
#[derive(Debug, Clone)]
pub struct EventBusStats {
    pub events_published: u64,
    pub events_processed: u64,
    pub event_failures: u64,
    pub active_handlers: usize,
    pub avg_processing_time_ms: u64,
}
