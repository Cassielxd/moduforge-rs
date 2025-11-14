use std::ops::BitAnd;

use crate::{
    error::{error_helpers, PoolResult},
};

use super::{MarkRef, NodeRef};

/// 为 NodeRef 实现自定义的 & 运算符，用于过滤特定类型的子节点
/// 当使用 & 运算符时，会保留指定类型的子节点，移除其他类型的子节点
impl<'a> BitAnd<String> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn bitand(
        self,
        node_type: String,
    ) -> Self::Output {
        let children =
            self.tree.children(&self.key.clone()).unwrap_or_default();
        let mut nodes_to_remove = Vec::new();

        for child_id in children.iter() {
            if let Some(node) = self.tree.get_node(&child_id) {
                if node.r#type != node_type {
                    nodes_to_remove.push(child_id.clone());
                }
            }
        }

        // 移除不匹配的节点
        if !nodes_to_remove.is_empty() {
            self.tree.remove_node(&self.key.clone(), nodes_to_remove)?;
        }

        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 NodeRef 实现自定义的 & 运算符，用于保留多个指定类型的子节点
/// 当使用 & 运算符时，会保留匹配任一指定类型的子节点，移除其他类型的子节点
impl<'a> BitAnd<Vec<String>> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn bitand(
        self,
        node_types: Vec<String>,
    ) -> Self::Output {
        let children =
            self.tree.children(&self.key.clone()).unwrap_or_default();
        let mut nodes_to_remove = Vec::new();

        for child_id in children.iter() {
            if let Some(node) = self.tree.get_node(&child_id) {
                let node_type_str = node.r#type.to_string();
                if !node_types.contains(&node_type_str) {
                    nodes_to_remove.push(child_id.clone());
                }
            }
        }

        // 移除不匹配的节点
        if !nodes_to_remove.is_empty() {
            self.tree.remove_node(&self.key.clone(), nodes_to_remove)?;
        }

        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 MarkRef 实现自定义的 & 运算符，用于保留指定名称的标记
/// 当使用 & 运算符时，会保留指定名称的标记，移除其他标记
impl<'a> BitAnd<String> for MarkRef<'a> {
    type Output = PoolResult<MarkRef<'a>>;
    fn bitand(
        self,
        mark_name: String,
    ) -> Self::Output {
        let noderef = self.tree.get_node(&self.key.clone());
        match noderef {
            Some(_) => {
                self.tree.remove_mark(&self.key.clone(), &[mark_name])?;
                Ok(MarkRef::new(self.tree, self.key.clone()))
            },
            None => Err(error_helpers::node_not_found(self.key.clone())),
        }
    }
}

/// 为 MarkRef 实现自定义的 & 运算符，用于保留多个指定名称的标记
/// 当使用 & 运算符时，会保留匹配任一指定名称的标记，移除其他标记
impl<'a> BitAnd<Vec<String>> for MarkRef<'a> {
    type Output = PoolResult<MarkRef<'a>>;
    fn bitand(
        self,
        mark_names: Vec<String>,
    ) -> Self::Output {
        let noderef = self.tree.get_node(&self.key.clone());
        match noderef {
            Some(_) => {
                self.tree.remove_mark(&self.key.clone(), &mark_names)?;
                Ok(MarkRef::new(self.tree, self.key.clone()))
            },
            None => Err(error_helpers::node_not_found(self.key.clone())),
        }
    }
}
