use std::sync::Arc;

use moduforge_core::{
    middleware::{Middleware, MiddlewareResult},
    EditorResult,
};
use moduforge_state::{State, Transaction};

pub struct Middleware1;
#[async_trait::async_trait]
impl Middleware for Middleware1 {
    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> EditorResult<()> {
        println!("Middleware1 before_dispatch");
        Ok(())
    }
    async fn after_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> EditorResult<MiddlewareResult> {
        println!("Middleware1 after_dispatch");
        Ok(MiddlewareResult::new(EditorResult::Ok(())))
    }
}

pub struct Middleware2;
#[async_trait::async_trait]
impl Middleware for Middleware2 {
    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> EditorResult<()> {
        println!("Middleware2 before_dispatch");
        Ok(())
    }
    async fn after_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> EditorResult<MiddlewareResult> {
        println!("Middleware2 after_dispatch");
        Ok(MiddlewareResult::new(EditorResult::Ok(())))
    }
}
