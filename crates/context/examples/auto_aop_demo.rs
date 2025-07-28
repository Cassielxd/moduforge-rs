//! 自动AOP拦截演示
//! 
//! 展示如何使用AOP代理系统自动拦截服务方法调用，
//! 无需手动调用apply_aspects

use mf_context::*;
use std::{fmt::Debug, sync::Arc, collections::HashMap};
use async_trait::async_trait;

// ============ 业务服务定义 ============

/// 用户服务
#[derive(Debug, Default, Component)]
#[component(name = "user_service", lifecycle = "singleton")]
pub struct UserService {
    users: HashMap<String, String>,
}

impl UserService {
    pub async fn create_user(&self, name: &str) -> ContainerResult<String> {
        println!("执行用户创建逻辑: {}", name);
        let user_id = format!("user_{}", std::process::id());
        Ok(user_id)
    }
    
    pub async fn get_user(&self, id: &str) -> ContainerResult<Option<String>> {
        println!("执行用户查询逻辑: {}", id);
        if id == "error" {
            return Err(ContainerError::ComponentCreationFailed {
                name: "UserService".to_string(),
                source: anyhow::anyhow!("用户不存在"),
            });
        }
        Ok(Some(format!("用户数据_{}", id)))
    }
    
    pub async fn update_user(&self, id: &str, name: &str) -> ContainerResult<()> {
        println!("执行用户更新逻辑: {} -> {}", id, name);
        Ok(())
    }
    
    pub async fn delete_user(&self, id: &str) -> ContainerResult<()> {
        println!("执行用户删除逻辑: {}", id);
        Ok(())
    }
}

/// 订单服务
#[derive(Debug, Default, Component)]
#[component(name = "order_service", lifecycle = "singleton")]
pub struct OrderService {
    orders: HashMap<String, String>,
}

impl OrderService {
    pub async fn create_order(&self, user_id: &str, amount: f64) -> ContainerResult<String> {
        println!("执行订单创建逻辑: 用户={}, 金额={}", user_id, amount);
        let order_id = format!("order_{}", std::process::id());
        Ok(order_id)
    }
    
    pub async fn get_order(&self, id: &str) -> ContainerResult<Option<String>> {
        println!("执行订单查询逻辑: {}", id);
        Ok(Some(format!("订单数据_{}", id)))
    }
}

// ============ AOP切面定义 ============

/// 日志切面
#[derive(Debug, Default, BeforeAspect)]
#[aspect(name = "自动日志切面", pointcut = "*Service::*", priority = 1)]
pub struct AutoLoggingAspect;

#[async_trait]
impl BeforeAspect for AutoLoggingAspect {
    async fn before(&self, context: &mut MethodContext) -> ContainerResult<()> {
        let args_str = if !context.args.is_empty() {
            format!(" 参数=[{}]", context.args.join(", "))
        } else {
            String::new()
        };
        
        println!("🔍 [{}] -> {}::{}{}", 
                self.name(), 
                context.target_type, 
                context.method_name,
                args_str);
        Ok(())
    }
    
    fn name(&self) -> &str {
        "自动日志切面"
    }
    
    fn priority(&self) -> i32 {
        1
    }
}

/// 性能监控切面
#[derive(Debug, Default, AfterAspect)]
#[aspect(name = "自动性能监控", type_pattern = "*Service", method_pattern = "*", priority = 2)]
pub struct AutoPerformanceAspect;

#[async_trait]
impl AfterAspect for AutoPerformanceAspect {
    async fn after(&self, context: &MethodContext, _result: &ContainerResult<Box<dyn std::any::Any + Send + Sync>>) {
        let elapsed = context.elapsed().as_millis();
        println!("⏱️  [{}] {}::{} 执行完成，耗时: {}ms", 
                self.name(),
                context.target_type, 
                context.method_name,
                elapsed);
    }
    
    fn name(&self) -> &str {
        "自动性能监控"
    }
    
