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
        self.tree.remove_node(&self.key, vec![node_id])?;
        Ok(NodeRef::new(self.tree, self.key))
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
/// 为 NodeRef 实现自定义的 - 运算符，用于删除单个节点
/// 当使用 - 运算符时，会从当前节点的子节点列表中移除指定的节点
impl<'a> Sub<usize> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn sub(
        self,
        index: usize,
    ) -> Self::Output {
        self.tree.remove_node_by_index(&self.key.clone().into(), index)?;
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
        self.tree
            .remove_mark(&self.key.clone().into(), &[mark.r#type.clone()])?;
        Ok(MarkRef::new(self.tree, self.key.clone()))
    }
}

impl<'a> Sub<String> for MarkRef<'a> {
    type Output = PoolResult<MarkRef<'a>>;
    fn sub(
        self,
        mark_name: String,
    ) -> Self::Output {
        self.tree.remove_mark_by_name(&self.key.clone().into(), &mark_name)?;
        Ok(MarkRef::new(self.tree, self.key.clone()))
    }
}
impl<'a> Sub<Vec<String>> for MarkRef<'a> {
    type Output = PoolResult<MarkRef<'a>>;
    fn sub(
        self,
        mark_names: Vec<String>,
    ) -> Self::Output {
        self.tree.remove_mark(&self.key.clone().into(), &mark_names)?;
        Ok(MarkRef::new(self.tree, self.key.clone()))
    }
}
