//! 状态管理Actor - 基于ractor框架实现
//!
//! 此Actor负责管理所有状态操作，确保状态更新的线程安全和串行执行。

use ractor::{Actor, ActorRef, ActorProcessingErr};
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::{
    debug::debug, error::ForgeResult, history_manager::HistoryManager,
    types::HistoryEntryWithMeta,
};

use mf_state::state::State;

use super::ActorSystemResult;

/// 状态管理消息类型
#[derive(Debug)]
pub enum StateMessage {
    /// 获取当前状态
    GetState { reply: oneshot::Sender<Arc<State>> },
    /// 应用事务（包含元信息）
    ApplyTransaction {
        transaction: mf_state::Transaction,
        description: String,
        meta: serde_json::Value,
        reply: oneshot::Sender<ForgeResult<Arc<State>>>,
    },
    /// 批量应用事务
    ApplyTransactionBatch {
        transactions: Vec<mf_state::Transaction>,
        description: String,
        meta: serde_json::Value,
        reply: oneshot::Sender<ForgeResult<Arc<State>>>,
    },
    /// 撤销操作
    Undo { reply: oneshot::Sender<ForgeResult<Arc<State>>> },
    /// 重做操作
    Redo { reply: oneshot::Sender<ForgeResult<Arc<State>>> },
    /// 跳转到指定历史位置
    Jump { steps: isize, reply: oneshot::Sender<ForgeResult<Arc<State>>> },
    /// 获取历史记录信息
    GetHistoryInfo { reply: oneshot::Sender<HistoryInfo> },
    /// 创建状态快照
    CreateSnapshot { reply: oneshot::Sender<StateSnapshot> },
    /// 记录已应用的事务到历史（不实际应用事务）
    RecordTransactions {
        state: Arc<State>,
        transactions: Vec<Arc<mf_state::Transaction>>,
        description: String,
        meta: serde_json::Value,
        reply: oneshot::Sender<ForgeResult<()>>,
    },
}

// StateMessage 自动实现 ractor::Message (Debug + Send + 'static)

/// 历史记录信息
#[derive(Debug, Clone)]
pub struct HistoryInfo {
    pub current_index: usize,
    pub total_entries: usize,
    pub can_undo: bool,
    pub can_redo: bool,
}

/// 状态快照
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub state: Arc<State>,
    pub timestamp: std::time::SystemTime,
    pub version: u64,
}

/// 状态Actor内部状态
pub struct StateActorState {
    /// 当前状态
    current_state: Arc<State>,
    /// 历史记录管理器
    history_manager: HistoryManager<HistoryEntryWithMeta>,
    /// 状态版本计数器
    version_counter: u64,
}

/// 状态管理Actor
pub struct StateActor;

#[ractor::async_trait]
impl Actor for StateActor {
    type Msg = StateMessage;
    type State = StateActorState;
    type Arguments = (Arc<State>, HistoryManager<HistoryEntryWithMeta>);

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        (initial_state, history_manager): Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        debug!("启动状态管理Actor");

        Ok(StateActorState {
            current_state: initial_state,
            history_manager,
            version_counter: 0,
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            StateMessage::GetState { reply } => {
                let _ = reply.send(state.current_state.clone());
            },

            StateMessage::ApplyTransaction {
                transaction,
                description,
                meta,
                reply,
            } => {
                let result = self
                    .apply_transaction_logic(
                        state,
                        transaction,
                        description,
                        meta,
                    )
                    .await;

                let _ = reply.send(result);
            },

            StateMessage::ApplyTransactionBatch {
                transactions,
                description,
                meta,
                reply,
            } => {
                let result = self
                    .apply_transaction_batch_logic(
                        state,
                        transactions,
                        description,
                        meta,
                    )
                    .await;

                let _ = reply.send(result);
            },

            StateMessage::Undo { reply } => {
                let result = self.undo_logic(state).await;
                let _ = reply.send(result);
            },

            StateMessage::Redo { reply } => {
                let result = self.redo_logic(state).await;
                let _ = reply.send(result);
            },

            StateMessage::Jump { steps, reply } => {
                let result = self.jump_logic(state, steps).await;
                let _ = reply.send(result);
            },

            StateMessage::GetHistoryInfo { reply } => {
                let info = self.get_history_info_logic(state);
                let _ = reply.send(info);
            },

            StateMessage::CreateSnapshot { reply } => {
                let snapshot = StateSnapshot {
                    state: state.current_state.clone(),
                    timestamp: std::time::SystemTime::now(),
                    version: state.version_counter,
                };
                let _ = reply.send(snapshot);
            },

            StateMessage::RecordTransactions {
                state: new_state,
                transactions,
                description,
                meta,
                reply,
            } => {
                if transactions.is_empty() {
                    let _ = reply.send(Ok(()));
                    return Ok(());
                }

                // 记录事务到历史（不应用，因为已经在外部应用过了）
                let entry = if transactions.len() == 1 {
                    HistoryEntryWithMeta::new(transactions[0].clone(), new_state, description, meta)
                } else {
                    HistoryEntryWithMeta::new_batch(transactions, new_state, description, meta)
                };

                state.history_manager.insert(entry);
                state.version_counter += 1;

                let _ = reply.send(Ok(()));
            },
        }

        Ok(())
    }

    async fn post_stop(
        &self,
        _myself: ActorRef<Self::Msg>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        debug!("停止状态管理Actor");
        Ok(())
    }
}