    fn priority(&self) -> i32 {
        2
    }
}

/// 错误处理切面
#[derive(Debug, Default, AfterThrowingAspect)]
#[aspect(name = "自动错误处理", pointcut = "*Service::*", priority = 3)]
pub struct AutoErrorHandlingAspect;

#[async_trait]
impl AfterThrowingAspect for AutoErrorHandlingAspect {
    async fn after_throwing(&self, context: &MethodContext, error: &ContainerError) {
        println!("❌ [{}] {}::{} 发生错误: {} (耗时: {}ms)", 
                self.name(), 
                context.target_type, 
                context.method_name,
                error,
                context.elapsed().as_millis());
        
        // 自动错误报告
        println!("📊 [{}] 已自动记录错误到监控系统", self.name());
    }
    
    fn name(&self) -> &str {
        "自动错误处理"
    }
    
    fn priority(&self) -> i32 {
        3
    }
}

// ============ 自动AOP代理服务 ============

/// 用户服务的AOP代理
pub struct UserServiceProxy {
    wrapper: AopProxyWrapper<UserService>,
}

impl UserServiceProxy {
    pub fn new(inner: Arc<UserService>) -> Self {
        Self { 
            wrapper: AopProxyWrapper::new(inner),
        }
    }
    
    /// 自动应用AOP的创建用户方法
    pub async fn create_user(&self, name: &str) -> ContainerResult<String> {
        let args = vec![format!("{:?}", name)];
        let inner: Arc<UserService> = self.wrapper.inner().clone();
        self.wrapper.proxy_call(
            "create_user",
            args,
            || async move {
                inner.create_user(name).await
            }
        ).await
    }
    
    /// 自动应用AOP的获取用户方法  
    pub async fn get_user(&self, id: &str) -> ContainerResult<Option<String>> {
        let args = vec![format!("{:?}", id)];
        let inner = self.wrapper.inner().clone();
        self.wrapper.proxy_call(
            "get_user",
            args,
            || async move {
                inner.get_user(id).await
            }
        ).await
    }
    
    /// 自动应用AOP的更新用户方法
    pub async fn update_user(&self, id: &str, name: &str) -> ContainerResult<()> {
        let args = vec![format!("{:?}", id), format!("{:?}", name)];
        let inner = self.wrapper.inner().clone();
        self.wrapper.proxy_call(
            "update_user",
            args,
            || async move {
                inner.update_user(id, name).await
            }
        ).await
    }
    
    /// 自动应用AOP的删除用户方法
    pub async fn delete_user(&self, id: &str) -> ContainerResult<()> {
        let args = vec![format!("{:?}", id)];
        let inner = self.wrapper.inner().clone();
        self.wrapper.proxy_call(
            "delete_user",
            args,
            || async move {
                inner.delete_user(id).await
            }
        ).await
    }
}

/// 订单服务的AOP代理
pub struct OrderServiceProxy {
    wrapper: AopProxyWrapper<OrderService>,
}

impl OrderServiceProxy {
    pub fn new(inner: Arc<OrderService>) -> Self {
        Self { 
            wrapper: AopProxyWrapper::new(inner),
        }
    }
    
    /// 自动应用AOP的创建订单方法
    pub async fn create_order(&self, user_id: &str, amount: f64) -> ContainerResult<String> {
        let args = vec![format!("{:?}", user_id), format!("{:?}", amount)];
        let inner = self.wrapper.inner().clone();
        self.wrapper.proxy_call(
            "create_order",
            args,
            || async move {
                inner.create_order(user_id, amount).await
            }
        ).await
    }
    
    /// 自动应用AOP的获取订单方法
    pub async fn get_order(&self, id: &str) -> ContainerResult<Option<String>> {
        let args = vec![format!("{:?}", id)];
        let inner = self.wrapper.inner().clone();
        self.wrapper.proxy_call(
            "get_order",
            args,
            || async move {
                inner.get_order(id).await
            }
        ).await
    }
}

