use std::ops::Sub;

use crate::{mark::Mark, types::NodeId};

use super::{MarkRef, NodeRef};

/// 为 NodeRef 实现自定义的 - 运算符，用于删除单个节点
/// 当使用 - 运算符时，会从当前节点的子节点列表中移除指定的节点
impl<'a> Sub<NodeId> for NodeRef<'a> {
    type Output = ();
    fn sub(self, node_id: NodeId) -> Self::Output {
        let _ = self.tree.remove_node(&self.key.into(), vec![node_id]);
    }
}

/// 为 NodeRef 实现自定义的 - 运算符，用于删除多个节点
/// 当使用 - 运算符时，会从当前节点的子节点列表中移除指定的多个节点
impl<'a> Sub<Vec<NodeId>> for NodeRef<'a> {
    type Output = ();
    fn sub(self, node_ids: Vec<NodeId>) -> Self::Output {
        let _ = self.tree.remove_node(&self.key.into(), node_ids);
    }
}

/// 为 MarkRef 实现自定义的 - 运算符，用于删除单个标记
/// 当使用 - 运算符时，会从当前标记列表中移除指定的标记
impl<'a> Sub<Mark> for MarkRef<'a> {
    type Output = ();
    fn sub(self, mark: Mark) -> Self::Output {
        let _ = self.tree.remove_mark(&self.key.into(), mark);
    }
}

/// 为 MarkRef 实现自定义的 - 运算符，用于删除多个标记
/// 当使用 - 运算符时，会从当前标记列表中移除指定的多个标记
/// 注意：这里使用循环逐个删除标记，而不是一次性删除所有标记
impl<'a> Sub<Vec<Mark>> for MarkRef<'a> {
    type Output = ();
    fn sub(self, marks: Vec<Mark>) -> Self::Output {
        for mark in marks {
            let _ = self.tree.remove_mark(&self.key, mark);
        }
    }
}
