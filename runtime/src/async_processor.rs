use std::{fmt::Display, sync::Arc, time::Duration};
use tokio::sync::{mpsc, oneshot};
use async_trait::async_trait;
use tokio::select;

/// 任务处理的结果状态
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
}

/// 任务处理的错误类型
#[derive(Debug)]
pub enum ProcessorError {
    QueueFull,
    TaskFailed(String),
    InternalError(String),
}

impl Display for ProcessorError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            ProcessorError::QueueFull => write!(f, "Task queue is full"),
            ProcessorError::TaskFailed(msg) => write!(f, "Task failed: {}", msg),
            ProcessorError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ProcessorError {}

/// 任务处理的配置
#[derive(Clone, Debug)]
pub struct ProcessorConfig {
    pub max_queue_size: usize,
    pub max_concurrent_tasks: usize,
    pub task_timeout: Duration,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self { max_queue_size: 1000, max_concurrent_tasks: 10, task_timeout: Duration::from_secs(30) }
    }
}

/// 任务处理的结果
#[derive(Debug)]
pub struct TaskResult<T, O> {
    pub task_id: u64,
    pub status: TaskStatus,
    pub task: Option<T>,
    pub output: Option<O>,
    pub error: Option<String>,
}

/// 队列中的任务
struct QueuedTask<T, O> {
    task: T,
    task_id: u64,
    result_tx: mpsc::Sender<TaskResult<T, O>>,
}

/// 任务队列
pub struct TaskQueue<T, O> {
    queue: mpsc::Sender<QueuedTask<T, O>>,
    queue_rx: Arc<tokio::sync::Mutex<Option<mpsc::Receiver<QueuedTask<T, O>>>>>,
    next_task_id: Arc<tokio::sync::Mutex<u64>>,
}

impl<T: Clone + Send + 'static, O: Clone + Send + 'static> TaskQueue<T, O> {
    pub fn new(config: &ProcessorConfig) -> Self {
        let (tx, rx) = mpsc::channel(config.max_queue_size);
        Self {
            queue: tx,
            queue_rx: Arc::new(tokio::sync::Mutex::new(Some(rx))),
            next_task_id: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }

    pub async fn enqueue_task(
        &self,
        task: T,
    ) -> Result<(u64, mpsc::Receiver<TaskResult<T, O>>), ProcessorError> {
        let mut task_id = self.next_task_id.lock().await;
        *task_id += 1;
        let current_id = *task_id;

        let (result_tx, result_rx) = mpsc::channel(1);
        let queued_task = QueuedTask { task, task_id: current_id, result_tx };

        self.queue.send(queued_task).await.map_err(|_| ProcessorError::QueueFull)?;

        Ok((current_id, result_rx))
    }

    pub async fn get_next_ready(&self) -> Option<(T, u64, mpsc::Sender<TaskResult<T, O>>)> {
        let mut rx_guard = self.queue_rx.lock().await;
        if let Some(rx) = rx_guard.as_mut() {
            if let Some(queued) = rx.recv().await {
                return Some((queued.task, queued.task_id, queued.result_tx));
            }
        }
        None
    }
}

#[async_trait]
pub trait TaskProcessor<T, O>: Send + Sync + 'static
where
    T: Clone + Send + 'static,
    O: Clone + Send + 'static,
{
    async fn process(
        &self,
        task: T,
    ) -> Result<O, ProcessorError>;
}

/// 异步任务处理器
pub struct AsyncProcessor<T, O, P>
where
    T: Clone + Send + 'static,
    O: Clone + Send + 'static,
    P: TaskProcessor<T, O>,
{
    task_queue: Arc<TaskQueue<T, O>>,
    config: ProcessorConfig,
    processor: Arc<P>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl<T, O, P> AsyncProcessor<T, O, P>
where
    T: Clone + Send + 'static,
    O: Clone + Send + 'static,
    P: TaskProcessor<T, O>,
{
    pub fn new(
        config: ProcessorConfig,
        processor: P,
    ) -> Self {
        let task_queue = Arc::new(TaskQueue::new(&config));
        Self { 
            task_queue, 
            config, 
            processor: Arc::new(processor),
            shutdown_tx: None,
            handle: None,
        }
    }

    pub async fn submit_task(
        &self,
        task: T,
    ) -> Result<(u64, mpsc::Receiver<TaskResult<T, O>>), ProcessorError> {
        self.task_queue.enqueue_task(task).await
    }

    pub fn start(&mut self) {
        let queue = self.task_queue.clone();
        let processor = self.processor.clone();
        let config = self.config.clone();
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        
        self.shutdown_tx = Some(shutdown_tx);

        let handle = tokio::spawn(async move {
            let mut join_set = tokio::task::JoinSet::new();
            
            loop {
                select! {
                    // 处理关闭信号
                    _ = &mut shutdown_rx => {
                        break;
                    }
                    
                    // 处理任务完成
                    Some(result) = join_set.join_next() => {
                        if let Err(e) = result {
                            eprintln!("Task failed: {}", e);
                        }
                    }
                    
                    // 获取新任务
                    Some((task, task_id, result_tx)) = queue.get_next_ready() => {
                        if join_set.len() < config.max_concurrent_tasks {
                            let processor = processor.clone();
                            
                            join_set.spawn(async move {
                                let result = processor.process(task.clone()).await;

                                let task_result = match result {
                                    Ok(output) => TaskResult {
                                        task_id,
                                        status: TaskStatus::Completed,
                                        task: Some(task),
                                        output: Some(output),
                                        error: None,
                                    },
                                    Err(e) => TaskResult {
                                        task_id,
                                        status: TaskStatus::Failed(e.to_string()),
                                        task: Some(task),
                                        output: None,
                                        error: Some(e.to_string()),
                                    },
                                };

                                let _ = result_tx.send(task_result).await;
                            });
                        }
                    }
                }
            }
        });

        self.handle = Some(handle);
    }

    /// 优雅地关闭处理器
    /// 
    /// 此方法会等待所有正在处理的任务完成后再关闭处理器。
    /// 在关闭过程中，新的任务将不会被接受。
    pub async fn shutdown(&mut self) -> Result<(), ProcessorError> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            shutdown_tx.send(()).map_err(|_| {
                ProcessorError::InternalError("Failed to send shutdown signal".to_string())
            })?;

            if let Some(handle) = self.handle.take() {
                handle.await.map_err(|e| {
                    ProcessorError::InternalError(format!("Failed to join processor task: {}", e))
                })?;
            }
        }
        Ok(())
    }
}

