use std::ops::Sub;

use crate::{
    error::{PoolResult},
    mark::Mark,
    types::NodeId,
};

use super::{MarkRef, NodeRef};

/// 为 NodeRef 实现自定义的 - 运算符，用于删除单个节点
/// 当使用 - 运算符时，会从当前节点的子节点列表中移除指定的节点
impl<'a> Sub<NodeId> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn sub(
        self,
        node_id: NodeId,
    ) -> Self::Output {
        self.tree.remove_node(&self.key.clone().into(), vec![node_id])?;
        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 NodeRef 实现自定义的 - 运算符，用于删除多个节点
/// 当使用 - 运算符时，会从当前节点的子节点列表中移除指定的多个节点
impl<'a> Sub<Vec<NodeId>> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn sub(
        self,
        node_ids: Vec<NodeId>,
    ) -> Self::Output {
        self.tree.remove_node(&self.key.clone().into(), node_ids)?;
        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 MarkRef 实现自定义的 - 运算符，用于删除单个标记
/// 当使用 - 运算符时，会从当前标记列表中移除指定的标记
impl<'a> Sub<Mark> for MarkRef<'a> {
    type Output = PoolResult<MarkRef<'a>>;
    fn sub(
        self,
        mark: Mark,
    ) -> Self::Output {
        self.tree.remove_mark(&self.key.clone().into(), mark)?;
        Ok(MarkRef::new(self.tree, self.key.clone()))
    }
}

/// 为 MarkRef 实现自定义的 - 运算符，用于删除多个标记
/// 当使用 - 运算符时，会从当前标记列表中移除指定的多个标记
/// 注意：这里使用循环逐个删除标记，而不是一次性删除所有标记
impl<'a> Sub<Vec<Mark>> for MarkRef<'a> {
    type Output = PoolResult<MarkRef<'a>>;
    fn sub(
        self,
        marks: Vec<Mark>,
    ) -> Self::Output {
        for mark in marks {
            self.tree.remove_mark(&self.key.clone().into(), mark)?;
        }
        Ok(MarkRef::new(self.tree, self.key.clone()))
    }
}
