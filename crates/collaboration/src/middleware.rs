use std::sync::Arc;

use async_trait::async_trait;
use mf_core::{middleware::Middleware, ForgeResult};
use mf_state::{State, Transaction};
use tracing::error;

use crate::SyncService;

#[derive(Debug)]
pub struct YrsMiddleware {
    pub sync_service: Arc<SyncService>,
    pub room_id: String,
}

#[async_trait]
impl Middleware for YrsMiddleware {
    fn name(&self) -> String {
        "YrsMiddleware".to_string()
    }
    async fn after_dispatch(
        &self,
        _state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> ForgeResult<Option<Transaction>> {
        tracing::info!(
            "同步事务: {} 个到房间 {}",
            transactions.len(),
            self.room_id
        );

        // 同步每个事务
        if let Err(e) = self
            .sync_service
            .handle_transaction_applied(transactions, &self.room_id)
            .await
        {
            error!("YRS同步失败: {}", e);
        }

        Ok(None)
    }
}
