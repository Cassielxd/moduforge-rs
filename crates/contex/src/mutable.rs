use std::{
    sync::Arc,
    fmt::Debug,
};

use async_trait::async_trait;
use parking_lot::{Mutex, RwLock};
use tokio::sync::{Mutex as AsyncMutex, RwLock as AsyncRwLock};

use crate::{Component, ContainerResult, Lifecycle};

/// 可变组件trait - 标识支持内部可变性的组件
#[async_trait]
pub trait MutableComponent: Component {
    /// 是否支持并发读写（使用RwLock）
    fn supports_concurrent_read() -> bool
    where
        Self: Sized,
    {
        false
    }
    
    /// 是否需要异步锁
    fn requires_async_lock() -> bool
    where
        Self: Sized,
    {
        false
    }
}

/// 同步互斥锁包装器
#[derive(Debug)]
pub struct MutexWrapper<T: MutableComponent> {
    inner: Arc<Mutex<T>>,
}

impl<T: MutableComponent> MutexWrapper<T> {
    pub fn new(component: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(component)),
        }
    }
    
    /// 获取可变引用的闭包访问
    pub fn with_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.lock();
        f(&mut *guard)
    }
    
    /// 获取不可变引用的闭包访问
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.inner.lock();
        f(&*guard)
    }
    
    /// 尝试获取可变引用（非阻塞）
    pub fn try_with_mut<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.inner.try_lock().map(|mut guard| f(&mut *guard))
    }
}

#[async_trait]
impl<T: MutableComponent> Component for MutexWrapper<T> {
    fn component_name() -> &'static str
    where
        Self: Sized,
    {
        T::component_name()
    }
    
    fn lifecycle() -> Lifecycle
    where
        Self: Sized,
    {
        T::lifecycle()
    }
    
    async fn initialize(&self) -> ContainerResult<()> {
        // For synchronous locks, we need to avoid holding locks across await
        // Use a simple approach: most components don't need complex async initialization
        Ok(())
    }
    
    async fn destroy(&self) -> ContainerResult<()> {
        // Similarly, most components don't need complex async destruction
        Ok(())
    }
}

/// 读写锁包装器（支持并发读）
#[derive(Debug)]
pub struct RwLockWrapper<T: MutableComponent> {
    inner: Arc<RwLock<T>>,
}

impl<T: MutableComponent> RwLockWrapper<T> {
    pub fn new(component: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(component)),
        }
    }
    
    /// 获取可变引用的闭包访问
    pub fn with_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.write();
        f(&mut *guard)
    }
    
    /// 获取不可变引用的闭包访问
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.inner.read();
        f(&*guard)
    }
    
    /// 尝试获取可变引用（非阻塞）
    pub fn try_with_mut<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.inner.try_write().map(|mut guard| f(&mut *guard))
    }
    
    /// 尝试获取不可变引用（非阻塞）
    pub fn try_with<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&T) -> R,
    {
        self.inner.try_read().map(|guard| f(&*guard))
    }
}

#[async_trait]
impl<T: MutableComponent> Component for RwLockWrapper<T> {
    fn component_name() -> &'static str
    where
        Self: Sized,
    {
        T::component_name()
    }
    
    fn lifecycle() -> Lifecycle
    where
        Self: Sized,
    {
        T::lifecycle()
    }
    
    async fn initialize(&self) -> ContainerResult<()> {
        // For synchronous locks, we need to avoid holding locks across await
        Ok(())
    }
    
    async fn destroy(&self) -> ContainerResult<()> {
        // Similarly, most components don't need complex async destruction
        Ok(())
    }
}

/// 异步互斥锁包装器
#[derive(Debug)]
pub struct AsyncMutexWrapper<T: MutableComponent> {
    inner: Arc<AsyncMutex<T>>,
}

impl<T: MutableComponent> AsyncMutexWrapper<T> {
    pub fn new(component: T) -> Self {
        Self {
            inner: Arc::new(AsyncMutex::new(component)),
        }
    }
    
    /// 获取可变引用的异步闭包访问
    pub async fn with_mut<F, Fut, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let mut guard = self.inner.lock().await;
        f(&mut *guard).await
    }
    
    /// 获取不可变引用的异步闭包访问
    pub async fn with<F, Fut, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let guard = self.inner.lock().await;
        f(&*guard).await
    }
    
    /// 尝试获取可变引用（非阻塞）
    pub fn try_with_mut<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.inner.try_lock().ok().map(|mut guard| f(&mut *guard))
    }
}

#[async_trait]
impl<T: MutableComponent> Component for AsyncMutexWrapper<T> {
    fn component_name() -> &'static str
    where
        Self: Sized,
    {
        T::component_name()
    }
    
    fn lifecycle() -> Lifecycle
    where
        Self: Sized,
    {
        T::lifecycle()
    }
    
    async fn initialize(&self) -> ContainerResult<()> {
        let guard = self.inner.lock().await;
        guard.initialize().await
    }
    
    async fn destroy(&self) -> ContainerResult<()> {
        let guard = self.inner.lock().await;
        guard.destroy().await
    }
}

/// 异步读写锁包装器
#[derive(Debug)]
pub struct AsyncRwLockWrapper<T: MutableComponent> {
    inner: Arc<AsyncRwLock<T>>,
}

impl<T: MutableComponent> AsyncRwLockWrapper<T> {
    pub fn new(component: T) -> Self {
        Self {
            inner: Arc::new(AsyncRwLock::new(component)),
        }
    }
    
    /// 获取可变引用的异步闭包访问
    pub async fn with_mut<F, Fut, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let mut guard = self.inner.write().await;
        f(&mut *guard).await
    }
    
    /// 获取不可变引用的异步闭包访问
    pub async fn with<F, Fut, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let guard = self.inner.read().await;
        f(&*guard).await
    }
    
    /// 尝试获取可变引用（非阻塞）
    pub async fn try_with_mut<F, Fut, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        if let Ok(mut guard) = self.inner.try_write() {
            Some(f(&mut *guard).await)
        } else {
            None
        }
    }
    
    /// 尝试获取不可变引用（非阻塞）
    pub async fn try_with<F, Fut, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&T) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        if let Ok(guard) = self.inner.try_read() {
            Some(f(&*guard).await)
        } else {
            None
        }
    }
}

#[async_trait]
impl<T: MutableComponent> Component for AsyncRwLockWrapper<T> {
    fn component_name() -> &'static str
    where
        Self: Sized,
    {
        T::component_name()
    }
    
    fn lifecycle() -> Lifecycle
    where
        Self: Sized,
    {
        T::lifecycle()
    }
    
    async fn initialize(&self) -> ContainerResult<()> {
        let guard = self.inner.read().await;
        guard.initialize().await
    }
    
    async fn destroy(&self) -> ContainerResult<()> {
        let guard = self.inner.read().await;
        guard.destroy().await
    }
}