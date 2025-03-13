use std::{
    fmt::{Display, Formatter},
    sync::Arc,
    time::{Duration},
};

use moduforge_core::state::{state::State, transaction::Transaction};
use tokio::sync::{mpsc, Mutex};

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

#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub transaction_id: u64,
    pub status: TransactionStatus,
    pub transaction: Option<Transaction>,
    pub error: Option<String>,
    pub state: Option<State>,
}

#[derive(Debug)]
struct QueuedTransaction {
    state: Arc<State>,
    transaction: Transaction,
    result_tx: mpsc::Sender<TransactionResult>,
}

#[derive(Debug)]
pub struct TransactionQueue {
    queue: mpsc::Sender<QueuedTransaction>,
    queue_rx: Arc<Mutex<Option<mpsc::Receiver<QueuedTransaction>>>>,
}

#[derive(Clone, Debug)]
pub struct TransactionQueueConfig {
    pub max_queue_size: usize,
    pub max_concurrent_transactions: usize,
    pub transaction_timeout: Duration,
}

impl Default for TransactionQueueConfig {
    fn default() -> Self {
        Self { max_queue_size: 1000, max_concurrent_transactions: 10, transaction_timeout: Duration::from_secs(30) }
    }
}

impl TransactionQueue {
    pub fn new(config: TransactionQueueConfig) -> Self {
        let (tx, rx) = mpsc::channel::<QueuedTransaction>(config.max_queue_size);

        Self { queue: tx, queue_rx: Arc::new(Mutex::new(Some(rx))) }
    }

    pub async fn enqueue_transaction(
        &self,
        state: Arc<State>,
        transaction: Transaction,
    ) -> Result<(u64, mpsc::Receiver<TransactionResult>)> {
        let id: u64 = transaction.id;
        let (result_tx, result_rx) = mpsc::channel(1);

        let queued_transaction = QueuedTransaction { state, transaction, result_tx };

        // 尝试发送到队列，如果队列满了会返回错误
        self.queue.send(queued_transaction).await.map_err(|_| FlowError::QueueFull)?;

        Ok((id, result_rx))
    }

    pub async fn get_next_ready(&self) -> Option<(Arc<State>, Transaction, mpsc::Sender<TransactionResult>)> {
        let mut rx_guard = self.queue_rx.lock().await;
        if let Some(rx) = rx_guard.as_mut() {
            if let Some(queued) = rx.recv().await {
                return Some((queued.state, queued.transaction, queued.result_tx));
            }
        }

        None
    }
}

#[derive(Clone)]
pub struct FlowEngine {
    transaction_queue: Arc<TransactionQueue>,
}

impl FlowEngine {
    pub fn new() -> Result<Self> {
        let transaction_queue = Arc::new(TransactionQueue::new(TransactionQueueConfig::default()));
        let engine = Self { transaction_queue };
        // 启动事务处理器
        engine.start_transaction_processor()?;

        Ok(engine)
    }

    pub async fn submit_transaction(
        &self,
        state: Arc<State>,
        transaction: Transaction,
    ) -> Result<(u64, mpsc::Receiver<TransactionResult>)> {
        // 提交到队列
        let (id, rx) = self.transaction_queue.enqueue_transaction(state.clone(), transaction).await?;

        Ok((id, rx))
    }

    pub async fn process_transaction(
        &self,
        state: Arc<State>,
        tr: Transaction,
    ) -> Result<TransactionResult> {
        let mut tr: Transaction = tr;
        // 执行插件的前置处理
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

    pub fn start_transaction_processor(&self) -> Result<()> {
        let queue = self.transaction_queue.clone();
        let engine = self.clone();

        tokio::spawn(async move {
            loop {
                if let Some((state, transaction, result_tx)) = queue.get_next_ready().await {
                    let transaction_id = transaction.id;
                    let result = engine.process_transaction(state, transaction).await;

                    match result {
                        Ok(result) => {
                            // 发送结果
                            let _ = result_tx.send(result).await;
                        },
                        Err(e) => {
                            let error_result = TransactionResult {
                                transaction_id,
                                status: TransactionStatus::Failed(e.to_string()),
                                error: Some(e.to_string()),
                                state: None,
                                transaction: None,
                            };
                            // 发送错误结果
                            let _ = result_tx.send(error_result).await;
                        },
                    }
                }
            }
        });

        Ok(())
    }

    // 批量提交事务
    pub async fn submit_transactions(
        &self,
        state: Arc<State>,
        transactions: Vec<Transaction>,
    ) -> Result<Vec<(u64, mpsc::Receiver<TransactionResult>)>> {
        let mut results = Vec::new();

        for transaction in transactions {
            let result = self.submit_transaction(state.clone(), transaction).await?;
            results.push(result);
        }

        Ok(results)
    }
}
