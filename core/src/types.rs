use std::{collections::HashMap, sync::Arc, time::SystemTime};
use async_trait::async_trait;
use moduforge_state::{State, StateConfig};

use crate::{
    event::{Event, EventHandler},
    extension::Extension,
    mark::Mark,
    middleware::{Middleware, MiddlewareStack},
    node::Node,
    EditorResult,
};
use moduforge_model::{node_pool::NodePool, schema::AttributeSpec};

#[async_trait]
pub trait NodePoolFnTrait: Send + Sync + std::fmt::Debug {
    async fn create(
        &self,
        config: &StateConfig,
    ) -> EditorResult<NodePool>;
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
    event_handlers: Vec<Arc<dyn EventHandler<Event>>>,
    middleware_stack: MiddlewareStack,
}
impl EditorOptions {
    pub fn get_middleware_stack(&self) -> MiddlewareStack {
        self.middleware_stack.clone()
    }
    pub fn set_middleware_stack(
        mut self,
        middleware_stack: MiddlewareStack,
    ) -> Self {
        self.middleware_stack = middleware_stack;
        self
    }
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
    pub fn add_extension(
        mut self,
        extension: Extensions,
    ) -> Self {
        self.extensions.push(extension);
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

    pub fn get_event_handlers(&self) -> Vec<Arc<dyn EventHandler<Event>>> {
        self.event_handlers.clone()
    }
    pub fn set_event_handlers(
        mut self,
        event_handlers: Vec<Arc<dyn EventHandler<Event>>>,
    ) -> Self {
        self.event_handlers = event_handlers;
        self
    }
}

#[derive(Default)]
pub struct EditorOptionsBuilder {
    content: Content,
    extensions: Vec<Extensions>,
    history_limit: Option<usize>,
    event_handlers: Vec<Arc<dyn EventHandler<Event>>>,
    middleware_stack: MiddlewareStack,
}

impl EditorOptionsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content(
        mut self,
        content: Content,
    ) -> Self {
        self.content = content;
        self
    }
    pub fn extensions(
        mut self,
        extensions: Vec<Extensions>,
    ) -> Self {
        self.extensions = extensions;
        self
    }
    pub fn add_extension(
        mut self,
        extension: Extensions,
    ) -> Self {
        self.extensions.push(extension);
        self
    }

    pub fn history_limit(
        mut self,
        limit: usize,
    ) -> Self {
        self.history_limit = Some(limit);
        self
    }

    pub fn event_handlers(
        mut self,
        handlers: Vec<Arc<dyn EventHandler<Event>>>,
    ) -> Self {
        self.event_handlers = handlers;
        self
    }

    pub fn add_event_handler(
        mut self,
        handler: Arc<dyn EventHandler<Event>>,
    ) -> Self {
        self.event_handlers.push(handler);
        self
    }

    pub fn middleware_stack(
        mut self,
        stack: MiddlewareStack,
    ) -> Self {
        self.middleware_stack = stack;
        self
    }
    pub fn add_middleware<T: Middleware + 'static>(
        mut self,
        middleware: T,
    ) -> Self {
        self.middleware_stack.add(middleware);
        self
    }

    pub fn build(self) -> EditorOptions {
        EditorOptions {
            content: self.content,
            extensions: self.extensions,
            history_limit: self.history_limit,
            event_handlers: self.event_handlers,
            middleware_stack: self.middleware_stack,
        }
    }
}


/// 带元信息的历史记录项
#[derive(Debug, Clone)]
pub struct HistoryEntryWithMeta {
    /// 状态快照
    pub state: Arc<State>,
    
    /// 操作描述
    pub description: String,
    
    /// 时间戳
    pub timestamp: SystemTime,
    
    pub meta: serde_json::Value,
}


impl HistoryEntryWithMeta {
    pub fn new(
        state: Arc<State>,
        description: String,
        meta: serde_json::Value,
    ) -> Self {
        Self {
            state,
            description,
            timestamp: SystemTime::now(),
            meta,
        }
    }
}