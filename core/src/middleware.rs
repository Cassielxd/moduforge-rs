use std::sync::Arc;

use crate::error::EditorResult;
use moduforge_state::{transaction::Transaction, state::State};

/// 表示中间件处理结果的结构体
pub struct MiddlewareResult {
    /// 原始处理结果
    pub result: EditorResult<()>,
    /// 需要额外处理的事务列表
    pub additional_transaction: Option<Transaction>,
}

impl MiddlewareResult {
    /// 创建一个只包含结果的处理结果
    pub fn new(result: EditorResult<()>) -> Self {
        Self { result, additional_transaction: None }
    }

    /// 创建一个包含结果和额外事务的处理结果
    pub fn with_transactions(
        result: EditorResult<()>,
        transaction: Option<Transaction>,
    ) -> Self {
        Self { result, additional_transaction: transaction }
    }
}

/// Middleware trait that can be implemented for transaction processing
#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    /// Process the transaction before it reaches the core dispatch
    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> EditorResult<()>;

    /// Process the result after the core dispatch
    /// Returns a MiddlewareResult that may contain additional transactions to be processed
    async fn after_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> EditorResult<MiddlewareResult>;
}

/// Type alias for a boxed middleware
pub type BoxedMiddleware = Arc<dyn Middleware>;

/// Middleware stack that holds multiple middleware
#[derive(Clone)]
pub struct MiddlewareStack {
    pub middlewares: Vec<BoxedMiddleware>,
}

impl MiddlewareStack {
    pub fn new() -> Self {
        Self { middlewares: Vec::new() }
    }

    pub fn add<M>(
        &mut self,
        middleware: M,
    ) where
        M: Middleware + 'static,
    {
        self.middlewares.push(Arc::new(middleware));
    }

    pub fn is_empty(&self) -> bool {
        self.middlewares.is_empty()
    }
}

impl Default for MiddlewareStack {
    fn default() -> Self {
        Self::new()
    }
}
