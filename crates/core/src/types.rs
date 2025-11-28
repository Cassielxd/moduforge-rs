use std::{collections::HashMap, sync::Arc};
use async_trait::async_trait;
use mf_state::StateConfig;

use crate::{
    event::{Event, EventHandler},
    extension::Extension,
    mark::Mark,
    middleware::MiddlewareStack,
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

impl GlobalAttributeItem {
    /// Get the first attribute key for testing purposes
    pub fn key(&self) -> Option<&str> {
        self.attributes.keys().next().map(|s| s.as_str())
    }

    /// Get all attribute keys
    pub fn keys(&self) -> Vec<&str> {
        self.attributes.keys().map(|s| s.as_str()).collect()
    }

    /// Check if has attribute key
    pub fn has_key(
        &self,
        key: &str,
    ) -> bool {
        self.attributes.contains_key(key)
    }
}

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
    event_handlers: Vec<Arc<dyn EventHandler<Event> + Send + Sync>>,
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
        let factory = schema.factory();
        let (nodes, marks) = factory.definitions();
        // 重建节点扩展
        for (name, node_type) in nodes {
            let node = crate::node::Node::create(name, node_type.spec.clone());
            extensions.push(Extensions::N(node));
        }

        // 重建标记扩展
        for (name, mark_type) in marks {
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

    pub fn get_event_handlers(
        &self
    ) -> Vec<Arc<dyn EventHandler<Event> + Send + Sync>> {
        self.event_handlers.clone()
    }
    pub fn set_event_handlers(
        mut self,
        event_handlers: Vec<Arc<dyn EventHandler<Event> + Send + Sync>>,
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
    event_handlers: Vec<Arc<dyn EventHandler<Event> + Send + Sync>>,
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
        handlers: Vec<Arc<dyn EventHandler<Event> + Send + Sync>>,
    ) -> Self {
        self.event_handlers = handlers;
        self
    }

    pub fn add_event_handler(
        mut self,
        handler: Arc<dyn EventHandler<Event> + Send + Sync>,
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
    pub fn add_middleware<T>(
        mut self,
        middleware: T,
    ) -> Self
    where
        T: crate::middleware::MiddlewareGeneric<
                mf_model::node_pool::NodePool,
                mf_model::schema::Schema,
            > + 'static,
    {
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

// Re-export from generic module
pub use crate::generic::types::{
    HistoryEntryWithMetaGeneric, ProcessorResultGeneric, TaskParamsGeneric, TransactionStatus,
};

// ==================== 向后兼容类型别名 ====================

/// 默认 HistoryEntryWithMeta 类型（向后兼容）
pub type HistoryEntryWithMeta = HistoryEntryWithMetaGeneric<
    mf_model::node_pool::NodePool,
    mf_model::schema::Schema,
>;

/// 默认 ProcessorResult 类型（向后兼容）
pub type ProcessorResult = ProcessorResultGeneric<
    mf_model::node_pool::NodePool,
    mf_model::schema::Schema,
>;

/// 默认 TaskParams 类型（向后兼容）
pub type TaskParams = TaskParamsGeneric<
    mf_model::node_pool::NodePool,
    mf_model::schema::Schema,
>;
