use std::sync::Arc;

use async_trait::async_trait;
use moduforge_core::{middleware::Middleware, ForgeResult};
use moduforge_state::{State, Transaction};
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
        state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> ForgeResult<Option<Transaction>> {
        tracing::info!(
            "同步事务: {} 个到房间 {}",
            transactions.len(),
            self.room_id
        );

        let state = match state {
            Some(s) => s,
            None => {
                error!(
                    "YrsMiddleware after_dispatch called without state, cannot sync."
                );
                return Ok(None);
            },
        };

        // 使用新的批量二进制格式同步
        if let Err(e) = self
            .sync_service
            .handle_transaction_applied(
                &self.room_id,
                transactions,
                &state,
                None,
            )
            .await
        {
            error!("YRS同步失败: {}", e);
        }

        Ok(None)
    }
}
