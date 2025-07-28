//! AOP宏自动注册演示
//! 
//! 展示如何使用宏自动注册AOP切面，包括：
//! - BeforeAspect: 前置切面自动注册
//! - AfterAspect: 后置切面自动注册  
//! - AfterReturningAspect: 返回后切面自动注册
//! - AfterThrowingAspect: 异常后切面自动注册
//! - AroundAspect: 环绕切面自动注册

use mf_context::*;
use std::{fmt::Debug, sync::Arc};
use async_trait::async_trait;

// ============ 业务服务 ============

/// 用户服务
#[derive(Debug, Default, Component)]
#[component(name = "user_service", lifecycle = "singleton")]
pub struct UserService {
    users: std::collections::HashMap<String, String>,
}

impl UserService {
    pub async fn create_user(&self, name: &str) -> ContainerResult<String> {
        println!("创建用户: {}", name);
        let user_id = format!("user_{}", std::process::id());
        Ok(user_id)
    }
    
    pub async fn get_user(&self, id: &str) -> ContainerResult<Option<String>> {
        println!("查找用户: {}", id);
        if id == "error" {
            return Err(ContainerError::ComponentCreationFailed {
                name: "UserService".to_string(),
                source: anyhow::anyhow!("用户不存在"),
            });
        }
        Ok(Some(format!("用户数据_{}", id)))
    }
    
    pub async fn update_user(&self, id: &str, name: &str) -> ContainerResult<()> {
        println!("更新用户: {} -> {}", id, name);
        Ok(())
    }
    
    pub async fn delete_user(&self, id: &str) -> ContainerResult<()> {
        println!("删除用户: {}", id);
        Ok(())
    }
}

/// 订单服务
#[derive(Debug, Default, Component)]
#[component(name = "order_service", lifecycle = "singleton")]
pub struct OrderService {
    orders: std::collections::HashMap<String, String>,
}

impl OrderService {
    pub async fn create_order(&self, user_id: &str, amount: f64) -> ContainerResult<String> {
        println!("创建订单: 用户={}, 金额={}", user_id, amount);
        let order_id = format!("order_{}", std::process::id());
        Ok(order_id)
    }
    
    pub async fn get_order(&self, id: &str) -> ContainerResult<Option<String>> {
        println!("查找订单: {}", id);
        Ok(Some(format!("订单数据_{}", id)))
    }
}

// ============ AOP切面定义（使用宏自动注册） ============

/// 日志切面 - 前置通知
#[derive(Debug, Default, BeforeAspect)]
#[aspect(name = "日志前置切面", pointcut = "*Service::*", priority = 1)]
pub struct LoggingBeforeAspect;

#[async_trait]
impl BeforeAspect for LoggingBeforeAspect {
    async fn before(&self, context: &mut MethodContext) -> ContainerResult<()> {
        let args_str = if !context.args.is_empty() {
            format!(" 参数=[{}]", context.args.join(", "))
        } else {
            String::new()
        };
        
        println!("[{}] -> {}::{}{}", 
                self.name(), 
                context.target_type, 
                context.method_name,
                args_str);
        Ok(())
    }
    
    fn name(&self) -> &str {
        "日志前置切面"
    }
    
    fn priority(&self) -> i32 {
        1
    }
}

/// 性能监控切面 - 后置通知
#[derive(Debug, Default, AfterAspect)]
#[aspect(name = "性能监控切面", type_pattern = "*Service", method_pattern = "*", priority = 2)]
pub struct PerformanceAfterAspect {
    threshold_ms: u64,
}

impl PerformanceAfterAspect {
    pub fn new(threshold_ms: u64) -> Self {
        Self { threshold_ms }
    }
}

#[async_trait]
impl AfterAspect for PerformanceAfterAspect {
    async fn after(&self, context: &MethodContext, _result: &ContainerResult<Box<dyn std::any::Any + Send + Sync>>) {
        let elapsed = context.elapsed().as_millis() as u64;
        if elapsed > self.threshold_ms {
            println!("[{}] 慢方法: {}::{} 耗时 {}ms (阈值: {}ms)", 
                    self.name(),
                    context.target_type, 
                    context.method_name,
                    elapsed,
                    self.threshold_ms);
        } else {
            println!("[{}] 方法执行完成: {}::{} 耗时 {}ms", 
                    self.name(),
                    context.target_type, 
                    context.method_name,
                    elapsed);
        }
    }
    
    fn name(&self) -> &str {
        "性能监控切面"
    }
    
    fn priority(&self) -> i32 {
        2
    }
}

/// 成功日志切面 - 返回后通知
#[derive(Debug, Default, AfterReturningAspect)]
#[aspect(name = "成功日志切面", pointcut = "UserService::*", priority = 3)]
pub struct SuccessLoggingAspect;

#[async_trait]
impl AfterReturningAspect for SuccessLoggingAspect {
    async fn after_returning(&self, context: &MethodContext, _result: &Box<dyn std::any::Any + Send + Sync>) {
        println!("[{}] <- {}::{} 成功 耗时={}ms", 
                self.name(), 
                context.target_type, 
                context.method_name,
                context.elapsed().as_millis());
    }
    
    fn name(&self) -> &str {
        "成功日志切面"
    }
    
    fn priority(&self) -> i32 {
        3
    }
}

