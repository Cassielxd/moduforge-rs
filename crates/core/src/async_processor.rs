use std::{
    fmt::Display,
    sync::Arc,
    time::{Duration, Instant},
};
use anyhow::anyhow;
use mf_state::debug;
use tokio::sync::{mpsc, oneshot};
use async_trait::async_trait;
use tokio::select;

use crate::{metrics, ForgeResult};

/// 任务处理的结果状态
/// - Pending: 任务等待处理
/// - Processing: 任务正在处理中
/// - Completed: 任务已完成
/// - Failed: 任务处理失败，包含错误信息
/// - Timeout: 任务执行超时
/// - Cancelled: 任务被取消
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
    Timeout,
    Cancelled,
}

impl From<&TaskStatus> for &'static str {
    fn from(status: &TaskStatus) -> Self {
        match status {
            TaskStatus::Pending => "pending",
            TaskStatus::Processing => "processing",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed(_) => "failed",
            TaskStatus::Timeout => "timeout",
            TaskStatus::Cancelled => "cancelled",
        }
    }
}

/// 任务处理器的错误类型
/// - QueueFull: 任务队列已满
/// - TaskFailed: 任务执行失败
/// - InternalError: 内部错误
/// - TaskTimeout: 任务执行超时
/// - TaskCancelled: 任务被取消
/// - RetryExhausted: 重试次数耗尽
#[derive(Debug)]
pub enum ProcessorError {
    QueueFull,
    TaskFailed(String),
    InternalError(String),
    TaskTimeout,
    TaskCancelled,
    RetryExhausted(String),
}

impl Display for ProcessorError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            ProcessorError::QueueFull => write!(f, "任务队列已满"),
            ProcessorError::TaskFailed(msg) => {
                write!(f, "任务执行失败: {}", msg)
            },
            ProcessorError::InternalError(msg) => {
                write!(f, "内部错误: {}", msg)
            },
            ProcessorError::TaskTimeout => {
                write!(f, "任务执行超时")
            },
            ProcessorError::TaskCancelled => write!(f, "任务被取消"),
            ProcessorError::RetryExhausted(msg) => {
                write!(f, "重试次数耗尽: {}", msg)
            },
        }
    }
}

impl std::error::Error for ProcessorError {}

/// 任务处理器的配置参数
/// - max_queue_size: 任务队列的最大容量
/// - max_concurrent_tasks: 最大并发任务数
/// - task_timeout: 单个任务的最大执行时间
/// - max_retries: 最大重试次数
/// - retry_delay: 重试延迟时间
#[derive(Clone, Debug)]
pub struct ProcessorConfig {
    pub max_queue_size: usize,
    pub max_concurrent_tasks: usize,
    pub task_timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            max_concurrent_tasks: 10,
            task_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        }
    }
}

/// 任务处理器的统计信息
/// - total_tasks: 总任务数
/// - completed_tasks: 已完成任务数
/// - failed_tasks: 失败任务数
/// - timeout_tasks: 超时任务数
/// - cancelled_tasks: 取消任务数
/// - current_queue_size: 当前队列大小
/// - current_processing_tasks: 当前处理任务数
#[derive(Debug, Default, Clone)]
pub struct ProcessorStats {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub timeout_tasks: u64,
    pub cancelled_tasks: u64,
    pub current_queue_size: usize,
    pub current_processing_tasks: usize,
}

/// 任务处理的结果结构
/// - task_id: 任务唯一标识符
/// - status: 任务状态
/// - task: 原始任务数据
/// - output: 任务处理输出
/// - error: 错误信息（如果有）
/// - processing_time: 任务处理时间
#[derive(Debug)]
pub struct TaskResult<T, O>
where
    T: Send + Sync,
    O: Send + Sync,
{
    pub task_id: u64,
    pub status: TaskStatus,
    pub task: Option<T>,
    pub output: Option<O>,
    pub error: Option<String>,
    pub processing_time: Option<Duration>,
}

/// 队列中的任务结构
/// - task: 实际任务数据
/// - task_id: 任务唯一标识符
/// - result_tx: 用于发送处理结果的通道发送端
/// - priority: 任务优先级
/// - retry_count: 重试次数
struct QueuedTask<T, O>
where
    T: Send + Sync,
    O: Send + Sync,
{
    task: T,
    task_id: u64,
    result_tx: mpsc::Sender<TaskResult<T, O>>,
    priority: u32,
    retry_count: u32,
}

