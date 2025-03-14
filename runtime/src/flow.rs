use std::{
    fmt::{Display, Formatter},
    sync::Arc,
    time::Duration,
};

use moduforge_core::state::{state::State, transaction::Transaction};
use crate::async_processor::{TaskProcessor, AsyncProcessor, ProcessorConfig, ProcessorError, TaskResult, TaskStatus};
use async_trait::async_trait;

pub type Result<T> = std::result::Result<T, FlowError>;

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
    Rolled,
    NotFound,
}

#[derive(Debug)]
pub enum FlowError {
    QueueFull,
    TransactionNotFound,
    TransactionTimeout,
    TransactionFailed(String),
    PluginError(String),
    StateError(String),
    InvalidTransaction(String),
    InternalError(String),
}

impl Display for FlowError {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            FlowError::QueueFull => write!(f, "Transaction queue is full"),
            FlowError::TransactionNotFound => write!(f, "Transaction not found"),
            FlowError::TransactionTimeout => write!(f, "Transaction timed out"),
            FlowError::TransactionFailed(msg) => write!(f, "Transaction failed: {}", msg),
            FlowError::PluginError(msg) => write!(f, "Plugin error: {}", msg),
            FlowError::StateError(msg) => write!(f, "State error: {}", msg),
            FlowError::InvalidTransaction(msg) => write!(f, "Invalid transaction: {}", msg),
            FlowError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for FlowError {}

impl From<ProcessorError> for FlowError {
    fn from(error: ProcessorError) -> Self {
        match error {
            ProcessorError::QueueFull => FlowError::QueueFull,
            ProcessorError::TaskFailed(msg) => FlowError::TransactionFailed(msg),
            ProcessorError::InternalError(msg) => FlowError::InternalError(msg),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub transaction_id: u64,
    pub status: TransactionStatus,
    pub transaction: Option<Transaction>,
    pub error: Option<String>,
    pub state: Option<State>,
}
/// 事务处理器
pub struct TransactionProcessor;
pub type TaskParams = (Arc<State>, Transaction);

#[async_trait]
impl TaskProcessor<TaskParams, TransactionResult> for TransactionProcessor {
    async fn process(
        &self,
        (state, mut tr): TaskParams,
    ) -> std::result::Result<TransactionResult, ProcessorError> {
        match state.apply(&mut tr).await {
            Ok(state) => Ok(TransactionResult {
                transaction_id: tr.id,
                status: TransactionStatus::Completed,
                error: None,
                state: Some(state),
                transaction: Some(tr),
            }),
            Err(e) => Ok(TransactionResult {
                transaction_id: tr.id,
                status: TransactionStatus::Failed(e.to_string()),
                error: None,
                state: None,
                transaction: None,
            }),
        }
    }
}

#[derive(Clone)]
pub struct FlowEngine {
    processor: Arc<AsyncProcessor<TaskParams, TransactionResult, TransactionProcessor>>,
}

impl FlowEngine {
    pub fn new() -> Result<Self> {
        let config = ProcessorConfig::default();
        let processor = AsyncProcessor::new(config, TransactionProcessor);
        processor.start();

        Ok(Self { processor: Arc::new(processor) })
    }

    pub async fn submit_transaction(
        &self,
        params: TaskParams,
    ) -> Result<(u64, tokio::sync::mpsc::Receiver<TaskResult<TaskParams, TransactionResult>>)> {
        self.processor.submit_task(params).await.map_err(Into::into)
    }

    pub async fn submit_transactions(
        &self,
        paramss: Vec<TaskParams>,
    ) -> Result<Vec<(u64, tokio::sync::mpsc::Receiver<TaskResult<TaskParams, TransactionResult>>)>> {
        let mut results = Vec::new();
        for transaction in paramss {
            let result = self.submit_transaction(transaction).await?;
            results.push(result);
        }
        Ok(results)
    }
}