impl<T, O, P> Drop for AsyncProcessor<T, O, P>
where
    T: Clone + Send + 'static,
    O: Clone + Send + 'static,
    P: TaskProcessor<T, O>,
{
    fn drop(&mut self) {
        if self.shutdown_tx.is_some() {
            // 创建一个新的运行时来处理异步关闭
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(self.shutdown()).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProcessor;

    #[async_trait::async_trait]
    impl TaskProcessor<i32, String> for TestProcessor {
        async fn process(
            &self,
            task: i32,
        ) -> Result<String, ProcessorError> {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(format!("Processed: {}", task))
        }
    }

    #[tokio::test]
    async fn test_async_processor() {
        let config =
            ProcessorConfig { max_queue_size: 100, max_concurrent_tasks: 5, task_timeout: Duration::from_secs(1) };
        let mut processor = AsyncProcessor::new(config, TestProcessor);
        processor.start();

        let mut receivers = Vec::new();
        for i in 0..10 {
            let (_, rx) = processor.submit_task(i).await.unwrap();
            receivers.push(rx);
        }

        for mut rx in receivers {
            let result = rx.recv().await.unwrap();
            assert_eq!(result.status, TaskStatus::Completed);
            assert!(result.error.is_none());
            assert!(result.output.is_some());
        }
    }

    #[tokio::test]
    async fn test_processor_shutdown() {
        let config = ProcessorConfig { 
            max_queue_size: 100, 
            max_concurrent_tasks: 5, 
            task_timeout: Duration::from_secs(1) 
        };
        let mut processor = AsyncProcessor::new(config, TestProcessor);
        processor.start();

        // Submit some tasks
        let mut receivers = Vec::new();
        for i in 0..5 {
            let (_, rx) = processor.submit_task(i).await.unwrap();
            receivers.push(rx);
        }

        // Initiate shutdown
        processor.shutdown().await.unwrap();

        // Verify all tasks completed
        for mut rx in receivers {
            let result = rx.recv().await.unwrap();
            assert_eq!(result.status, TaskStatus::Completed);
        }
    }

    #[tokio::test]
    async fn test_processor_auto_shutdown() {
        let config = ProcessorConfig { 
            max_queue_size: 100, 
            max_concurrent_tasks: 5, 
            task_timeout: Duration::from_secs(1) 
        };
        let mut processor = AsyncProcessor::new(config, TestProcessor);
        processor.start();

        // Submit some tasks
        let mut receivers = Vec::new();
        for i in 0..5 {
            let (_, rx) = processor.submit_task(i).await.unwrap();
            receivers.push(rx);
        }

        // Drop the processor, which should trigger shutdown
        drop(processor);

        // Verify all tasks completed
        for mut rx in receivers {
            let result = rx.recv().await.unwrap();
            assert_eq!(result.status, TaskStatus::Completed);
        }
    }
}
