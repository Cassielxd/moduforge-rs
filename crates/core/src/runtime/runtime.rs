use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;

use crate::{
    config::ForgeConfig,
    debug::{debug, info},
    error::{error_utils, ForgeResult},
    event::{Event, EventBus},
    extension_manager::ExtensionManager,
    helpers::{
        create_doc, event_helper::EventHelper, history_helper::HistoryHelper,
        middleware_helper::MiddlewareHelper,
    },
    history_manager::HistoryManager,
    metrics,
    runtime::{runtime_trait::RuntimeTrait, sync_flow::FlowEngine},
    types::{HistoryEntryWithMeta, ProcessorResult, RuntimeOptions},
};

use mf_model::{node_pool::NodePool, schema::Schema};
use mf_state::{
    ops::GlobalResourceManager,
    state::{State, StateConfig},
    transaction::{Command, Transaction},
};

/// Editor 结构体代表编辑器的核心功能实现
/// 负责管理文档状态、事件处理、插件系统和存储等核心功能
pub struct ForgeRuntime {
    event_bus: EventBus<Event>,
    state: Arc<State>,
    flow_engine: Arc<FlowEngine>,
    extension_manager: ExtensionManager,
    history_manager: HistoryManager<HistoryEntryWithMeta>,
    options: RuntimeOptions,
    config: ForgeConfig,
}
impl ForgeRuntime {
    /// 创建新的编辑器实例
    ///
    /// 此方法会自动从以下位置加载XML schema配置：
    /// 1. 优先使用 `config.extension.xml_schema_paths` 中配置的路径
    /// 2. 如果没有配置，则尝试加载默认的 `schema/main.xml`
    /// 3. 如果都没有，则使用默认配置
    ///
    /// # 参数
    /// * `options` - 编辑器配置选项
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - 编辑器实例或错误
    #[cfg_attr(
        feature = "dev-tracing",
        tracing::instrument(
            skip(options),
            fields(crate_name = "core", runtime_type = "sync")
        )
    )]
    pub async fn create(options: RuntimeOptions) -> ForgeResult<Self> {
        Self::create_with_config(options, ForgeConfig::default()).await
    }

    /// 从指定路径的XML schema文件创建编辑器实例
    ///
    /// # 参数
    /// * `xml_schema_path` - XML schema文件路径
    /// * `options` - 可选的RuntimeOptions配置
    /// * `config` - 可选的ForgeConfig配置
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - 编辑器实例或错误
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::ForgeRuntime;
    ///
    /// // 从指定路径加载schema
    /// let runtime = ForgeRuntime::from_xml_schema_path(
    ///     "./schemas/document.xml",
    ///     None,
    ///     None
    /// ).await?;
    /// ```
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(options, config), fields(
        crate_name = "core",
        schema_path = xml_schema_path
    )))]
    pub async fn from_xml_schema_path(
        xml_schema_path: &str,
        options: Option<RuntimeOptions>,
        config: Option<ForgeConfig>,
    ) -> ForgeResult<Self> {
        let mut config = config.unwrap_or_default();
        config.extension.xml_schema_paths = vec![xml_schema_path.to_string()];
        Self::create_with_config(options.unwrap_or_default(), config).await
    }

    /// 合并RuntimeOptions和ExtensionManager
    ///
    /// # 参数
    /// * `options` - 可选的RuntimeOptions
    /// * `extension_manager` - ExtensionManager实例
    ///
    /// # 返回值
    /// * `RuntimeOptions` - 合并后的选项
    fn merge_options_with_extensions(
        options: Option<RuntimeOptions>,
        extension_manager: ExtensionManager,
    ) -> RuntimeOptions {
        match options {
            Some(opts) => {
                // 从ExtensionManager获取extensions并合并到现有选项中
                let schema = extension_manager.get_schema();
                let mut xml_extensions = Vec::new();
                let factory = schema.factory();
                let (nodes, marks) = factory.definitions();
                // 重建节点扩展
                for (name, node_type) in nodes {
                    let node =
                        crate::node::Node::create(name, node_type.spec.clone());
                    xml_extensions.push(crate::types::Extensions::N(node));
                }

                // 重建标记扩展
                for (name, mark_type) in marks {
                    let mark =
                        crate::mark::Mark::new(name, mark_type.spec.clone());
                    xml_extensions.push(crate::types::Extensions::M(mark));
                }

                // 合并扩展（XML扩展优先）
                let existing_extensions = opts.get_extensions();
                xml_extensions.extend(existing_extensions);
                opts.set_extensions(xml_extensions)
            },
            None => {
                // 如果没有提供选项，从ExtensionManager创建新的
                RuntimeOptions::from_extension_manager(extension_manager)
            },
        }
    }

    /// 从多个XML schema文件创建编辑器实例
    ///
    /// # 参数
    /// * `xml_schema_paths` - XML schema文件路径列表
    /// * `options` - 可选的RuntimeOptions配置
    /// * `config` - 可选的ForgeConfig配置
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - 编辑器实例或错误
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(xml_schema_paths, options, config), fields(
        crate_name = "core",
        schema_count = xml_schema_paths.len(),
        runtime_type = "sync"
    )))]
    pub async fn from_xml_schemas(
        xml_schema_paths: &[&str],
        options: Option<RuntimeOptions>,
        config: Option<ForgeConfig>,
    ) -> ForgeResult<Self> {
        let mut config = config.unwrap_or_default();
        config.extension.xml_schema_paths =
            xml_schema_paths.iter().map(|s| s.to_string()).collect();
        Self::create_with_config(options.unwrap_or_default(), config).await
    }

    /// 从XML内容字符串创建编辑器实例
    ///
    /// # 参数
    /// * `xml_content` - XML schema内容
    /// * `options` - 可选的RuntimeOptions配置
    /// * `config` - 可选的ForgeConfig配置
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - 编辑器实例或错误
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::{ForgeRuntime, types::RuntimeOptions};
    ///
    /// let xml = r#"<schema>...</schema>"#;
    ///
    /// // 使用默认选项
    /// let runtime = ForgeRuntime::from_xml_content(xml, None, None).await?;
    ///
    /// // 使用自定义选项
    /// let mut options = RuntimeOptions::default();
    /// options.set_history_limit(Some(100));
    /// let runtime = ForgeRuntime::from_xml_content(xml, Some(options), None).await?;
    /// ```
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(xml_content, options, config), fields(
        crate_name = "core",
        content_size = xml_content.len(),
        runtime_type = "sync"
    )))]
    pub async fn from_xml_content(
        xml_content: &str,
        options: Option<RuntimeOptions>,
        config: Option<ForgeConfig>,
    ) -> ForgeResult<Self> {
        let extension_manager = ExtensionManager::from_xml_string(xml_content)?;
        let final_options =
            Self::merge_options_with_extensions(options, extension_manager);

        Self::create_with_config(final_options, config.unwrap_or_default())
            .await
    }

    /// 使用指定配置创建编辑器实例
    ///
    /// 此方法会自动从以下位置加载XML schema配置：
    /// 1. 优先使用 `config.extension.xml_schema_paths` 中配置的路径
    /// 2. 如果没有配置，则尝试加载默认的 `schema/main.xml`
    /// 3. 如果都没有，则使用默认配置
    ///
    /// # 参数
    /// * `options` - 编辑器配置选项
    /// * `config` - 编辑器配置
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - 编辑器实例或错误
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(options, config), fields(
        crate_name = "core",
        runtime_type = "sync",
        has_middleware = !options.get_middleware_stack().is_empty()
    )))]
    pub async fn create_with_config(
        options: RuntimeOptions,
        config: ForgeConfig,
    ) -> ForgeResult<Self> {
        let start_time = Instant::now();
        info!("正在创建新的编辑器实例");

        // 构建扩展管理器 - 自动处理XML schema
        let extension_manager =
            Self::create_extension_manager(&options, &config)?;

        debug!("已初始化扩展管理器");

        let op_state = GlobalResourceManager::new();
        for op_fn in extension_manager.get_op_fns() {
            op_fn(&op_state)?;
        }

        let mut state_config = StateConfig {
            schema: Some(extension_manager.get_schema()),
            doc: None,
            stored_marks: None,
            plugins: Some(extension_manager.get_plugins().clone()),
            resource_manager: Some(Arc::new(op_state)),
        };
        create_doc::create_doc(&options.get_content(), &mut state_config)
            .await?;
        let state: State = State::create(state_config).await?;

        let state: Arc<State> = Arc::new(state);
        debug!("已创建编辑器状态");

        // 使用 EventHelper 创建并初始化事件总线
        let event_bus = EventHelper::create_and_init_event_bus(
            &config,
            &options,
            state.clone(),
        )
        .await?;

        let runtime = ForgeRuntime {
            event_bus,
            state: state.clone(),
            flow_engine: Arc::new(FlowEngine::new()?),
            extension_manager,
            history_manager: HistoryManager::with_config(
                HistoryEntryWithMeta::new(
                    state.clone(),
                    "创建工程项目".to_string(),
                    serde_json::Value::Null,
                ),
                config.history.clone(),
            ),
            options,
            config,
        };
        info!("编辑器实例创建成功");
        metrics::editor_creation_duration(start_time.elapsed());
        Ok(runtime)
    }

    /// 创建扩展管理器 - 自动处理XML schema配置
    ///
    /// # 参数
    /// * `options` - 运行时选项
    /// * `config` - 编辑器配置
    ///
    /// # 返回值
    /// * `ForgeResult<ExtensionManager>` - 扩展管理器实例或错误
    fn create_extension_manager(
        options: &RuntimeOptions,
        config: &ForgeConfig,
    ) -> ForgeResult<ExtensionManager> {
        crate::helpers::runtime_common::ExtensionManagerHelper::create_extension_manager(
            options, config,
        )
    }

    /// 销毁编辑器实例
    #[cfg_attr(
        feature = "dev-tracing",
        tracing::instrument(skip(self), fields(crate_name = "core"))
    )]
    pub async fn destroy(&mut self) -> ForgeResult<()> {
        debug!("正在销毁编辑器实例");
        EventHelper::destroy_event_bus(&mut self.event_bus).await?;
        debug!("编辑器实例销毁成功");
        Ok(())
    }

    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, event), fields(
        crate_name = "core",
        event_type = std::any::type_name_of_val(&event)
    )))]
    pub async fn emit_event(
        &mut self,
        event: Event,
    ) -> ForgeResult<()> {
        EventHelper::emit_event(&mut self.event_bus, event).await
    }
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        crate_name = "core",
        tr_id = %transaction.id,
        middleware_count = self.options.get_middleware_stack().middlewares.len()
    )))]
    pub async fn run_before_middleware(
        &mut self,
        transaction: &mut Transaction,
    ) -> ForgeResult<()> {
        MiddlewareHelper::run_before_middleware(
            transaction,
            &self.options.get_middleware_stack(),
            &self.config,
        )
        .await
    }
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, state, transactions), fields(
        crate_name = "core",
        has_state = state.is_some(),
        tr_count = transactions.len(),
        middleware_count = self.options.get_middleware_stack().middlewares.len()
    )))]
    pub async fn run_after_middleware(
        &mut self,
        state: &mut Option<Arc<State>>,
        transactions: &mut Vec<Arc<Transaction>>,
    ) -> ForgeResult<()> {
        debug!("执行后置中间件链");
        for middleware in &self.options.get_middleware_stack().middlewares {
            let start_time = Instant::now();
            let timeout = std::time::Duration::from_millis(
                self.config.performance.middleware_timeout_ms,
            );
            let middleware_result = match tokio::time::timeout(
                timeout,
                middleware.after_dispatch(state.clone(), transactions),
            )
            .await
            {
                Ok(Ok(result)) => {
                    metrics::middleware_execution_duration(
                        start_time.elapsed(),
                        "after",
                        middleware.name().as_str(),
                    );
                    result
                },
                Ok(Err(e)) => {
                    return Err(error_utils::middleware_error(format!(
                        "后置中间件执行失败: {e}"
                    )));
                },
                Err(_) => {
                    return Err(error_utils::middleware_error(format!(
                        "后置中间件执行超时（{}ms）",
                        self.config.performance.middleware_timeout_ms
                    )));
                },
            };

            if let Some(mut transaction) = middleware_result {
                // 由运行时统一提交事务，再通过相同的 flow 引擎处理
                transaction.commit()?;
                let task_result = self
                    .flow_engine
                    .submit((self.state.clone(), transaction))
                    .await;
                let Some(ProcessorResult { result: Some(result), .. }) =
                    task_result.output
                else {
                    return Err(error_utils::state_error(
                        "附加事务处理结果无效".to_string(),
                    ));
                };
                *state = Some(result.state);
                transactions.extend(result.transactions);
            }
        }
        Ok(())
    }
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, command), fields(
        crate_name = "core",
        command_name = %command.name()
    )))]
    pub async fn command(
        &mut self,
        command: Arc<dyn Command>,
    ) -> ForgeResult<()> {
        debug!("正在执行命令: {}", command.name());
        metrics::command_executed(command.name().as_str());
        let mut tr = self.get_tr();
        command.execute(&mut tr).await?;
        tr.commit()?;
        self.dispatch(tr).await
    }

    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, command, meta), fields(
        crate_name = "core",
        command_name = %command.name(),
        description = %description
    )))]
    pub async fn command_with_meta(
        &mut self,
        command: Arc<dyn Command>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        debug!("正在执行命令: {}", command.name());
        metrics::command_executed(command.name().as_str());
        let mut tr = self.get_tr();
        command.execute(&mut tr).await?;
        tr.commit()?;
        self.dispatch_with_meta(tr, description, meta).await
    }

    /// 处理编辑器事务的核心方法
    ///
    /// # 参数
    /// * `transaction` - 要处理的事务对象
    ///
    /// # 返回值
    /// * `EditorResult<()>` - 处理结果，成功返回 Ok(()), 失败返回错误
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        crate_name = "core",
        tr_id = %transaction.id,
        runtime_type = "sync"
    )))]
    pub async fn dispatch(
        &mut self,
        transaction: Transaction,
    ) -> ForgeResult<()> {
        self.dispatch_with_meta(
            transaction,
            "".to_string(),
            serde_json::Value::Null,
        )
        .await
    }
    /// 更新编辑器状态并记录到历史记录 包含描述和元信息
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction, meta), fields(
        crate_name = "core",
        tr_id = %transaction.id,
        description = %description,
        runtime_type = "sync"
    )))]
    pub async fn dispatch_with_meta(
        &mut self,
        transaction: Transaction,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        metrics::transaction_dispatched();
        let old_id = self.get_state().version;
        // 保存当前事务的副本，用于中间件处理
        let mut current_transaction = transaction;
        self.run_before_middleware(&mut current_transaction).await?;

        // 应用事务到编辑器状态，获取新的状态和产生的事务列表
        let task_result = self
            .flow_engine
            .submit((self.state.clone(), current_transaction.clone()))
            .await;
        let Some(ProcessorResult { result: Some(result), .. }) =
            task_result.output
        else {
            return Err(error_utils::state_error(
                "任务处理结果无效".to_string(),
            ));
        };
        // 使用 Option 来避免不必要的克隆
        let mut state_update = None;
        let mut transactions = Vec::new();
        transactions.extend(result.transactions);
        // 检查最后一个事务是否改变了文档
        if transactions.last().is_some() {
            state_update = Some(result.state);
        }
        // 执行后置中间件链，允许中间件在事务应用后执行额外操作
        self.run_after_middleware(&mut state_update, &mut transactions).await?;

        // 如果有新的状态，更新编辑器状态并记录到历史记录
        if let Some(state) = state_update {
            self.update_state_with_meta(state.clone(), description, meta)
                .await?;
            self.emit_event(Event::TrApply(old_id, transactions, state))
                .await?;
        }
        Ok(())
    }
    /// 更新编辑器状态并记录到历史记录 不包含描述和元信息
    pub async fn update_state(
        &mut self,
        state: Arc<State>,
    ) -> ForgeResult<()> {
        self.update_state_with_meta(
            state,
            "".to_string(),
            serde_json::Value::Null,
        )
        .await
    }
    /// 更新编辑器状态并记录到历史记录 包含描述和元信息
    pub async fn update_state_with_meta(
        &mut self,
        state: Arc<State>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        self.state = state.clone();
        HistoryHelper::insert(
            &mut self.history_manager,
            state,
            description,
            meta,
        );
        Ok(())
    }

    pub async fn register_plugin(&mut self) -> ForgeResult<()> {
        info!("正在注册新插件");
        let state = self
            .get_state()
            .reconfigure(StateConfig {
                schema: Some(self.get_schema()),
                doc: Some(self.get_state().doc()),
                stored_marks: None,
                plugins: Some(self.get_state().plugins().await),
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
    ) -> ForgeResult<()> {
        info!("正在注销插件: {}", plugin_key);
        let ps = self
            .get_state()
            .plugins()
            .await
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

    pub fn get_options(&self) -> &RuntimeOptions {
        &self.options
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &ForgeConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(
        &mut self,
        config: ForgeConfig,
    ) {
        self.config = config;
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
        self.get_state().tr()
    }

    pub fn undo(&mut self) {
        self.state = HistoryHelper::undo(&mut self.history_manager);
    }

    pub fn redo(&mut self) {
        self.state = HistoryHelper::redo(&mut self.history_manager);
    }

    pub fn jump(
        &mut self,
        n: isize,
    ) {
        self.state = HistoryHelper::jump(&mut self.history_manager, n);
    }
    pub fn get_history_manager(&self) -> &HistoryManager<HistoryEntryWithMeta> {
        &self.history_manager
    }
}
impl Drop for ForgeRuntime {
    fn drop(&mut self) {
        // 在 Drop 中只能使用同步方法
        EventHelper::destroy_event_bus_blocking(&mut self.event_bus);
    }
}

// ==================== RuntimeTrait 实现 ====================

#[async_trait]
impl RuntimeTrait for ForgeRuntime {
    async fn dispatch(
        &mut self,
        transaction: Transaction,
    ) -> ForgeResult<()> {
        self.dispatch(transaction).await
    }

    async fn dispatch_with_meta(
        &mut self,
        transaction: Transaction,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        self.dispatch_with_meta(transaction, description, meta).await
    }

    async fn command(
        &mut self,
        command: Arc<dyn Command>,
    ) -> ForgeResult<()> {
        self.command(command).await
    }

    async fn command_with_meta(
        &mut self,
        command: Arc<dyn Command>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        self.command_with_meta(command, description, meta).await
    }

    async fn get_state(&self) -> ForgeResult<Arc<State>> {
        Ok(self.get_state().clone())
    }

    async fn get_tr(&self) -> ForgeResult<Transaction> {
        Ok(self.get_tr())
    }

    async fn get_schema(&self) -> ForgeResult<Arc<Schema>> {
        Ok(self.get_schema())
    }

    async fn undo(&mut self) -> ForgeResult<()> {
        self.undo();
        Ok(())
    }

    async fn redo(&mut self) -> ForgeResult<()> {
        self.redo();
        Ok(())
    }

    async fn jump(
        &mut self,
        steps: isize,
    ) -> ForgeResult<()> {
        self.jump(steps);
        Ok(())
    }

    fn get_config(&self) -> &ForgeConfig {
        self.get_config()
    }

    fn update_config(
        &mut self,
        config: ForgeConfig,
    ) {
        self.update_config(config);
    }

    fn get_options(&self) -> &RuntimeOptions {
        self.get_options()
    }

    async fn destroy(&mut self) -> ForgeResult<()> {
        self.destroy().await
    }
}
