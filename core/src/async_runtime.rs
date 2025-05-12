use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};

use crate::{
    error_utils,
    event::Event,
    flow::{FlowEngine, ProcessorResult},
    types::EditorOptions,
    EditorResult,
};
use moduforge_state::{
    debug,
    state::TransactionResult,
    transaction::{Command, Transaction},
    State,
};
use crate::runtime::Editor;

/// 性能监控配置
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub enable_monitoring: bool,
    pub middleware_timeout_ms: u64,
    pub log_threshold_ms: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: false,
            middleware_timeout_ms: 500,
            log_threshold_ms: 50,
        }
    }
}

/// Editor 结构体代表编辑器的核心功能实现
/// 负责管理文档状态、事件处理、插件系统和存储等核心功能
pub struct AsyncEditor {
    base: Editor,
    flow_engine: FlowEngine,
    perf_config: PerformanceConfig,
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
        let base = Editor::create(options).await?;
        Ok(AsyncEditor {
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
    pub async fn command(
        &mut self,
        command: Arc<dyn Command>,
    ) -> EditorResult<()> {
        let cmd_name = command.name();
        debug!("正在执行命令: {}", cmd_name);

        // 创建事务并应用命令
        let mut tr = self.get_tr();
        command.execute(&mut tr).await.map_err(|e| {
            error_utils::state_error(format!(
                "命令执行失败: {}",
                e
            ))
        })?;

        // 使用高性能处理引擎处理事务
        match self.dispatch_flow(tr).await {
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
    pub async fn dispatch_flow(
        &mut self,
        transaction: Transaction,
    ) -> EditorResult<()> {
        let start_time = std::time::Instant::now();
        let mut current_transaction = transaction;

        // 前置中间件处理
        let middleware_start = std::time::Instant::now();
        self.run_before_middleware(&mut current_transaction).await?;
        self.log_performance("前置中间件处理", middleware_start.elapsed());

        // 使用 flow_engine 提交事务
        let flow_start = std::time::Instant::now();
        let (_id, mut rx) = self
            .flow_engine
            .submit_transaction((
                self.base.get_state().clone(),
                current_transaction,
            ))
            .await?;
        self.log_performance("提交事务", flow_start.elapsed());

        // 等待任务结果
        let recv_start = std::time::Instant::now();
        let Some(task_result) = rx.recv().await else {
            return Err(error_utils::state_error(
                "无法接收任务结果".to_string(),
            ));
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
        if let Some(tr) = transactions.last() {
            if tr.doc_changed() {
                current_state = Some(Arc::new(result.state));
            }
        }

        // 执行后置中间件链
        let after_start = std::time::Instant::now();
        self.run_after_middleware(&mut current_state, &mut transactions)
            .await?;
        self.log_performance("后置中间件处理", after_start.elapsed());

        // 更新状态并广播事件
        if let Some(state) = current_state {
            let update_start = std::time::Instant::now();
            self.base.update_state(state.clone()).await?;
            self.log_performance("状态更新", update_start.elapsed());

            let event_start = std::time::Instant::now();
            self.base
                .emit_event(Event::TrApply(Arc::new(transactions), state))
                .await?;
            self.log_performance("事件广播", event_start.elapsed());
        }

        self.log_performance("事务处理总耗时", start_time.elapsed());
        Ok(())
    }

    pub async fn run_before_middleware(
        &mut self,
        transaction: &mut Transaction,
    ) -> EditorResult<()> {
        debug!("执行前置中间件链");
        for middleware in
            &self.base.get_options().get_middleware_stack().middlewares
        {
            let timeout = std::time::Duration::from_millis(500);
            if let Err(e) = tokio::time::timeout(
                timeout,
                middleware.before_dispatch(transaction),
            )
            .await
            {
                return Err(error_utils::middleware_error(format!(
                    "中间件执行超时: {}",
                    e
                )));
            }
        }
        Ok(())
    }
    pub async fn run_after_middleware(
        &mut self,
        state: &mut Option<Arc<State>>,
        transactions: &mut Vec<Transaction>,
    ) -> EditorResult<()> {
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

            if let Some(transaction) = middleware_result.additional_transaction
            {
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

                let Some(task_result) = rx.recv().await else {
                    debug!("接收事务处理结果失败");
                    return Ok(());
                };

                let Some(ProcessorResult { result: Some(result), .. }) =
                    task_result.output
                else {
                    debug!("处理结果无效");
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
