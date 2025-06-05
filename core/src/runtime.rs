use std::sync::Arc;

use crate::{
    error::{error_utils, EditorResult},
    event::{Event, EventBus},
    extension_manager::ExtensionManager,
    helpers::create_doc,
    history_manager::HistoryManager,
    types::EditorOptions,
};

use moduforge_model::{node_pool::NodePool, schema::Schema};
use moduforge_state::{
    debug, error, info,
    ops::GlobalResourceManager,
    state::{State, StateConfig, TransactionResult},
    transaction::{Command, Transaction},
};

/// 默认中间件超时时间（毫秒）
const DEFAULT_MIDDLEWARE_TIMEOUT_MS: u64 = 500;

/// Editor 结构体代表编辑器的核心功能实现
/// 负责管理文档状态、事件处理、插件系统和存储等核心功能
pub struct Editor {
    event_bus: EventBus<Event>,
    state: Arc<State>,
    extension_manager: ExtensionManager,
    history_manager: HistoryManager<Arc<State>>,
    options: EditorOptions,
}
unsafe impl Send for Editor {}
unsafe impl Sync for Editor {}
impl Editor {
    /// 创建新的编辑器实例
    /// options: 编辑器配置选项
    pub async fn create(options: EditorOptions) -> EditorResult<Self> {
        info!("正在创建新的编辑器实例");
        let extension_manager =
            ExtensionManager::new(&options.get_extensions())?;
        debug!("已初始化扩展管理器");

        let event_bus = EventBus::new();
        debug!("已创建文档和事件总线");
        let op_state = GlobalResourceManager::new();
        for op_fn in extension_manager.get_op_fns() {
            op_fn(&op_state)?;
        }

        let mut config = StateConfig {
            schema: Some(extension_manager.get_schema()),
            doc: None,
            stored_marks: None,
            plugins: Some(extension_manager.get_plugins().clone()),
            resource_manager: Some(Arc::new(op_state)),
        };
        create_doc::create_doc(&options.get_content(), &mut config).await?;
        let state: State = State::create(config).await?;

        let state: Arc<State> = Arc::new(state);
        debug!("已创建编辑器状态");

        let mut runtime = Editor {
            event_bus,
            state: state.clone(),
            extension_manager,
            history_manager: HistoryManager::new(
                state,
                options.get_history_limit(),
            ),
            options,
        };
        runtime.init().await?;
        info!("编辑器实例创建成功");
        Ok(runtime)
    }

    /// 初始化编辑器，设置事件处理器并启动事件循环
    async fn init(&mut self) -> EditorResult<()> {
        debug!("正在初始化编辑器");
        self.event_bus.add_event_handlers(self.options.get_event_handlers())?;
        self.event_bus.start_event_loop();
        debug!("事件总线已启动");

        self.event_bus
            .broadcast_blocking(Event::Create(self.state.clone()))
            .map_err(|e| {
                error!("广播创建事件失败: {}", e);
                error_utils::event_error(format!(
                    "Failed to broadcast create event: {}",
                    e
                ))
            })?;
        debug!("已广播创建事件");
        Ok(())
    }

    /// 销毁编辑器实例
    pub async fn destroy(&mut self) -> EditorResult<()> {
        debug!("正在销毁编辑器实例");
        // 广播销毁事件
        self.event_bus.broadcast(Event::Destroy).await?;
        // 停止事件循环
        self.event_bus.broadcast(Event::Stop).await?;
        debug!("编辑器实例销毁成功");
        Ok(())
    }

    pub async fn emit_event(
        &mut self,
        event: Event,
    ) -> EditorResult<()> {
        self.event_bus.broadcast(event).await?;
        Ok(())
    }
    pub async fn run_before_middleware(
        &mut self,
        transaction: &mut Transaction,
    ) -> EditorResult<()> {
        debug!("执行前置中间件链");
        for middleware in &self.options.get_middleware_stack().middlewares {
            let timeout = std::time::Duration::from_millis(DEFAULT_MIDDLEWARE_TIMEOUT_MS);
            match tokio::time::timeout(
                timeout,
                middleware.before_dispatch(transaction),
            )
            .await
            {
                Ok(Ok(())) => {
                    // 中间件执行成功
                    continue;
                },
                Ok(Err(e)) => {
                    return Err(error_utils::middleware_error(format!(
                        "前置中间件执行失败: {}",
                        e
                    )));
                },
                Err(_) => {
                    return Err(error_utils::middleware_error(format!(
                        "前置中间件执行超时（{}ms）",
                        DEFAULT_MIDDLEWARE_TIMEOUT_MS
                    )));
                },
            }
        }
        transaction.commit();
        Ok(())
    }
    pub async fn run_after_middleware(
        &mut self,
        state: &mut Option<Arc<State>>,
        transactions: &mut Vec<Transaction>,
    ) -> EditorResult<()> {
        debug!("执行后置中间件链");
        for middleware in &self.options.get_middleware_stack().middlewares {
            let timeout = std::time::Duration::from_millis(DEFAULT_MIDDLEWARE_TIMEOUT_MS);
            let middleware_result = match tokio::time::timeout(
                timeout,
                middleware.after_dispatch(state.clone(), transactions),
            )
            .await
            {
                Ok(Ok(result)) => result,
                Ok(Err(e)) => {
                    return Err(error_utils::middleware_error(format!(
                        "后置中间件执行失败: {}",
                        e
                    )));
                },
                Err(_) => {
                    return Err(error_utils::middleware_error(format!(
                        "后置中间件执行超时（{}ms）",
                        DEFAULT_MIDDLEWARE_TIMEOUT_MS
                    )));
                },
            };

            if let Some(mut transaction) = middleware_result.additional_transaction
            {
                transaction.commit();
                let TransactionResult { state: new_state, transactions: trs } =
                    self.state.apply(transaction).await.map_err(|e| {
                        error_utils::state_error(format!(
                            "附加事务应用失败: {}",
                            e
                        ))
                    })?;
                *state = Some(Arc::new(new_state));
                transactions.extend(trs);
            }
        }
        Ok(())
    }
    pub async fn command(
        &mut self,
        command: Arc<dyn Command>,
    ) -> EditorResult<()> {
        debug!("正在执行命令: {}", command.name());
        let mut tr = self.get_tr();
        command.execute(&mut tr).await?;
        tr.commit();
        self.dispatch(tr).await
    }

