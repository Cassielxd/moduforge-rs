use thiserror::Error;

/// 统一的 Forge 错误类型
///
/// 这个枚举定义了 ModuForge 核心模块中可能出现的所有错误类型，
/// 提供了结构化的错误处理和更好的错误分类。
#[derive(Error, Debug)]
pub enum ForgeError {
    /// 状态管理相关错误
    #[error("状态错误: {message}")]
    State {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 事件系统相关错误
    #[error("事件错误: {message}")]
    Event {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 中间件相关错误
    #[error("中间件错误: {message}")]
    Middleware {
        message: String,
        middleware_name: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 扩展和插件相关错误
    #[error("扩展错误: {message}")]
    Extension {
        message: String,
        extension_name: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 事务处理相关错误
    #[error("事务错误: {message}")]
    Transaction {
        message: String,
        transaction_id: Option<u64>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 历史记录相关错误
    #[error("历史记录错误: {message}")]
    History {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 配置相关错误
    #[error("配置错误: {message}")]
    Config {
        message: String,
        config_key: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 存储相关错误
    #[error("存储错误: {message}")]
    Storage {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 缓存相关错误
    #[error("缓存错误: {message}")]
    Cache {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 引擎相关错误
    #[error("引擎错误: {message}")]
    Engine {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 超时错误
    #[error("操作超时: {operation} (超时时间: {timeout_ms}ms)")]
    Timeout {
        operation: String,
        timeout_ms: u64,
    },

    /// 资源不足错误
    #[error("资源不足: {resource_type}")]
    ResourceExhausted {
        resource_type: String,
        current_usage: Option<usize>,
        limit: Option<usize>,
    },

    /// 并发错误
    #[error("并发错误: {message}")]
    Concurrency {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 验证错误
    #[error("验证失败: {message}")]
    Validation {
        message: String,
        field: Option<String>,
    },

    /// 外部依赖错误
    #[error("外部依赖错误: {dependency}")]
    ExternalDependency {
        dependency: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// 内部错误（不应该发生的错误）
    #[error("内部错误: {message}")]
    Internal {
        message: String,
        location: Option<String>,
    },

    /// 兼容性错误，用于包装其他错误类型
    #[error("其他错误: {0}")]
    Other(#[from] anyhow::Error),
}

/// 错误结果类型别名
pub type ForgeResult<T> = Result<T, ForgeError>;

impl ForgeError {
    /// 获取错误代码，用于程序化处理
    pub fn error_code(&self) -> &'static str {
        match self {
            ForgeError::State { .. } => "STATE_ERROR",
            ForgeError::Event { .. } => "EVENT_ERROR",
            ForgeError::Middleware { .. } => "MIDDLEWARE_ERROR",
            ForgeError::Extension { .. } => "EXTENSION_ERROR",
            ForgeError::Transaction { .. } => "TRANSACTION_ERROR",
            ForgeError::History { .. } => "HISTORY_ERROR",
            ForgeError::Config { .. } => "CONFIG_ERROR",
            ForgeError::Storage { .. } => "STORAGE_ERROR",
            ForgeError::Cache { .. } => "CACHE_ERROR",
            ForgeError::Engine { .. } => "ENGINE_ERROR",
            ForgeError::Timeout { .. } => "TIMEOUT_ERROR",
            ForgeError::ResourceExhausted { .. } => "RESOURCE_EXHAUSTED",
            ForgeError::Concurrency { .. } => "CONCURRENCY_ERROR",
            ForgeError::Validation { .. } => "VALIDATION_ERROR",
            ForgeError::ExternalDependency { .. } => "EXTERNAL_DEPENDENCY_ERROR",
            ForgeError::Internal { .. } => "INTERNAL_ERROR",
            ForgeError::Other(_) => "OTHER_ERROR",
        }
    }

    /// 检查错误是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ForgeError::Timeout { .. }
                | ForgeError::ResourceExhausted { .. }
                | ForgeError::Concurrency { .. }
                | ForgeError::ExternalDependency { .. }
        )
    }

    /// 检查错误是否为临时性错误
    pub fn is_temporary(&self) -> bool {
        matches!(
            self,
            ForgeError::Timeout { .. }
                | ForgeError::ResourceExhausted { .. }
                | ForgeError::Concurrency { .. }
        )
    }
}

/// 错误构造工具函数
///
/// 这些函数提供了便捷的方式来创建各种类型的错误，
/// 同时保持与现有代码的向后兼容性。
pub mod error_utils {
    use super::*;

    /// 将任意错误转换为 ForgeError
    pub fn map_error<T, E: std::error::Error + Send + Sync + 'static>(
        result: Result<T, E>,
        context: &str,
    ) -> ForgeResult<T> {
        result.map_err(|e| ForgeError::Other(anyhow::anyhow!("{}: {}", context, e)))
    }

    /// 创建状态错误
    pub fn state_error(msg: impl Into<String>) -> ForgeError {
        ForgeError::State {
            message: msg.into(),
            source: None,
        }
    }

    /// 创建带源错误的状态错误
    pub fn state_error_with_source(
        msg: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> ForgeError {
        ForgeError::State {
            message: msg.into(),
            source: Some(Box::new(source)),
        }
    }

    /// 创建事件错误
    pub fn event_error(msg: impl Into<String>) -> ForgeError {
        ForgeError::Event {
            message: msg.into(),
            source: None,
        }
    }

    /// 创建带源错误的事件错误
    pub fn event_error_with_source(
        msg: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> ForgeError {
        ForgeError::Event {
            message: msg.into(),
            source: Some(Box::new(source)),
        }
    }

    /// 创建中间件错误
    pub fn middleware_error(msg: impl Into<String>) -> ForgeError {
        ForgeError::Middleware {
            message: msg.into(),
            middleware_name: None,
            source: None,
        }
    }

    /// 创建带中间件名称的中间件错误
    pub fn middleware_error_with_name(
        msg: impl Into<String>,
        middleware_name: impl Into<String>,
    ) -> ForgeError {
        ForgeError::Middleware {
            message: msg.into(),
            middleware_name: Some(middleware_name.into()),
            source: None,
        }
    }

    /// 创建带源错误的中间件错误
    pub fn middleware_error_with_source(
        msg: impl Into<String>,
        middleware_name: Option<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> ForgeError {
        ForgeError::Middleware {
            message: msg.into(),
            middleware_name,
            source: Some(Box::new(source)),
        }
    }

    /// 创建扩展错误
    pub fn extension_error(msg: impl Into<String>) -> ForgeError {
        ForgeError::Extension {
            message: msg.into(),
            extension_name: None,
            source: None,
        }
    }

    /// 创建带扩展名称的扩展错误
    pub fn extension_error_with_name(
        msg: impl Into<String>,
        extension_name: impl Into<String>,
    ) -> ForgeError {
        ForgeError::Extension {
            message: msg.into(),
            extension_name: Some(extension_name.into()),
            source: None,
        }
    }

    /// 创建插件错误（扩展错误的别名，保持向后兼容）
    pub fn plugin_error(msg: impl Into<String>) -> ForgeError {
        extension_error(msg)
    }

    /// 创建事务错误
    pub fn transaction_error(msg: impl Into<String>) -> ForgeError {
        ForgeError::Transaction {
            message: msg.into(),
            transaction_id: None,
            source: None,
        }
    }

    /// 创建带事务ID的事务错误
    pub fn transaction_error_with_id(
        msg: impl Into<String>,
        transaction_id: u64,
    ) -> ForgeError {
        ForgeError::Transaction {
            message: msg.into(),
            transaction_id: Some(transaction_id),
            source: None,
        }
    }

    /// 创建历史记录错误
    pub fn history_error(msg: impl Into<String>) -> ForgeError {
        ForgeError::History {
            message: msg.into(),
            source: None,
        }
    }

    /// 创建配置错误
    pub fn config_error(msg: impl Into<String>) -> ForgeError {
        ForgeError::Config {
            message: msg.into(),
            config_key: None,
            source: None,
        }
    }

    /// 创建带配置键的配置错误
    pub fn config_error_with_key(
        msg: impl Into<String>,
        config_key: impl Into<String>,
    ) -> ForgeError {
        ForgeError::Config {
            message: msg.into(),
            config_key: Some(config_key.into()),
            source: None,
        }
    }

    /// 创建存储错误
    pub fn storage_error(msg: impl Into<String>) -> ForgeError {
        ForgeError::Storage {
            message: msg.into(),
            source: None,
        }
    }

    /// 创建缓存错误
    pub fn cache_error(msg: impl Into<String>) -> ForgeError {
        ForgeError::Cache {
            message: msg.into(),
            source: None,
        }
    }

    /// 创建引擎错误
    pub fn engine_error(msg: impl Into<String>) -> ForgeError {
        ForgeError::Engine {
            message: msg.into(),
            source: None,
        }
    }

    /// 创建超时错误
    pub fn timeout_error(operation: impl Into<String>) -> ForgeError {
        ForgeError::Timeout {
            operation: operation.into(),
            timeout_ms: 0, // 默认值，可以根据需要调整
        }
    }

    /// 创建带超时时间的超时错误
    pub fn timeout_error_with_duration(operation: impl Into<String>, timeout_ms: u64) -> ForgeError {
        ForgeError::Timeout {
            operation: operation.into(),
            timeout_ms,
        }
    }

    /// 创建运行时错误
    pub fn runtime_error(msg: impl Into<String>) -> ForgeError {
        ForgeError::Engine {
            message: msg.into(),
            source: None,
        }
    }

    /// 创建资源不足错误
    pub fn resource_exhausted_error(resource_type: impl Into<String>) -> ForgeError {
        ForgeError::ResourceExhausted {
            resource_type: resource_type.into(),
            current_usage: None,
            limit: None,
        }
    }

    /// 创建带使用量信息的资源不足错误
    pub fn resource_exhausted_error_with_usage(
        resource_type: impl Into<String>,
        current_usage: usize,
        limit: usize,
    ) -> ForgeError {
        ForgeError::ResourceExhausted {
            resource_type: resource_type.into(),
            current_usage: Some(current_usage),
            limit: Some(limit),
        }
    }

    /// 创建并发错误
    pub fn concurrency_error(msg: impl Into<String>) -> ForgeError {
        ForgeError::Concurrency {
            message: msg.into(),
            source: None,
        }
    }

    /// 创建验证错误
    pub fn validation_error(msg: impl Into<String>) -> ForgeError {
        ForgeError::Validation {
            message: msg.into(),
            field: None,
        }
    }

    /// 创建带字段信息的验证错误
    pub fn validation_error_with_field(
        msg: impl Into<String>,
        field: impl Into<String>,
    ) -> ForgeError {
        ForgeError::Validation {
            message: msg.into(),
            field: Some(field.into()),
        }
    }

    /// 创建外部依赖错误
    pub fn external_dependency_error(
        dependency: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> ForgeError {
        ForgeError::ExternalDependency {
            dependency: dependency.into(),
            source: Box::new(source),
        }
    }

    /// 创建内部错误
    pub fn internal_error(msg: impl Into<String>) -> ForgeError {
        ForgeError::Internal {
            message: msg.into(),
            location: None,
        }
    }

    /// 创建带位置信息的内部错误
    pub fn internal_error_with_location(
        msg: impl Into<String>,
        location: impl Into<String>,
    ) -> ForgeError {
        ForgeError::Internal {
            message: msg.into(),
            location: Some(location.into()),
        }
    }
}

// 错误转换实现
impl From<crate::config::ConfigValidationError> for ForgeError {
    fn from(err: crate::config::ConfigValidationError) -> Self {
        ForgeError::Validation {
            field: Some("config".to_string()),
            message: err.to_string(),
        }
    }
}
