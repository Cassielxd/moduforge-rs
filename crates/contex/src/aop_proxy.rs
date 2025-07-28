//! AOP代理系统
//! 
//! 提供自动AOP拦截功能，让服务方法调用自动应用切面

use std::sync::Arc;
use async_trait::async_trait;
use crate::{ContainerResult, aop::apply_aspects};

/// AOP代理trait，为服务提供自动切面拦截
#[async_trait]
pub trait AopProxy<T> {
    /// 创建代理实例
    fn create_proxy(inner: Arc<T>) -> Self;
    
    /// 获取内部服务实例
    fn inner(&self) -> &Arc<T>;
}

/// 自动代理宏，为服务方法添加AOP拦截
#[macro_export]
macro_rules! aop_proxy {
    (
        $service_type:ty,
        $proxy_name:ident,
        {
            $(
                async fn $method_name:ident(&self $(, $param_name:ident: $param_type:ty)*) -> $return_type:ty $method_body:block
            )*
        }
    ) => {
        pub struct $proxy_name {
            inner: Arc<$service_type>,
        }
        
        #[async_trait::async_trait]
        impl AopProxy<$service_type> for $proxy_name {
            fn create_proxy(inner: Arc<$service_type>) -> Self {
                Self { inner }
            }
            
            fn inner(&self) -> &Arc<$service_type> {
                &self.inner
            }
        }
        
        impl $proxy_name {
            $(
                pub async fn $method_name(&self $(, $param_name: $param_type)*) -> $return_type {
                    let inner = self.inner.clone();
                    let args = vec![$(format!("{:?}", $param_name)),*];
                    
                    apply_aspects(
                        stringify!($service_type),
                        stringify!($method_name),
                        args,
                        || async move {
                            inner.$method_name($($param_name,)*).await
                        }
                    ).await
                }
            )*
        }
    };
}

/// 更简单的代理创建宏
#[macro_export] 
macro_rules! create_aop_proxy {
    ($service:expr, $service_type:ty) => {{
        AopProxyWrapper::new($service)
    }};
}

/// 通用AOP代理包装器
pub struct AopProxyWrapper<T> {
    inner: Arc<T>,
    type_name: &'static str,
}

impl<T> AopProxyWrapper<T> {
    pub fn new(inner: Arc<T>) -> Self {
        Self {
            inner,
            type_name: std::any::type_name::<T>(),
        }
    }
    
    pub fn inner(&self) -> &Arc<T> {
        &self.inner
    }
    
    /// 代理异步方法调用
    pub async fn proxy_call<F, Fut, R>(&self, method_name: &str, args: Vec<String>, f: F) -> ContainerResult<R>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = ContainerResult<R>> + Send,
        R: Send + 'static,
    {
        apply_aspects(self.type_name, method_name, args, f).await
    }
}

/// 自动生成代理方法的宏
#[macro_export]
macro_rules! proxy_method {
    ($proxy:expr, $method_name:ident, $($arg:expr),*) => {{
        let args = vec![$(format!("{:?}", $arg)),*];
        let inner = $proxy.inner().clone();
        $proxy.proxy_call(
            stringify!($method_name),
            args,
            || async move {
                inner.$method_name($($arg,)*).await
            }
        ).await
    }};
}