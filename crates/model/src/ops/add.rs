use std::ops::Add;

use serde_json::Value;

use crate::{
    attrs::Attrs,
    error::{PoolResult},
    mark::Mark,
    node::Node,
    node_type::NodeEnum,
};

use super::{AttrsRef, MarkRef, NodeRef};

/// 为 NodeRef 实现自定义的 + 运算符，用于添加单个节点
/// 当使用 + 运算符时，会将新节点添加到当前节点的子节点列表中
impl<'a> Add<Node> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn add(
        self,
        node: Node,
    ) -> Self::Output {
        self.tree.add_node(&self.key.clone(), &vec![node])?;
        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}
/// 为 NodeRef 实现自定义的 + 运算符，用于在指定位置添加单个节点
/// 当使用 + 运算符时，会将新节点添加到当前节点的子节点列表中的指定位置
impl<'a> Add<(usize, Node)> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn add(
        self,
        (index, node): (usize, Node),
    ) -> Self::Output {
        self.tree.add_at_index(&self.key.clone(), index, &node)?;
        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 NodeRef 实现自定义的 + 运算符，用于添加多个节点
/// 当使用 + 运算符时，会将多个新节点添加到当前节点的子节点列表中
impl<'a> Add<Vec<Node>> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn add(
        self,
        nodes: Vec<Node>,
    ) -> Self::Output {
        self.tree.add_node(&self.key.clone(), &nodes)?;
        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}
impl<'a> Add<NodeEnum> for NodeRef<'a> {
    type Output = PoolResult<NodeRef<'a>>;
    fn add(
        self,
        nodes: NodeEnum,
    ) -> Self::Output {
        self.tree.add(&self.key.clone(), vec![nodes])?;
        Ok(NodeRef::new(self.tree, self.key.clone()))
    }
}

/// 为 MarkRef 实现自定义的 + 运算符，用于添加单个标记
/// 当使用 + 运算符时，会将新标记添加到当前标记的列表中
impl<'a> Add<Mark> for MarkRef<'a> {
    type Output = PoolResult<MarkRef<'a>>;
    fn add(
        self,
        mark: Mark,
    ) -> Self::Output {
        self.tree.add_mark(&self.key.clone(), &vec![mark])?;
        Ok(MarkRef::new(self.tree, self.key.clone()))
    }
}

/// 为 MarkRef 实现自定义的 + 运算符，用于添加多个标记
/// 当使用 + 运算符时，会将多个新标记添加到当前标记的列表中
impl<'a> Add<Vec<Mark>> for MarkRef<'a> {
    type Output = PoolResult<MarkRef<'a>>;
    fn add(
        self,
        marks: Vec<Mark>,
    ) -> Self::Output {
        self.tree.add_mark(&self.key.clone(), &marks)?;
        Ok(MarkRef::new(self.tree, self.key.clone()))
    }
}

/// 为 AttrsRef 实现自定义的 + 运算符，用于添加属性
/// 当使用 + 运算符时，会更新当前节点的属性
impl<'a> Add<Attrs> for AttrsRef<'a> {
    type Output = PoolResult<AttrsRef<'a>>;
    fn add(
        self,
        attrs: Attrs,
    ) -> Self::Output {
        self.tree.update_attr(&self.key.clone(), attrs.attrs)?;
        Ok(AttrsRef::new(self.tree, self.key.clone()))
    }
}
impl<'a> Add<(String, Value)> for AttrsRef<'a> {
    type Output = PoolResult<AttrsRef<'a>>;
    fn add(
        self,
        (key, value): (String, Value),
    ) -> Self::Output {
        self.tree
            .update_attr(&self.key.clone(), imbl::hashmap! {key=>value})?;
        Ok(AttrsRef::new(self.tree, self.key.clone()))
    }
}

/// 为 AttrsRef 实现自定义的 + 运算符，用于直接添加属性映射
/// 当使用 + 运算符时，会直接使用提供的属性映射更新当前节点的属性
impl<'a> Add<imbl::HashMap<String, Value>> for AttrsRef<'a> {
    type Output = PoolResult<AttrsRef<'a>>;
    fn add(
        self,
        attrs: imbl::HashMap<String, Value>,
    ) -> Self::Output {
        self.tree.update_attr(&self.key.clone(), attrs)?;
        Ok(AttrsRef::new(self.tree, self.key.clone()))
    }
}
