use std::{fmt::Debug, sync::Arc};

use async_channel::{Receiver, Sender};
use moduforge_core::state::{state::State, transaction::Transaction};

// 事件类型定义
#[derive(Clone)]
pub enum Event {
    Apply(Arc<Transaction>, Arc<State>), // 事务应用后 + 是否成功
    None,
}

#[derive(Clone)]
pub struct EventBus {
    tx: Sender<Event>,
    rt:Receiver<Event>
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, rt) = async_channel::bounded(100);
        Self { tx ,rt}
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        self.rt.clone()
    }

    pub async fn broadcast(&self, event: Event) -> Result<(), async_channel::SendError<Event>> {
        self.tx.send(event).await?;
        Ok(())
    }
}

// 事件处理器特征
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync + Debug {
    async fn handle(&self, event: &Event);
}

// 事件上下文
pub struct EventContext {
    pub state: Arc<State>,
}
