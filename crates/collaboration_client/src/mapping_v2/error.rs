use thiserror::Error;
use std::any::TypeId;

/// 转换错误类型 - 提供详细的错误信息和上下文
#[derive(Error, Debug, Clone)]
pub enum ConversionError {
    #[error("不支持的步骤类型: {step_type} (TypeId: {type_id:?})")]
    UnsupportedStepType { step_type: String, type_id: TypeId },

    #[error("步骤验证失败: {reason}, 步骤: {step_name}")]
    ValidationFailed { step_name: String, reason: String },

    #[error("Yrs 事务操作失败: {operation}, 原因: {reason}")]
    YrsTransactionFailed { operation: String, reason: String },

    #[error("节点操作失败: {node_id}, 操作: {operation}, 原因: {reason}")]
    NodeOperationFailed { node_id: String, operation: String, reason: String },

    #[error("属性操作失败: 节点 {node_id}, 属性 {attr_key}, 原因: {reason}")]
    AttributeOperationFailed {
        node_id: String,
        attr_key: String,
        reason: String,
    },

    #[error(
        "标记操作失败: 节点 {node_id}, 标记类型 {mark_type}, 原因: {reason}"
    )]
    MarkOperationFailed { node_id: String, mark_type: String, reason: String },

    #[error("序列化失败: {reason}")]
    SerializationFailed { reason: String },

    #[error(
        "权限不足: 用户 {user_id} 无法执行操作 {operation} 在节点 {node_id}"
    )]
    PermissionDenied { user_id: String, operation: String, node_id: String },

    #[error(
        "并发冲突: 节点 {node_id} 在客户端 {local_client} 和 {remote_client} 之间存在冲突"
    )]
    ConcurrencyConflict {
        node_id: String,
        local_client: String,
        remote_client: String,
    },

    #[error("自定义错误: {message}")]
    Custom { message: String },
}

impl ConversionError {
    /// 创建不支持步骤类型错误
    pub fn unsupported_step<T: 'static>(step_name: &str) -> Self {
        Self::UnsupportedStepType {
            step_type: step_name.to_string(),
            type_id: TypeId::of::<T>(),
        }
    }

    /// 创建验证失败错误
    pub fn validation_failed(
        step_name: &str,
        reason: &str,
    ) -> Self {
        Self::ValidationFailed {
            step_name: step_name.to_string(),
            reason: reason.to_string(),
        }
    }

    /// 创建节点操作失败错误
    pub fn node_operation_failed(
        node_id: &str,
        operation: &str,
        reason: &str,
    ) -> Self {
        Self::NodeOperationFailed {
            node_id: node_id.to_string(),
            operation: operation.to_string(),
            reason: reason.to_string(),
        }
    }

    /// 创建权限不足错误
    pub fn permission_denied(
        user_id: &str,
        operation: &str,
        node_id: &str,
    ) -> Self {
        Self::PermissionDenied {
            user_id: user_id.to_string(),
            operation: operation.to_string(),
            node_id: node_id.to_string(),
        }
    }
}

/// 转换结果类型别名
pub type ConversionResult<T> = Result<T, ConversionError>;

/// 可恢复的转换错误 - 支持重试机制
#[derive(Error, Debug, Clone)]
pub enum RecoverableError {
    #[error("临时网络错误: {reason}")]
    TemporaryNetworkError { reason: String },

    #[error("资源暂时不可用: {resource}")]
    ResourceTemporarilyUnavailable { resource: String },

    #[error("事务冲突，可重试: {reason}")]
    RetryableTransactionConflict { reason: String },
}

impl From<RecoverableError> for ConversionError {
    fn from(err: RecoverableError) -> Self {
        Self::Custom { message: err.to_string() }
    }
}
