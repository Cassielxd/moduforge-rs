use std::fmt;
use std::io;

/// 编辑器核心错误类型
#[derive(Debug)]
pub enum EditorError {
    /// 状态相关错误
    StateError(String),
    /// 存储相关错误
    StorageError(String),
    /// 事件处理错误
    EventError(String),
    /// 插件相关错误
    PluginError(String),
    /// IO操作错误
    IoError(io::Error),
    /// 配置错误
    ConfigError(String),
    /// 事务处理错误
    TransactionError(String),
    /// 历史记录错误
    HistoryError(String),
    /// 引擎错误
    EngineError(String),
    /// 缓存错误
    CacheError(String),
    /// 未知错误
    Unknown(String),
}

impl fmt::Display for EditorError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            EditorError::StateError(msg) => write!(f, "State error: {}", msg),
            EditorError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            EditorError::EventError(msg) => write!(f, "Event error: {}", msg),
            EditorError::PluginError(msg) => write!(f, "Plugin error: {}", msg),
            EditorError::IoError(err) => write!(f, "IO error: {}", err),
            EditorError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            EditorError::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
            EditorError::HistoryError(msg) => write!(f, "History error: {}", msg),
            EditorError::EngineError(msg) => write!(f, "Engine error: {}", msg),
            EditorError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            EditorError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for EditorError {}

impl From<io::Error> for EditorError {
    fn from(err: io::Error) -> Self {
        EditorError::IoError(err)
    }
}

impl From<String> for EditorError {
    fn from(err: String) -> Self {
        EditorError::Unknown(err)
    }
}

impl From<&str> for EditorError {
    fn from(err: &str) -> Self {
        EditorError::Unknown(err.to_string())
    }
}

/// 错误结果类型别名
pub type EditorResult<T> = Result<T, EditorError>;

/// 错误处理工具函数
pub mod error_utils {
    use super::*;

    /// 将错误转换为更具体的错误类型
    pub fn map_error<T, E: std::error::Error>(
        result: Result<T, E>,
        context: &str,
    ) -> EditorResult<T> {
        result.map_err(|e| EditorError::Unknown(format!("{}: {}", context, e)))
    }

    /// 创建状态错误
    pub fn state_error(msg: impl Into<String>) -> EditorError {
        EditorError::StateError(msg.into())
    }

    /// 创建存储错误
    pub fn storage_error(msg: impl Into<String>) -> EditorError {
        EditorError::StorageError(msg.into())
    }

    /// 创建事件错误
    pub fn event_error(msg: impl Into<String>) -> EditorError {
        EditorError::EventError(msg.into())
    }

    /// 创建插件错误
    pub fn plugin_error(msg: impl Into<String>) -> EditorError {
        EditorError::PluginError(msg.into())
    }

    /// 创建配置错误
    pub fn config_error(msg: impl Into<String>) -> EditorError {
        EditorError::ConfigError(msg.into())
    }

    /// 创建事务错误
    pub fn transaction_error(msg: impl Into<String>) -> EditorError {
        EditorError::TransactionError(msg.into())
    }

    /// 创建历史记录错误
    pub fn history_error(msg: impl Into<String>) -> EditorError {
        EditorError::HistoryError(msg.into())
    }

    /// 创建引擎错误
    pub fn engine_error(msg: impl Into<String>) -> EditorError {
        EditorError::EngineError(msg.into())
    }

    /// 创建缓存错误
    pub fn cache_error(msg: impl Into<String>) -> EditorError {
        EditorError::CacheError(msg.into())
    }
}
