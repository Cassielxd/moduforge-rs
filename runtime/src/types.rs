use std::{collections::HashMap, env::current_dir, path::PathBuf, sync::Arc};

use crate::{event::EventHandler, extension::Extension, mark::Mark, node::Node};
use bincode::{Decode, Encode};
use moduforge_core::{
    model::{node_pool::NodePool, schema::AttributeSpec},
    transform::ConcreteStep,
};
use serde::{Deserialize, Serialize};
use zen_engine::{
    DecisionEngine, handler::custom_node_adapter::NoopCustomNode, loader::FilesystemLoader,
};
pub type Engine = Arc<DecisionEngine<FilesystemLoader, NoopCustomNode>>;
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

#[derive(Clone, Debug, Default)]
pub enum Content {
    NodePoolBinary(Vec<u8>),
    NodePool(NodePool),
    Snapshot(Vec<u8>),
    #[default]
    None,
}

#[derive(Clone, Debug)]
pub struct StorageOptions {
    pub storage_path: PathBuf,

    pub l2_path: PathBuf,
}
impl Default for StorageOptions {
    fn default() -> Self {
        let path = current_dir().unwrap().join("./data");
        Self {
            l2_path: path.join("db"),
            storage_path: path,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct EditorOptions {
    content: Content,
    extensions: Vec<Extensions>,
    history_limit: Option<usize>,
    snapshot_interval: Option<usize>,
    rules_path: Option<PathBuf>,
    event_handlers: Vec<Arc<dyn EventHandler>>,
    storage_option: Option<StorageOptions>,
}
impl EditorOptions {
    pub fn get_content(&self) -> Content {
        self.content.clone()
    }
    pub fn set_content(mut self, content: Content) -> Self {
        self.content = content;
        self
    }
    pub fn get_extensions(&self) -> Vec<Extensions> {
        self.extensions.clone()
    }
    pub fn set_extensions(mut self, extensions: Vec<Extensions>) -> Self {
        self.extensions = extensions;
        self
    }
    pub fn get_history_limit(&self) -> Option<usize> {
        self.history_limit
    }
    pub fn set_history_limit(mut self, history_limit: usize) -> Self {
        self.history_limit = Some(history_limit);
        self
    }
    pub fn get_snapshot_interval(&self) -> Option<usize> {
        self.snapshot_interval
    }
    pub fn set_snapshot_interval(mut self, snapshot_interval: usize) -> Self {
        self.snapshot_interval = Some(snapshot_interval);
        self
    }
    pub fn get_rules_path(&self) -> Option<PathBuf> {
        self.rules_path.clone()
    }
    pub fn set_rules_path(mut self, rules_path: PathBuf) -> Self {
        self.rules_path = Some(rules_path);
        self
    }
    pub fn get_event_handlers(&self) -> Vec<Arc<dyn EventHandler>> {
        self.event_handlers.clone()
    }
    pub fn set_event_handlers(mut self, event_handlers: Vec<Arc<dyn EventHandler>>) -> Self {
        self.event_handlers = event_handlers;
        self
    }
    pub fn get_storage_option(&self) -> Option<StorageOptions> {
        self.storage_option.clone()
    }
    pub fn set_storage_option(mut self, storage_option: StorageOptions) -> Self {
        self.storage_option = Some(storage_option);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Decode, Encode)]
pub enum StepExt {
    ConcreteStep(ConcreteStep),
}
