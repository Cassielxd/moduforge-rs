use std::{
    any::Any,
    collections::HashMap,
    sync::Arc,
};

use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::{Mutex, RwLock as AsyncRwLock};

use crate::{
    component::{Component, ComponentResolver},
    error::{ContainerError, ContainerResult},
    lifecycle::Lifecycle,
    registry::global_registry,
    global_container,
    mutable::{MutableComponent, MutexWrapper, RwLockWrapper, AsyncMutexWrapper, AsyncRwLockWrapper},
    aop_proxy::AopProxyWrapper,
};

/// 作用域上下文
#[derive(Debug, Clone)]
pub struct ScopeContext {
    pub id: String,
    pub name: String,
    pub parent: Option<String>,
}

impl ScopeContext {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            parent: None,
        }
    }
    
    pub fn with_parent(name: impl Into<String>, parent: &ScopeContext) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            parent: Some(parent.id.clone()),
        }
    }
}

/// 依赖注入容器
pub struct Container {
    /// 单例实例缓存
    singleton_instances: DashMap<String, Arc<dyn Any + Send + Sync>>,
    /// 作用域实例缓存 (scope_id -> component_name -> instance)
    scoped_instances: AsyncRwLock<HashMap<String, DashMap<String, Arc<dyn Any + Send + Sync>>>>,
    /// 当前作用域栈
    scope_stack: AsyncRwLock<Vec<ScopeContext>>,
    /// 初始化状态
    initialized: Mutex<bool>,
    /// 正在解析的组件（用于循环依赖检测）
    resolving: AsyncRwLock<std::collections::HashSet<String>>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            singleton_instances: DashMap::new(),
            scoped_instances: AsyncRwLock::new(HashMap::new()),
            scope_stack: AsyncRwLock::new(Vec::new()),
            initialized: Mutex::new(false),
            resolving: AsyncRwLock::new(std::collections::HashSet::new()),
        }
    }
    
    /// 初始化容器
    pub async fn initialize(&self) -> ContainerResult<()> {
        let mut initialized = self.initialized.lock().await;
        if *initialized {
            return Ok(());
        }
        
        // 检查循环依赖
        global_registry().check_circular_dependencies()?;
        
        // 预创建所有单例组件
        let components = global_registry().get_all_components();
        for component_info in components {
            if component_info.lifecycle == Lifecycle::Singleton {
                self.resolve_by_name(&component_info.name).await?;
            }
        }
        
        *initialized = true;
        Ok(())
    }
    
    /// 创建新的作用域
    pub async fn create_scope(&self, name: impl Into<String>) -> ScopeContext {
        let scope = {
            let stack = self.scope_stack.read().await;
            if let Some(parent) = stack.last() {
                ScopeContext::with_parent(name, parent)
            } else {
                ScopeContext::new(name)
            }
        };
        
        // 添加到作用域栈
        {
            let mut stack = self.scope_stack.write().await;
            stack.push(scope.clone());
        }
        
        // 初始化作用域实例缓存
        {
            let mut scoped_instances = self.scoped_instances.write().await;
            scoped_instances.insert(scope.id.clone(), DashMap::new());
        }
        
        scope
    }
    
    /// 退出作用域
    pub async fn exit_scope(&self, scope: &ScopeContext) -> ContainerResult<()> {
        // 从作用域栈中移除
        {
            let mut stack = self.scope_stack.write().await;
            stack.retain(|s| s.id != scope.id);
        }
        
        // 清理作用域实例并调用销毁方法
        {
            let mut scoped_instances = self.scoped_instances.write().await;
            if let Some(scope_cache) = scoped_instances.remove(&scope.id) {
                for entry in scope_cache.into_iter() {
                    // Note: We can't directly downcast to dyn Component here
                    // In a real implementation, we'd need to store component types
                    // or implement a proper cleanup mechanism
                    println!("Cleaning up component: {}", entry.0);
                }
            }
        }
        
        Ok(())
    }
    
    /// 在指定作用域中执行
    pub async fn with_scope<F, R>(&self, name: impl Into<String>, f: F) -> ContainerResult<R>
    where
        F: FnOnce(&Self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ContainerResult<R>> + Send>>,
    {
        let scope = self.create_scope(name).await;
        let result = f(self).await;
        self.exit_scope(&scope).await?;
        result
    }
    
    /// 获取当前作用域
    async fn get_current_scope(&self) -> Option<ScopeContext> {
        let stack = self.scope_stack.read().await;
        stack.last().cloned()
    }
    
    /// 通过名称解析组件
    pub async fn resolve_by_name(&self, name: &str) -> ContainerResult<Arc<dyn Any + Send + Sync>> {
        // 检查是否正在解析（循环依赖检测）
        {
            let mut resolving = self.resolving.write().await;
            if resolving.contains(name) {
                return Err(ContainerError::CircularDependency {
                    components: vec![name.to_string()],
                });
            }
            resolving.insert(name.to_string());
        }
        
        let result = self.resolve_by_name_internal(name).await;
        
        // 清理解析状态
        {
            let mut resolving = self.resolving.write().await;
            resolving.remove(name);
        }
        
        result
    }
    
    /// 为实例创建AOP代理包装器（如果需要）
    fn create_aop_proxy_if_needed(&self, instance: Arc<dyn Any + Send + Sync>, auto_proxy: bool) -> Arc<dyn Any + Send + Sync> {
        if !auto_proxy {
            return instance;
        }
        
        // 由于Rust的类型系统限制，我们无法在运行时为任意类型创建动态代理
        // 因此，我们采用一个标记机制：为启用auto_proxy的组件设置一个特殊标记
        // 在实际使用中，开发者需要在服务方法中手动调用apply_aspects或使用代理宏
        
        println!("🔧 [容器] 为组件启用自动代理模式 (类型: {})", 
                std::any::type_name_of_val(&*instance));
        
        // 我们可以创建一个包装结构来标记这个实例已启用AOP
        // 但由于类型擦除的限制，目前只能返回原实例
        // 开发者需要在服务实现中使用AOP宏或手动调用apply_aspects
        
        instance
    }
    
    async fn resolve_by_name_internal(&self, name: &str) -> ContainerResult<Arc<dyn Any + Send + Sync>> {
        let component_ref = global_registry()
            .get_component_by_name(name)
            .ok_or_else(|| ContainerError::ComponentNotFound {
                name: name.to_string(),
            })?;
            
        let lifecycle = component_ref.info.lifecycle;
        let component_name = component_ref.info.name.clone();
        let auto_proxy = component_ref.info.auto_proxy;
        
        match lifecycle {
            Lifecycle::Singleton => {
                // 检查单例缓存
                if let Some(instance) = self.singleton_instances.get(&component_name) {
                    return Ok(instance.clone());
                }
                
                // 创建新实例
                let factory = &component_ref.factory;
                // We need to pass self as Arc<Container> - this needs refactoring
                let self_arc = global_container();
                let mut instance = factory(self_arc).await?;
                
                // 应用AOP代理（如果需要）
                instance = self.create_aop_proxy_if_needed(instance, auto_proxy);
                
                // 缓存实例
                self.singleton_instances.insert(component_name, instance.clone());
                Ok(instance)
            }
            
            Lifecycle::Scoped => {
                let current_scope = self.get_current_scope().await
                    .ok_or_else(|| ContainerError::LifecycleError {
                        message: format!("No active scope for scoped component: {}", name),
                    })?;
                
                // 检查作用域缓存
                {
                    let scoped_instances = self.scoped_instances.read().await;
                    if let Some(scope_cache) = scoped_instances.get(&current_scope.id) {
                        if let Some(instance) = scope_cache.get(&component_name) {
                            return Ok(instance.clone());
                        }
                    }
                }
                
                // 创建新实例
                let factory = &component_ref.factory;
                // We need to pass self as Arc<Container> - this needs refactoring
                let self_arc = global_container();
                let mut instance = factory(self_arc).await?;
                
                // 应用AOP代理（如果需要）
                instance = self.create_aop_proxy_if_needed(instance, auto_proxy);
                
                // 缓存到作用域
                {
                    let scoped_instances = self.scoped_instances.read().await;
                    if let Some(scope_cache) = scoped_instances.get(&current_scope.id) {
                        scope_cache.insert(component_name, instance.clone());
                    }
                }
                
                Ok(instance)
            }
            
            Lifecycle::Transient => {
                // 每次都创建新实例
                let factory = &component_ref.factory;
                let self_arc = global_container();
                let mut instance = factory(self_arc).await?;
                
                // 应用AOP代理（如果需要）
                instance = self.create_aop_proxy_if_needed(instance, auto_proxy);
                
                Ok(instance)
            }
        }
    }
}

