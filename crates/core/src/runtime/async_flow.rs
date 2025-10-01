use std::{sync::Arc};

use crate::{
    runtime::async_processor::{
        AsyncProcessor, ProcessorError, TaskProcessor, TaskResult,
    },
    config::ProcessorConfig,
    debug::debug,
    types::{ProcessorResult, TaskParams, TransactionStatus},
    ForgeResult,
};
use async_trait::async_trait;

/// 事务处理器
pub struct TransactionProcessor;

#[async_trait]
impl TaskProcessor<TaskParams, ProcessorResult> for TransactionProcessor {
    async fn process(
        &self,
        (state, tr): TaskParams,
    ) -> std::result::Result<ProcessorResult, ProcessorError> {
        match state.apply(tr).await {
            Ok(result) => Ok(ProcessorResult {
                status: TransactionStatus::Completed,
                error: None,
                result: Some(result),
            }),
            Err(e) => Ok(ProcessorResult {
                status: TransactionStatus::Failed(e.to_string()),
                error: None,
                result: None,
            }),
        }
    }
}

#[derive(Clone)]
pub struct FlowEngine {
    processor:
        Arc<AsyncProcessor<TaskParams, ProcessorResult, TransactionProcessor>>,
}

impl FlowEngine {
    pub async fn new() -> ForgeResult<Self> {
        let config = ProcessorConfig::default();
        let mut processor = AsyncProcessor::new(config, TransactionProcessor);
        processor.start().await.map_err(|e| {
            crate::error::error_utils::engine_error(format!(
                "启动异步处理器失败: {e}"
            ))
        })?;

        Ok(Self { processor: Arc::new(processor) })
    }

    pub async fn submit_transaction(
        &self,
        params: TaskParams,
    ) -> ForgeResult<(
        u64,
        tokio::sync::mpsc::Receiver<TaskResult<TaskParams, ProcessorResult>>,
    )> {
        self.processor.submit_task(params, 0).await.map_err(Into::into)
    }

    pub async fn submit_transactions(
        &self,
        paramss: Vec<TaskParams>,
    ) -> ForgeResult<
        Vec<(
            u64,
            tokio::sync::mpsc::Receiver<
                TaskResult<TaskParams, ProcessorResult>,
            >,
        )>,
    > {
        let mut results = Vec::new();
        for transaction in paramss {
            let result = self.submit_transaction(transaction).await?;
            results.push(result);
        }
        Ok(results)
    }

    /// 关闭流引擎
    ///
    /// 注意：由于 processor 被包装在 Arc 中，这个方法只能发送关闭信号
    /// 实际的关闭需要等到所有 Arc 引用都被释放
    pub async fn shutdown(&self) -> ForgeResult<()> {
        // 由于 processor 在 Arc 中，我们无法获取可变引用来调用 shutdown
        // 这是设计上的限制，实际的关闭会在 Drop 时自动发生
        debug!("FlowEngine shutdown 被调用，实际关闭将在 Drop 时发生");
        Ok(())
    }
}
