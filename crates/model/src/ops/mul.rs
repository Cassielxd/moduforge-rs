use std::ops::Mul;

use crate::{error::PoolResult, id_generator::IdGenerator, types::NodeId};

use super::NodeRef;

/// 为 NodeRef 实现自定义的 * 运算符，用于复制当前节点N次
/// 当使用 * 运算符时，会将当前节点复制指定次数并添加到父节点中
impl<'a> Mul<usize> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn mul(
        self,
        count: usize,
    ) -> Self::Output {
        // 获取当前节点
        if let Some(current_node) = self.tree.get_node(&self.key.clone().into())
        {
            let mut nodes = Vec::new();
            for _ in 0..count {
                // 创建节点的副本
                let mut node = current_node.as_ref().clone();
                node.id = IdGenerator::get_id();
                node.content = imbl::Vector::new();
                nodes.push(node);
            }
            // 添加到当前节点的父节点中
            if let Some(parent) =
                self.tree.get_parent_node(&self.key.clone().into())
            {
                self.tree.add_node(&parent.id, &nodes)?;
            }
        }
        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 NodeRef 实现自定义的 * 运算符，用于复制指定节点到当前位置
/// 当使用 * 运算符时，会将指定节点复制到当前节点的子节点列表中
impl<'a> Mul<NodeId> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn mul(
        self,
        node_id: NodeId,
    ) -> Self::Output {
        // 获取要复制的节点
        if let Some(source_node) = self.tree.get_node(&node_id) {
            let mut node = source_node.as_ref().clone();
            node.id = IdGenerator::get_id();
            node.content = imbl::Vector::new();
            // 添加到当前节点
            self.tree.add_node(&self.key.clone().into(), &vec![node])?;
        }
        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 NodeRef 实现自定义的 * 运算符，用于批量复制多个指定节点
/// 当使用 * 运算符时，会将指定的多个节点复制到当前节点的子节点列表中
impl<'a> Mul<Vec<NodeId>> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn mul(
        self,
        node_ids: Vec<NodeId>,
    ) -> Self::Output {
        let mut cloned_nodes = Vec::new();

        for node_id in node_ids {
            if let Some(source_node) = self.tree.get_node(&node_id) {
                let mut node = source_node.as_ref().clone();
                node.id = IdGenerator::get_id();
                node.content = imbl::Vector::new();
                cloned_nodes.push(node);
            }
        }

        if !cloned_nodes.is_empty() {
            self.tree.add_node(&self.key.clone().into(), &cloned_nodes)?;
        }

        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}
