use std::sync::Arc;

use crate::{
    event::{Event, EventBus},
    extension_manager::ExtensionManager,
    flow::{FlowEngine, ProcessorResult},
    helpers::create_doc,
    history_manager::HistoryManager,
    traits::{EditorBase, EditorCore},
    types::EditorOptions,
};
use async_trait::async_trait;
use moduforge_core::{
    debug,
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
    pub async fn create(
        options: EditorOptions
    ) -> Result<Self, Box<dyn std::error::Error>> {
        debug!("正在创建新的编辑器实例");
        let extension_manager =
            ExtensionManager::new(&options.get_extensions());
        debug!("已初始化扩展管理器");
        let doc = create_doc::create_doc(
            &extension_manager.get_schema(),
            &options.get_content(),
        )
        .await;
        let event_bus = EventBus::new();
        debug!("已创建文档和事件总线");
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
            history_manager: HistoryManager::new(
                state,
                options.get_history_limit(),
            ),
            options,
        };

        let mut runtime = Editor { base, flow_engine: FlowEngine::new()? };
        runtime.init().await?;
        debug!("编辑器实例创建成功");
        Ok(runtime)
    }

    /// 初始化编辑器，设置事件处理器并启动事件循环
    async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.base
            .event_bus
            .add_event_handlers(self.base.options.get_event_handlers())
            .await?;
        self.base.event_bus.start_event_loop();
        self.base
            .event_bus
            .broadcast_blocking(Event::Create(self.base.state.clone()))?;
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
        let (_id, mut rx) = self
            .flow_engine
            .submit_transaction((self.base.state.clone(), transaction))
            .await?;

        let Some(task_result) = rx.recv().await else {
            return Ok(());
        };
        let Some(ProcessorResult { result: Some(mut result), .. }) =
            task_result.output
        else {
            return Ok(());
        };

        self.base.state = Arc::new(result.state);

        if let Some(tr) = result.transactions.pop() {
            if tr.doc_changed() {
                self.base.history_manager.insert(self.base.state.clone());
                self.base
                    .event_bus
                    .broadcast(Event::TrApply(
                        Arc::new(tr),
                        self.base.state.clone(),
                    ))
                    .await?;
            }
        }
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
