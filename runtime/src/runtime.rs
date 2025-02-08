use std::sync::Arc;

use moduforge_core::{model::schema::Schema, state::{state::State, transaction::Transaction}, transform::transform::Transform};

use crate::event::{EventBus, EventHandler, Event};

pub struct RuntimeOptions {}

pub struct Runtime {
    state: Arc<State>,
    schema: Schema,
    event_bus: EventBus,
    handlers: Vec<Arc<dyn EventHandler>>,
}

impl Runtime {
    pub fn new(state: State, schema: Schema) -> Self {
        let event_bus = EventBus::new();
        Runtime {
            state:Arc::new(state),
            schema,
            event_bus,
            handlers: vec![],
        }
    }

    pub fn get_event_bus(&self) -> &EventBus {
        &self.event_bus
    }
    /// 启动事件循环
    fn start_event_loop(&self) {
        let mut rx = self.event_bus.subscribe();
        let handlers = self.handlers.clone();
        tokio::spawn(async move{
            while let Ok(event) = rx.recv().await {
                for handler in &handlers {
                    handler.handle(&event);
                }
            }
        });
       
    }
    /// 处理事务事件，并生成增量记录。
    async fn dispatch(&mut self,transaction:Transaction)-> Result<(), Box<dyn std::error::Error>> {
        let mut transaction = transaction;
        self.state = Arc::new(self.state.apply(&mut transaction).await?) ;
        if transaction.doc_changed(){
           let event_bus = self.get_event_bus();
           event_bus.publish(Event::Apply(Arc::new(transaction),self.state.clone()));
        }
        Ok(())
    }
}



