use crate::types::NodeId;

/// Represents all possible errors that can occur in the node pool operations.
#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    /// Error occurs when attempting to add a node with an ID that already exists in the pool.
    #[error("重复的节点 ID: {0}")]
    DuplicateNodeId(NodeId),

    /// Error occurs when trying to access a parent node that doesn't exist in the pool.
    #[error("父节点不存在: {0}")]
    ParentNotFound(NodeId),

    /// Error occurs when trying to access a child node that doesn't exist in the pool.
    #[error("子节点不存在: {0}")]
    ChildNotFound(NodeId),

    /// Error occurs when trying to access a node that doesn't exist in the pool.
    #[error("节点不存在: {0}")]
    NodeNotFound(NodeId),

    /// Error occurs when a node exists in the pool but has no parent.
    #[error("检测到孤立节点: {0}")]
    OrphanNode(NodeId),

    /// Error occurs when attempting to establish an invalid parent-child relationship.
    #[error(
        "无效的父子关系: 子节点 {child} 不是父节点 {alleged_parent} 的子节点"
    )]
    InvalidParenting { child: NodeId, alleged_parent: NodeId },

    /// Error occurs when attempting to replace a node with a different ID.
    #[error(
        "节点ID不匹配: 新节点ID({nodeid})与要替换的节点ID({new_node_id})不一致"
    )]
    InvalidNodeId { nodeid: NodeId, new_node_id: NodeId },

    /// Error occurs when attempting to perform operations on an empty pool.
    #[error("节点池为空")]
    EmptyPool,

    /// Error occurs when attempting to create a cycle in the node hierarchy.
    #[error("检测到循环引用: 节点 {0} 不能成为自己的祖先")]
    CyclicReference(NodeId),

    /// Error occurs when attempting to move a node to an invalid position.
    #[error("无效的节点移动: 无法将节点 {0} 移动到目标位置")]
    InvalidNodeMove(NodeId),

    /// Error occurs when attempting to perform operations on a locked node.
    #[error("节点 {0} 已被锁定，无法执行操作")]
    NodeLocked(NodeId),

    /// Error occurs when attempting to perform operations on a deleted node.
    #[error("节点 {0} 已被删除")]
    NodeDeleted(NodeId),

    /// Error occurs when attempting to remove the root node.
    #[error("无法删除根节点")]
    CannotRemoveRoot,
}

impl PoolError {
    /// Returns the node ID associated with this error, if any.
    pub fn node_id(&self) -> Option<&NodeId> {
        match self {
            PoolError::DuplicateNodeId(id) => Some(id),
            PoolError::ParentNotFound(id) => Some(id),
            PoolError::ChildNotFound(id) => Some(id),
            PoolError::NodeNotFound(id) => Some(id),
            PoolError::OrphanNode(id) => Some(id),
            PoolError::InvalidParenting { child, .. } => Some(child),
            PoolError::InvalidNodeId { nodeid, .. } => Some(nodeid),
            PoolError::CyclicReference(id) => Some(id),
            PoolError::InvalidNodeMove(id) => Some(id),
            PoolError::NodeLocked(id) => Some(id),
            PoolError::NodeDeleted(id) => Some(id),
            PoolError::EmptyPool => None,
            PoolError::CannotRemoveRoot => None,
        }
    }
}

/// A type alias for Result that uses PoolError as the error type.
pub type PoolResult<T> = Result<T, PoolError>;
