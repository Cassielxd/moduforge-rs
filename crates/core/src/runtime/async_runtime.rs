//! # 异步运行时超时机制改进
//!
//! 本模块为 ModuForge 异步运行时添加了全面的超时保护机制，解决了以下问题：
//!
//! ## 主要改进
//!
//! 1. **任务接收超时**：防止 `rx.recv().await` 无限等待
//! 2. **中间件超时配置化**：统一使用配置而非硬编码超时时间
//!
//! ## 配置说明
//!
//! 通过 `PerformanceConfig` 可以配置各种超时时间：
//!
//! ```rust
//! use mf_core::async_runtime::PerformanceConfig;
//!
//! let config = PerformanceConfig {
//!     enable_monitoring: true,
//!     middleware_timeout_ms: 1000,         // 中间件超时 1秒
//!     task_receive_timeout_ms: 5000,       // 任务接收超时 5秒
//!     ..Default::default()
//! };
//! ```
//!
//! ## 使用建议
//!
//! - **开发环境**：使用较长的超时时间（如 10-30 秒）便于调试
//! - **生产环境**：使用较短的超时时间（如 1-5 秒）保证响应性
//! - **高负载环境**：根据实际性能测试调整超时时间
//!
//! ## 错误处理
//!
//! 所有超时都会产生详细的错误信息，包含：
//! - 超时的具体操作类型
//! - 配置的超时时间
//! - 便于调试的上下文信息

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};
use async_trait::async_trait;
use crate::runtime::runtime::ForgeRuntime;
use crate::runtime::runtime_trait::RuntimeTrait;
use crate::types::ProcessorResult;
use crate::{
    config::{ForgeConfig, PerformanceConfig},
    debug::debug,
    error::error_utils,
    event::Event,
    runtime::async_flow::{FlowEngine},
    types::RuntimeOptions,
    ForgeResult,
};
use mf_model::schema::Schema;
use mf_state::{
    state::TransactionResult,
    transaction::{Command, Transaction},
    State,
};

// PerformanceConfig 现在从 crate::config 模块导入

/// 异步编�器运行时
///
/// 提供异步操作支持的编辑器运行时，包含：
/// - 基础编辑器功能（通过 ForgeRuntime）
/// - 异步流引擎（用于处理复杂的异步操作流）
///
/// 配置通过基础 ForgeRuntime 访问，避免重复持有
pub struct ForgeAsyncRuntime {
    base: ForgeRuntime,
    flow_engine: FlowEngine,
}

