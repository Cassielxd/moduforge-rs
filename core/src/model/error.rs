use super::types::NodeId;



#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    #[error("重复的节点 ID: {0}")]
    DuplicateNodeId(NodeId),
    #[error("父节点不存在: {0}")]
    ParentNotFound(NodeId),
    #[error("子节点不存在: {0}")]
    ChildNotFound(NodeId),
    #[error("节点不存在: {0}")]
    NodeNotFound(NodeId),
    #[error("Orphan node detected: {0}")]
    OrphanNode(NodeId),
    #[error("Invalid parenting: child {child} not found in parent {alleged_parent}'s children")]
    InvalidParenting {
        child: NodeId,
        alleged_parent: NodeId,
    },
}
