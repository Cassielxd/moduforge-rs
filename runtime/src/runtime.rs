use std::sync::Arc;

use crate::{
    event::{Event, EventBus, EventHandler},
    extension_manager::ExtensionManager,
    history_manager::HistoryManager,
    types::{Content, Extensions},
};
use moduforge_core::{
    model::{
        node_pool::{self, NodePool},
        schema::Schema,
    },
    state::{
        state::{State, StateConfig},
        transaction::Transaction,
    },
    transform::transform::Transform,
};
use moduforge_delta::from_binary;
use moduforge_delta::snapshot::FullSnapshot;
use tokio::{select, signal};
#[derive(Clone, Debug)]
pub struct RuntimeOptions {
    pub content: Content,
    pub extensions: Vec<Extensions>,
    pub history_limit: Option<usize>,
    pub event_handlers: Vec<Arc<dyn EventHandler>>,
}

pub struct Runtime {
    event_bus: EventBus,

    state: Arc<State>,
    extension_manager: ExtensionManager,
    history_manager: HistoryManager<Arc<State>>,
    options: RuntimeOptions,
}

impl Runtime {
    pub async fn create(options: RuntimeOptions) -> Self {
        let event_bus = EventBus::new();
        let extension_manager = ExtensionManager::new(options.extensions.clone());
        let doc = match &options.content {
            Content::NodePoolBinary(items) => {
                if let Ok(node_pool) = from_binary::<NodePool>(items) {
                    Some(Arc::new(node_pool))
                } else {
                    panic!("NodePoolBinary二进制格式数据异常");
                }
            }
            Content::NodePool(node_pool) => Some(Arc::new(node_pool.clone())),
            Content::Snapshot(items) => {
                if let Ok(full_snapshot) = from_binary::<FullSnapshot>(&items) {
                    Some(full_snapshot.node_pool.clone())
                } else {
                    panic!("Snapshot二进制格式数据异常");
                }
            }
            Content::None => None,
        };

        let state: State = State::create(StateConfig {
            schema: Some(extension_manager.get_schema()),
            doc,
            stored_marks: None,
            plugins: Some(extension_manager.get_plugins().clone()),
        })
        .await
        .unwrap();
        let state: Arc<State> = Arc::new(state);
        Runtime {
            event_bus,
            history_manager: HistoryManager::new(state.clone(), options.history_limit.clone()),
            options,
            extension_manager,
            state,
        }
    }
    pub fn doc(&self)-> Arc<NodePool>{
        self.get_state().doc()
    }
    pub fn get_state(&self) -> &Arc<State> {
        &self.state
    }
    pub fn get_schema(&self) -> Arc<Schema> {
        self.extension_manager.get_schema()
    }
    pub fn get_tr(&self) -> Transaction {
        self.get_state().tr()
    }

    pub fn get_event_bus(&self) -> &EventBus {
        &self.event_bus
    }
    /// 启动事件循环
    pub fn start_event_loop(&self) {
        let rx = self.event_bus.subscribe();
        let handlers = self.options.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = rx.recv() => match event {
                        Ok(event) => {
                            for handler in &handlers.event_handlers {
                                handler.handle(&event).await;
                            }
                        },
                        Err(_) => {
                            println!("跳出了");
                            break;
                        },
                    },
                    shutdown_signal = Box::pin(signal::ctrl_c()) => {
                        match shutdown_signal {
                            Ok(()) => {
                                println!("事件管理器,接收到关闭信号，正在退出...");
                                break;
                            },
                            Err(e) => {
                                eprintln!("事件管理器,处理关闭信号时出错: {}", e);
                                break;
                            }
                        }
                    },
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
        if !transaction.doc_changed() {
            return Ok(());
        }

        let event_bus = self.get_event_bus();
        event_bus
            .broadcast(Event::Apply(Arc::new(transaction), self.state.clone()))
            .await?;
        Ok(())
    }
    /// 注册插件
    #[allow(dead_code)]
    async fn register_plugin(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self
            .get_state()
            .reconfigure(StateConfig {
                schema: Some(self.get_schema()),
                doc: Some(self.get_state().doc()),
                stored_marks: None,
                plugins: Some(self.get_state().plugins().clone()),
            })
            .await?;
        self.state = Arc::new(state);
        Ok(())
    }
    /// 注销插件
    #[allow(dead_code)]
    async fn unregister_plugin(
        &mut self,
        plugin_key: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ps = self
            .get_state()
            .plugins()
            .iter()
            .filter(|p| p.key().key != plugin_key)
            .cloned()
            .collect();
        let state = self
            .get_state()
            .reconfigure(StateConfig {
                schema: Some(self.get_schema().clone()),
                doc: Some(self.get_state().doc()),
                stored_marks: None,
                plugins: Some(ps),
            })
            .await?;
        self.state = Arc::new(state);
        Ok(())
    }
    pub fn undo(&mut self) {
        self.history_manager.jump(-1);
        self.state = self.history_manager.get_present();
    }

    pub fn redo(&mut self) {
        self.history_manager.jump(1);
        self.state = self.history_manager.get_present();
    }
}
