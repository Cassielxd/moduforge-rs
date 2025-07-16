use std::{sync::Arc, time::Duration};

use crate::{
    runtime::sync_processor::{
        ProcessorError, SyncProcessor, TaskProcessor, TaskResult,
    },
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
        Arc<SyncProcessor<TaskParams, ProcessorResult, TransactionProcessor>>,
}

impl FlowEngine {
    pub fn new() -> ForgeResult<Self> {
        let processor =
            SyncProcessor::new(TransactionProcessor, 3, Duration::from_secs(1));
        Ok(Self { processor: Arc::new(processor) })
    }

    pub async fn submit(
        &self,
        params: TaskParams,
    ) -> TaskResult<TaskParams, ProcessorResult> {
        self.processor.process_task(params).await
    }
}
