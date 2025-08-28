/// 调试工具模块
/// 
/// 提供条件编译的调试输出宏，只在开发模式下启用

/// 条件调试宏 - 只在启用 debug-logs 特性时才输出
#[cfg(feature = "debug-logs")]
pub use mf_state::debug;

/// 生产环境下的空调试宏
#[cfg(not(feature = "debug-logs"))]
macro_rules! debug {
    ($($arg:tt)*) => {
        () // 生产环境下不输出任何调试信息，返回unit类型
    };
}

#[cfg(not(feature = "debug-logs"))]
pub(crate) use debug;

/// 条件信息输出宏 - 信息输出在生产环境中保留
pub use mf_state::info;

/// 警告和错误在生产环境中保留
pub use mf_state::{warn, error};