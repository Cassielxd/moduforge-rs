use std::ops::Shr;

use crate::{error::PoolResult, mark::Mark, node::Node};

use super::{MarkRef, NodeRef};

/// 为 NodeRef 实现自定义的 >> 运算符，用于在子节点列表末尾添加单个节点
/// 当使用 >> 运算符时，会将新节点添加到当前节点的子节点列表的末尾位置
impl<'a> Shr<Node> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn shr(
        self,
        node: Node,
    ) -> Self::Output {
        // 添加到末尾（等同于标准的add操作）
        self.tree.add_node(&self.key.clone().into(), &vec![node])?;
        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 NodeRef 实现自定义的 >> 运算符，用于在子节点列表末尾添加多个节点
/// 当使用 >> 运算符时，会将多个新节点添加到当前节点的子节点列表的末尾位置
impl<'a> Shr<Vec<Node>> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn shr(
        self,
        nodes: Vec<Node>,
    ) -> Self::Output {
        if !nodes.is_empty() {
            self.tree.add_node(&self.key.clone().into(), &nodes)?;
        }
        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 NodeRef 实现自定义的 >> 运算符，用于将当前节点向右移动指定位置
/// 当使用 >> 运算符时，会将当前节点在其父节点的子节点列表中向右移动指定位置数
impl<'a> Shr<usize> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn shr(
        self,
        positions: usize,
    ) -> Self::Output {
        // 获取当前节点在父节点中的位置
        if let Some(parent) =
            self.tree.get_parent_node(&self.key.clone().into())
        {
            let siblings: imbl::Vector<String> =
                self.tree.children(&parent.id).unwrap_or_default();

            if let Some(current_index) =
                siblings.iter().position(|id| id.clone() == self.key)
            {
                // 计算新位置，不能超过列表长度
                let max_index = siblings.len().saturating_sub(1);
                let new_index = (current_index + positions).min(max_index);

                // 如果位置有变化，执行移动
                if new_index != current_index {
                    //这里只需要修改  content 中的顺序就行，不需要删除和添加
                    let mut node = self
                        .tree
                        .get_node(&self.key.clone().into())
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

/// 为 MarkRef 实现自定义的 >> 运算符，用于在标记列表末尾添加单个标记
/// 当使用 >> 运算符时，会将新标记添加到当前标记列表的末尾位置
impl<'a> Shr<Mark> for MarkRef<'a> {
    type Output = PoolResult<MarkRef<'a>>;
    fn shr(
        self,
        mark: Mark,
    ) -> Self::Output {
        // 添加到末尾（等同于标准的add操作）
        self.tree.add_mark(&self.key.clone().into(), &vec![mark])?;
        Ok(MarkRef::new(self.tree, self.key.clone()))
    }
}

/// 为 MarkRef 实现自定义的 >> 运算符，用于在标记列表末尾添加多个标记
/// 当使用 >> 运算符时，会将多个新标记添加到当前标记列表的末尾位置
impl<'a> Shr<Vec<Mark>> for MarkRef<'a> {
    type Output = PoolResult<MarkRef<'a>>;
    fn shr(
        self,
        marks: Vec<Mark>,
    ) -> Self::Output {
        if !marks.is_empty() {
            self.tree.add_mark(&self.key.clone().into(), &marks)?;
        }
        Ok(MarkRef::new(self.tree, self.key.clone()))
    }
}