impl StateActor {
    /// 应用单个事务
    async fn apply_transaction_logic(
        &self,
        actor_state: &mut StateActorState,
        transaction: mf_state::Transaction,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<Arc<State>> {
        // 应用事务
        let result =
            actor_state.current_state.apply(transaction.clone()).await?;
        actor_state.current_state = result.state;

        // 增加版本号
        actor_state.version_counter += 1;

        // 保存事务到历史（包含状态快照）
        actor_state.history_manager.insert(HistoryEntryWithMeta::new(
            Arc::new(transaction),
            actor_state.current_state.clone(),
            description,
            meta,
        ));

        Ok(actor_state.current_state.clone())
    }

    /// 批量应用事务
    async fn apply_transaction_batch_logic(
        &self,
        actor_state: &mut StateActorState,
        transactions: Vec<mf_state::Transaction>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<Arc<State>> {
        let mut transaction_arcs = Vec::new();

        // 逐个应用事务
        for tr in transactions {
            let result = actor_state.current_state.apply(tr.clone()).await?;
            actor_state.current_state = result.state;
            transaction_arcs.push(Arc::new(tr));
        }

        // 增加版本号
        actor_state.version_counter += 1;

        // 批量保存事务到历史（包含状态快照）
        actor_state.history_manager.insert(HistoryEntryWithMeta::new_batch(
            transaction_arcs,
            actor_state.current_state.clone(),
            description,
            meta,
        ));

        Ok(actor_state.current_state.clone())
    }

    /// 撤销逻辑 - 使用状态快照直接切换 (O(1) 性能)
    async fn undo_logic(
        &self,
        actor_state: &mut StateActorState,
    ) -> ForgeResult<Arc<State>> {
        if !actor_state.history_manager.can_undo() {
            return Ok(actor_state.current_state.clone());
        }

        // 更新历史位置
        actor_state.history_manager.jump(-1);

        // 获取撤销后的状态（直接使用快照）
        let entry = actor_state.history_manager.get_present();
        actor_state.current_state = entry.state.clone();

        // 记录指标
        crate::metrics::history_operation("undo");

        Ok(actor_state.current_state.clone())
    }

    /// 重做逻辑 - 使用状态快照直接切换 (O(1) 性能)
    async fn redo_logic(
        &self,
        actor_state: &mut StateActorState,
    ) -> ForgeResult<Arc<State>> {
        if !actor_state.history_manager.can_redo() {
            return Ok(actor_state.current_state.clone());
        }

        // 更新历史位置
        actor_state.history_manager.jump(1);

        // 获取重做后的状态（直接使用快照）
        let entry = actor_state.history_manager.get_present();
        actor_state.current_state = entry.state.clone();

        // 记录指标
        crate::metrics::history_operation("redo");

        Ok(actor_state.current_state.clone())
    }

    /// 跳转逻辑 - 使用状态快照直接跳转
    async fn jump_logic(
        &self,
        actor_state: &mut StateActorState,
        steps: isize,
    ) -> ForgeResult<Arc<State>> {
        if steps == 0 {
            return Ok(actor_state.current_state.clone());
        }

        // 更新历史位置
        actor_state.history_manager.jump(steps);

        // 获取跳转后的状态（直接使用快照）
        let entry = actor_state.history_manager.get_present();
        actor_state.current_state = entry.state.clone();

        // 记录指标
        crate::metrics::history_operation("jump");

        Ok(actor_state.current_state.clone())
    }

    /// 计算事务的逆向操作（已废弃，混合方案直接使用状态快照）
    #[allow(dead_code)]
    fn invert_transaction(
        &self,
        tr: &mf_state::Transaction,
        current_state: &State,
    ) -> ForgeResult<mf_state::Transaction> {

        let mut inverted_tr = mf_state::Transaction::new(current_state);

        // 反向遍历步骤（LIFO）
        for step in tr.steps.iter().rev() {
            if let Some(inverted_step) =
                step.invert(&current_state.doc().get_inner())
            {
                inverted_tr.step(inverted_step)?;
            }
        }

        Ok(inverted_tr)
    }

    /// 获取历史记录信息
    fn get_history_info_logic(
        &self,
        actor_state: &StateActorState,
    ) -> HistoryInfo {
        // 假设HistoryManager有这些方法，如果没有需要添加
        let current_index =
            actor_state.history_manager.get_history().past.len();
        let total_entries = actor_state.history_manager.get_history_length();

        HistoryInfo {
            current_index,
            total_entries,
            can_undo: current_index > 0,
            can_redo: current_index < total_entries.saturating_sub(1),
        }
    }
}

/// 状态Actor管理器
pub struct StateActorManager;

impl StateActorManager {
    /// 启动状态Actor
    pub async fn start(
        initial_state: Arc<State>,
        history_manager: HistoryManager<HistoryEntryWithMeta>,
    ) -> ActorSystemResult<ActorRef<StateMessage>> {
        let (actor_ref, _handle) = Actor::spawn(
            Some("StateActor".to_string()),
            StateActor,
            (initial_state, history_manager),
        )
        .await
        .map_err(|e| super::ActorSystemError::ActorStartupFailed {
            actor_name: "StateActor".to_string(),
            source: e,
        })?;

        debug!("状态管理Actor启动成功");
        Ok(actor_ref)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_state_actor_basic_operations() {
        // 创建测试状态和历史管理器
        // 注意：这需要实际的State实现，这里只是占位测试
    }

    #[tokio::test]
    async fn test_state_actor_history_operations() {
        // 测试撤销/重做功能
    }
}
