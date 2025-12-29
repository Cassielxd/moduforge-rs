// Re-export from generic module
pub use crate::generic::extension::{
    ExtensionGeneric, OpFnGeneric, OpFnItemGeneric, NodeTransformFnGeneric,
};

// ==================== 向后兼容类型别名 ====================

/// 默认 Extension 类型（向后兼容）
pub type Extension = ExtensionGeneric<
    mf_model::node_pool::NodePool,
    mf_model::schema::Schema,
>;

/// 默认 OpFn 类型（向后兼容）
pub type OpFn = OpFnGeneric<
    mf_model::node_pool::NodePool,
    mf_model::schema::Schema,
>;

/// 默认 OpFnItem 类型（向后兼容）
pub type OpFnItem = OpFnItemGeneric<
    mf_model::node_pool::NodePool,
    mf_model::schema::Schema,
>;

/// 默认 NodeTransformFn 类型（向后兼容）
pub type NodeTransformFn = NodeTransformFnGeneric<
    mf_model::node_pool::NodePool,
    mf_model::schema::Schema,
>;
