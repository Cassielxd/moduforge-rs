use std::ops::Shl;

use crate::{error::PoolResult, node::Node};

use super::{NodeRef};

/// 为 NodeRef 实现自定义的 << 运算符，用于在子节点列表开头插入单个节点
/// 当使用 << 运算符时，会将新节点插入到当前节点的子节点列表的开头位置
impl<'a> Shl<Node> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn shl(
        self,
        node: Node,
    ) -> Self::Output {
        // 在索引0处插入节点（开头位置）
        self.tree.add_at_index(&self.key.clone(), 0, &node)?;
        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 NodeRef 实现自定义的 << 运算符，用于在子节点列表开头插入多个节点
/// 当使用 << 运算符时，会将多个新节点依次插入到当前节点的子节点列表的开头位置
impl<'a> Shl<Vec<Node>> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn shl(
        self,
        nodes: Vec<Node>,
    ) -> Self::Output {
        // 反向插入，确保节点顺序正确
        for (i, node) in nodes.into_iter().enumerate() {
            self.tree.add_at_index(&self.key.clone(), i, &node)?;
        }
        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 NodeRef 实现自定义的 << 运算符，用于在指定数量的位置处插入节点
/// 当使用 << 运算符时，会将当前节点向左移动指定位置数
impl<'a> Shl<usize> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn shl(
        self,
        positions: usize,
    ) -> Self::Output {
        // 获取当前节点在父节点中的位置
        if let Some(parent) = self.tree.get_parent_node(&self.key.clone())
        {
            let siblings = self.tree.children(&parent.id).unwrap_or_default();

            if let Some(current_index) =
                siblings.iter().position(|id| id.clone() == self.key)
            {
                // 计算新位置，不能小于0
                let new_index = current_index.saturating_sub(positions);

                // 如果位置有变化，执行移动
                if new_index != current_index {
                    //这里只需要修改  content 中的顺序就行，不需要删除和添加
                    let mut node = self
                        .tree
                        .get_node(&self.key.clone())
                        .unwrap()
                        .as_ref()
                        .clone();
                    let mut content = node.content.clone();
                    content.swap(current_index, new_index);
                    node.content = content;
                    self.tree.update_node(node)?;
                }
            }
        }

        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}
