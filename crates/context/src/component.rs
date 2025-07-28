use std::{
    any::{Any, TypeId},
    fmt::Debug,
    future::Future,
    pin::Pin,
    sync::Arc,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{lifecycle::Lifecycle, error::ContainerResult};

use crate::container::Container;

/// 组件工厂函数类型
pub type ComponentFactory = Box<
    dyn Fn(Arc<Container>) -> Pin<Box<dyn Future<Output = ContainerResult<Arc<dyn Any + Send + Sync>>> + Send>>
        + Send
        + Sync,
>;

/// 组件解析器trait，用于解析依赖
#[async_trait]
pub trait ComponentResolver: Send + Sync {
    async fn resolve_by_name(&self, name: &str) -> ContainerResult<Arc<dyn Any + Send + Sync>>;
}

/// 组件trait，所有可注入的组件都必须实现此trait
#[async_trait]
pub trait Component: Any + Send + Sync + Debug {
    /// 组件名称，默认为类型名
    fn component_name() -> &'static str
    where
        Self: Sized,
    {
        std::any::type_name::<Self>()
    }
    
    /// 组件生命周期，默认为单例
    fn lifecycle() -> Lifecycle
    where
        Self: Sized,
    {
        Lifecycle::Singleton  
    }
    
    /// 异步初始化方法，组件创建后会被调用
    async fn initialize(&self) -> ContainerResult<()> {
        Ok(())
    }
    
    /// 异步销毁方法，组件销毁前会被调用
    async fn destroy(&self) -> ContainerResult<()> {
        Ok(())
    }
}

/// 组件依赖信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub name: String,
    pub type_name: String,
    pub optional: bool,
}

/// 组件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub type_name: String,
    pub type_id: String, // TypeId的字符串表示
    pub lifecycle: Lifecycle,
    pub dependencies: Vec<DependencyInfo>,
    pub initialized: bool,
    pub auto_proxy: bool, // 是否自动创建AOP代理
}

impl ComponentInfo {
    pub fn new<T: Component>() -> Self {
        Self {
            name: T::component_name().to_string(),
            type_name: std::any::type_name::<T>().to_string(),
            type_id: format!("{:?}", TypeId::of::<T>()),
            lifecycle: T::lifecycle(),
            dependencies: Vec::new(),
            initialized: false,
            auto_proxy: false, // 默认不启用自动代理
        }
    }
    
    /// 创建带自动代理配置的组件信息
    pub fn new_with_auto_proxy<T: Component>(auto_proxy: bool) -> Self {
        Self {
            name: T::component_name().to_string(),
            type_name: std::any::type_name::<T>().to_string(),
            type_id: format!("{:?}", TypeId::of::<T>()),
            lifecycle: T::lifecycle(),
            dependencies: Vec::new(),
            initialized: false,
            auto_proxy,
        }
    }
}

/// 组件实例包装器
pub struct ComponentInstance {
    pub info: ComponentInfo,
    pub instance: Option<Arc<dyn Any + Send + Sync>>,
    pub factory: ComponentFactory,
}

impl Debug for ComponentInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentInstance")
            .field("info", &self.info)
            .field("has_instance", &self.instance.is_some())
            .finish()
    }
}