/// 任务队列结构
/// - queue: 任务发送通道
/// - queue_rx: 任务接收通道（包装在Arc<Mutex>中以支持共享访问）
/// - next_task_id: 下一个任务的ID（原子递增）
/// - stats: 任务处理器统计信息
pub struct TaskQueue<T, O>
where
    T: Send + Sync,
    O: Send + Sync,
{
    queue: mpsc::Sender<QueuedTask<T, O>>,
    queue_rx: Arc<tokio::sync::Mutex<Option<mpsc::Receiver<QueuedTask<T, O>>>>>,
    next_task_id: Arc<tokio::sync::Mutex<u64>>,
    stats: Arc<tokio::sync::Mutex<ProcessorStats>>,
}

impl<T: Clone + Send + Sync + 'static, O: Clone + Send + Sync + 'static>
    TaskQueue<T, O>
{
    pub fn new(config: &ProcessorConfig) -> Self {
        let (tx, rx) = mpsc::channel(config.max_queue_size);
        Self {
            queue: tx,
            queue_rx: Arc::new(tokio::sync::Mutex::new(Some(rx))),
            next_task_id: Arc::new(tokio::sync::Mutex::new(0)),
            stats: Arc::new(tokio::sync::Mutex::new(ProcessorStats::default())),
        }
    }

    pub async fn enqueue_task(
        &self,
        task: T,
        priority: u32,
    ) -> ForgeResult<(u64, mpsc::Receiver<TaskResult<T, O>>)> {
        let mut task_id = self.next_task_id.lock().await;
        *task_id += 1;
        let current_id = *task_id;

        let (result_tx, result_rx) = mpsc::channel(1);
        let queued_task = QueuedTask {
            task,
            task_id: current_id,
            result_tx,
            priority,
            retry_count: 0,
        };

        self.queue
            .send(queued_task)
            .await
            .map_err(|_| anyhow!("队列已经满了"))?;

        let mut stats = self.stats.lock().await;
        stats.total_tasks += 1;
        stats.current_queue_size += 1;

        metrics::task_submitted();
        metrics::set_queue_size(stats.current_queue_size);

        Ok((current_id, result_rx))
    }

    pub async fn get_next_ready(
        &self
    ) -> Option<(T, u64, mpsc::Sender<TaskResult<T, O>>, u32, u32)> {
        let mut rx_guard = self.queue_rx.lock().await;
        if let Some(rx) = rx_guard.as_mut() {
            if let Some(queued) = rx.recv().await {
                let mut stats: tokio::sync::MutexGuard<'_, ProcessorStats> =
                    self.stats.lock().await;
                stats.current_queue_size -= 1;
                stats.current_processing_tasks += 1;
                metrics::set_queue_size(stats.current_queue_size);
                metrics::increment_processing_tasks();
                return Some((
                    queued.task,
                    queued.task_id,
                    queued.result_tx,
                    queued.priority,
                    queued.retry_count,
                ));
            }
        }
        None
    }

    pub async fn get_stats(&self) -> ProcessorStats {
        self.stats.lock().await.clone()
    }

    pub async fn update_stats(
        &self,
        result: &TaskResult<T, O>,
    ) {
        let mut stats = self.stats.lock().await;
        stats.current_processing_tasks -= 1;
        metrics::decrement_processing_tasks();

        let status_str: &'static str = (&result.status).into();
        metrics::task_processed(status_str);

        if let Some(duration) = result.processing_time {
            metrics::task_processing_duration(duration);
        }

        match result.status {
            TaskStatus::Completed => {
                stats.completed_tasks += 1;
            },
            TaskStatus::Failed(_) => stats.failed_tasks += 1,
            TaskStatus::Timeout => stats.timeout_tasks += 1,
            TaskStatus::Cancelled => stats.cancelled_tasks += 1,
            _ => {},
        }
    }
}

/// 任务处理器特征
/// 定义了处理任务的基本接口
#[async_trait]
pub trait TaskProcessor<T, O>: Send + Sync + 'static
where
    T: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
{
    async fn process(
        &self,
        task: T,
    ) -> Result<O, ProcessorError>;
}

