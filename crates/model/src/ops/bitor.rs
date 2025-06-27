use std::ops::BitOr;

use crate::{
    error::PoolResult, id_generator::IdGenerator, mark::Mark, node::Node,
    types::NodeId,
};

use super::{MarkRef, NodeRef};

/// 为 NodeRef 实现自定义的 | 运算符，用于合并另一个节点的所有子节点
/// 当使用 | 运算符时，会将另一个节点的所有子节点复制到当前节点中
impl<'a> BitOr<NodeId> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn bitor(
        self,
        other_node_id: NodeId,
    ) -> Self::Output {
        // 获取另一个节点的所有子节点
        let other_children =
            self.tree.children(&other_node_id).unwrap_or_default();
        let mut nodes_to_add = Vec::new();

        for child_id in other_children {
            if let Some(child_node) = self.tree.get_node(&child_id) {
                let mut node = child_node.0.as_ref().clone();
                node.id = IdGenerator::get_id();
                nodes_to_add.push(node);
            }
        }

        if !nodes_to_add.is_empty() {
            self.tree.add_node(&self.key.clone().into(), &nodes_to_add)?;
        }
        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 NodeRef 实现自定义的 | 运算符，用于合并多个节点的子节点
/// 当使用 | 运算符时，会将多个节点的所有子节点复制到当前节点中
impl<'a> BitOr<Vec<NodeId>> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn bitor(
        self,
        other_node_ids: Vec<NodeId>,
    ) -> Self::Output {
        let mut all_nodes_to_add = Vec::new();

        for node_id in other_node_ids {
            let children = self.tree.children(&node_id).unwrap_or_default();
            for child_id in children {
                if let Some(child_node) = self.tree.get_node(&child_id) {
                    let mut node = child_node.0.as_ref().clone();
                    node.id = IdGenerator::get_id();
                    all_nodes_to_add.push(node);
                }
            }
        }

        if !all_nodes_to_add.is_empty() {
            self.tree.add_node(&self.key.clone().into(), &all_nodes_to_add)?;
        }

        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 NodeRef 实现自定义的 | 运算符，用于直接合并节点列表
/// 当使用 | 运算符时，会将提供的节点列表合并到当前节点中
impl<'a> BitOr<Vec<Node>> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn bitor(
        self,
        nodes: Vec<Node>,
    ) -> Self::Output {
        if !nodes.is_empty() {
            self.tree.add_node(&self.key.clone().into(), &nodes)?;
        }
        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 MarkRef 实现自定义的 | 运算符，用于合并标记（去重）
/// 当使用 | 运算符时，会将新标记添加到当前标记列表中，如果标记已存在则不重复添加
impl<'a> BitOr<Mark> for MarkRef<'a> {
    type Output = PoolResult<MarkRef<'a>>;
    fn bitor(
        self,
        mark: Mark,
    ) -> Self::Output {
        // 检查标记是否已存在
        let existing_marks =
            self.tree.get_marks(&self.key.clone().into()).unwrap_or_default();
        let mark_exists = existing_marks.iter().any(|existing_mark| {
            existing_mark.r#type == mark.r#type
                && existing_mark.attrs == mark.attrs
        });

        if !mark_exists {
            self.tree.add_mark(&self.key.clone().into(), &vec![mark])?;
        }

        Ok(MarkRef::new(self.tree, self.key.clone()))
    }
}

/// 为 MarkRef 实现自定义的 | 运算符，用于合并多个标记（去重）
/// 当使用 | 运算符时，会将多个新标记添加到当前标记列表中，自动去重
impl<'a> BitOr<Vec<Mark>> for MarkRef<'a> {
    type Output = PoolResult<MarkRef<'a>>;
    fn bitor(
        self,
        marks: Vec<Mark>,
    ) -> Self::Output {
        let existing_marks =
            self.tree.get_marks(&self.key.clone().into()).unwrap_or_default();
        let mut unique_marks = Vec::new();

        for mark in marks {
            let mark_exists = existing_marks.iter().any(|existing_mark| {
                existing_mark.r#type == mark.r#type
                    && existing_mark.attrs == mark.attrs
            });

            if !mark_exists {
                unique_marks.push(mark);
            }
        }

        if !unique_marks.is_empty() {
            self.tree.add_mark(&self.key.clone().into(), &unique_marks)?;
        }

        Ok(MarkRef::new(self.tree, self.key.clone()))
    }
}
