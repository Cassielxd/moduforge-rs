use crate::types::NodeId;
use anyhow::{bail, Context, Result};

/// Error messages for node pool operations
pub mod error_messages {
    pub const DUPLICATE_NODE: &str = "重复的节点 ID";
    pub const PARENT_NOT_FOUND: &str = "父节点不存在";
    pub const CHILD_NOT_FOUND: &str = "子节点不存在";
    pub const NODE_NOT_FOUND: &str = "节点不存在";
    pub const ORPHAN_NODE: &str = "检测到孤立节点";
    pub const INVALID_PARENTING: &str = "无效的父子关系";
    pub const INVALID_NODE_ID: &str = "节点ID不匹配";
    pub const EMPTY_POOL: &str = "节点池为空";
    pub const CYCLIC_REFERENCE: &str = "检测到循环引用";
    pub const INVALID_NODE_MOVE: &str = "无效的节点移动";
    pub const NODE_LOCKED: &str = "节点已被锁定，无法执行操作";
    pub const NODE_DELETED: &str = "节点已被删除";
    pub const CANNOT_REMOVE_ROOT: &str = "无法删除根节点";
}

/// Helper functions for creating node pool errors
pub mod error_helpers {
    use super::*;
    use anyhow::bail;

    pub fn duplicate_node(id: NodeId) -> anyhow::Error {
        anyhow::anyhow!("{}: {}", error_messages::DUPLICATE_NODE, id)
    }

    pub fn parent_not_found(id: NodeId) -> anyhow::Error {
        anyhow::anyhow!("{}: {}", error_messages::PARENT_NOT_FOUND, id)
    }

    pub fn child_not_found(id: NodeId) -> anyhow::Error {
        anyhow::anyhow!("{}: {}", error_messages::CHILD_NOT_FOUND, id)
    }

    pub fn node_not_found(id: NodeId) -> anyhow::Error {
        anyhow::anyhow!("{}: {}", error_messages::NODE_NOT_FOUND, id)
    }

    pub fn orphan_node(id: NodeId) -> anyhow::Error {
        anyhow::anyhow!("{}: {}", error_messages::ORPHAN_NODE, id)
    }

    pub fn invalid_parenting(
        child: NodeId,
        alleged_parent: NodeId,
    ) -> anyhow::Error {
        anyhow::anyhow!(
            "{}: 子节点 {} 不是父节点 {} 的子节点",
            error_messages::INVALID_PARENTING,
            child,
            alleged_parent
        )
    }

    pub fn invalid_node_id(
        nodeid: NodeId,
        new_node_id: NodeId,
    ) -> anyhow::Error {
        anyhow::anyhow!(
            "{}: 新节点ID({})与要替换的节点ID({})不一致",
            error_messages::INVALID_NODE_ID,
            nodeid,
            new_node_id
        )
    }

    pub fn empty_pool() -> anyhow::Error {
        anyhow::anyhow!(error_messages::EMPTY_POOL)
    }

    pub fn cyclic_reference(id: NodeId) -> anyhow::Error {
        anyhow::anyhow!(
            "{}: 节点 {} 不能成为自己的祖先",
            error_messages::CYCLIC_REFERENCE,
            id
        )
    }

    pub fn invalid_node_move(id: NodeId) -> anyhow::Error {
        anyhow::anyhow!(
            "{}: 无法将节点 {} 移动到目标位置",
            error_messages::INVALID_NODE_MOVE,
            id
        )
    }

    pub fn node_locked(id: NodeId) -> anyhow::Error {
        anyhow::anyhow!("{}: {}", error_messages::NODE_LOCKED, id)
    }

    pub fn node_deleted(id: NodeId) -> anyhow::Error {
        anyhow::anyhow!("{}: {}", error_messages::NODE_DELETED, id)
    }

    pub fn cannot_remove_root() -> anyhow::Error {
        anyhow::anyhow!(error_messages::CANNOT_REMOVE_ROOT)
    }
}

/// A type alias for Result that uses anyhow::Error as the error type.
pub type PoolResult<T> = Result<T>;
