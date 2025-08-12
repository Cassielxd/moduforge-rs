use crate::{tree::Tree, types::NodeId};

pub mod add;
pub mod bitand;
pub mod bitor;
pub mod mul;
pub mod shl;
pub mod shr;
pub mod sub;

// 用于处理节点赋值的包装器
pub struct NodeRef<'a> {
    tree: &'a mut Tree,
    key: NodeId,
}

impl<'a> NodeRef<'a> {
    pub fn new(
        tree: &'a mut Tree,
        key: NodeId,
    ) -> Self {
        Self { tree, key }
    }
}

impl<'a> std::ops::Deref for NodeRef<'a> {
    type Target = Tree;

    fn deref(&self) -> &Self::Target {
        self.tree
    }
}

impl<'a> std::ops::DerefMut for NodeRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tree
    }
}

// 用于处理节点赋值的包装器
pub struct MarkRef<'a> {
    tree: &'a mut Tree,
    key: NodeId,
}

impl<'a> MarkRef<'a> {
    pub fn new(
        tree: &'a mut Tree,
        key: NodeId,
    ) -> Self {
        Self { tree, key }
    }
}

impl<'a> std::ops::Deref for MarkRef<'a> {
    type Target = Tree;

    fn deref(&self) -> &Self::Target {
        self.tree
    }
}

impl<'a> std::ops::DerefMut for MarkRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tree
    }
}

pub struct AttrsRef<'a> {
    tree: &'a mut Tree,
    key: NodeId,
}

impl<'a> AttrsRef<'a> {
    pub fn new(
        tree: &'a mut Tree,
        key: NodeId,
    ) -> Self {
        Self { tree, key }
    }
}

impl<'a> std::ops::Deref for AttrsRef<'a> {
    type Target = Tree;

    fn deref(&self) -> &Self::Target {
        self.tree
    }
}

impl<'a> std::ops::DerefMut for AttrsRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tree
    }
}