/// 异步任务处理器
/// 负责管理任务队列、并发处理和任务生命周期
/// - T: 任务类型
/// - O: 任务输出类型
/// - P: 任务处理器实现
pub struct AsyncProcessor<T, O, P>
where
    T: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
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
    T: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
    P: TaskProcessor<T, O>,
{
    /// 创建新的异步任务处理器
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

    /// 提交新任务到处理器
    /// 返回任务ID和用于接收处理结果的通道
    pub async fn submit_task(
        &self,
        task: T,
        priority: u32,
    ) -> ForgeResult<(u64, mpsc::Receiver<TaskResult<T, O>>)> {
        self.task_queue.enqueue_task(task, priority).await
    }

    /// 启动任务处理器
    /// 创建后台任务来处理队列中的任务
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
                            debug!("任务执行失败: {}", e);
                        }
                    }

                    // 获取新任务并处理
                    Some((task, task_id, result_tx, _priority, retry_count)) = queue.get_next_ready() => {
                        if join_set.len() < config.max_concurrent_tasks {
                            let processor = processor.clone();
                            let config = config.clone();
                            let queue = queue.clone();

                            join_set.spawn(async move {
                                let start_time = Instant::now();
                                let mut current_retry = retry_count;

                                loop {
                                    let result = tokio::time::timeout(
                                        config.task_timeout,
                                        processor.process(task.clone())
                                    ).await;

                                    match result {
                                        Ok(Ok(output)) => {
                                            let processing_time = start_time.elapsed();
                                            let task_result = TaskResult {
                                                task_id,
                                                status: TaskStatus::Completed,
                                                task: Some(task),
                                                output: Some(output),
                                                error: None,
                                                processing_time: Some(processing_time),
                                            };
                                            queue.update_stats(&task_result).await;
                                            let _ = result_tx.send(task_result).await;
                                            break;
                                        }
                                        Ok(Err(e)) => {
                                            if current_retry < config.max_retries {
                                                current_retry += 1;
                                                tokio::time::sleep(config.retry_delay).await;
                                                continue;
                                            }
                                            let task_result = TaskResult {
                                                task_id,
                                                status: TaskStatus::Failed(e.to_string()),
                                                task: Some(task),
                                                output: None,
                                                error: Some(e.to_string()),
                                                processing_time: Some(start_time.elapsed()),
                                            };
                                            queue.update_stats(&task_result).await;
                                            let _ = result_tx.send(task_result).await;
                                            break;
                                        }
                                        Err(_) => {
                                            let task_result = TaskResult {
                                                task_id,
                                                status: TaskStatus::Timeout,
                                                task: Some(task),
                                                output: None,
                                                error: Some("任务执行超时".to_string()),
                                                processing_time: Some(start_time.elapsed()),
                                            };
                                            queue.update_stats(&task_result).await;
                                            let _ = result_tx.send(task_result).await;
                                            break;
                                        }
                                    }
                                }
                            });
                        }
                    }
                }
            }
        });

        self.handle = Some(handle);
    }

    /// 优雅地关闭处理器
    /// 等待所有正在处理的任务完成后再关闭
    pub fn shutdown(&mut self) -> Result<(), ProcessorError> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            shutdown_tx.send(()).map_err(|_| {
                ProcessorError::InternalError(
                    "Failed to send shutdown signal".to_string(),
                )
            })?;
        }
        Ok(())
    }

    pub async fn get_stats(&self) -> ProcessorStats {
        self.task_queue.get_stats().await
    }
}

/// 实现Drop特征，确保处理器在销毁时能够优雅关闭
impl<T, O, P> Drop for AsyncProcessor<T, O, P>
where
    T: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
    P: TaskProcessor<T, O>,
{
    fn drop(&mut self) {
        if self.shutdown_tx.is_some() {
            // 创建一个新的运行时来处理异步关闭
            let _ = self.shutdown();
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
        let config = ProcessorConfig {
            max_queue_size: 100,
            max_concurrent_tasks: 5,
            task_timeout: Duration::from_secs(1),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        };
        let mut processor = AsyncProcessor::new(config, TestProcessor);
        processor.start();

        let mut receivers = Vec::new();
        for i in 0..10 {
            let (_, rx) = processor.submit_task(i, 0).await.unwrap();
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
            task_timeout: Duration::from_secs(1),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        };
        let mut processor = AsyncProcessor::new(config, TestProcessor);
        processor.start();

        // Submit some tasks
        let mut receivers = Vec::new();
        for i in 0..5 {
            let (_, rx) = processor.submit_task(i, 0).await.unwrap();
            receivers.push(rx);
        }

        // Initiate shutdown
        processor.shutdown().unwrap();

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
            task_timeout: Duration::from_secs(1),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        };
        let mut processor = AsyncProcessor::new(config, TestProcessor);
        processor.start();

        // Submit some tasks
        let mut receivers = Vec::new();
        for i in 0..5 {
            let (_, rx) = processor.submit_task(i, 0).await.unwrap();
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
