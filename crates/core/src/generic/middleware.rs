//! 泛型中间件系统
//!
//! 定义了支持任意 DataContainer 和 SchemaDefinition 组合的中间件系统。

use std::sync::Arc;

use crate::error::ForgeResult;
use mf_model::traits::{DataContainer, SchemaDefinition};
use mf_state::{
    state::StateGeneric,
    transaction::TransactionGeneric,
};

// ==================== 泛型中间件 Trait ====================

/// 可以用于事务处理的中间件 trait（泛型版本）
///
/// 支持任意 DataContainer 和 SchemaDefinition 组合
#[async_trait::async_trait]
pub trait MiddlewareGeneric<C, S>: Send + Sync
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 返回中间件的名称
    fn name(&self) -> String;

    /// 在事务到达核心分发之前处理事务
    async fn before_dispatch(
        &self,
        _transaction: &mut TransactionGeneric<C, S>,
    ) -> ForgeResult<()> {
        Ok(())
    }

    /// 在核心分发之后处理结果
    /// 返回一个可能包含需要额外处理的事务的 MiddlewareResult
    async fn after_dispatch(
        &self,
        _state: Option<Arc<StateGeneric<C, S>>>,
        _transactions: &[Arc<TransactionGeneric<C, S>>],
    ) -> ForgeResult<Option<TransactionGeneric<C, S>>> {
        Ok(None)
    }
}

// ==================== 泛型中间件堆栈 ====================

/// Middleware stack that holds multiple middleware（泛型版本）
#[derive(Clone)]
pub struct MiddlewareStackGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub middlewares: Vec<Arc<dyn MiddlewareGeneric<C, S>>>,
}

impl<C, S> MiddlewareStackGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub fn new() -> Self {
        Self { middlewares: Vec::new() }
    }

    pub fn add<M>(
        &mut self,
        middleware: M,
    ) where
        M: MiddlewareGeneric<C, S> + 'static,
    {
        self.middlewares.push(Arc::new(middleware));
    }

    pub fn is_empty(&self) -> bool {
        self.middlewares.is_empty()
    }
}

impl<C, S> Default for MiddlewareStackGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
