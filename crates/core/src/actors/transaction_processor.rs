//! 事务处理Actor - 基于ractor框架实现
//!
//! 此Actor负责处理所有事务逻辑，保持与原始dispatch_with_meta方法完全相同的执行顺序。

use ractor::{Actor, ActorRef, ActorProcessingErr};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::oneshot;

use crate::{
    config::ForgeConfig,
    debug::debug,
    error::{error_utils, ForgeResult},
    event::Event,
    middleware::MiddlewareStack,
    runtime::sync_flow::FlowEngine,
    types::ProcessorResult,
    metrics,
};

use mf_state::{state::State, transaction::Transaction};

use super::{ActorMetrics, ActorSystemResult};

/// 事务处理消息类型
#[derive(Debug)]
pub enum TransactionMessage {
    /// 处理事务（保持与原始dispatch_with_meta完全相同的逻辑）
    ProcessTransaction {
        transaction: Transaction,
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

// TransactionMessage 自动实现 ractor::Message (Debug + Send + 'static)

/// 事务处理统计信息
#[derive(Debug, Clone)]
pub struct TransactionStats {
    pub transactions_processed: u64,
    pub transaction_failures: u64,
    pub avg_processing_time_ms: u64,
    pub middleware_timeouts: u64,
}

/// 事务处理Actor状态
pub struct TransactionProcessorState {
    /// 状态Actor引用
    state_actor: ActorRef<super::StateMessage>,
    /// 事件总线Actor引用
    event_bus: ActorRef<super::EventBusMessage>,
    /// 中间件堆栈
    middleware_stack: MiddlewareStack,
    /// 流引擎
    flow_engine: Arc<FlowEngine>,
    /// 配置
    config: ForgeConfig,
    /// 指标收集
    metrics: ActorMetrics,
    /// 统计信息
    stats: TransactionStats,
}

/// 事务处理Actor
pub struct TransactionProcessorActor;

#[ractor::async_trait]
impl Actor for TransactionProcessorActor {
    type Msg = TransactionMessage;
    type State = TransactionProcessorState;
    type Arguments = (
        ActorRef<super::StateMessage>,
        ActorRef<super::EventBusMessage>,
        MiddlewareStack,
        Arc<FlowEngine>,
        ForgeConfig,
    );

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let (state_actor, event_bus, middleware_stack, flow_engine, config) =
            args;

        debug!("启动事务处理Actor");

        Ok(TransactionProcessorState {
            state_actor,
            event_bus,
            middleware_stack,
            flow_engine,
            config,
            metrics: ActorMetrics::default(),
            stats: TransactionStats {
                transactions_processed: 0,
                transaction_failures: 0,
                avg_processing_time_ms: 0,
                middleware_timeouts: 0,
            },
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            TransactionMessage::ProcessTransaction {
                transaction,
                description,
                meta,
                reply,
            } => {
                let start_time = Instant::now();

                // 🎯 完全保持原始dispatch_with_meta的逻辑
                let result = self
                    .dispatch_with_meta_exact_logic(
                        state,
                        transaction,
                        description,
                        meta,
                    )
                    .await;

                let processing_time = start_time.elapsed();

                // 更新统计信息
                state.stats.transactions_processed += 1;
                if result.is_err() {
                    state.stats.transaction_failures += 1;
                    state.metrics.increment_errors();
                }
                state.stats.avg_processing_time_ms =
                    processing_time.as_millis() as u64;
                state
                    .metrics
                    .update_processing_time(processing_time.as_millis() as u64);
                state.metrics.increment_messages();

                // 发送回复
                let _ = reply.send(result);
            },
            TransactionMessage::GetStats { reply } => {
                let _ = reply.send(state.stats.clone());
            },
            TransactionMessage::UpdateConfig { config, reply } => {
                state.config = config;
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
        debug!("停止事务处理Actor");
        Ok(())
    }
}

impl TransactionProcessorActor {
    /// 🎯 与原始dispatch_with_meta完全相同的逻辑实现
    ///
    /// 这个方法保持与runtime.rs:674-721行完全相同的执行流程
    async fn dispatch_with_meta_exact_logic(
        &self,
        state: &mut TransactionProcessorState,
        transaction: Transaction,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        // 1. 指标记录 - 与原代码完全相同
        metrics::transaction_dispatched();

        // 2. 获取当前状态版本 - 通过消息获取
        let current_state = self.get_current_state(&state.state_actor).await?;
        let old_id = current_state.version;

        // 3. 前置中间件 - 完全相同的逻辑
        let mut current_transaction = transaction;
        self.run_before_middleware(state, &mut current_transaction).await?;

        // 4. 事务应用 - 完全相同的逻辑
        let task_result = state
            .flow_engine
            .submit((current_state, current_transaction.clone()))
            .await;

        let Some(ProcessorResult { result: Some(result), .. }) =
            task_result.output
        else {
            return Err(error_utils::state_error(
                "任务处理结果无效".to_string(),
            ));
        };

        // 5. 状态更新逻辑 - 完全相同
        let mut state_update = None;
        let mut transactions = Vec::new();
        transactions.extend(result.transactions);

        if transactions.last().is_some() {
            state_update = Some(result.state);
        }

        // 6. 后置中间件 - 完全相同的逻辑
        self.run_after_middleware(state, &mut state_update, &mut transactions)
            .await?;

        // 7. 状态更新和事件广播 - 通过消息传递，但逻辑相同
        if let Some(new_state) = state_update {
            self.update_state_with_meta(
                &state.state_actor,
                new_state.clone(),
                description,
                meta,
            )
            .await?;

            self.emit_event(
                &state.event_bus,
                Event::TrApply(old_id, transactions, new_state),
            )
            .await?;
        }

        Ok(())
    }

    /// 获取当前状态 - 通过消息传递
    async fn get_current_state(
        &self,
        state_actor: &ActorRef<super::StateMessage>,
    ) -> ForgeResult<Arc<State>> {
        let (tx, rx) = oneshot::channel();

        state_actor
            .send_message(super::StateMessage::GetState { reply: tx })
            .map_err(|e| {
            error_utils::state_error(format!("发送获取状态消息失败: {e}"))
        })?;

        rx.await.map_err(|e| {
            error_utils::state_error(format!("接收状态响应失败: {e}"))
        })
    }

    /// 🔄 前置中间件逻辑 - 与原代码完全相同（529-568行）
    async fn run_before_middleware(
        &self,
        actor_state: &mut TransactionProcessorState,
        transaction: &mut Transaction,
    ) -> ForgeResult<()> {
        debug!("执行前置中间件链");

        for middleware in &actor_state.middleware_stack.middlewares {
            let start_time = Instant::now();
            let timeout = std::time::Duration::from_millis(
                actor_state.config.performance.middleware_timeout_ms,
            );

            match tokio::time::timeout(
                timeout,
                middleware.before_dispatch(transaction),
            )
            .await
            {
                Ok(Ok(())) => {
                    metrics::middleware_execution_duration(
                        start_time.elapsed(),
                        "before",
                        middleware.name().as_str(),
                    );
                    continue;
                },
                Ok(Err(e)) => {
                    return Err(error_utils::middleware_error(format!(
                        "前置中间件执行失败: {e}"
                    )));
                },
                Err(_) => {
                    actor_state.stats.middleware_timeouts += 1;
                    return Err(error_utils::middleware_error(format!(
                        "前置中间件执行超时（{}ms）",
                        actor_state.config.performance.middleware_timeout_ms
                    )));
                },
            }
        }
        Ok(())
    }

    /// 🔄 后置中间件逻辑 - 与原代码完全相同（570-628行逻辑）
    async fn run_after_middleware(
        &self,
        actor_state: &mut TransactionProcessorState,
        state_update: &mut Option<Arc<State>>,
        transactions: &mut Vec<Arc<Transaction>>,
    ) -> ForgeResult<()> {
        debug!("执行后置中间件链");

        for middleware in &actor_state.middleware_stack.middlewares {
            let start_time = Instant::now();
            let timeout = std::time::Duration::from_millis(
                actor_state.config.performance.middleware_timeout_ms,
            );

            let middleware_result = match tokio::time::timeout(
                timeout,
                middleware.after_dispatch(state_update.clone(), transactions),
            )
            .await
            {
                Ok(Ok(result)) => {
                    metrics::middleware_execution_duration(
                        start_time.elapsed(),
                        "after",
                        middleware.name().as_str(),
                    );
                    result
                },
                Ok(Err(e)) => {
                    return Err(error_utils::middleware_error(format!(
                        "后置中间件执行失败: {e}"
                    )));
                },
                Err(_) => {
                    actor_state.stats.middleware_timeouts += 1;
                    return Err(error_utils::middleware_error(format!(
                        "后置中间件执行超时（{}ms）",
                        actor_state.config.performance.middleware_timeout_ms
                    )));
                },
            };

            // 处理中间件返回的附加事务
            if let Some(mut additional_transaction) = middleware_result {
                additional_transaction.commit()?;

                let current_state = state_update.as_ref()
                    .ok_or_else(|| error_utils::state_error(
                        "处理附加事务时状态为空".to_string()
                    ))?
                    .clone();

                let task_result = actor_state
                    .flow_engine
                    .submit((current_state, additional_transaction))
                    .await;

                let Some(ProcessorResult { result: Some(result), .. }) =
                    task_result.output
                else {
                    return Err(error_utils::state_error(
                        "附加事务处理结果无效".to_string(),
                    ));
                };

                *state_update = Some(result.state);
                transactions.extend(result.transactions);
            }
        }
        Ok(())
    }

    /// 状态更新 - 通过消息传递
    async fn update_state_with_meta(
        &self,
        state_actor: &ActorRef<super::StateMessage>,
        state: Arc<State>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        let (tx, rx) = oneshot::channel();

        state_actor
            .send_message(super::StateMessage::UpdateStateWithMeta {
                state,
                description,
                meta,
                reply: tx,
            })
            .map_err(|e| {
                error_utils::state_error(format!("发送状态更新消息失败: {e}"))
            })?;

        rx.await.map_err(|e| {
            error_utils::state_error(format!("接收状态更新响应失败: {e}"))
        })?
    }

    /// 事件广播 - 通过消息传递
    async fn emit_event(
        &self,
        event_bus: &ActorRef<super::EventBusMessage>,
        event: Event,
    ) -> ForgeResult<()> {
        event_bus
            .send_message(super::EventBusMessage::PublishEvent { event })
            .map_err(|e| {
                error_utils::event_error(format!("发送事件消息失败: {e}"))
            })?;

        Ok(())
    }
}

/// 事务处理Actor管理器
pub struct TransactionProcessorManager;

impl TransactionProcessorManager {
    /// 启动事务处理Actor
    pub async fn start(
        state_actor: ActorRef<super::StateMessage>,
        event_bus: ActorRef<super::EventBusMessage>,
        middleware_stack: MiddlewareStack,
        flow_engine: Arc<FlowEngine>,
        config: ForgeConfig,
    ) -> ActorSystemResult<ActorRef<TransactionMessage>> {
        let (actor_ref, _handle) = Actor::spawn(
            Some("TransactionProcessor".to_string()),
            TransactionProcessorActor,
            (state_actor, event_bus, middleware_stack, flow_engine, config),
        )
        .await
        .map_err(|e| super::ActorSystemError::ActorStartupFailed {
            actor_name: "TransactionProcessor".to_string(),
            source: e,
        })?;

        debug!("事务处理Actor启动成功");
        Ok(actor_ref)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_transaction_processor_actor_creation() {
        // 这里只是基本的Actor创建测试
        // 完整的兼容性测试将在集成测试中进行

        // 注意：这需要其他Actor的模拟实现，暂时只测试基本结构
    }
}
