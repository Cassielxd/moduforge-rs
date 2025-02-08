use std::collections::HashMap;

use moduforge_core::model::schema::{Attribute, AttributeSpec};

use crate::{extension::Extension, mark::Mark, node::Node};

pub type GlobalAttributes = Vec<GlobalAttributeItem>;
#[derive(Clone, PartialEq, Debug, Eq, Default)]
pub struct GlobalAttributeItem {
    pub types: Vec<String>,
    pub attributes: HashMap<String, AttributeSpec>,
}

pub enum Extensions {
    N(Node),
    M(Mark),
    E(Extension),
}
