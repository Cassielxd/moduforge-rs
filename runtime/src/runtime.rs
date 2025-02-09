use std::sync::Arc;

use moduforge_core::{
    model::schema::Schema,
    state::{
        state::{State, StateConfig},
        transaction::Transaction,
    },
    transform::transform::Transform,
};

use crate::event::{Event, EventBus, EventHandler};

pub struct RuntimeOptions {}

pub struct Runtime {
    state: Arc<State>,
    schema: Arc<Schema>,
    event_bus: EventBus,
    handlers: Vec<Arc<dyn EventHandler>>,
}

impl Runtime {
    pub fn new(state: State, schema: Schema) -> Self {
        let event_bus = EventBus::new();
        Runtime {
            state: Arc::new(state),
            schema: Arc::new(schema),
            event_bus,
            handlers: vec![],
        }
    }
    pub fn add_handler(&mut self, handler: Arc<dyn EventHandler>) {
        self.handlers.push(handler);
    }
    pub fn get_state(&self) -> &Arc<State> {
        &self.state
    }
    pub fn get_schema(&self) -> &Schema {
        &self.schema
    }
    pub fn get_tr(&self) -> Transaction {
        self.state.tr()
    }

    pub fn get_event_bus(&self) -> &EventBus {
        &self.event_bus
    }
    /// 启动事件循环
    pub fn start_event_loop(&self) {
        let mut rx = self.event_bus.subscribe();
        let handlers = self.handlers.clone();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                for handler in &handlers {
                    handler.handle(&event);
                }
            }
        });
    }
    /// 处理事务事件，并生成增量记录。
    pub async fn dispatch(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut transaction = transaction;
        self.state = Arc::new(self.state.apply(&mut transaction).await?);
        if transaction.doc_changed() {
            let event_bus = self.get_event_bus();
            event_bus.broadcast(Event::Apply(Arc::new(transaction), self.state.clone())).await?;
        }
        Ok(())
    }
    /// 注册插件
    async fn register_plugin(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self
            .state
            .reconfigure(StateConfig {
                schema: Some(self.schema.clone()),
                doc: Some(self.state.doc()),
                stored_marks: None,
                plugins: Some(self.state.plugins().clone()),
            })
            .await?;
        self.state = Arc::new(state);
        Ok(())
    }
    /// 注销插件
    async fn unregister_plugin(
        &mut self,
        plugin_key: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ps = self
            .state
            .plugins()
            .iter()
            .filter(|p| p.key().key != plugin_key)
            .cloned()
            .collect();
        let state = self
            .state
            .reconfigure(StateConfig {
                schema: Some(self.schema.clone()),
                doc: Some(self.state.doc()),
                stored_marks: None,
                plugins: Some(ps),
            })
            .await?;
        self.state = Arc::new(state);
        Ok(())
    }
}