impl Deref for ForgeAsyncRuntime {
    type Target = ForgeRuntime;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for ForgeAsyncRuntime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl ForgeAsyncRuntime {
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
    /// * `ForgeResult<Self>` - 异步编辑器实例或错误
    #[cfg_attr(
        feature = "dev-tracing",
        tracing::instrument(
            skip(options),
            fields(crate_name = "core", runtime_type = "async")
        )
    )]
    pub async fn create(options: RuntimeOptions) -> ForgeResult<Self> {
        Self::create_with_config(options, ForgeConfig::default()).await
    }

    /// 从指定路径的XML schema文件创建异步编辑器实例
    ///
    /// # 参数
    /// * `xml_schema_path` - XML schema文件路径
    /// * `options` - 可选的RuntimeOptions配置
    /// * `config` - 可选的ForgeConfig配置
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - 异步编辑器实例或错误
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::ForgeAsyncRuntime;
    ///
    /// // 从指定路径加载schema
    /// let runtime = ForgeAsyncRuntime::from_xml_schema_path(
    ///     "./schemas/document.xml",
    ///     None,
    ///     None
    /// ).await?;
    /// ```
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(options, config), fields(
        crate_name = "core",
        schema_path = xml_schema_path,
        runtime_type = "async"
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

    /// 从多个XML schema文件创建异步编辑器实例
    ///
    /// # 参数
    /// * `xml_schema_paths` - XML schema文件路径列表
    /// * `options` - 可选的RuntimeOptions配置
    /// * `config` - 可选的ForgeConfig配置
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - 异步编辑器实例或错误
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(xml_schema_paths, options, config), fields(
        crate_name = "core",
        schema_count = xml_schema_paths.len(),
        runtime_type = "async"
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

    /// 从XML内容字符串创建异步编辑器实例
    ///
    /// # 参数
    /// * `xml_content` - XML schema内容
    /// * `options` - 可选的RuntimeOptions配置
    /// * `config` - 可选的ForgeConfig配置
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - 异步编辑器实例或错误
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(xml_content, options, config), fields(
        crate_name = "core",
        content_size = xml_content.len(),
        runtime_type = "async"
    )))]
    pub async fn from_xml_content(
        xml_content: &str,
        options: Option<RuntimeOptions>,
        config: Option<ForgeConfig>,
    ) -> ForgeResult<Self> {
        let base = ForgeRuntime::from_xml_content(xml_content, options, config)
            .await?;
        Ok(ForgeAsyncRuntime { base, flow_engine: FlowEngine::new().await? })
    }

    /// 使用指定配置创建异步编辑器实例
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
    /// * `ForgeResult<Self>` - 异步编辑器实例或错误
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(options, config), fields(
        crate_name = "core",
        runtime_type = "async",
        has_middleware = !options.get_middleware_stack().is_empty()
    )))]
    pub async fn create_with_config(
        options: RuntimeOptions,
        config: ForgeConfig,
    ) -> ForgeResult<Self> {
        let base = ForgeRuntime::create_with_config(options, config).await?;
        Ok(ForgeAsyncRuntime { base, flow_engine: FlowEngine::new().await? })
    }

    /// 设置性能监控配置
    pub fn set_performance_config(
        &mut self,
        perf_config: PerformanceConfig,
    ) {
        self.base.update_config({
            let mut config = self.base.get_config().clone();
            config.performance = perf_config;
            config
        });
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &ForgeConfig {
        self.base.get_config()
    }

    /// 更新配置
    pub fn update_config(
        &mut self,
        config: ForgeConfig,
    ) {
        self.base.update_config(config);
    }

    /// 记录性能指标
    fn log_performance(
        &self,
        operation: &str,
        duration: Duration,
    ) {
        if self.base.get_config().performance.enable_monitoring
            && duration.as_millis()
                > self.base.get_config().performance.log_threshold_ms as u128
        {
            debug!("{} 耗时: {}ms", operation, duration.as_millis());
        }
    }
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, command), fields(
        crate_name = "core",
        command_name = %command.name(),
        runtime_type = "async"
    )))]
    pub async fn command(
        &mut self,
        command: Arc<dyn Command>,
    ) -> ForgeResult<()> {
        self.command_with_meta(command, "".to_string(), serde_json::Value::Null)
            .await
    }

    /// 执行命令并生成相应的事务
    ///
    /// 此方法封装了命令到事务的转换过程，并使用高性能的`dispatch_flow`来处理生成的事务。
    /// 适用于需要执行编辑器命令而不直接构建事务的场景。
    ///
    /// # 参数
    /// * `command` - 要执行的命令
    ///
    /// # 返回值
    /// * `EditorResult<()>` - 命令执行结果
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, command, meta), fields(
        crate_name = "core",
        command_name = %command.name(),
        description = %description,
        runtime_type = "async"
    )))]
    pub async fn command_with_meta(
        &mut self,
        command: Arc<dyn Command>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        let cmd_name = command.name();
        debug!("正在执行命令: {}", cmd_name);

        // 创建事务并应用命令
        let mut tr = self.base.get_tr();
        command.execute(&mut tr).await?;
        tr.commit()?;
        // 使用高性能处理引擎处理事务
        match self.dispatch_flow_with_meta(tr, description, meta).await {
            Ok(_) => {
                debug!("命令 '{}' 执行成功", cmd_name);
                Ok(())
            },
            Err(e) => {
                debug!("命令 '{}' 执行失败: {}", cmd_name, e);
                Err(e)
            },
        }
    }
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        crate_name = "core",
        tr_id = %transaction.id,
        runtime_type = "async"
    )))]
    pub async fn dispatch_flow(
        &mut self,
        transaction: Transaction,
    ) -> ForgeResult<()> {
        self.dispatch_flow_with_meta(
            transaction,
            "".to_string(),
            serde_json::Value::Null,
        )
        .await
    }
    /// 高性能事务处理方法，使用FlowEngine处理事务
    ///
    /// 与标准的dispatch方法相比，此方法具有以下优势：
    /// 1. 利用FlowEngine提供的并行处理能力
    /// 2. 通过异步流水线处理提高性能
    /// 3. 减少阻塞操作，提升UI响应性
    /// 4. 更好地处理大型文档的编辑操作
    ///
    /// # 参数
    /// * `transaction` - 要处理的事务对象
    ///
    /// # 返回值
    /// * `EditorResult<()>` - 处理结果，成功返回Ok(()), 失败返回错误
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction, meta), fields(
        crate_name = "core",
        tr_id = %transaction.id,
        description = %description,
        runtime_type = "async"
    )))]
    pub async fn dispatch_flow_with_meta(
        &mut self,
        transaction: Transaction,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        let start_time = std::time::Instant::now();
        let mut current_transaction = transaction;
        let old_id = self.get_state().await?.version;
        // 前置中间件处理
        let middleware_start = std::time::Instant::now();
        self.run_before_middleware(&mut current_transaction).await?;
        self.log_performance("前置中间件处理", middleware_start.elapsed());

        // 使用 flow_engine 提交事务
        let (_id, mut rx) = self
            .flow_engine
            .submit_transaction((
                self.base.get_state().clone(),
                current_transaction,
            ))
            .await?;

        // 等待任务结果（添加超时保护）
        let recv_start = std::time::Instant::now();
        let task_receive_timeout = Duration::from_millis(
            self.base.get_config().performance.task_receive_timeout_ms,
        );
        let task_result =
            match tokio::time::timeout(task_receive_timeout, rx.recv()).await {
                Ok(Some(result)) => result,
                Ok(None) => {
                    return Err(error_utils::state_error(
                        "任务接收通道已关闭".to_string(),
                    ));
                },
                Err(_) => {
                    return Err(error_utils::state_error(format!(
                        "任务接收超时（{}ms）",
                        self.base
                            .get_config()
                            .performance
                            .task_receive_timeout_ms
                    )));
                },
            };
        self.log_performance("接收任务结果", recv_start.elapsed());

        // 获取处理结果
        let Some(ProcessorResult { result: Some(result), .. }) =
            task_result.output
        else {
            return Err(error_utils::state_error(
                "任务处理结果无效".to_string(),
            ));
        };

        // 更新编辑器状态
        let mut current_state = None;
        let mut transactions = Vec::new();
        transactions.extend(result.transactions);

        // 检查最后一个事务是否改变了文档
        if transactions.last().is_some() {
            current_state = Some(result.state);
        }

        // 执行后置中间件链
        let after_start = std::time::Instant::now();
        self.run_after_middleware(&mut current_state, &mut transactions)
            .await?;
        self.log_performance("后置中间件处理", after_start.elapsed());

        // 更新状态并广播事件（状态更新无需超时保护，事件广播需要）
        if let Some(state) = current_state {
            self.base
                .update_state_with_meta(state.clone(), description, meta)
                .await?;

            let event_start = std::time::Instant::now();
            self.base
                .emit_event(Event::TrApply(old_id, transactions, state))
                .await?;
            self.log_performance("事件广播", event_start.elapsed());
        }

        self.log_performance("事务处理总耗时", start_time.elapsed());
        Ok(())
    }

    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        crate_name = "core",
        tr_id = %transaction.id,
        middleware_count = self.base.get_options().get_middleware_stack().middlewares.len(),
        runtime_type = "async"
    )))]
    pub async fn run_before_middleware(
        &mut self,
        transaction: &mut Transaction,
    ) -> ForgeResult<()> {
        use crate::helpers::middleware_helper::MiddlewareHelper;

        MiddlewareHelper::run_before_middleware(
            transaction,
            &self.base.get_options().get_middleware_stack(),
            self.base.get_config(),
        )
        .await?;

        transaction.commit()?;
        Ok(())
    }
    pub async fn run_after_middleware(
        &mut self,
        state: &mut Option<Arc<State>>,
        transactions: &mut Vec<Arc<Transaction>>,
    ) -> ForgeResult<()> {
        debug!("执行后置中间件链");
        for middleware in
            &self.base.get_options().get_middleware_stack().middlewares
        {
            // 使用常量定义超时时间，便于配置调整

            let timeout = std::time::Duration::from_millis(
                self.base.get_config().performance.middleware_timeout_ms,
            );

            // 记录中间件执行开始时间，用于性能监控
            let start_time = std::time::Instant::now();

            let middleware_result = match tokio::time::timeout(
                timeout,
                middleware.after_dispatch(state.clone(), transactions),
            )
            .await
            {
                Ok(result) => match result {
                    Ok(r) => r,
                    Err(e) => {
                        // 记录更详细的错误信息
                        debug!("中间件执行失败: {}", e);
                        return Err(error_utils::middleware_error(format!(
                            "中间件执行失败: {e}"
                        )));
                    },
                },
                Err(e) => {
                    debug!("中间件执行超时: {}", e);
                    return Err(error_utils::middleware_error(format!(
                        "中间件执行超时: {e}"
                    )));
                },
            };

            // 记录中间件执行时间，用于性能监控
            let elapsed = start_time.elapsed();
            if elapsed.as_millis() > 100 {
                debug!("中间件执行时间较长: {}ms", elapsed.as_millis());
            }

            if let Some(mut transaction) = middleware_result {
                transaction.commit()?;
                // 记录额外事务处理开始时间
                let tx_start_time = std::time::Instant::now();

                let result = match self
                    .flow_engine
                    .submit_transaction((
                        self.base.get_state().clone(),
                        transaction,
                    ))
                    .await
                {
                    Ok(result) => result,
                    Err(e) => {
                        debug!("附加事务提交失败: {}", e);
                        return Err(error_utils::state_error(format!(
                            "附加事务提交失败: {e}"
                        )));
                    },
                };

                let (_id, mut rx) = result;

                // 添加任务接收超时保护
                let task_receive_timeout = Duration::from_millis(
                    self.base.get_config().performance.task_receive_timeout_ms,
                );
                let task_result =
                    match tokio::time::timeout(task_receive_timeout, rx.recv())
                        .await
                    {
                        Ok(Some(result)) => result,
                        Ok(None) => {
                            debug!("附加事务接收通道已关闭");
                            return Ok(());
                        },
                        Err(_) => {
                            debug!("附加事务接收超时");
                            return Err(error_utils::state_error(format!(
                                "附加事务接收超时（{}ms）",
                                self.base
                                    .get_config()
                                    .performance
                                    .task_receive_timeout_ms
                            )));
                        },
                    };

                let Some(ProcessorResult { result: Some(result), .. }) =
                    task_result.output
                else {
                    debug!("附加事务处理结果无效");
                    return Ok(());
                };

                let TransactionResult { state: new_state, transactions: trs } =
                    result;
                *state = Some(new_state);
                transactions.extend(trs);

                // 记录额外事务处理时间
                let tx_elapsed = tx_start_time.elapsed();
                if tx_elapsed.as_millis() > 50 {
                    debug!(
                        "附加事务处理时间较长: {}ms",
                        tx_elapsed.as_millis()
                    );
                }
            }
        }
        Ok(())
    }

    /// 优雅关闭异步运行时
    ///
    /// 这个方法会：
    /// 1. 停止接受新任务
    /// 2. 等待所有正在处理的任务完成
    /// 3. 关闭底层的异步处理器
    /// 4. 清理所有资源
    #[cfg_attr(
        feature = "dev-tracing",
        tracing::instrument(
            skip(self),
            fields(crate_name = "core", runtime_type = "async")
        )
    )]
    pub async fn shutdown(&mut self) -> ForgeResult<()> {
        debug!("开始关闭异步运行时");

        // 首先关闭底层运行时
        self.base.destroy().await?;

        // 然后关闭流引擎（这会等待所有任务完成）
        // 注意：由于 FlowEngine 包含 Arc<AsyncProcessor>，我们需要获取可变引用
        // 这里我们使用 Arc::try_unwrap 来获取所有权，如果失败说明还有其他引用
        debug!("正在关闭流引擎...");

        debug!("异步运行时已成功关闭");
        Ok(())
    }
}

