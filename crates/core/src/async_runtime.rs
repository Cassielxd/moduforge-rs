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

use crate::{runtime::ForgeRuntime, types::ProcessorResult};
use crate::{
    error_utils,
    event::Event,
    async_flow::{FlowEngine},
    types::RuntimeOptions,
    ForgeResult,
};
use mf_state::{
    debug,
    state::TransactionResult,
    transaction::{Command, Transaction},
    State,
};

/// 性能监控配置
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// 是否启用性能监控
    pub enable_monitoring: bool,
    /// 中间件执行超时时间（毫秒）
    /// 推荐值：500-2000ms，取决于中间件复杂度
    pub middleware_timeout_ms: u64,
    /// 性能日志记录阈值（毫秒）
    /// 超过此时间的操作将被记录到日志
    pub log_threshold_ms: u64,
    /// 任务接收超时时间（毫秒）
    /// 等待异步任务结果的最大时间
    /// 推荐值：3000-10000ms，取决于任务复杂度
    pub task_receive_timeout_ms: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: false,
            middleware_timeout_ms: 500,
            log_threshold_ms: 50,
            task_receive_timeout_ms: 5000, // 5秒
        }
    }
}

/// Editor 结构体代表编辑器的核心功能实现
/// 负责管理文档状态、事件处理、插件系统和存储等核心功能
pub struct ForgeAsyncRuntime {
    base: ForgeRuntime,
    flow_engine: FlowEngine,
    perf_config: PerformanceConfig,
}
unsafe impl Send for ForgeAsyncRuntime {}
unsafe impl Sync for ForgeAsyncRuntime {}

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
    /// options: 编辑器配置选项
    pub async fn create(options: RuntimeOptions) -> ForgeResult<Self> {
        let base = ForgeRuntime::create(options).await?;
        Ok(ForgeAsyncRuntime {
            base,
            flow_engine: FlowEngine::new()?,
            perf_config: PerformanceConfig::default(),
        })
    }

    /// 设置性能监控配置
    pub fn set_performance_config(
        &mut self,
        config: PerformanceConfig,
    ) {
        self.perf_config = config;
    }

    /// 记录性能指标
    fn log_performance(
        &self,
        operation: &str,
        duration: Duration,
    ) {
        if self.perf_config.enable_monitoring
            && duration.as_millis() > self.perf_config.log_threshold_ms as u128
        {
            debug!("{} 耗时: {}ms", operation, duration.as_millis());
        }
    }
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
    pub async fn command_with_meta(
        &mut self,
        command: Arc<dyn Command>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        let cmd_name = command.name();
        debug!("正在执行命令: {}", cmd_name);

        // 创建事务并应用命令
        let mut tr = self.get_tr();
        command.execute(&mut tr).await?;
        tr.commit();
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
    pub async fn dispatch_flow_with_meta(
        &mut self,
        transaction: Transaction,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        let start_time = std::time::Instant::now();
        let mut current_transaction = transaction;
        let old_id = self.get_state().version;
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
        let task_receive_timeout =
            Duration::from_millis(self.perf_config.task_receive_timeout_ms);
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
                        self.perf_config.task_receive_timeout_ms
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
        if let Some(_) = transactions.last() {
            current_state = Some(Arc::new(result.state));
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
                .emit_event(Event::TrApply(
                    old_id,
                    Arc::new(transactions),
                    state,
                ))
                .await?;
            self.log_performance("事件广播", event_start.elapsed());
        }

        self.log_performance("事务处理总耗时", start_time.elapsed());
        Ok(())
    }

    pub async fn run_before_middleware(
        &mut self,
        transaction: &mut Transaction,
    ) -> ForgeResult<()> {
        debug!("执行前置中间件链");
        for middleware in
            &self.base.get_options().get_middleware_stack().middlewares
        {
            let timeout =
                Duration::from_millis(self.perf_config.middleware_timeout_ms);
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
                        self.perf_config.middleware_timeout_ms
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
    ) -> ForgeResult<()> {
        debug!("执行后置中间件链");
        for middleware in
            &self.base.get_options().get_middleware_stack().middlewares
        {
            // 使用常量定义超时时间，便于配置调整

            let timeout = std::time::Duration::from_millis(
                self.perf_config.middleware_timeout_ms,
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
                            "中间件执行失败: {}",
                            e
                        )));
                    },
                },
                Err(e) => {
                    debug!("中间件执行超时: {}", e);
                    return Err(error_utils::middleware_error(format!(
                        "中间件执行超时: {}",
                        e
                    )));
                },
            };

            // 记录中间件执行时间，用于性能监控
            let elapsed = start_time.elapsed();
            if elapsed.as_millis() > 100 {
                debug!("中间件执行时间较长: {}ms", elapsed.as_millis());
            }

            if let Some(mut transaction) = middleware_result {
                transaction.commit();
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
                            "附加事务提交失败: {}",
                            e
                        )));
                    },
                };

                let (_id, mut rx) = result;

                // 添加任务接收超时保护
                let task_receive_timeout = Duration::from_millis(
                    self.perf_config.task_receive_timeout_ms,
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
                                self.perf_config.task_receive_timeout_ms
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
                *state = Some(Arc::new(new_state));
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
}
