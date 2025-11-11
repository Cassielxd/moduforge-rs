//! 事件总线Actor - 基于ractor框架实现
//!
//! 此Actor负责事件的发布和订阅，保持与原始EventBus完全相同的行为。

use ractor::{Actor, ActorRef, ActorProcessingErr};
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::{
    config::EventConfig,
    debug::debug,
    error::{error_utils, ForgeResult},
    event::{Event, EventHandler, HandlerId},
};

use super::{ActorSystemResult, ActorMetrics};

/// 事件总线消息类型
#[derive(Debug)]
pub enum EventBusMessage {
    /// 发布事件
    PublishEvent { event: Event },
    /// 添加事件处理器
    AddHandler {
        handler: Arc<dyn EventHandler<Event> + Send + Sync>,
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

// EventBusMessage 自动实现 ractor::Message (Debug + Send + 'static)

/// 事件总线统计信息
#[derive(Debug, Clone)]
pub struct EventBusStats {
    pub events_published: u64,
    pub events_processed: u64,
    pub event_failures: u64,
    pub active_handlers: usize,
    pub avg_processing_time_ms: u64,
}

/// 事件总线Actor状态
pub struct EventBusActorState {
    /// 事件处理器列表
    handlers: Vec<(HandlerId, Arc<dyn EventHandler<Event> + Send + Sync>)>,
    /// 下一个处理器ID
    next_handler_id: HandlerId,
    /// 配置
    config: EventConfig,
    /// 指标收集
    metrics: ActorMetrics,
    /// 统计信息
    stats: EventBusStats,
}

/// 事件总线Actor
pub struct EventBusActor;

#[ractor::async_trait]
impl Actor for EventBusActor {
    type Msg = EventBusMessage;
    type State = EventBusActorState;
    type Arguments = EventConfig;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        config: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        debug!("启动事件总线Actor");

        Ok(EventBusActorState {
            handlers: Vec::new(),
            next_handler_id: 1,
            config,
            metrics: ActorMetrics::default(),
            stats: EventBusStats {
                events_published: 0,
                events_processed: 0,
                event_failures: 0,
                active_handlers: 0,
                avg_processing_time_ms: 0,
            },
        })
    }

    async fn post_stop(
        &self,
        _myself: ActorRef<Self::Msg>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        debug!("停止事件总线Actor");
        Ok(())
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            EventBusMessage::PublishEvent { event } => {
                let start_time = std::time::Instant::now();

                // 🎯 与原始事件广播逻辑完全相同
                let result = self.broadcast_event_logic(state, event).await;

                let processing_time = start_time.elapsed();
                state.stats.events_published += 1;

                if result.is_err() {
                    state.stats.event_failures += 1;
                    state.metrics.increment_errors();
                }

                state.stats.avg_processing_time_ms =
                    processing_time.as_millis() as u64;
                state
                    .metrics
                    .update_processing_time(processing_time.as_millis() as u64);
                state.metrics.increment_messages();

                // 注意：PublishEvent通常不需要回复，因为它是"fire and forget"模式
                if let Err(e) = result {
                    debug!("事件发布失败: {}", e);
                }
            },

            EventBusMessage::AddHandler { handler, reply } => {
                let handler_id = state.next_handler_id;
                state.next_handler_id += 1;

                state.handlers.push((handler_id, handler));
                state.stats.active_handlers = state.handlers.len();

                let _ = reply.send(handler_id);
            },

            EventBusMessage::RemoveHandler { handler_id, reply } => {
                let initial_len = state.handlers.len();
                state.handlers.retain(|(id, _)| *id != handler_id);

                let result = if state.handlers.len() < initial_len {
                    state.stats.active_handlers = state.handlers.len();
                    Ok(())
                } else {
                    Err(error_utils::event_error(format!(
                        "事件处理器 {handler_id} 不存在"
                    )))
                };

                let _ = reply.send(result);
            },

            EventBusMessage::GetStats { reply } => {
                let _ = reply.send(state.stats.clone());
            },

            EventBusMessage::UpdateConfig { config, reply } => {
                state.config = config;
                let _ = reply.send(Ok(()));
            },
        }

        Ok(())
    }
}

