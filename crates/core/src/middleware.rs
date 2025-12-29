use std::sync::Arc;

use mf_model::{
    node_pool::NodePool,
    schema::Schema,
};

// ==================== 泛型中间件 Trait ====================

// Re-export from generic module
pub use crate::generic::middleware::{MiddlewareGeneric, MiddlewareStackGeneric};

// ==================== 向后兼容类型别名 ====================

/// 默认 Middleware trait（向后兼容）
///
/// 使用 NodePool 作为容器，Schema 作为模式定义
pub type Middleware = dyn MiddlewareGeneric<NodePool, Schema>;

/// 用于事务处理的中间件类型别名
pub type ArcMiddleware = Arc<Middleware>;

/// 默认 MiddlewareStack 类型（向后兼容）
///
/// 使用 NodePool 作为容器，Schema 作为模式定义
pub type MiddlewareStack = MiddlewareStackGeneric<NodePool, Schema>;
