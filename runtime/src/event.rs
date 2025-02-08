use std::sync::Arc;

use moduforge_core::state::{state::State, transaction::Transaction};
use tokio::sync::broadcast::{self, Receiver, Sender};
// 事件类型定义
#[derive(Clone)]
pub enum Event {
    Apply(Arc<Transaction>, Arc<State>), // 事务应用后 + 是否成功

}

#[derive(Clone)]
pub struct EventBus {
    tx: Sender<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        self.tx.subscribe()
    }

    pub fn publish(&self, event: Event) {
        let _ = self.tx.send(event);
    }
}

// 事件处理器特征
pub trait EventHandler: Send + Sync {
    fn handle(&self, event: &Event);
}
