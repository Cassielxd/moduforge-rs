//! 历史管理辅助模块
//!
//! 提供统一的历史管理逻辑，包括：
//! - 撤销（undo）操作
//! - 重做（redo）操作
//! - 跳转（jump）操作
//! - 性能指标记录

use crate::{history_manager::HistoryManager, metrics, types::HistoryEntryWithMeta};
use mf_state::state::State;
use std::sync::Arc;

/// 历史管理辅助器
pub struct HistoryHelper;

impl HistoryHelper {
    /// 执行撤销操作
    ///
    /// # 参数
    /// * `history_manager` - 历史管理器的可变引用
    ///
    /// # 返回值
    /// * `Arc<State>` - 撤销后的状态
    pub fn undo(
        history_manager: &mut HistoryManager<HistoryEntryWithMeta>
    ) -> Arc<State> {
        history_manager.jump(-1);
        let state = history_manager.get_present().state.clone();
        metrics::history_operation("undo");
        state
    }

    /// 执行重做操作
    ///
    /// # 参数
    /// * `history_manager` - 历史管理器的可变引用
    ///
    /// # 返回值
    /// * `Arc<State>` - 重做后的状态
    pub fn redo(
        history_manager: &mut HistoryManager<HistoryEntryWithMeta>
    ) -> Arc<State> {
        history_manager.jump(1);
        let state = history_manager.get_present().state.clone();
        metrics::history_operation("redo");
        state
    }

    /// 执行跳转操作
    ///
    /// # 参数
    /// * `history_manager` - 历史管理器的可变引用
    /// * `steps` - 跳转步数（正数前进，负数后退）
    ///
    /// # 返回值
    /// * `Arc<State>` - 跳转后的状态
    pub fn jump(
        history_manager: &mut HistoryManager<HistoryEntryWithMeta>,
        steps: isize,
    ) -> Arc<State> {
        history_manager.jump(steps);
        let state = history_manager.get_present().state.clone();
        metrics::history_operation("jump");
        state
    }

    /// 插入新的历史记录
    ///
    /// # 参数
    /// * `history_manager` - 历史管理器的可变引用
    /// * `state` - 新状态
    /// * `description` - 描述信息
    /// * `meta` - 元数据
    pub fn insert(
        history_manager: &mut HistoryManager<HistoryEntryWithMeta>,
        state: Arc<State>,
        description: String,
        meta: serde_json::Value,
    ) {
        history_manager.insert(HistoryEntryWithMeta::new(
            state,
            description,
            meta,
        ));
    }

    /// 获取当前状态
    ///
    /// # 参数
    /// * `history_manager` - 历史管理器的引用
    ///
    /// # 返回值
    /// * `Arc<State>` - 当前状态
    pub fn get_current_state(
        history_manager: &HistoryManager<HistoryEntryWithMeta>
    ) -> Arc<State> {
        history_manager.get_present().state.clone()
    }
}
