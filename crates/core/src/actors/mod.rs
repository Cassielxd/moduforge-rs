//! Actor系统模块 - 基于ractor框架的实现
//!
//! 本模块使用ractor框架重构ModuForge的核心架构，实现Actor模式的并发设计。
//!
//! ## 架构设计
//!
//! - **TransactionProcessorActor**: 事务处理Actor，负责处理所有事务逻辑
//! - **StateActor**: 状态管理Actor，确保状态操作的线程安全
//! - **EventBusActor**: 事件总线Actor，处理事件的发布和订阅
//! - **ExtensionManagerActor**: 扩展管理Actor，负责插件系统
//! - **ForgeActorSystem**: Actor系统管理器，协调所有Actor
//!
//! ## 设计原则
//!
//! 1. **保持现有逻辑不变**: 所有业务逻辑保持与原实现完全相同
//! 2. **消息驱动**: 所有组件间通信通过消息传递
//! 3. **故障隔离**: Actor失败不影响其他Actor
//! 4. **性能优化**: 利用Actor模式的并发优势

pub mod event_bus;
pub mod extension_manager;
pub mod state_actor;
pub mod system;
pub mod transaction_processor;

// 重新导出核心类型
pub use transaction_processor::{TransactionProcessorActor, TransactionMessage};
pub use state_actor::{StateActor, StateMessage};
pub use event_bus::{EventBusActor, EventBusMessage};
pub use extension_manager::{ExtensionManagerActor, ExtensionMessage};
pub use system::{ForgeActorSystem, ActorSystemConfig};

use ractor::{SpawnErr};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::oneshot;

/// Actor系统错误类型
#[derive(Debug, Error)]
pub enum ActorSystemError {
    #[error("Actor启动失败: {actor_name} - {source}")]
    ActorStartupFailed { actor_name: String, source: SpawnErr },

    #[error("Actor通信失败: {message}")]
    CommunicationFailed { message: String },

    #[error("Actor系统关闭失败: {message}")]
    ShutdownFailed { message: String },

    #[error("配置错误: {message}")]
    ConfigurationError { message: String },

    #[error("超时错误: {operation}")]
    TimeoutError { operation: String },

    #[error("其他错误: {message}")]
    Other { message: String },
}

/// Actor系统结果类型
pub type ActorSystemResult<T> = Result<T, ActorSystemError>;

/// Actor消息包装器
///
/// 用于在保持现有API兼容性的同时，支持ractor的消息系统
#[derive(Debug)]
pub struct MessageWrapper<T> {
    pub inner: T,
    pub reply_to: Option<oneshot::Sender<crate::ForgeResult<()>>>,
}

impl<T> MessageWrapper<T> {
    pub fn new(inner: T) -> Self {
        Self { inner, reply_to: None }
    }

    pub fn with_reply(
        inner: T,
        reply_to: oneshot::Sender<crate::ForgeResult<()>>,
    ) -> Self {
        Self { inner, reply_to: Some(reply_to) }
    }

    pub fn reply(
        self,
        result: crate::ForgeResult<()>,
    ) {
        if let Some(sender) = self.reply_to {
            let _ = sender.send(result);
        }
    }
}

// 移除手动Message实现 - ractor 0.15.8 自动为满足条件的类型实现Message
// 只需要确保消息类型满足: Debug + Send + 'static

/// Actor管理器trait
///
/// 定义了Actor的生命周期管理接口
#[async_trait::async_trait]
pub trait ActorManager {
    type Config;
    type Handle;

    /// 启动Actor
    async fn start(config: Self::Config) -> ActorSystemResult<Self::Handle>;

    /// 停止Actor
    async fn stop(handle: Self::Handle) -> ActorSystemResult<()>;

    /// 检查Actor是否健康
    async fn health_check(handle: &Self::Handle) -> bool;
}

/// Actor指标收集
pub struct ActorMetrics {
    /// 消息处理计数
    pub messages_processed: Arc<std::sync::atomic::AtomicU64>,
    /// 错误计数
    pub errors_count: Arc<std::sync::atomic::AtomicU64>,
    /// 平均处理时间
    pub avg_processing_time: Arc<std::sync::atomic::AtomicU64>,
}

impl Default for ActorMetrics {
    fn default() -> Self {
        Self {
            messages_processed: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            errors_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            avg_processing_time: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
}

impl ActorMetrics {
    pub fn increment_messages(&self) {
        self.messages_processed
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn increment_errors(&self) {
        self.errors_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn update_processing_time(
        &self,
        duration_ms: u64,
    ) {
        self.avg_processing_time
            .store(duration_ms, std::sync::atomic::Ordering::Relaxed);
    }
}