    /// 处理编辑器事务的核心方法
    ///
    /// # 参数
    /// * `transaction` - 要处理的事务对象
    ///
    /// # 返回值
    /// * `EditorResult<()>` - 处理结果，成功返回 Ok(()), 失败返回错误
    pub async fn dispatch(
        &mut self,
        transaction: Transaction,
    ) -> EditorResult<()> {
        let old_id = self.get_state().version;
        // 保存当前事务的副本，用于中间件处理
        let mut current_transaction = transaction;
        self.run_before_middleware(&mut current_transaction).await?;

        // 应用事务到编辑器状态，获取新的状态和产生的事务列表
        let TransactionResult { state, mut transactions } =
            self.state.apply(current_transaction).await?;

        // 使用 Option 来避免不必要的克隆
        let mut state_update = None;

        // 检查最后一个事务是否改变了文档
        if let Some(tr) = transactions.last() {
            if tr.doc_changed() {
                // 如果文档发生变化，更新当前状态并广播事务应用事件
                state_update = Some(Arc::new(state));
            }
        }
        // 执行后置中间件链，允许中间件在事务应用后执行额外操作
        self.run_after_middleware(&mut state_update, &mut transactions).await?;

        // 如果有新的状态，更新编辑器状态并记录到历史记录
        if let Some(state) = state_update {
            self.update_state(state.clone()).await?;
            self.emit_event(Event::TrApply(
                old_id,
                Arc::new(transactions),
                state,
            ))
            .await?;
        }

        Ok(())
    }
    pub async fn update_state(
        &mut self,
        state: Arc<State>,
    ) -> EditorResult<()> {
        self.state = state;
        self.history_manager.insert(self.state.clone());
        Ok(())
    }

    pub async fn register_plugin(&mut self) -> EditorResult<()> {
        info!("正在注册新插件");
        let state = self
            .get_state()
            .reconfigure(StateConfig {
                schema: Some(self.get_schema()),
                doc: Some(self.get_state().doc()),
                stored_marks: None,
                plugins: Some(self.get_state().plugins().clone()),
                resource_manager: Some(
                    self.get_state().resource_manager().clone(),
                ),
            })
            .await?;
        self.update_state(Arc::new(state)).await?;
        info!("插件注册成功");
        Ok(())
    }

    pub async fn unregister_plugin(
        &mut self,
        plugin_key: String,
    ) -> EditorResult<()> {
        info!("正在注销插件: {}", plugin_key);
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
                resource_manager: Some(
                    self.get_state().resource_manager().clone(),
                ),
            })
            .await?;
        self.update_state(Arc::new(state)).await?;
        info!("插件注销成功");
        Ok(())
    }

    /// 共享的基础实现方法
    pub fn doc(&self) -> Arc<NodePool> {
        self.state.doc()
    }

    pub fn get_options(&self) -> &EditorOptions {
        &self.options
    }

    pub fn get_state(&self) -> &Arc<State> {
        &self.state
    }

    pub fn get_schema(&self) -> Arc<Schema> {
        self.extension_manager.get_schema()
    }

    pub fn get_event_bus(&self) -> &EventBus<Event> {
        &self.event_bus
    }

    pub fn get_tr(&self) -> Transaction {
        let tr = self.get_state().tr();
        tr
    }

    pub fn undo(&mut self) {
        self.history_manager.jump(-1);
        self.state = self.history_manager.get_present();
    }

    pub fn redo(&mut self) {
        self.history_manager.jump(1);
        self.state = self.history_manager.get_present();
    }

    pub fn jump(
        &mut self,
        n: isize,
    ) {
        self.history_manager.jump(n);
        self.state = self.history_manager.get_present();
    }
}
impl Drop for Editor {
    fn drop(&mut self) {
        self.event_bus.destroy();
    }
}
