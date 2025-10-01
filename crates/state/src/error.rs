use anyhow::Result;

/// A type alias for Result that uses anyhow::Error as the error type.
pub type StateResult<T> = Result<T>;

/// Helper functions for creating common error types
#[allow(clippy::module_inception)]
pub mod error {
    use anyhow::anyhow;

    /// Creates a plugin initialization error
    pub fn plugin_init_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow!("插件状态初始化失败: {}", msg.into())
    }

    /// Creates a plugin apply error
    pub fn plugin_apply_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow!("插件状态应用失败: {}", msg.into())
    }

    /// Creates a transaction error
    pub fn transaction_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow!("事务应用失败: {}", msg.into())
    }

    /// Creates a configuration error
    pub fn configuration_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow!("配置错误: {}", msg.into())
    }

    /// Creates a field operation error
    pub fn field_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow!("字段操作失败: {}", msg.into())
    }

    /// Creates a schema error
    pub fn schema_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow!("Schema错误: {}", msg.into())
    }

    /// Creates a plugin not found error
    pub fn plugin_not_found(msg: impl Into<String>) -> anyhow::Error {
        anyhow!("插件未找到: {}", msg.into())
    }

    /// Creates an invalid plugin state error
    pub fn invalid_plugin_state(msg: impl Into<String>) -> anyhow::Error {
        anyhow!("插件状态无效: {}", msg.into())
    }

    /// Creates a serialization error
    pub fn serialize_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow!("序列化失败: {}", msg.into())
    }

    /// Creates a deserialization error
    pub fn deserialize_error(msg: impl Into<String>) -> anyhow::Error {
        anyhow!("反序列化失败: {}", msg.into())
    }
}
