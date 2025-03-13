use std::{path::Path, sync::Arc};

use crate::{
    cache::{cache::DocumentCache, CacheKey},
    engine_manager::EngineManager,
    event::{Event, EventBus, EventHandler},
    event_handler::SnapshotHandler,
    extension_manager::ExtensionManager,
    flow::{FlowEngine, TransactionResult},
    helpers::create_doc,
    history_manager::HistoryManager,
    storage_manager::StorageManager,
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

/// Editor 结构体代表编辑器的核心功能实现
/// 负责管理文档状态、事件处理、插件系统和存储等核心功能
pub struct Editor {
    /// 事件总线，用于处理编辑器内的事件分发
    event_bus: EventBus,
    /// 当前文档状态
    state: Arc<State>,
    flow_engine: FlowEngine,
    /// 插件管理器，负责插件的加载和管理
    extension_manager: ExtensionManager,
    /// 存储管理器，负责文档的持久化存储
    storage_manager: Arc<StorageManager>,
    /// 引擎管理器，处理规则引擎相关的操作
    engine_manager: EngineManager,
    /// 历史记录管理器，用于实现撤销/重做功能
    history_manager: HistoryManager<Arc<State>>,
    /// 编辑器配置选项
    options: EditorOptions,
}

impl Editor {
    /// 创建新的编辑器实例
    /// options: 编辑器配置选项
    pub async fn create(options: EditorOptions) -> Self {
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
        .await
        .unwrap();
        let state: Arc<State> = Arc::new(state);

        let mut runtime = Editor {
            event_bus,
            history_manager: HistoryManager::new(state.clone(), options.get_history_limit()),
            storage_manager,
            engine_manager: EngineManager::create(options.get_rules_path()),
            options,
            extension_manager,
            state,
            flow_engine: FlowEngine::new().unwrap(),
        };
        runtime.init().await;
        runtime
    }
    /// 初始化编辑器，设置事件处理器并启动事件循环
    async fn init(&mut self) {
        let default_event_handlers = init_event_handler(
            &self.storage_manager,
            self.options.get_event_handlers(),
            self.options.get_storage_option(),
            self.options.get_history_limit(),
        );
        self.event_bus.add_event_handlers(default_event_handlers).await;
        self.event_bus.start_event_loop();
        let _ = self.event_bus.broadcast_blocking(Event::Create(self.state.clone()));
    }
    /// 获取当前文档内容
    pub fn doc(&self) -> Arc<NodePool> {
        self.get_state().doc()
    }
    /// 获取编辑器配置选项
    pub fn get_options(&self) -> &EditorOptions {
        &self.options
    }
    // 获取引擎管理器实例
    pub fn get_engine_manager(&self) -> &EngineManager {
        &self.engine_manager
    }
    /// 根据缓存键获取文档快照
    pub fn get_snapshot(
        &self,
        key: &CacheKey,
    ) -> Option<Arc<NodePool>> {
        self.storage_manager.get_snapshot(key)
    }
    /// 获取当前状态
    pub fn get_state(&self) -> &Arc<State> {
        &self.state
    }
    /// 获取文档模式定义
    pub fn get_schema(&self) -> Arc<Schema> {
        self.extension_manager.get_schema()
    }
    /// 将当前文档导出为zip文件
    /// output_path: 输出文件路径
    pub async fn export_zip(
        &self,
        output_path: &Path,
    ) {
        if let Err(err) = self.storage_manager.export_zip(&self.state, output_path).await {
            eprintln!("导出zip文件失败:{}", err);
        }
    }
    /// 创建新的事务实例
    /// 返回: 包含当前状态和引擎配置的事务对象
    pub fn get_tr(&self) -> Transaction {
        let mut tr = self.get_state().tr();
        let engine = self.engine_manager.engine.clone();
        tr.set_meta("engine", engine);
        tr
    }
    /// 执行自定义命令
    /// command: 要执行的命令
    /// 返回: 执行结果
    pub async fn command(
        &mut self,
        command: Arc<dyn Command>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut tr = self.get_tr();
        tr.transaction(command).await;
        self.dispatch(tr).await?;
        Ok(())
    }
    /// 获取事件总线实例
    pub fn get_event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// 处理事务并更新状态
    /// transaction: 要处理的事务
    /// 返回: 处理结果
    pub async fn dispatch(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (_id, mut rx) = self.flow_engine.submit_transaction(self.state.clone(), transaction).await?;
        match rx.recv().await {
            Some(TransactionResult { transaction_id: _, status: _, transaction, error: _, state }) => {
                self.state = Arc::new(state.unwrap());
                let tr = transaction.unwrap();
                if !tr.doc_changed() {
                    return Ok(());
                }
                self.history_manager.insert(self.state.clone());
                let event_bus = self.get_event_bus();
                event_bus.broadcast(Event::TrApply(Arc::new(tr), self.state.clone())).await?;
            },
            None => {
                println!("transaction is not found");
            },
        }

        Ok(())
    }
    /// 注册新插件
    /// 更新状态以包含新插件的配置
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
        self.state = Arc::new(state);
        Ok(())
    }
    /// 执行撤销操作
    pub fn undo(&mut self) {
        self.history_manager.jump(-1);
        self.state = self.history_manager.get_present();
    }
    /// 执行重做操作
    pub fn redo(&mut self) {
        self.history_manager.jump(1);
        self.state = self.history_manager.get_present();
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
