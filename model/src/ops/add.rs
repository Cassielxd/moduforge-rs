use std::ops::Add;

use serde_json::Value;

use crate::{attrs::Attrs, mark::Mark, node::Node};

use super::{AttrsRef, MarkRef, NodeRef};

/// 为 NodeRef 实现自定义的 + 运算符，用于添加单个节点
/// 当使用 + 运算符时，会将新节点添加到当前节点的子节点列表中
impl<'a> Add<Node> for NodeRef<'a> {
    type Output = ();
    fn add(self, node: Node) -> Self::Output {
        let _ = self.tree.add_node(&self.key.into(), &vec![node]);
    }
}

/// 为 NodeRef 实现自定义的 + 运算符，用于添加多个节点
/// 当使用 + 运算符时，会将多个新节点添加到当前节点的子节点列表中
impl<'a> Add<Vec<Node>> for NodeRef<'a> {
    type Output = ();
    fn add(self, nodes: Vec<Node>) -> Self::Output {
        let _ = self.tree.add_node(&self.key.into(), &nodes);
    }
}

/// 为 MarkRef 实现自定义的 + 运算符，用于添加单个标记
/// 当使用 + 运算符时，会将新标记添加到当前标记的列表中
impl<'a> Add<Mark> for MarkRef<'a> {
    type Output = ();
    fn add(self, mark: Mark) -> Self::Output {
        let _ = self.tree.add_mark(&self.key.into(), &vec![mark]);
    }
}

/// 为 MarkRef 实现自定义的 + 运算符，用于添加多个标记
/// 当使用 + 运算符时，会将多个新标记添加到当前标记的列表中
impl<'a> Add<Vec<Mark>> for MarkRef<'a> {
    type Output = ();
    fn add(self, marks: Vec<Mark>) -> Self::Output {
        let _ = self.tree.add_mark(&self.key.into(), &marks);
    }
}

/// 为 AttrsRef 实现自定义的 + 运算符，用于添加属性
/// 当使用 + 运算符时，会更新当前节点的属性
impl<'a> Add<Attrs> for AttrsRef<'a> {
    type Output = ();
    fn add(self, attrs: Attrs) -> Self::Output {
        let _ = self.tree.update_attr(&self.key.into(), attrs.attrs);
    }
}

/// 为 AttrsRef 实现自定义的 + 运算符，用于直接添加属性映射
/// 当使用 + 运算符时，会直接使用提供的属性映射更新当前节点的属性
impl<'a> Add<im::HashMap<String, Value>> for AttrsRef<'a> {
    type Output = ();
    fn add(self, attrs: im::HashMap<String, Value>) -> Self::Output {
        let _ = self.tree.update_attr(&self.key.into(), attrs);
    }
}
