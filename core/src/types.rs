use std::{collections::HashMap, default, sync::Arc};
use async_trait::async_trait;

use crate::{event::EventHandler, extension::Extension, mark::Mark, node::Node};
use moduforge_model::{
    node_pool::NodePool,
    schema::{AttributeSpec, Schema},
};

#[async_trait]
pub trait NodePoolFnTrait: Send + Sync + std::fmt::Debug {
    async fn create(
        &self,
        schema: &Schema,
    ) -> NodePool;
}

pub type GlobalAttributes = Vec<GlobalAttributeItem>;
#[derive(Clone, PartialEq, Debug, Eq, Default)]
pub struct GlobalAttributeItem {
    pub types: Vec<String>,
    pub attributes: HashMap<String, AttributeSpec>,
}

unsafe impl Send for GlobalAttributeItem {}
unsafe impl Sync for GlobalAttributeItem {}

#[derive(Clone)]
pub enum Extensions {
    N(Node),
    M(Mark),
    E(Extension),
}

#[derive(Clone, Default)]
pub enum Content {
    NodePool(NodePool),
    NodePoolFn(Arc<dyn NodePoolFnTrait>),
    #[default]
    None,
}

#[derive(Clone, Default)]
pub struct EditorOptions {
    content: Content,
    extensions: Vec<Extensions>,
    history_limit: Option<usize>,
    event_handlers: Vec<Arc<dyn EventHandler>>,
}
impl EditorOptions {
    pub fn get_content(&self) -> Content {
        self.content.clone()
    }
    pub fn set_content(
        mut self,
        content: Content,
    ) -> Self {
        self.content = content;
        self
    }
    pub fn get_extensions(&self) -> Vec<Extensions> {
        self.extensions.clone()
    }
    pub fn set_extensions(
        mut self,
        extensions: Vec<Extensions>,
    ) -> Self {
        self.extensions = extensions;
        self
    }
    pub fn get_history_limit(&self) -> Option<usize> {
        self.history_limit
    }
    pub fn set_history_limit(
        mut self,
        history_limit: usize,
    ) -> Self {
        self.history_limit = Some(history_limit);
        self
    }

    pub fn get_event_handlers(&self) -> Vec<Arc<dyn EventHandler>> {
        self.event_handlers.clone()
    }
    pub fn set_event_handlers(
        mut self,
        event_handlers: Vec<Arc<dyn EventHandler>>,
    ) -> Self {
        self.event_handlers = event_handlers;
        self
    }
}
