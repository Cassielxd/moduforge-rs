use std::sync::Arc;

use crate::{
    event::{Event, EventBus},
    extension_manager::ExtensionManager,
    flow::{FlowEngine, ProcessorResult},
    helpers::create_doc,
    history_manager::HistoryManager,
    traits::{EditorBase, EditorCore},
    types::EditorOptions,
    middleware::{Middleware, MiddlewareStack},
};
use async_trait::async_trait;
use moduforge_model::{node_pool::NodePool, schema::Schema};
use moduforge_state::{
    debug,
    state::{State, StateConfig},
    transaction::{Command, Transaction},
};
/// Editor 结构体代表编辑器的核心功能实现
/// 负责管理文档状态、事件处理、插件系统和存储等核心功能
pub struct Editor {
    base: EditorBase,
    flow_engine: FlowEngine,
    middleware_stack: MiddlewareStack,
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

        let mut runtime = Editor {
            base,
            flow_engine: FlowEngine::new()?,
            middleware_stack: MiddlewareStack::new(),
        };
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

    /// 添加中间件到中间件栈
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

    /// 处理编辑器事务的核心方法
    ///
    /// # 参数
    /// * `transaction` - 要处理的事务对象
    ///
    /// # 返回值
    /// * `Result<(), Self::Error>` - 处理结果，成功返回 Ok(()), 失败返回错误
    async fn dispatch(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), Self::Error> {
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
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("中间件执行超时: {}", e),
                )));
            }
        }

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
        for middleware in &self.middleware_stack.middlewares {
            // 调用中间件的 after_dispatch 方法，添加超时保护
            let timeout = std::time::Duration::from_millis(100);
            let middleware_result = match tokio::time::timeout(
                timeout,
                middleware.after_dispatch(current_state.clone()),
            )
            .await
            {
                Ok(result) => result.map_err(|e| Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("中间件错误: {}", e),
                )))?,
                Err(e) => {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        format!("中间件执行超时: {}", e),
                    )));
                }
            };

            // 如果中间件返回了额外的事务，则应用该事务
            if let Some(transaction) = middleware_result.additional_transaction {
                let (_id, mut rx) = self
                    .flow_engine
                    .submit_transaction((self.base.state.clone(), transaction))
                    .await?;

                let Some(task_result) = rx.recv().await else {
                    continue;
                };
                
                let Some(ProcessorResult { result: Some(result), .. }) =
                    task_result.output
                else {
                    continue;
                };

                current_state = Some(Arc::new(result.state));
               
            }
        }
        if let Some(state) = current_state {
            self.base.state = state;
            self.base.history_manager.insert(self.base.state.clone());
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
