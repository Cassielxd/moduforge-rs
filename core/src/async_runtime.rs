use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::{
    event::{Event, EventBus},
    extension_manager::ExtensionManager,
    flow::{FlowEngine, ProcessorResult},
    helpers::create_doc,
    history_manager::HistoryManager,
    middleware::MiddlewareStack,
    types::EditorOptions,
    EditorResult,
};
use moduforge_state::{
    debug,
    state::{State, StateConfig},
    transaction::Transaction,
};
use crate::runtime::Editor;
/// Editor 结构体代表编辑器的核心功能实现
/// 负责管理文档状态、事件处理、插件系统和存储等核心功能
pub struct AsyncEditor {
    base: Editor,
    flow_engine: FlowEngine,
}
impl Deref for AsyncEditor {
    type Target = Editor;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for AsyncEditor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl AsyncEditor {
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

        let base = Editor {
            event_bus,
            state: state.clone(),
            extension_manager,
            history_manager: HistoryManager::new(
                state,
                options.get_history_limit(),
            ),
            options,
            middleware_stack: MiddlewareStack::new(),
        };

        let mut runtime = AsyncEditor { base, flow_engine: FlowEngine::new()? };
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

    /// 销毁编辑器实例
    pub async fn destroy(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("正在销毁编辑器实例");
        // 广播销毁事件
        self.base.event_bus.broadcast(Event::Destroy).await?;
        // 停止事件循环
        self.base.event_bus.broadcast(Event::Stop).await?;
        debug!("编辑器实例销毁成功");
        Ok(())
    }

    pub async fn dispatch(
        &mut self,
        transaction: Transaction,
    ) -> EditorResult<()> {
        // 保存当前事务的副本，用于中间件处理
        let mut current_transaction = transaction;
        self.run_before_middleware(&mut current_transaction).await?;

        // 使用 flow_engine 提交事务
        let (_id, mut rx) = self
            .flow_engine
            .submit_transaction((self.base.state.clone(), current_transaction))
            .await?;

        // 等待任务结果
        let Some(task_result) = rx.recv().await else {
            return Ok(());
        };

        // 获取处理结果
        let Some(ProcessorResult { result: Some(result), .. }) =
            task_result.output
        else {
            return Ok(());
        };

        // 更新编辑器状态
        let mut current_state = Some(Arc::new(result.state));

        // 检查最后一个事务是否改变了文档
        if let Some(tr) = result.transactions.last() {
            if tr.doc_changed() {
                // 如果文档发生变化，更新历史记录并广播事务应用事件
                self.base.history_manager.insert(self.base.state.clone());

                // 使用 clone 的引用计数而不是深度克隆
                let transactions = Arc::new(result.transactions);

                self.base
                    .event_bus
                    .broadcast(Event::TrApply(
                        transactions,
                        current_state.clone().unwrap(),
                    ))
                    .await?;
            }
        }
        // 执行后置中间件链，允许中间件在事务应用后执行额外操作
        self.run_after_middleware(&mut current_state).await?;
        if let Some(state) = current_state {
            self.base.state = state;
            self.base.history_manager.insert(self.base.state.clone());
        }
        Ok(())
    }
}
