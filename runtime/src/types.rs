use std::collections::HashMap;

use moduforge_core::model::{
    node_pool::NodePool,
    schema::{Attribute, AttributeSpec},
};

use crate::{extension::Extension, mark::Mark, node::Node};

pub type GlobalAttributes = Vec<GlobalAttributeItem>;
#[derive(Clone, PartialEq, Debug, Eq, Default)]
pub struct GlobalAttributeItem {
    pub types: Vec<String>,
    pub attributes: HashMap<String, AttributeSpec>,
}
#[derive(Clone, Debug)]
pub enum Extensions {
    N(Node),
    M(Mark),
    E(Extension),
}

#[derive(Clone, Debug)]
pub enum Content {
    NodePoolBinary(Vec<u8>),
    NodePool(NodePool),
    Snapshot(Vec<u8>),
    None,
}
