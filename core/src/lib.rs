pub mod logging;
pub mod model;
pub mod state;
pub mod transform;

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
