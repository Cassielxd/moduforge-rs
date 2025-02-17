use std::{collections::HashMap, env::current_dir, path::PathBuf};

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
impl Default for Content {
    fn default() -> Self {
        Content::None
    }
}

#[derive(Clone, Debug)]
pub struct StorageOptions {
    pub storage_path: PathBuf,
    pub snapshot_path: PathBuf,
    pub delta_path: PathBuf,
    pub l2_path: PathBuf,
}
impl Default for StorageOptions {
    fn default() -> Self {
        let path = current_dir().unwrap().join("./data");
        Self {
            snapshot_path: path.join("snapshot"),
            delta_path: path.join("delta"),
            l2_path: path.join("db"),
            storage_path: path,
        }
    }
}
