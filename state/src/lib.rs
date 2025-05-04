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
pub mod logging;
pub mod ops;
pub mod plugin;
pub mod resource;
pub mod resource_table;
pub mod state;
pub mod transaction;
pub use state::{State, StateConfig, Configuration};
pub use transaction::Transaction;
pub use tracing::{info, debug, warn, error};
/// 初始化日志系统
///
/// # 参数
/// * `level` - 日志级别，可选值：
///   - trace: 最详细的跟踪信息
///   - debug: 调试信息
///   - info: 一般信息
///   - warn: 警告信息
///   - error: 错误信息
/// * `file_path` - 日志文件路径，如果为 None 则只输出到控制台
///
/// # 示例
/// ```
/// use moduforge_core::init_logging;
///
/// // 只输出到控制台
/// init_logging("debug", None)?;
///
/// // 同时输出到文件和控制台
/// init_logging("info", Some("logs/moduforge.log"))?;
/// ```
pub fn init_logging(
    level: &str,
    file_path: Option<&str>,
) -> anyhow::Result<()> {
    use std::path::PathBuf;

    let level = match level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    let file_path = file_path.map(PathBuf::from);
    logging::init_logging(Some(level), file_path)
}
