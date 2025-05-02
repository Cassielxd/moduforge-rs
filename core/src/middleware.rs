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

/// 可以用于事务处理的中间件 trait
#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
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
    ) -> EditorResult<MiddlewareResult>;
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