impl EventBusActor {
    /// 🎯 与原始事件广播逻辑完全相同
    ///
    /// 对应原始EventBus::broadcast的逻辑
    async fn broadcast_event_logic(
        &self,
        actor_state: &mut EventBusActorState,
        event: Event,
    ) -> ForgeResult<()> {
        debug!("广播事件: {}", event.name());

        let mut processing_errors = Vec::new();
        let event_name = event.name();

        // 并行处理所有事件处理器（与原始实现相同）
        let mut tasks = Vec::new();

        for (handler_id, handler) in &actor_state.handlers {
            let handler_clone = handler.clone();
            let event_clone = event.clone();
            let handler_id = *handler_id;

            // 创建处理任务
            let task = tokio::spawn(async move {
                let result = handler_clone.handle(&event_clone).await;
                (handler_id, result)
            });

            tasks.push(task);
        }

        // 等待所有处理器完成
        for task in tasks {
            match task.await {
                Ok((handler_id, Ok(()))) => {
                    actor_state.stats.events_processed += 1;
                    debug!(
                        "事件处理器 {} 成功处理事件 {}",
                        handler_id, event_name
                    );
                },
                Ok((handler_id, Err(e))) => {
                    processing_errors.push(format!(
                        "处理器 {handler_id} 处理事件 {event_name} 失败: {e}"
                    ));
                    actor_state.stats.event_failures += 1;
                },
                Err(e) => {
                    processing_errors
                        .push(format!("事件处理任务执行失败: {e}"));
                    actor_state.stats.event_failures += 1;
                },
            }
        }

        // 错误处理策略（与原始实现相同）
        if !processing_errors.is_empty() {
            let error_summary = processing_errors.join("; ");
            debug!("事件处理过程中出现错误: {}", error_summary);

            // 根据配置决定是否抛出错误
            // 如果处理失败，记录错误但继续处理其他handlers
            if false {
                // TODO: 可以考虑添加fail_on_handler_error配置
                return Err(error_utils::event_error(format!(
                    "事件 {event_name} 处理失败: {error_summary}"
                )));
            }
        }

        Ok(())
    }
}

/// 事件总线Actor管理器
pub struct EventBusActorManager;

impl EventBusActorManager {
    /// 启动事件总线Actor
    pub async fn start(
        config: EventConfig
    ) -> ActorSystemResult<ActorRef<EventBusMessage>> {
        let (actor_ref, _handle) = Actor::spawn(
            Some("EventBusActor".to_string()),
            EventBusActor,
            config,
        )
        .await
        .map_err(|e| super::ActorSystemError::ActorStartupFailed {
            actor_name: "EventBusActor".to_string(),
            source: e,
        })?;

        debug!("事件总线Actor启动成功");
        Ok(actor_ref)
    }

    /// 向事件总线添加处理器（便捷方法）
    pub async fn add_handlers(
        event_bus: &ActorRef<EventBusMessage>,
        handlers: Vec<Arc<dyn EventHandler<Event> + Send + Sync>>,
    ) -> ForgeResult<Vec<HandlerId>> {
        let mut handler_ids = Vec::new();

        for handler in handlers {
            let (tx, rx) = oneshot::channel();

            event_bus
                .send_message(EventBusMessage::AddHandler {
                    handler,
                    reply: tx,
                })
                .map_err(|e| {
                    error_utils::event_error(format!(
                        "发送添加处理器消息失败: {e}"
                    ))
                })?;

            let handler_id = rx.await.map_err(|e| {
                error_utils::event_error(format!("接收处理器ID失败: {e}"))
            })?;

            handler_ids.push(handler_id);
        }

        Ok(handler_ids)
    }
}
