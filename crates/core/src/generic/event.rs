//! 泛型事件类型定义
//!
//! 定义了支持任意 DataContainer 和 SchemaDefinition 组合的事件系统。

use std::sync::Arc;

use mf_model::traits::{DataContainer, SchemaDefinition};
use mf_state::{
    state::StateGeneric,
    transaction::TransactionGeneric,
};

/// 事件类型定义（泛型版本）
///
///支持任意 DataContainer 和 SchemaDefinition 组合
#[derive(Debug, Clone)]
pub enum EventGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 状态创建事件
    Create(Arc<StateGeneric<C, S>>),

    /// 事务应用事件 (old_state, new_state, transactions)
    /// 统一使用新旧状态模式，与 Undo/Redo 保持一致
    TrApply {
        old_state: Arc<StateGeneric<C, S>>,
        new_state: Arc<StateGeneric<C, S>>,
        transactions: Vec<Arc<TransactionGeneric<C, S>>>,
    },

    /// 撤销事件 (old_state, new_state, undone_transactions)
    /// 包含被撤销的事务列表，供其他组件（如搜索索引）使用
    Undo {
        old_state: Arc<StateGeneric<C, S>>,
        new_state: Arc<StateGeneric<C, S>>,
        transactions: Vec<Arc<TransactionGeneric<C, S>>>,
    },

    /// 重做事件 (old_state, new_state, redone_transactions)
    /// 包含重做的事务列表，供其他组件（如搜索索引）使用
    Redo {
        old_state: Arc<StateGeneric<C, S>>,
        new_state: Arc<StateGeneric<C, S>>,
        transactions: Vec<Arc<TransactionGeneric<C, S>>>,
    },

    /// 历史跳转事件 (old_state, new_state, transactions, steps)
    /// 当用户跳转到历史中的特定位置时触发
    /// transactions 包含跳转过程中所有被影响的事务
    Jump {
        old_state: Arc<StateGeneric<C, S>>,
        new_state: Arc<StateGeneric<C, S>>,
        transactions: Vec<Arc<TransactionGeneric<C, S>>>,
        steps: isize,
    },

    /// 事务失败事件
    /// 当事务应用失败时触发，供错误处理和日志记录使用
    TrFailed {
        state: Arc<StateGeneric<C, S>>,
        transaction: TransactionGeneric<C, S>,
        error: String,
    },

    /// 历史清空事件
    /// 当历史记录被清空时触发
    HistoryCleared,

    /// 销毁事件
    Destroy,

    /// 停止事件（需要重启）
    Stop,
}

impl<C, S> EventGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub fn name(&self) -> &'static str {
        match self {
            EventGeneric::Create(_) => "Create",
            EventGeneric::TrApply { .. } => "TrApply",
            EventGeneric::Undo { .. } => "Undo",
            EventGeneric::Redo { .. } => "Redo",
            EventGeneric::Jump { .. } => "Jump",
            EventGeneric::TrFailed { .. } => "TrFailed",
            EventGeneric::HistoryCleared => "HistoryCleared",
            EventGeneric::Destroy => "Destroy",
            EventGeneric::Stop => "Stop",
        }
    }
}
