//! 运行时统一接口定义
//!
//! 此模块定义了 RuntimeTrait,为三种运行时实现提供统一接口:
//! - ForgeRuntime (同步运行时)
//! - ForgeActorRuntime (Actor运行时)
//! - ForgeAsyncRuntime (异步运行时)
//!
//! 通过统一接口,用户可以:
//! 1. 轻松切换不同运行时实现
//! 2. 编写运行时无关的业务逻辑
//! 3. 使用trait对象实现运行时抽象

// Re-export from generic module
pub use crate::generic::runtime::RuntimeTraitGeneric;

// ==================== 向后兼容类型别名 ====================

/// 默认 RuntimeTrait 类型（向后兼容）
///
/// 使用 NodePool 作为容器，Schema 作为模式定义
pub type RuntimeTrait = dyn RuntimeTraitGeneric<
    mf_model::node_pool::NodePool,
    mf_model::schema::Schema,
>;