impl Container {
    /// 解析特定类型的组件
    pub async fn resolve<T: Component + 'static>(&self) -> ContainerResult<Arc<T>> {
        let component_ref = global_registry()
            .get_component_by_type::<T>()
            .ok_or_else(|| ContainerError::ComponentNotFound {
                name: T::component_name().to_string(),
            })?;
            
        let component_name = component_ref.info.name.clone();
        drop(component_ref);
        
        let instance = self.resolve_by_name(&component_name).await?;
        
        // Use Arc::downcast instead of downcast_arc
        let any_arc = instance as Arc<dyn Any + Send + Sync>;
        any_arc.downcast::<T>()
            .map_err(|_| ContainerError::TypeMismatch {
                expected: std::any::type_name::<T>().to_string(),
                actual: "Unknown".to_string(),
            })
    }
    
    /// 解析可变组件（自动选择合适的包装器）
    pub async fn resolve_mutable<T>(&self) -> ContainerResult<Arc<dyn Any + Send + Sync>>
    where
        T: MutableComponent + 'static,
    {
        let component = self.resolve::<T>().await?;
        let wrapped = if T::requires_async_lock() {
            if T::supports_concurrent_read() {
                Arc::new(AsyncRwLockWrapper::new(Arc::try_unwrap(component).map_err(|_| {
                    ContainerError::ComponentCreationFailed {
                        name: T::component_name().to_string(),
                        source: anyhow::anyhow!("Failed to unwrap Arc for mutable component"),
                    }
                })?)) as Arc<dyn Any + Send + Sync>
            } else {
                Arc::new(AsyncMutexWrapper::new(Arc::try_unwrap(component).map_err(|_| {
                    ContainerError::ComponentCreationFailed {
                        name: T::component_name().to_string(),
                        source: anyhow::anyhow!("Failed to unwrap Arc for mutable component"),
                    }
                })?)) as Arc<dyn Any + Send + Sync>
            }
        } else {
            if T::supports_concurrent_read() {
                Arc::new(RwLockWrapper::new(Arc::try_unwrap(component).map_err(|_| {
                    ContainerError::ComponentCreationFailed {
                        name: T::component_name().to_string(),
                        source: anyhow::anyhow!("Failed to unwrap Arc for mutable component"),
                    }
                })?)) as Arc<dyn Any + Send + Sync>
            } else {
                Arc::new(MutexWrapper::new(Arc::try_unwrap(component).map_err(|_| {
                    ContainerError::ComponentCreationFailed {
                        name: T::component_name().to_string(),
                        source: anyhow::anyhow!("Failed to unwrap Arc for mutable component"),
                    }
                })?)) as Arc<dyn Any + Send + Sync>
            }
        };
        
        Ok(wrapped)
    }
    
    /// 解析为Mutex包装的组件
    pub async fn resolve_mutex<T>(&self) -> ContainerResult<Arc<MutexWrapper<T>>>
    where
        T: MutableComponent + 'static,
    {
        let component = self.resolve::<T>().await?;
        let unwrapped = Arc::try_unwrap(component).map_err(|_| {
            ContainerError::ComponentCreationFailed {
                name: T::component_name().to_string(),
                source: anyhow::anyhow!("Failed to unwrap Arc for mutable component"),
            }
        })?;
        Ok(Arc::new(MutexWrapper::new(unwrapped)))
    }
    
    /// 解析为RwLock包装的组件
    pub async fn resolve_rwlock<T>(&self) -> ContainerResult<Arc<RwLockWrapper<T>>>
    where
        T: MutableComponent + 'static,
    {
        let component = self.resolve::<T>().await?;
        let unwrapped = Arc::try_unwrap(component).map_err(|_| {
            ContainerError::ComponentCreationFailed {
                name: T::component_name().to_string(),
                source: anyhow::anyhow!("Failed to unwrap Arc for mutable component"),
            }
        })?;
        Ok(Arc::new(RwLockWrapper::new(unwrapped)))
    }
    
    /// 解析为异步Mutex包装的组件
    pub async fn resolve_async_mutex<T>(&self) -> ContainerResult<Arc<AsyncMutexWrapper<T>>>
    where
        T: MutableComponent + 'static,
    {
        let component = self.resolve::<T>().await?;
        let unwrapped = Arc::try_unwrap(component).map_err(|_| {
            ContainerError::ComponentCreationFailed {
                name: T::component_name().to_string(),
                source: anyhow::anyhow!("Failed to unwrap Arc for mutable component"),
            }
        })?;
        Ok(Arc::new(AsyncMutexWrapper::new(unwrapped)))
    }
    
    /// 解析为异步RwLock包装的组件
    pub async fn resolve_async_rwlock<T>(&self) -> ContainerResult<Arc<AsyncRwLockWrapper<T>>>
    where
        T: MutableComponent + 'static,
    {
        let component = self.resolve::<T>().await?;
        let unwrapped = Arc::try_unwrap(component).map_err(|_| {
            ContainerError::ComponentCreationFailed {
                name: T::component_name().to_string(),
                source: anyhow::anyhow!("Failed to unwrap Arc for mutable component"),
            }
        })?;
        Ok(Arc::new(AsyncRwLockWrapper::new(unwrapped)))
    }
}

#[async_trait]
impl ComponentResolver for Container {
    async fn resolve_by_name(&self, name: &str) -> ContainerResult<Arc<dyn Any + Send + Sync>> {
        self.resolve_by_name(name).await
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}