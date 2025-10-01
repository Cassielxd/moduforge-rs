use std::sync::Arc;

use async_trait::async_trait;
use mf_core::{
    history_manager::HistoryManager, types::HistoryEntryWithMeta, ForgeResult,
};
use mf_model::node_pool::NodePool;
use mf_state::{transaction::Command, State, Transaction};


#[async_trait]
pub trait EditorTrait: Send + Sync {
    async fn doc(&self) -> Arc<NodePool>;
    async fn get_state(&self) -> Arc<State>;
    async fn get_history_manager(
        &self
    ) -> Option<&HistoryManager<HistoryEntryWithMeta>> {
        None
    }
    async fn command(
        &mut self,
        command: Arc<dyn Command>,
    ) -> ForgeResult<()>;

    async fn command_with_meta(
        &mut self,
        command: Arc<dyn Command>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()>;
    async fn dispatch_flow(
        &mut self,
        transaction: Transaction,
    ) -> ForgeResult<()>;
    async fn dispatch_flow_with_meta(
        &mut self,
        transaction: Transaction,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()>;
}
