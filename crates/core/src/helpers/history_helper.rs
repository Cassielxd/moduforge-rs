//! 历史管理辅助模块（混合方案：状态快照 + 事务列表）
//!
//! 提供统一的历史管理逻辑，包括：
//! - 撤销（undo）操作 - 使用状态快照实现 O(1) 性能
//! - 重做（redo）操作 - 使用状态快照实现 O(1) 性能
//! - 跳转（jump）操作 - 使用状态快照实现快速跳转
//! - 性能指标记录
//! - 事件触发，供其他组件（如搜索索引）响应

use crate::{history_manager::HistoryManager, metrics, types::HistoryEntryWithMeta};
use mf_state::{state::State, Transaction};
use std::sync::Arc;

/// 历史操作结果
#[derive(Debug, Clone)]
pub struct HistoryOperationResult {
    /// 旧状态（操作前）
    pub old_state: Arc<State>,
    /// 新状态（操作后）
    pub new_state: Arc<State>,
    /// 相关的事务列表
    pub transactions: Vec<Arc<Transaction>>,
}

/// 历史管理辅助器
pub struct HistoryHelper;

impl HistoryHelper {
    /// 执行撤销操作 - 使用状态快照直接切换 (O(1) 性能)
    ///
    /// # 参数
    /// * `history_manager` - 历史管理器的可变引用
    /// * `current_state` - 当前状态
    ///
    /// # 返回值
    /// * `Option<HistoryOperationResult>` - 撤销结果（包含状态和事务信息）
    pub fn undo(
        history_manager: &mut HistoryManager<HistoryEntryWithMeta>,
        current_state: Arc<State>,
    ) -> Option<HistoryOperationResult> {
        if !history_manager.can_undo() {
            return None;
        }

        let old_entry = history_manager.get_present();
        let old_state = current_state;
        let transactions = old_entry.transactions.clone();

        // 更新历史位置
        history_manager.jump(-1);

        // 获取撤销后的状态（直接使用快照）
        let new_entry = history_manager.get_present();
        let new_state = new_entry.state.clone();

        metrics::history_operation("undo");

        Some(HistoryOperationResult { old_state, new_state, transactions })
    }

    /// 执行重做操作 - 使用状态快照直接切换 (O(1) 性能)
    ///
    /// # 参数
    /// * `history_manager` - 历史管理器的可变引用
    /// * `current_state` - 当前状态
    ///
    /// # 返回值
    /// * `Option<HistoryOperationResult>` - 重做结果（包含状态和事务信息）
    pub fn redo(
        history_manager: &mut HistoryManager<HistoryEntryWithMeta>,
        current_state: Arc<State>,
    ) -> Option<HistoryOperationResult> {
        if !history_manager.can_redo() {
            return None;
        }

        let old_state = current_state;

        // 更新历史位置
        history_manager.jump(1);

        // 获取重做后的状态（直接使用快照）
        let new_entry = history_manager.get_present();
        let new_state = new_entry.state.clone();
        let transactions = new_entry.transactions.clone();

        metrics::history_operation("redo");

        Some(HistoryOperationResult { old_state, new_state, transactions })
    }

    /// 执行跳转操作 - 使用状态快照直接跳转
    ///
    /// # 参数
    /// * `history_manager` - 历史管理器的可变引用
    /// * `current_state` - 当前状态
    /// * `steps` - 跳转步数（正数前进，负数后退）
    ///
    /// # 返回值
    /// * `Option<HistoryOperationResult>` - 跳转结果
    ///
    /// # 注意
    /// 跳转时收集的 transactions 是从当前位置到目标位置之间所有被影响的事务：
    /// - steps < 0（后退）：收集被撤销的事务
    /// - steps > 0（前进）：收集被重做的事务
    pub fn jump(
        history_manager: &mut HistoryManager<HistoryEntryWithMeta>,
        current_state: Arc<State>,
        steps: isize,
    ) -> Option<HistoryOperationResult> {
        if steps == 0 {
            return None;
        }

        let old_state = current_state;
        let history = history_manager.get_history();
        let current_index = history.past.len();

        // 收集跳转过程中所有被影响的事务
        let mut transactions = Vec::new();

        if steps < 0 {
            // 后退：收集被撤销的事务（从当前位置往回）
            let abs_steps = (-steps) as usize;
            let start_index = current_index.saturating_sub(abs_steps);

            for i in (start_index..current_index).rev() {
                if let Some(entry) = history.past.get(i) {
                    transactions.extend(entry.transactions.clone());
                }
            }
        } else {
            // 前进：收集被重做的事务（从当前位置往前）
            let steps = steps as usize;

            // 从 future 中获取要重做的事务
            for i in 0..steps.min(history.future.len()) {
                if let Some(entry) = history.future.get(i) {
                    transactions.extend(entry.transactions.clone());
                }
            }
        }

        // 更新历史位置
        history_manager.jump(steps);

        // 获取跳转后的状态（直接使用快照）
        let new_entry = history_manager.get_present();
        let new_state = new_entry.state.clone();

        metrics::history_operation("jump");

        Some(HistoryOperationResult { old_state, new_state, transactions })
    }

    /// 插入新的历史记录
    ///
    /// # 参数
    /// * `history_manager` - 历史管理器的可变引用
    /// * `state` - 应用事务后的状态
    /// * `transactions` - 事务列表
    /// * `description` - 描述信息
    /// * `meta` - 元数据
    pub fn insert(
        history_manager: &mut HistoryManager<HistoryEntryWithMeta>,
        state: Arc<State>,
        transactions: Vec<Arc<Transaction>>,
        description: String,
        meta: serde_json::Value,
    ) {
        if transactions.is_empty() {
            return;
        }

        let entry = if transactions.len() == 1 {
            HistoryEntryWithMeta::new(
                transactions[0].clone(),
                state,
                description,
                meta,
            )
        } else {
            HistoryEntryWithMeta::new_batch(
                transactions,
                state,
                description,
                meta,
            )
        };
        history_manager.insert(entry);
    }
}