/// 异常处理切面 - 异常后通知
#[derive(Debug, Default, AfterThrowingAspect)]
#[aspect(name = "异常处理切面", type_pattern = "*Service", method_pattern = "get_*", priority = 4)]
pub struct ErrorHandlingAspect;

#[async_trait]
impl AfterThrowingAspect for ErrorHandlingAspect {
    async fn after_throwing(&self, context: &MethodContext, error: &ContainerError) {
        println!("[{}] <- {}::{} 错误: {} 耗时={}ms", 
                self.name(), 
                context.target_type, 
                context.method_name,
                error,
                context.elapsed().as_millis());
        
        // 这里可以添加错误处理逻辑，比如发送告警、记录错误日志等
        println!("[{}] 已记录错误信息到监控系统", self.name());
    }
    
    fn name(&self) -> &str {
        "异常处理切面"
    }
    
    fn priority(&self) -> i32 {
        4
    }
}

/// 权限检查切面 - 环绕通知
#[derive(Debug, Default, AroundAspect)]
#[aspect(name = "权限检查切面", pointcut = "*Service::delete_*", priority = 0)]
pub struct SecurityAspect;

#[async_trait]
impl AroundAspect for SecurityAspect {
    async fn before_proceed(&self, context: &mut MethodContext) -> ContainerResult<bool> {
        println!("[{}] 检查权限: {}::{}", 
                self.name(),
                context.target_type, 
                context.method_name);
        
        // 模拟权限检查
        if context.method_name.starts_with("delete_") {
            println!("[{}] 权限检查通过: 允许删除操作", self.name());
            context.set_attribute("permission", "granted");
            Ok(true) // 允许继续执行
        } else {
            Ok(true)
        }
    }
    
    async fn after_proceed(&self, context: &MethodContext, result: &ContainerResult<Box<dyn std::any::Any + Send + Sync>>) {
        if let Some(permission) = context.get_attribute("permission") {
            match result {
                Ok(_) => println!("[{}] 权限操作完成: {} ({})", 
                                self.name(), 
                                context.method_name, 
                                permission),
                Err(_) => println!("[{}] 权限操作失败: {} ({})", 
                                 self.name(), 
                                 context.method_name, 
                                 permission),
            }
        }
    }
    
    fn name(&self) -> &str {
        "权限检查切面"
    }
    
    fn priority(&self) -> i32 {
        0 // 最高优先级，最先执行
    }
}

// ============ 演示函数 ============

/// 演示AOP切面功能
async fn demo_aop_functionality() -> ContainerResult<()> {
    let container = global_container();
    
    // 获取服务
    let user_service: Arc<UserService> = container.resolve().await?;
    let order_service: Arc<OrderService> = container.resolve().await?;
    
    println!("\n=== 演示前置切面和成功切面 ===");
    let user_id = apply_aspects(
        "UserService",
        "create_user",
        vec!["Alice".to_string()],
        || user_service.create_user("Alice")
    ).await?;
    
    println!("\n=== 演示性能监控切面 ===");
    apply_aspects(
        "UserService",
        "get_user", 
        vec![user_id.clone()],
        || user_service.get_user(&user_id)
    ).await?;
    
    println!("\n=== 演示异常处理切面 ===");
    let _ = apply_aspects(
        "UserService",
        "get_user",
        vec!["error".to_string()],
        || user_service.get_user("error")
    ).await;
    
    println!("\n=== 演示环绕切面（权限检查） ===");
    apply_aspects(
        "UserService", 
        "delete_user",
        vec![user_id.clone()],
        || user_service.delete_user(&user_id)
    ).await?;
    
    println!("\n=== 演示订单服务切面 ===");
    let order_id = apply_aspects(
        "OrderService",
        "create_order", 
        vec![user_id.clone(), "99.99".to_string()],
        || order_service.create_order(&user_id, 99.99)
    ).await?;
    
    apply_aspects(
        "OrderService",
        "get_order",
        vec![order_id.clone()],
        || order_service.get_order(&order_id)
    ).await?;
    
    println!("\n=== 演示更新操作（仅前置和后置切面） ===");
    apply_aspects(
        "UserService",
        "update_user",
        vec![user_id.clone(), "Alice Smith".to_string()],
        || user_service.update_user(&user_id, "Alice Smith")
    ).await?;
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ModuForge AOP 宏自动注册演示 ===\n");
    
    // 初始化容器（切面会通过宏自动注册）
    initialize_container().await?;
    
    println!("=== 切面自动注册信息 ===");
    let aspect_manager = get_aspect_manager();
    let manager = aspect_manager.read().unwrap();
    
    // 显示注册的切面信息
    println!("已注册的切面：");
    println!("- 前置切面: LoggingBeforeAspect");
    println!("- 后置切面: PerformanceAfterAspect");  
    println!("- 返回后切面: SuccessLoggingAspect");
    println!("- 异常后切面: ErrorHandlingAspect");
    println!("- 环绕切面: SecurityAspect");
    
    drop(manager); // 释放读锁
    
    // 演示AOP功能
    demo_aop_functionality().await?;
    
    println!("\n=== AOP宏演示完成 ===");
    println!("所有切面都通过宏自动注册，无需手动调用注册函数！");
    
    Ok(())
}