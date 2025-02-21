use std::sync::Arc;

use crate::{
    cache::{CacheKey, cache::DocumentCache},
    engine_manager::EngineManager,
    event::{Event, EventBus, EventHandler},
    event_handler::SnapshotHandler,
    extension_manager::ExtensionManager,
    helpers::create_doc,
    history_manager::HistoryManager,
    snapshot_manager::SnapshotManager,
    types::{EditorOptions, StorageOptions},
};
use moduforge_core::{
    model::{node_pool::NodePool, schema::Schema},
    state::{
        state::{State, StateConfig},
        transaction::{Command, Transaction},
    },
    transform::transform::Transform,
};

/// 编辑器
pub struct Editor {
    event_bus: EventBus,
    state: Arc<State>,
    extension_manager: ExtensionManager,
    snapshot_manager: Arc<SnapshotManager>,
    engine_manager: EngineManager,
    history_manager: HistoryManager<Arc<State>>,
    options: EditorOptions,
    storage: StorageOptions,
}

impl Editor {
    pub async fn create(options: EditorOptions) -> Self {
        let extension_manager = ExtensionManager::new(&options.get_extensions());

        let doc = create_doc::create_doc(&options.get_content());
        let storage = match &options.get_storage_option() {
            Some(o) => o.clone(),
            None => StorageOptions::default(),
        };
        let cache: Arc<DocumentCache> = DocumentCache::new(&storage);
        let snapshot_manager = SnapshotManager::create(cache);
        let event_bus = EventBus::new();
        let state: State = State::create(StateConfig {
            schema: Some(extension_manager.get_schema()),
            doc,
            stored_marks: None,
            plugins: Some(extension_manager.get_plugins().clone()),
        })
        .await
        .unwrap();
        let state: Arc<State> = Arc::new(state);

        let mut runtime = Editor {
            event_bus,
            history_manager: HistoryManager::new(state.clone(), options.get_history_limit()),
            snapshot_manager,
            engine_manager: EngineManager::create(options.get_rules_path()),
            options,
            extension_manager,
            state,
            storage,
        };
        runtime.init().await;
        runtime
    }
    pub async fn init(&mut self) {
        let default_event_handlers = init_event_handler(
            &self.snapshot_manager,
            self.options.get_event_handlers(),
            self.storage.clone(),
            self.options.get_history_limit(),
        );
        self.event_bus
            .add_event_handlers(default_event_handlers)
            .await;
        self.event_bus.start_event_loop();
        let _ = self
            .event_bus
            .broadcast_blocking(Event::Create(self.state.clone()));
    }

    pub fn doc(&self) -> Arc<NodePool> {
        self.get_state().doc()
    }
    pub fn get_options(&self) -> &EditorOptions {
        &self.options
    }
    pub fn get_engine_manager(&self) -> &EngineManager {
        &self.engine_manager
    }
    pub fn get_snapshot(&self, key: &CacheKey) -> Option<Arc<NodePool>> {
        self.snapshot_manager.get_snapshot(key)
    }
    pub fn get_state(&self) -> &Arc<State> {
        &self.state
    }
    pub fn get_schema(&self) -> Arc<Schema> {
        self.extension_manager.get_schema()
    }
    /// 获取新的事物
    pub fn get_tr(&self) -> Transaction {
        let mut tr = self.get_state().tr();
        let engine = self.engine_manager.engine.clone();
        tr.set_meta("engine", engine);
        tr
    }
    /// 执行自定义命令
    pub async fn command(
        &mut self,
        command: Arc<dyn Command>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut tr = self.get_tr();
        tr.transaction(command).await;
        self.dispatch(tr).await?;
        Ok(())
    }
    pub fn get_event_bus(&self) -> &EventBus {
        &self.event_bus
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
            .broadcast(Event::TrApply(Arc::new(transaction), self.state.clone()))
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
            .filter(|p| p.key != plugin_key)
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

pub fn init_event_handler(
    snapshot_manager: &Arc<SnapshotManager>,
    event_handlers: Vec<Arc<dyn EventHandler>>,
    storage: StorageOptions,
    snapshot_interval: Option<usize>,
) -> Vec<Arc<dyn EventHandler>> {
    let mut default_event_handlers: Vec<Arc<dyn EventHandler>> = vec![SnapshotHandler::new(
        storage.clone(),
        snapshot_interval.unwrap_or(50),
        snapshot_manager.clone(),
    )];
    default_event_handlers.append(&mut event_handlers.clone());
    default_event_handlers
}
