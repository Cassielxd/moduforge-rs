use std::{
    sync::Arc,
};

use moduforge_state::{
    state::{State, TransactionResult},
    transaction::Transaction,
};
use crate::{
    async_processor::{
        AsyncProcessor, ProcessorConfig, ProcessorError, TaskProcessor,
        TaskResult,
    },
    EditorResult,
};
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
    Rolled,
    NotFound,
}

#[derive(Debug, Clone)]
pub struct ProcessorResult {
    pub status: TransactionStatus,
    pub error: Option<String>,
    pub result: Option<TransactionResult>,
}
/// 事务处理器
pub struct TransactionProcessor;
pub type TaskParams = (Arc<State>, Transaction);

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
    pub fn new() -> EditorResult<Self> {
        let config = ProcessorConfig::default();
        let mut processor = AsyncProcessor::new(config, TransactionProcessor);
        processor.start();

        Ok(Self { processor: Arc::new(processor) })
    }

    pub async fn submit_transaction(
        &self,
        params: TaskParams,
    ) -> EditorResult<(
        u64,
        tokio::sync::mpsc::Receiver<TaskResult<TaskParams, ProcessorResult>>,
    )> {
        self.processor.submit_task(params, 0).await.map_err(Into::into)
    }

    pub async fn submit_transactions(
        &self,
        paramss: Vec<TaskParams>,
    ) -> EditorResult<
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
}
