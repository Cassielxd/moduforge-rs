use std::{collections::HashMap, sync::Arc, time::SystemTime};
use async_trait::async_trait;
use mf_state::{state::TransactionResult, State, StateConfig, Transaction};

use crate::{
    event::{Event, EventHandler},
    extension::Extension,
    mark::Mark,
    middleware::{Middleware, MiddlewareStack},
    node::Node,
    ForgeResult,
};
use mf_model::{node_pool::NodePool, schema::AttributeSpec};

#[async_trait]
pub trait NodePoolFnTrait: Send + Sync + std::fmt::Debug {
    async fn create(
        &self,
        config: &StateConfig,
    ) -> ForgeResult<NodePool>;
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
pub struct RuntimeOptions {
    content: Content,
    extensions: Vec<Extensions>,
    history_limit: Option<usize>,
    event_handlers: Vec<Arc<dyn EventHandler<Event>>>,
    middleware_stack: MiddlewareStack,
}
impl RuntimeOptions {
    /// 从ExtensionManager创建RuntimeOptions
    ///
    /// # 参数
    /// * `extension_manager` - ExtensionManager实例
    ///
    /// # 返回值
    /// * `Self` - 新的RuntimeOptions实例
    pub fn from_extension_manager(
        extension_manager: crate::extension_manager::ExtensionManager
    ) -> Self {
        // 从ExtensionManager获取schema并重建extensions
        let schema = extension_manager.get_schema();
        let mut extensions = Vec::new();

        // 重建节点扩展
        for (name, node_type) in &schema.nodes {
            let node = crate::node::Node::create(name, node_type.spec.clone());
            extensions.push(Extensions::N(node));
        }

        // 重建标记扩展
        for (name, mark_type) in &schema.marks {
            let mark = crate::mark::Mark::new(name, mark_type.spec.clone());
            extensions.push(Extensions::M(mark));
        }

        Self {
            content: Content::None,
            extensions,
            history_limit: None,
            event_handlers: Vec::new(),
            middleware_stack: MiddlewareStack::default(),
        }
    }

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

    pub fn build(self) -> RuntimeOptions {
        RuntimeOptions {
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
        Self { state, description, timestamp: SystemTime::now(), meta }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
    Rolled,
    NotFound,
}

#[derive(Debug, Clone)]
pub struct ProcessorResult {
    pub status: TransactionStatus,
    pub error: Option<String>,
    pub result: Option<TransactionResult>,
}

pub type TaskParams = (Arc<State>, Transaction);
