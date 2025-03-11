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
  #[error("检测到孤立节点: {0}")]
  OrphanNode(NodeId),
  #[error("无效的:子节点 {child} 没在  {alleged_parent} 找到 's")]
  InvalidParenting { child: NodeId, alleged_parent: NodeId },
}
