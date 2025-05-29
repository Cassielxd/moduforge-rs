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

pub mod async_processor;
pub mod async_runtime;
pub mod error;
pub mod event;
pub mod extension;
pub mod extension_manager;
pub mod flow;
pub mod helpers;
pub mod history_manager;
pub mod sync_processor;

pub mod mark;
pub mod metrics;
pub mod middleware;
pub mod node;
pub mod runtime;
pub mod types;
pub use error::{EditorResult, error_utils};
