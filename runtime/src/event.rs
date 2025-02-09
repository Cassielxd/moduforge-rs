use std::sync::Arc;

use moduforge_core::state::{state::State, transaction::Transaction};
use tokio::sync::broadcast::{self};
use async_broadcast::{Sender, Receiver};
// 事件类型定义
#[derive(Clone)]
pub enum Event {
    Apply(Arc<Transaction>, Arc<State>), // 事务应用后 + 是否成功
    None
}

#[derive(Clone)]
pub struct EventBus {
    tx: Sender<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = async_broadcast::broadcast(100);
        Self { tx }
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        self.tx.new_receiver()
    }

    pub async fn broadcast(&self, event: Event) -> Result<(), async_broadcast::SendError<Event>> {
        self.tx.broadcast(event).await?;
        Ok(())
    }


}

// 事件处理器特征
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &Event);
}

// 事件上下文
pub struct EventContext {
    pub state: Arc<State>
}