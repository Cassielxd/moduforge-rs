use std::{path::Path, sync::Arc};

use crate::{
    cache::{cache::DocumentCache, CacheKey},
    event::{Event, EventBus, EventHandler},
    event_handler::SnapshotHandler,
    extension_manager::ExtensionManager,
    flow::{FlowEngine, ProcessorResult},
    helpers::create_doc,
    history_manager::HistoryManager,
    storage_manager::StorageManager,
    traits::{EditorBase, EditorCore},
    types::{EditorOptions, StorageOptions},
};
use async_trait::async_trait;
use moduforge_core::{
    model::{node_pool::NodePool, schema::Schema},
    state::{
        state::{State, StateConfig},
        transaction::{Command, Transaction},
    },
    transform::transform::Transform,
};

/// Editor 结构体代表编辑器的核心功能实现
/// 负责管理文档状态、事件处理、插件系统和存储等核心功能
pub struct Editor {
    base: EditorBase,
    flow_engine: FlowEngine,
}

impl Editor {
    /// 创建新的编辑器实例
    /// options: 编辑器配置选项
    pub async fn create(options: EditorOptions) -> Result<Self, Box<dyn std::error::Error>> {
        let extension_manager = ExtensionManager::new(&options.get_extensions());

        let doc = create_doc::create_doc(&options.get_content());
        let storage = options.get_storage_option();
        let cache: Arc<DocumentCache> = DocumentCache::new(&storage);
        let storage_manager = StorageManager::create(cache);
        let event_bus = EventBus::new();
        let state: State = State::create(StateConfig {
            schema: Some(extension_manager.get_schema()),
            doc,
            stored_marks: None,
            plugins: Some(extension_manager.get_plugins().clone()),
        })
        .await?;
        let state: Arc<State> = Arc::new(state);

        let base = EditorBase {
            event_bus,
            state: state.clone(),
            extension_manager,
            storage_manager,
            history_manager: HistoryManager::new(state, options.get_history_limit()),
            options,
        };

        let mut runtime = Editor { base, flow_engine: FlowEngine::new()? };
        runtime.init().await?;
        Ok(runtime)
    }

    /// 初始化编辑器，设置事件处理器并启动事件循环
    async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let default_event_handlers = init_event_handler(
            &self.base.storage_manager,
            self.base.options.get_event_handlers(),
            self.base.options.get_storage_option(),
            self.base.options.get_history_limit(),
        );
        self.base.event_bus.add_event_handlers(default_event_handlers).await?;
        self.base.event_bus.start_event_loop();
        self.base.event_bus.broadcast_blocking(Event::Create(self.base.state.clone()))?;
        Ok(())
    }
}
#[async_trait]
impl EditorCore for Editor {
    type Error = Box<dyn std::error::Error>;

    fn doc(&self) -> Arc<NodePool> {
        self.base.doc()
    }

    fn get_options(&self) -> &EditorOptions {
        self.base.get_options()
    }


    fn get_snapshot(
        &self,
        key: &CacheKey,
    ) -> Option<Arc<NodePool>> {
        self.base.get_snapshot(key)
    }

    fn get_state(&self) -> &Arc<State> {
        self.base.get_state()
    }

    fn get_schema(&self) -> Arc<Schema> {
        self.base.get_schema()
    }

    fn get_event_bus(&self) -> &EventBus {
        self.base.get_event_bus()
    }

    fn get_tr(&self) -> Transaction {
        self.base.get_tr()
    }

    async fn command(
        &mut self,
        command: Arc<dyn Command>,
    ) -> Result<(), Self::Error> {
        let mut tr = self.get_tr();
        tr.transaction(command).await;
        self.dispatch(tr).await
    }

    async fn dispatch(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), Self::Error> {
        let (_id, mut rx) = self.flow_engine.submit_transaction((self.base.state.clone(), transaction)).await?;
        match rx.recv().await {
            Some(task_result) => {
                if let Some(ProcessorResult { result, status: _, error: _ }) = task_result.output {
                    if let Some(result) = result {
                        self.base.state = Arc::new(result.state);
                        let mut trs = result.trs;
                        let tr = trs.pop().unwrap();
                        if !tr.doc_changed() {
                            return Ok(());
                        }
                        self.base.history_manager.insert(self.base.state.clone());
                        let event_bus = self.get_event_bus();
                        event_bus.broadcast(Event::TrApply(Arc::new(tr), self.base.state.clone())).await?;
                    }
                }
            },
            None => {
                println!("transaction is not found");
            },
        }
        Ok(())
    }

    async fn export_zip(
        &self,
        output_path: &Path,
    ) -> Result<(), Self::Error> {
        self.base.storage_manager.export_zip(&self.base.state, output_path).await?;
        Ok(())
    }

    async fn register_plugin(&mut self) -> Result<(), Self::Error> {
        let state = self
            .get_state()
            .reconfigure(StateConfig {
                schema: Some(self.get_schema()),
                doc: Some(self.get_state().doc()),
                stored_marks: None,
                plugins: Some(self.get_state().plugins().clone()),
            })
            .await?;
        self.base.state = Arc::new(state);
        Ok(())
    }

    async fn unregister_plugin(
        &mut self,
        plugin_key: String,
    ) -> Result<(), Self::Error> {
        let ps = self.get_state().plugins().iter().filter(|p| p.key != plugin_key).cloned().collect();
        let state = self
            .get_state()
            .reconfigure(StateConfig {
                schema: Some(self.get_schema().clone()),
                doc: Some(self.get_state().doc()),
                stored_marks: None,
                plugins: Some(ps),
            })
            .await?;
        self.base.state = Arc::new(state);
        Ok(())
    }

    fn undo(&mut self) {
        self.base.undo()
    }

    fn redo(&mut self) {
        self.base.redo()
    }
}

/// 初始化事件处理器
/// storage_manager: 存储管理器实例
/// event_handlers: 自定义事件处理器列表
/// storage: 存储配置选项
/// snapshot_interval: 快照间隔配置
/// 返回: 包含默认处理器和自定义处理器的完整处理器列表
pub fn init_event_handler(
    storage_manager: &Arc<StorageManager>,
    event_handlers: Vec<Arc<dyn EventHandler>>,
    storage: StorageOptions,
    snapshot_interval: Option<usize>,
) -> Vec<Arc<dyn EventHandler>> {
    let mut default_event_handlers: Vec<Arc<dyn EventHandler>> =
        vec![SnapshotHandler::new(storage.clone(), snapshot_interval.unwrap_or(50), storage_manager.clone())];
    default_event_handlers.append(&mut event_handlers.clone());
    default_event_handlers
}
