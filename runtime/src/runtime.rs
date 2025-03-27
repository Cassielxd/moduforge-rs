use std::sync::Arc;

use crate::{
    error::{EditorError, EditorResult, error_utils},
    event::{Event, EventBus},
    extension_manager::ExtensionManager,
    helpers::create_doc,
    history_manager::HistoryManager,
    types::EditorOptions,
    traits::{EditorCore, EditorBase},
};
use async_trait::async_trait;
use moduforge_core::{
    model::{node_pool::NodePool, schema::Schema},
    state::{
        state::{State, StateConfig, TransactionResult},
        transaction::{Command, Transaction},
    },
    transform::transform::Transform,
    {info, debug, error},
};

/// Editor 结构体代表编辑器的核心功能实现
/// 负责管理文档状态、事件处理、插件系统和存储等核心功能
pub struct Editor {
    base: EditorBase,
}

impl Editor {
    /// 创建新的编辑器实例
    /// options: 编辑器配置选项
    pub async fn create(options: EditorOptions) -> EditorResult<Self> {
        info!("正在创建新的编辑器实例");
        let extension_manager = ExtensionManager::new(&options.get_extensions());
        debug!("已初始化扩展管理器");

        let doc = create_doc::create_doc(&extension_manager.get_schema(),&options.get_content()).await;
        let event_bus = EventBus::new();
        debug!("已创建文档和事件总线");

        let state: State = State::create(StateConfig {
            schema: Some(extension_manager.get_schema()),
            doc,
            stored_marks: None,
            plugins: Some(extension_manager.get_plugins().clone()),
        })
        .await
        .map_err(|e| {
            error!("创建状态失败: {}", e);
            error_utils::state_error(format!("Failed to create state: {}", e))
        })?;

        let state: Arc<State> = Arc::new(state);
        debug!("已创建编辑器状态");

        let base = EditorBase {
            event_bus,
            state: state.clone(),
            extension_manager,
            history_manager: HistoryManager::new(state, options.get_history_limit()),
            options,
        };

        let mut runtime = Editor { base };
        runtime.init().await?;
        info!("编辑器实例创建成功");
        Ok(runtime)
    }

    /// 初始化编辑器，设置事件处理器并启动事件循环
    async fn init(&mut self) -> EditorResult<()> {
        debug!("正在初始化编辑器");
        self.base.event_bus.add_event_handlers(self.base.options.get_event_handlers()).await?;
        self.base.event_bus.start_event_loop();
        debug!("事件总线已启动");

        self.base.event_bus.broadcast_blocking(Event::Create(self.base.state.clone())).map_err(|e| {
            error!("广播创建事件失败: {}", e);
            error_utils::event_error(format!("Failed to broadcast create event: {}", e))
        })?;
        debug!("已广播创建事件");
        Ok(())
    }
}

#[async_trait]
impl EditorCore for Editor {
    type Error = EditorError;

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
    ) -> EditorResult<()> {
        debug!("正在执行命令: {}", command.name());
        let mut tr = self.get_tr();
        tr.transaction(command).await;
        self.dispatch(tr).await
    }

    async fn dispatch(
        &mut self,
        transaction: Transaction,
    ) -> EditorResult<()> {
        let TransactionResult { state, mut transactions } = self
            .base
            .state
            .apply(transaction)
            .await
            .map_err(|e| error_utils::state_error(format!("Failed to apply transaction: {}", e)))?;

        if let Some(tr) = transactions.pop() {
            if tr.doc_changed() {
                self.base.state = Arc::new(state);
                self.base.history_manager.insert(self.base.state.clone());

                self.base
                    .event_bus
                    .broadcast(Event::TrApply(Arc::new(tr), self.base.state.clone()))
                    .await
                    .map_err(|e| error_utils::event_error(format!("Failed to broadcast transaction event: {}", e)))?;
            }
        }

        Ok(())
    }

    async fn register_plugin(&mut self) -> EditorResult<()> {
        info!("正在注册新插件");
        let state = self
            .get_state()
            .reconfigure(StateConfig {
                schema: Some(self.get_schema()),
                doc: Some(self.get_state().doc()),
                stored_marks: None,
                plugins: Some(self.get_state().plugins().clone()),
            })
            .await
            .map_err(|e| {
                error!("重新配置状态失败: {}", e);
                error_utils::state_error(format!("Failed to reconfigure state: {}", e))
            })?;
        self.base.state = Arc::new(state);
        info!("插件注册成功");
        Ok(())
    }

    async fn unregister_plugin(
        &mut self,
        plugin_key: String,
    ) -> EditorResult<()> {
        info!("正在注销插件: {}", plugin_key);
        let ps = self.get_state().plugins().iter().filter(|p| p.key != plugin_key).cloned().collect();
        let state = self
            .get_state()
            .reconfigure(StateConfig {
                schema: Some(self.get_schema().clone()),
                doc: Some(self.get_state().doc()),
                stored_marks: None,
                plugins: Some(ps),
            })
            .await
            .map_err(|e| {
                error!("重新配置状态失败: {}", e);
                error_utils::state_error(format!("Failed to reconfigure state: {}", e))
            })?;
        self.base.state = Arc::new(state);
        info!("插件注销成功");
        Ok(())
    }

    fn undo(&mut self) {
        debug!("执行撤销操作");
        self.base.undo()
    }

    fn redo(&mut self) {
        debug!("执行重做操作");
        self.base.redo()
    }
}
