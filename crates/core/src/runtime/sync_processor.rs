use std::{
    fmt::Display,
    sync::{Arc},
    time::{Duration, Instant},
    thread,
    marker::PhantomData,
};

use async_trait::async_trait;

use crate::metrics;

/// 任务处理的结果状态
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Completed,
    Failed(String),
}

impl From<&TaskStatus> for &'static str {
    fn from(status: &TaskStatus) -> Self {
        match status {
            TaskStatus::Completed => "completed",
            TaskStatus::Failed(_) => "failed",
        }
    }
}

/// 任务处理器的错误类型
#[derive(Debug)]
pub enum ProcessorError {
    TaskFailed(String),
    InternalError(String),
}

impl Display for ProcessorError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            ProcessorError::TaskFailed(msg) => {
                write!(f, "任务执行失败: {msg}")
            },
            ProcessorError::InternalError(msg) => {
                write!(f, "内部错误: {msg}")
            },
        }
    }
}

impl std::error::Error for ProcessorError {}

/// 任务处理的结果结构
#[derive(Debug)]
pub struct TaskResult<T, O>
where
    T: Send + Sync,
    O: Send + Sync,
{
    pub status: TaskStatus,
    pub task: Option<T>,
    pub output: Option<O>,
    pub error: Option<String>,
    pub processing_time: Duration,
}

/// 任务处理器特征
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

/// 同步任务处理器
pub struct SyncProcessor<T, O, P>
where
    T: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
    P: TaskProcessor<T, O>,
{
    processor: Arc<P>,
    max_retries: u32,
    retry_delay: Duration,
    _phantom: PhantomData<(T, O)>,
}

impl<T, O, P> SyncProcessor<T, O, P>
where
    T: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
    P: TaskProcessor<T, O>,
{
    pub fn new(
        processor: P,
        max_retries: u32,
        retry_delay: Duration,
    ) -> Self {
        Self {
            processor: Arc::new(processor),
            max_retries,
            retry_delay,
            _phantom: PhantomData,
        }
    }

    pub async fn process_task(
        &self,
        task: T,
    ) -> TaskResult<T, O> {
        metrics::task_submitted();
        let start_time = Instant::now();
        let mut current_retry = 0;

        loop {
            match self.processor.process(task.clone()).await {
                Ok(output) => {
                    let result = TaskResult {
                        status: TaskStatus::Completed,
                        task: Some(task),
                        output: Some(output),
                        error: None,
                        processing_time: start_time.elapsed(),
                    };
                    metrics::task_processing_duration(result.processing_time);
                    metrics::task_processed((&result.status).into());
                    return result;
                },
                Err(e) => {
                    if current_retry < self.max_retries {
                        current_retry += 1;
                        metrics::task_retried();
                        tokio::time::sleep(self.retry_delay).await;
                        continue;
                    }
                    let result = TaskResult {
                        status: TaskStatus::Failed(e.to_string()),
                        task: Some(task),
                        output: None,
                        error: Some(e.to_string()),
                        processing_time: start_time.elapsed(),
                    };
                    metrics::task_processing_duration(result.processing_time);
                    metrics::task_processed((&result.status).into());
                    return result;
                },
            }
        }
    }

    pub async fn process_task_with_retry(
        &self,
        task: T,
        max_retries: u32,
        retry_delay: Duration,
    ) -> TaskResult<T, O> {
        metrics::task_submitted();
        let start_time = Instant::now();
        let mut current_retry = 0;

        loop {
            match self.processor.process(task.clone()).await {
                Ok(output) => {
                    let result = TaskResult {
                        status: TaskStatus::Completed,
                        task: Some(task),
                        output: Some(output),
                        error: None,
                        processing_time: start_time.elapsed(),
                    };
                    metrics::task_processing_duration(result.processing_time);
                    metrics::task_processed((&result.status).into());
                    return result;
                },
                Err(e) => {
                    if current_retry < max_retries {
                        current_retry += 1;
                        metrics::task_retried();
                        thread::sleep(retry_delay);
                        continue;
                    }
                    let result = TaskResult {
                        status: TaskStatus::Failed(e.to_string()),
                        task: Some(task),
                        output: None,
                        error: Some(e.to_string()),
                        processing_time: start_time.elapsed(),
                    };
                    metrics::task_processing_duration(result.processing_time);
                    metrics::task_processed((&result.status).into());
                    return result;
                },
            }
        }
    }
}
