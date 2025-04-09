use std::sync::Arc;

use crate::{
    error::{EditorError, EditorResult, error_utils},
    event::{Event, EventBus},
    extension_manager::ExtensionManager,
    helpers::create_doc,
    history_manager::HistoryManager,
    types::EditorOptions,
    traits::{EditorCore, EditorBase},
    middleware::{Middleware, MiddlewareStack},
};
use async_trait::async_trait;

use moduforge_model::{node_pool::NodePool, schema::Schema};
use moduforge_state::{
    state::{State, StateConfig, TransactionResult},
    transaction::{Command, Transaction},
    {info, debug, error},
};

/// Editor 结构体代表编辑器的核心功能实现
/// 负责管理文档状态、事件处理、插件系统和存储等核心功能
pub struct Editor {
    base: EditorBase,
    middleware_stack: MiddlewareStack,
}

impl Editor {
    /// 创建新的编辑器实例
    /// options: 编辑器配置选项
    pub async fn create(options: EditorOptions) -> EditorResult<Self> {
        info!("正在创建新的编辑器实例");
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
            history_manager: HistoryManager::new(
                state,
                options.get_history_limit(),
            ),
            options,
        };

        let mut runtime =
            Editor { base, middleware_stack: MiddlewareStack::new() };
        runtime.init().await?;
        info!("编辑器实例创建成功");
        Ok(runtime)
    }

    /// 初始化编辑器，设置事件处理器并启动事件循环
    async fn init(&mut self) -> EditorResult<()> {
        debug!("正在初始化编辑器");
        self.base
            .event_bus
            .add_event_handlers(self.base.options.get_event_handlers())
            .await?;
        self.base.event_bus.start_event_loop();
        debug!("事件总线已启动");

        self.base
            .event_bus
            .broadcast_blocking(Event::Create(self.base.state.clone()))
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
    pub async fn destroy(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("正在销毁编辑器实例");
        // 广播销毁事件
        self.base.event_bus.broadcast(Event::Destroy).await?;
        // 停止事件循环
        self.base.event_bus.broadcast(Event::Stop).await?;
        debug!("编辑器实例销毁成功");
        Ok(())
    }

    /// Add a middleware to the stack
    pub fn add_middleware<M>(
        &mut self,
        middleware: M,
    ) where
        M: Middleware + 'static,
    {
        self.middleware_stack.add(middleware);
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

    /// 处理编辑器事务的核心方法
    ///
    /// # 参数
    /// * `transaction` - 要处理的事务对象
    ///
    /// # 返回值
    /// * `EditorResult<()>` - 处理结果，成功返回 Ok(()), 失败返回错误
    async fn dispatch(
        &mut self,
        transaction: Transaction,
    ) -> EditorResult<()> {
        // 保存当前事务的副本，用于中间件处理
        let mut current_transaction = transaction;

        // 执行前置中间件链，允许中间件在事务应用前修改事务
        for middleware in &self.middleware_stack.middlewares {
            // 添加超时保护，防止中间件执行时间过长
            let timeout = std::time::Duration::from_millis(100);
            if let Err(e) = tokio::time::timeout(
                timeout,
                middleware.before_dispatch(&mut current_transaction),
            )
            .await
            {
                return Err(error_utils::middleware_error(format!(
                    "Middleware execution timeout: {}",
                    e
                )));
            }
        }

        // 应用事务到编辑器状态，获取新的状态和产生的事务列表
        let TransactionResult { state, transactions } =
            self.base.state.apply(current_transaction).await.map_err(|e| {
                error_utils::state_error(format!(
                    "Failed to apply transaction: {}",
                    e
                ))
            })?;

        // 使用 Option 来避免不必要的克隆
        let mut state_update = None;

        // 检查最后一个事务是否改变了文档
        if let Some(tr) = transactions.last() {
            if tr.doc_changed() {
                // 如果文档发生变化，更新当前状态并广播事务应用事件
                state_update = Some(Arc::new(state));

                // 使用 clone 的引用计数而不是深度克隆
                let transactions = Arc::new(transactions);
                let current_state = self.base.state.clone();

                self.base
                    .event_bus
                    .broadcast(Event::TrApply(transactions, current_state))
                    .await
                    .map_err(|e| {
                        error_utils::event_error(format!(
                            "Failed to broadcast event: {}",
                            e
                        ))
                    })?;
            }
        }

        // 执行后置中间件链，允许中间件在事务应用后执行额外操作
        for middleware in &self.middleware_stack.middlewares {
            // 调用中间件的 after_dispatch 方法，添加超时保护
            let timeout = std::time::Duration::from_millis(100);
            let middleware_result = match tokio::time::timeout(
                timeout,
                middleware.after_dispatch(state_update.clone()),
            )
            .await
            {
                Ok(result) => result?,
                Err(e) => {
                    return Err(error_utils::middleware_error(format!(
                        "Middleware execution timeout: {}",
                        e
                    )));
                },
            };

            // 如果中间件返回了额外的事务，则应用该事务
            if let Some(transaction) = middleware_result.additional_transaction
            {
                let TransactionResult { state, transactions: _ } =
                    self.base.state.apply(transaction).await.map_err(|e| {
                        error_utils::state_error(format!(
                            "Failed to apply additional transaction: {}",
                            e
                        ))
                    })?;
                state_update = Some(Arc::new(state));
            }
        }

        // 如果有新的状态，更新编辑器状态并记录到历史记录
        if let Some(state) = state_update {
            self.base.state = state;
            // 使用 clone 的引用计数
            self.base.history_manager.insert(self.base.state.clone());
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
                error_utils::state_error(format!(
                    "Failed to reconfigure state: {}",
                    e
                ))
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
            .await
            .map_err(|e| {
                error!("重新配置状态失败: {}", e);
                error_utils::state_error(format!(
                    "Failed to reconfigure state: {}",
                    e
                ))
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