// ============ 演示函数 ============

/// 演示自动AOP拦截功能
async fn demo_auto_aop() -> ContainerResult<()> {
    let container = global_container();
    
    // 获取原始服务
    let user_service: Arc<UserService> = container.resolve().await?;
    let order_service: Arc<OrderService> = container.resolve().await?;
    
    // 创建AOP代理
    let user_proxy = UserServiceProxy::new(user_service);
    let order_proxy = OrderServiceProxy::new(order_service);
    
    println!("\n=== 演示自动AOP拦截 - 用户服务 ===");
    
    // 创建用户 - 自动触发日志和性能监控切面
    println!("\n>> 创建用户:");
    let user_id = user_proxy.create_user("Alice").await?;
    println!("✅ 用户创建成功: {}", user_id);
    
    // 查询用户 - 自动触发AOP切面
    println!("\n>> 查询用户:");
    if let Some(user_data) = user_proxy.get_user(&user_id).await? {
        println!("✅ 用户查询成功: {}", user_data);
    }
    
    // 更新用户 - 自动触发AOP切面
    println!("\n>> 更新用户:");
    user_proxy.update_user(&user_id, "Alice Smith").await?;
    println!("✅ 用户更新成功");
    
    // 删除用户 - 自动触发AOP切面
    println!("\n>> 删除用户:");
    user_proxy.delete_user(&user_id).await?;
    println!("✅ 用户删除成功");
    
    println!("\n=== 演示自动AOP拦截 - 订单服务 ===");
    
    // 创建订单 - 自动触发AOP切面
    println!("\n>> 创建订单:");
    let order_id = order_proxy.create_order(&user_id, 99.99).await?;
    println!("✅ 订单创建成功: {}", order_id);
    
    // 查询订单 - 自动触发AOP切面
    println!("\n>> 查询订单:");
    if let Some(order_data) = order_proxy.get_order(&order_id).await? {
        println!("✅ 订单查询成功: {}", order_data);
    }
    
    println!("\n=== 演示错误处理切面 ===");
    
    // 触发错误 - 自动触发错误处理切面
    println!("\n>> 查询不存在的用户:");
    let _ = user_proxy.get_user("error").await;
    println!("✅ 错误已被切面自动处理");
    
    Ok(())
}

/// 使用AopProxyWrapper的更简单方式
async fn demo_wrapper_proxy() -> ContainerResult<()> {
    let container = global_container();
    
    println!("\n=== 演示通用AOP代理包装器 ===");
    
    // 获取服务并创建代理包装器
    let user_service: Arc<UserService> = container.resolve().await?;
    let user_wrapper = AopProxyWrapper::new(user_service);
    
    // 使用代理包装器调用方法
    println!("\n>> 使用包装器创建用户:");
    let args = vec!["Bob".to_string()];
    let inner = user_wrapper.inner().clone();
    let user_id = user_wrapper.proxy_call(
        "create_user",
        args,
        || async move {
            inner.create_user("Bob").await
        }
    ).await?;
    println!("✅ 包装器用户创建成功: {}", user_id);
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ModuForge 自动AOP拦截演示 ===\n");
    
    // 初始化容器（切面会通过宏自动注册）
    initialize_container().await?;
    
    println!("✅ 切面自动注册完成:");
    println!("   - AutoLoggingAspect: 自动日志切面");
    println!("   - AutoPerformanceAspect: 自动性能监控切面");
    println!("   - AutoErrorHandlingAspect: 自动错误处理切面");
    
    // 演示自动AOP拦截
    demo_auto_aop().await?;
    
    // 演示通用代理包装器
    demo_wrapper_proxy().await?;
    
    println!("\n=== 自动AOP拦截演示完成 ===");
    println!("🎉 所有服务方法调用都自动应用了AOP切面！");
    println!("💡 无需手动调用 apply_aspects，代理会自动处理！");
    
    Ok(())
}