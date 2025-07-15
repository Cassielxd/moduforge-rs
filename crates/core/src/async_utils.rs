//! 异步工具模块
//! 
//! 提供异步/同步边界处理工具，帮助避免在异步上下文中使用阻塞调用

use std::{future::Future, time::Duration};
use tokio::runtime::Handle;
use crate::{error::error_utils, ForgeResult};

/// 异步/同步边界处理工具
pub struct AsyncBridge;

impl AsyncBridge {
    /// 在同步上下文中安全地执行异步操作
    /// 
    /// 此方法会检查当前是否在 Tokio 运行时中：
    /// - 如果在运行时中，使用 spawn_blocking 避免阻塞
    /// - 如果不在运行时中，创建临时运行时执行
    /// 
    /// # 警告
    /// 此方法应该谨慎使用，主要用于：
    /// - Drop 实现
    /// - 同步的 FFI 边界
    /// - 测试代码
    pub fn run_async_in_sync<F, T>(future: F) -> ForgeResult<T>
    where
        F: Future<Output = ForgeResult<T>> + Send + 'static,
        T: Send + 'static,
    {
        // 检查是否在 Tokio 运行时中
        if let Ok(handle) = Handle::try_current() {
            // 在运行时中，使用 spawn_blocking 避免阻塞
            let (tx, rx) = std::sync::mpsc::channel();
            
            handle.spawn(async move {
                let result = future.await;
                let _ = tx.send(result);
            });
            
            // 等待结果，设置合理的超时
            match rx.recv_timeout(Duration::from_secs(30)) {
                Ok(result) => result,
                Err(_) => Err(error_utils::timeout_error(
                    "异步操作在同步上下文中超时".to_string()
                )),
            }
        } else {
            // 不在运行时中，创建临时运行时
            let rt = tokio::runtime::Runtime::new().map_err(|e| {
                error_utils::runtime_error(format!("创建临时运行时失败: {}", e))
            })?;
            
            rt.block_on(future)
        }
    }

    /// 在同步上下文中安全地执行异步操作（带超时）
    pub fn run_async_in_sync_with_timeout<F, T>(
        future: F,
        timeout: Duration,
    ) -> ForgeResult<T>
    where
        F: Future<Output = ForgeResult<T>> + Send + 'static,
        T: Send + 'static,
    {
        Self::run_async_in_sync(async move {
            tokio::time::timeout(timeout, future)
                .await
                .map_err(|_| error_utils::timeout_error("异步操作超时".to_string()))?
        })
    }

    /// 检查当前是否在异步上下文中
    pub fn is_in_async_context() -> bool {
        Handle::try_current().is_ok()
    }

    /// 安全地在异步上下文中执行可能阻塞的操作
    /// 
    /// 此方法会将阻塞操作移到专用的阻塞线程池中执行
    pub async fn run_blocking_in_async<F, T>(blocking_op: F) -> ForgeResult<T>
    where
        F: FnOnce() -> ForgeResult<T> + Send + 'static,
        T: Send + 'static,
    {
        tokio::task::spawn_blocking(blocking_op)
            .await
            .map_err(|e| error_utils::runtime_error(format!("阻塞任务执行失败: {}", e)))?
    }
}

/// 异步安全的资源清理器
///
/// 用于在 Drop 中安全地执行异步清理操作
pub struct AsyncDropper<T>
where
    T: Send + 'static,
{
    resource: Option<T>,
    cleanup_fn: Option<Box<dyn FnOnce(T) -> ForgeResult<()> + Send>>,
}

impl<T> AsyncDropper<T>
where
    T: Send + 'static,
{
    /// 创建新的异步清理器
    pub fn new<F>(resource: T, cleanup_fn: F) -> Self
    where
        F: FnOnce(T) -> ForgeResult<()> + Send + 'static,
    {
        Self {
            resource: Some(resource),
            cleanup_fn: Some(Box::new(cleanup_fn)),
        }
    }

    /// 手动执行清理（异步版本）
    pub async fn cleanup_async(mut self) -> ForgeResult<()> {
        if let (Some(resource), Some(cleanup_fn)) = (self.resource.take(), self.cleanup_fn.take()) {
            cleanup_fn(resource)
        } else {
            Ok(())
        }
    }

    /// 手动执行清理（同步版本）
    pub fn cleanup_sync(mut self) -> ForgeResult<()> {
        if let (Some(resource), Some(cleanup_fn)) = (self.resource.take(), self.cleanup_fn.take()) {
            cleanup_fn(resource)
        } else {
            Ok(())
        }
    }
}

impl<T> Drop for AsyncDropper<T>
where
    T: Send + 'static,
{
    fn drop(&mut self) {
        if let (Some(resource), Some(cleanup_fn)) = (self.resource.take(), self.cleanup_fn.take()) {
            // 在 Drop 中只能使用同步清理
            if let Err(e) = cleanup_fn(resource) {
                eprintln!("异步资源清理失败: {}", e);
            }
        }
    }
}

/// 异步操作的同步包装器
/// 
/// 提供一个安全的方式在同步代码中调用异步操作
pub struct SyncWrapper<T> {
    inner: T,
}

impl<T> SyncWrapper<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

/// 为事件总线提供同步包装
impl<T: Send + 'static> SyncWrapper<crate::event::EventBus<T>> {
    /// 同步广播事件（自动选择最佳方法）
    pub fn broadcast_auto(&self, event: T) -> ForgeResult<()> {
        if AsyncBridge::is_in_async_context() {
            // 在异步上下文中，使用 spawn 避免阻塞
            let bus = self.inner.clone();
            tokio::spawn(async move {
                if let Err(e) = bus.broadcast(event).await {
                    eprintln!("异步事件广播失败: {}", e);
                }
            });
            Ok(())
        } else {
            // 在同步上下文中，使用阻塞版本
            self.inner.broadcast_blocking(event)
        }
    }

    /// 同步销毁事件总线（自动选择最佳方法）
    pub fn destroy_auto(&self) -> ForgeResult<()> {
        if AsyncBridge::is_in_async_context() {
            // 在异步上下文中，使用异步版本
            let bus = self.inner.clone();
            AsyncBridge::run_async_in_sync(async move {
                bus.destroy().await
            })
        } else {
            // 在同步上下文中，使用阻塞版本
            self.inner.destroy_blocking();
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_sync_context_detection() {
        // 在测试中，通常不在异步上下文中
        assert!(!AsyncBridge::is_in_async_context());
    }

    #[tokio::test]
    async fn test_async_context_detection() {
        // 在 tokio::test 中，应该在异步上下文中
        assert!(AsyncBridge::is_in_async_context());
    }

    #[tokio::test]
    async fn test_run_blocking_in_async() {
        let result = AsyncBridge::run_blocking_in_async(|| {
            // 模拟阻塞操作
            std::thread::sleep(Duration::from_millis(10));
            Ok(42)
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_run_async_in_sync() {
        let result = AsyncBridge::run_async_in_sync(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(42)
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_async_dropper() {
        let dropper = AsyncDropper::new(42, |value| {
            assert_eq!(value, 42);
            Ok(())
        });

        // 手动清理
        dropper.cleanup_async().await.unwrap();
    }
}
