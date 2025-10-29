//! ModuForge-RS 状态管理模块
//!
//! 该模块负责管理应用程序的状态，包括：
//! - 状态管理
//! - 事务处理
//! - 资源管理
//! - 插件系统
//! - 日志系统
//!
//! 主要组件：
//! - `error`: 错误类型和处理
//! - `gotham_state`: Gotham 状态管理
//! - `logging`: 日志系统
//! - `ops`: 操作定义
//! - `plugin`: 插件系统
//! - `resource`: 资源管理
//! - `resource_table`: 资源表
//! - `state`: 状态管理
//! - `transaction`: 事务处理
//!
//! 核心类型：
//! - `State`: 状态管理
//! - `StateConfig`: 状态配置
//! - `Configuration`: 配置管理
//! - `Transaction`: 事务处理

pub mod error;
pub mod gotham_state;
pub mod ops;
pub mod plugin;
pub mod resource;
pub mod resource_table;
pub mod state;
pub mod transaction;
pub use state::{State, StateConfig, Configuration};
pub use transaction::Transaction;
pub use tracing::{info, debug, warn, error};
