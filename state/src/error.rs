/// A type alias for Result that uses StateError as the error type.
pub type StateResult<T> = Result<T, StateError>;

/// Represents all possible errors that can occur in state management operations.
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    /// Error occurs when plugin state initialization fails
    #[error("插件状态初始化失败: {0}")]
    PluginInitError(String),

    /// Error occurs when applying plugin state fails
    #[error("插件状态应用失败: {0}")]
    PluginApplyError(String),

    /// Error occurs when transaction application fails
    #[error("事务应用失败: {0}")]
    TransactionError(String),

    /// Error occurs when configuration is invalid
    #[error("配置错误: {0}")]
    ConfigurationError(String),

    /// Error occurs when field operations fail
    #[error("字段操作失败: {0}")]
    FieldError(String),

    /// Error occurs when schema is missing or invalid
    #[error("Schema错误: {0}")]
    SchemaError(String),

    /// Error occurs when plugin is not found
    #[error("插件未找到: {0}")]
    PluginNotFound(String),

    /// Error occurs when plugin state is invalid
    #[error("插件状态无效: {0}")]
    InvalidPluginState(String),
}
