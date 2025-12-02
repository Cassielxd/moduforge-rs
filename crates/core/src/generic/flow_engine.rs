//! 流引擎的泛型定义
//!
//! 此模块包含 FlowEngine 的泛型版本，支持同步和异步处理。

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

use mf_model::traits::{DataContainer, SchemaDefinition};

use crate::{
    config::ProcessorConfig,
    debug::debug,
    runtime::{
        async_processor::{
            AsyncProcessor, ProcessorError as AsyncProcessorError,
            TaskProcessor as AsyncTaskProcessor, TaskResult as AsyncTaskResult,
        },
        sync_processor::{
            ProcessorError as SyncProcessorError, SyncProcessor,
            TaskProcessor as SyncTaskProcessor, TaskResult as SyncTaskResult,
        },
    },
    types::{ProcessorResultGeneric, TaskParamsGeneric, TransactionStatus},
    ForgeResult,
};

/// 事务处理器（泛型版本）
pub struct TransactionProcessorGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    _phantom: std::marker::PhantomData<(C, S)>,
}

impl<C, S> TransactionProcessorGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

/// 同步任务处理器实现
#[async_trait]
impl<C, S> SyncTaskProcessor<TaskParamsGeneric<C, S>, ProcessorResultGeneric<C, S>>
    for TransactionProcessorGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    async fn process(
        &self,
        (state, tr): TaskParamsGeneric<C, S>,
    ) -> std::result::Result<ProcessorResultGeneric<C, S>, SyncProcessorError> {
        match (&state).apply_generic(tr).await {
            Ok(result) => Ok(ProcessorResultGeneric {
                status: TransactionStatus::Completed,
                error: None,
                result: Some(result),
            }),
            Err(e) => Ok(ProcessorResultGeneric {
                status: TransactionStatus::Failed(e.to_string()),
                error: None,
                result: None,
            }),
        }
    }
}

/// 异步任务处理器实现
#[async_trait]
impl<C, S> AsyncTaskProcessor<TaskParamsGeneric<C, S>, ProcessorResultGeneric<C, S>>
    for TransactionProcessorGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    async fn process(
        &self,
        (state, tr): TaskParamsGeneric<C, S>,
    ) -> std::result::Result<ProcessorResultGeneric<C, S>, AsyncProcessorError> {
        match (&state).apply_generic(tr).await {
            Ok(result) => Ok(ProcessorResultGeneric {
                status: TransactionStatus::Completed,
                error: None,
                result: Some(result),
            }),
            Err(e) => Ok(ProcessorResultGeneric {
                status: TransactionStatus::Failed(e.to_string()),
                error: None,
                result: None,
            }),
        }
    }
}

/// 同步流引擎（泛型版本）
#[derive(Clone)]
pub struct SyncFlowEngineGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    processor: Arc<
        SyncProcessor<
            TaskParamsGeneric<C, S>,
            ProcessorResultGeneric<C, S>,
            TransactionProcessorGeneric<C, S>,
        >,
    >,
}

impl<C, S> SyncFlowEngineGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub fn new() -> ForgeResult<Self> {
        let processor = SyncProcessor::new(
            TransactionProcessorGeneric::new(),
            3,
            Duration::from_secs(1),
        );
        Ok(Self {
            processor: Arc::new(processor),
        })
    }

    pub async fn submit(
        &self,
        params: TaskParamsGeneric<C, S>,
    ) -> SyncTaskResult<TaskParamsGeneric<C, S>, ProcessorResultGeneric<C, S>> {
        self.processor.process_task(params).await
    }
}

/// 异步流引擎（泛型版本）
#[derive(Clone)]
pub struct AsyncFlowEngineGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    processor: Arc<
        AsyncProcessor<
            TaskParamsGeneric<C, S>,
            ProcessorResultGeneric<C, S>,
            TransactionProcessorGeneric<C, S>,
        >,
    >,
}

impl<C, S> AsyncFlowEngineGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub async fn new() -> ForgeResult<Self> {
        let config = ProcessorConfig::default();
        let mut processor =
            AsyncProcessor::new(config, TransactionProcessorGeneric::new());
        processor.start().await.map_err(|e| {
            crate::error::error_utils::engine_error(format!(
                "启动异步处理器失败: {e}"
            ))
        })?;

        Ok(Self {
            processor: Arc::new(processor),
        })
    }

    pub async fn submit_transaction(
        &self,
        params: TaskParamsGeneric<C, S>,
    ) -> ForgeResult<(
        u64,
        Receiver<AsyncTaskResult<TaskParamsGeneric<C, S>, ProcessorResultGeneric<C, S>>>,
    )> {
        self.processor.submit_task(params, 0).await
    }

    pub async fn submit_transactions(
        &self,
        paramss: Vec<TaskParamsGeneric<C, S>>,
    ) -> ForgeResult<
        Vec<(
            u64,
            Receiver<AsyncTaskResult<TaskParamsGeneric<C, S>, ProcessorResultGeneric<C, S>>>,
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
    pub async fn shutdown(&self) -> ForgeResult<()> {
        debug!("AsyncFlowEngineGeneric shutdown 被调用，实际关闭将在 Drop 时发生");
        Ok(())
    }
}
