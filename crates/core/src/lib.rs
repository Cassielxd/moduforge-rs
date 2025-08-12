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
pub mod error;
pub mod event;
pub mod extension;
pub mod extension_manager;
pub mod helpers;
pub mod history_manager;

pub mod mark;
pub mod metrics;
pub mod middleware;
pub mod node;
pub mod runtime;
pub mod schema_parser;
pub mod types;
pub use error::{ForgeResult, error_utils};
/// 重命名
pub mod model {
    pub use mf_model::*;
}
/// 重命名
pub mod state {
    pub use mf_state::*;
}
/// 重命名
pub mod transform {
    pub use mf_transform::*;
}

// 公共 API 导出
pub use runtime::async_processor::{
    AsyncProcessor, ProcessorError, TaskProcessor, TaskResult, TaskStatus,
};
pub use runtime::async_runtime::ForgeAsyncRuntime;
pub use config::{
    ForgeConfig, ForgeConfigBuilder, Environment, ProcessorConfig,
    PerformanceConfig, EventConfig, HistoryConfig, ExtensionConfig,
    CacheConfig, ConfigValidationError,
};
pub use error::ForgeError;
pub use event::{Event, EventBus, EventHandler};
pub use extension::Extension;
pub use extension_manager::{ExtensionManager, ExtensionManagerBuilder};
pub use history_manager::{History, HistoryManager};
pub use runtime::runtime::ForgeRuntime;
pub use schema_parser::{XmlSchemaParser, XmlSchemaSerializer, XmlSchemaError, XmlSchemaResult};
pub use runtime::sync_processor::{
    SyncProcessor, TaskProcessor as SyncTaskProcessor,
};
pub use types::*;