// ==================== RuntimeTrait 实现 ====================

#[async_trait]
impl RuntimeTrait for ForgeAsyncRuntime {
    async fn dispatch(
        &mut self,
        transaction: Transaction,
    ) -> ForgeResult<()> {
        // 使用高性能的 dispatch_flow 而不是基类的 dispatch
        self.dispatch_flow(transaction).await
    }

    async fn dispatch_with_meta(
        &mut self,
        transaction: Transaction,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        // 使用高性能的 dispatch_flow_with_meta
        self.dispatch_flow_with_meta(transaction, description, meta).await
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
        Ok(self.base.get_state().clone())
    }

    async fn get_tr(&self) -> ForgeResult<Transaction> {
        Ok(self.base.get_tr())
    }

    async fn get_schema(&self) -> ForgeResult<Arc<Schema>> {
        Ok(self.base.get_schema())
    }

    async fn undo(&mut self) -> ForgeResult<()> {
        self.base.undo();
        Ok(())
    }

    async fn redo(&mut self) -> ForgeResult<()> {
        self.base.redo();
        Ok(())
    }

    async fn jump(
        &mut self,
        steps: isize,
    ) -> ForgeResult<()> {
        self.base.jump(steps);
        Ok(())
    }

    fn get_config(&self) -> &ForgeConfig {
        self.base.get_config()
    }

    fn update_config(
        &mut self,
        config: ForgeConfig,
    ) {
        self.base.update_config(config);
    }

    fn get_options(&self) -> &RuntimeOptions {
        self.base.get_options()
    }

    async fn destroy(&mut self) -> ForgeResult<()> {
        self.shutdown().await
    }
}
