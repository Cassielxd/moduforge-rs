//! ModuForge-RS 核心模块
//!
//! 该模块提供了框架的核心功能，包括：
//! - 异步处理器和运行时
//! - 事件系统
//! - 扩展机制
//! - 流程控制
//! - 错误处理
//! - 历史记录管理
//! - 中间件支持
//! - 节点系统
//! - 类型定义
//!
//! 主要组件：
//! - `async_processor`: 异步任务处理器
//! - `async_runtime`: 异步运行时环境
//! - `error`: 错误类型和处理
//! - `event`: 事件系统
//! - `extension`: 扩展机制
//! - `flow`: 流程控制
//! - `history_manager`: 历史记录管理
//! - `middleware`: 中间件支持
//! - `node`: 节点系统
//! - `types`: 核心类型定义

pub mod config;
pub mod debug;
pub mod error;
pub mod error_helpers;
pub mod event;
pub mod extension;
pub mod extension_manager;
pub mod helpers;
pub mod history_manager;
#[cfg(test)]
pub mod test_helpers;

pub mod mark;
pub mod metrics;
pub mod middleware;
pub mod node;
pub mod runtime;
pub mod schema_parser;
pub mod types;

// 追踪初始化模块（开发环境专用）
pub mod tracing_init;

// 新的Actor系统模块
pub mod actors;

// 泛型运行时系统模块
pub mod generic;

pub use error::{ForgeResult, error_utils};
pub use error_helpers::{
    UnwrapHelpers, lock_helpers, collection_helpers, schema_helpers,
    state_helpers,
};

// 公共 API 导出
pub use runtime::async_processor::{
    AsyncProcessor, ProcessorError, TaskProcessor, TaskResult, TaskStatus,
};
pub use runtime::async_runtime::ForgeAsyncRuntime;
// 新的Actor运行时
pub use runtime::actor_runtime::ForgeActorRuntime;
// 运行时统一接口
pub use runtime::runtime_trait::{RuntimeTrait};
// 运行时构建器和自适应配置
pub use runtime::builder::{ForgeRuntimeBuilder, AnyRuntime};
pub use runtime::system_detector::{SystemResources, ResourceTier};
pub use runtime::adaptive::AdaptiveRuntimeSelector;
pub use config::{
    ForgeConfig, ForgeConfigBuilder, Environment, ProcessorConfig,
    PerformanceConfig, EventConfig, HistoryConfig, ExtensionConfig,
    CacheConfig, ConfigValidationError, RuntimeType, RuntimeConfig,
};
pub use error::ForgeError;
pub use event::{Event, EventBus, EventHandler};
pub use extension::Extension;
pub use extension_manager::{ExtensionManager, ExtensionManagerBuilder};
pub use history_manager::{History, HistoryManager};

pub use runtime::runtime::ForgeRuntime;
pub use schema_parser::{
    XmlSchemaParser, XmlSchemaSerializer, XmlSchemaError, XmlSchemaResult,
};
pub use runtime::sync_processor::{
    SyncProcessor, TaskProcessor as SyncTaskProcessor,
};
pub use types::*;

// Actor系统相关导出
pub use actors::{
    ForgeActorSystem, ActorSystemConfig,
    transaction_processor::{TransactionMessage, TransactionStats},
    state_actor::{StateMessage, HistoryInfo, StateSnapshot},
    event_bus::{EventBusMessage, EventBusStats},
};
