use anyhow::{Result};
/// 错误结果类型别名
pub type EditorResult<T> = Result<T>;

/// 错误处理工具函数
pub mod error_utils {
    use super::*;

    /// 将错误转换为更具体的错误类型
    pub fn map_error<T, E: std::error::Error>(
        result: Result<T, E>,
        context: &str,
    ) -> EditorResult<T> {
        result.map_err(|e| {
            anyhow::anyhow!(format!("未知的错误 {}: {}", context, e))
        })
    }

    /// 创建状态错误
    pub fn state_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!(format!("状态错误 {:?}", msg.into()))
    }

    /// 创建存储错误
    pub fn storage_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!(format!("存储相关错误 {:?}", msg.into()))
    }

    /// 创建事件错误
    pub fn event_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!(format!("事件处理错误 {:?}", msg.into()))
    }

    /// 创建插件错误
    pub fn plugin_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!(format!("插件相关错误 {:?}", msg.into()))
    }

    /// 创建配置错误
    pub fn config_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!(format!("配置错误 {:?}", msg.into()))
    }

    /// 创建事务错误
    pub fn transaction_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!(format!(" 事务处理错误 {:?}", msg.into()))
    }

    /// 创建历史记录错误
    pub fn history_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!(format!(" 历史记录错误 {:?}", msg.into()))
    }

    /// 创建引擎错误
    pub fn engine_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!(format!(" 引擎错误 {:?}", msg.into()))
    }

    /// 创建缓存错误
    pub fn cache_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!(format!("  缓存错误 {:?}", msg.into()))
    }

    /// 创建中间件错误
    pub fn middleware_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow::anyhow!(format!("  插件相关错误 {:?}", msg.into()))
    }
}
