use std::sync::Arc;

use crate::error::EditorResult;
use moduforge_state::{transaction::Transaction, state::State};

/// 可以用于事务处理的中间件 trait
#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    /// 返回中间件的名称
    fn name(&self) -> String;

    /// 在事务到达核心分发之前处理事务
    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> EditorResult<()>;

    /// 在核心分发之后处理结果
    /// 返回一个可能包含需要额外处理的事务的 MiddlewareResult
    async fn after_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> EditorResult<Option<Transaction>>;
}

/// 用于事务处理的中间件类型别名
pub type ArcMiddleware = Arc<dyn Middleware>;

/// Middleware stack that holds multiple middleware
#[derive(Clone)]
pub struct MiddlewareStack {
    pub middlewares: Vec<ArcMiddleware>,
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
