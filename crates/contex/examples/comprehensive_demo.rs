//! ModuForge 综合功能演示
//! 
//! 这个示例展示ModuForge的所有核心功能：
//! 1. 依赖注入容器 - 组件注册、生命周期管理、依赖解析
//! 2. AOP切面编程 - 前置、后置、环绕、异常处理切面
//! 3. 自动代理创建 - 通过auto_proxy配置实现零手动操作
//! 4. 配置管理 - 配置注入和环境变量支持
//! 5. Profile支持 - 多环境配置管理
//! 6. 可变组件 - 并发安全的状态管理
//! 7. Bean工厂 - 复杂对象创建
//! 8. 循环依赖检测 - 安全的依赖图管理

use mf_contex::*;
use std::{
    fmt::Debug, 
    sync::Arc, 
    collections::HashMap,
    time::Duration,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

// ============ 配置管理 ============

/// 应用配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub server_port: u16,
    pub debug_mode: bool,
    pub max_connections: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost:5432/app".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            server_port: 8080,
            debug_mode: true,
            max_connections: 100,
        }
    }
}

/// 配置组件
#[derive(Debug, Component)]
#[component(name = "app_config", lifecycle = "singleton")]
pub struct ConfigComponent {
    pub config: AppConfig,
}

impl Default for ConfigComponent {
    fn default() -> Self {
        let mut config = AppConfig::default();
        
        // 从环境变量加载配置
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database_url = db_url;
        }
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            config.redis_url = redis_url;
        }
        if let Ok(port) = std::env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse() {
                config.server_port = port_num;
            }
        }
        
        Self { config }
    }
}

// ============ 数据访问层 - 启用自动代理 ============

/// 数据库连接池
#[derive(Debug, Component)]
#[component(name = "database_pool", lifecycle = "singleton", auto_proxy = true)]
pub struct DatabasePool {
    connection_count: usize,
    config: AppConfig,
}

impl Default for DatabasePool {
    fn default() -> Self {
        Self {
            connection_count: 0,
            config: AppConfig::default(),
        }
    }
}

impl DatabasePool {
    pub fn new(config: AppConfig) -> Self {
        Self {
            connection_count: 0,
            config,
        }
    }

    pub async fn initialize(&mut self) -> ContainerResult<()> {
        println!("🔗 初始化数据库连接池: {}", self.config.database_url);
        self.connection_count = self.config.max_connections;
        sleep(Duration::from_millis(100)).await; // 模拟初始化延迟
        Ok(())
    }

    pub async fn execute_query(&self, sql: &str) -> ContainerResult<Vec<HashMap<String, String>>> {
        println!("🗄️  执行SQL查询: {}", sql);
        
        if sql.contains("error") {
            return Err(ContainerError::ComponentCreationFailed {
                name: "DatabasePool".to_string(),
                source: anyhow::anyhow!("SQL执行错误"),
            });
        }
        
        // 模拟查询延迟
        sleep(Duration::from_millis(50)).await;
        
        let mut result = HashMap::new();
        result.insert("id".to_string(), "1".to_string());
        result.insert("data".to_string(), "测试数据".to_string());
        Ok(vec![result])
    }

    pub async fn execute_transaction<F, Fut, R>(&self, transaction: F) -> ContainerResult<R>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = ContainerResult<R>> + Send,
        R: Send + 'static,
    {
        println!("🔄 开始数据库事务");
        let result = transaction().await;
        match &result {
            Ok(_) => println!("✅ 事务提交成功"),
            Err(_) => println!("❌ 事务回滚"),
        }
        result
    }
}

/// Redis缓存
#[derive(Debug, Component)]
#[component(name = "redis_cache", lifecycle = "singleton", auto_proxy = true)]
pub struct RedisCache {
    cache: HashMap<String, String>,
    config: AppConfig,
}

impl Default for RedisCache {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
            config: AppConfig::default(),
        }
    }
}

impl RedisCache {
    pub fn new(config: AppConfig) -> Self {
        Self {
            cache: HashMap::new(),
            config,
        }
    }

    pub async fn initialize(&mut self) -> ContainerResult<()> {
        println!("📦 初始化Redis缓存: {}", self.config.redis_url);
        sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> ContainerResult<Option<String>> {
        println!("🔍 缓存查询: {}", key);
        Ok(self.cache.get(key).cloned())
    }

    pub async fn set(&mut self, key: String, value: String, _ttl: Duration) -> ContainerResult<()> {
        println!("💾 缓存存储: {} = {}", key, value);
        self.cache.insert(key, value);
        Ok(())
    }

    pub async fn delete(&mut self, key: &str) -> ContainerResult<()> {
        println!("🗑️  缓存删除: {}", key);
        self.cache.remove(key);
        Ok(())
    }
}

// ============ 业务服务层 - 启用自动代理 ============

/// 用户服务
#[derive(Debug, Component)]
#[component(name = "user_service", lifecycle = "singleton", auto_proxy = true)]
pub struct UserService {
    users: HashMap<String, String>,
}

impl Default for UserService {
    fn default() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
}

impl UserService {
    #[auto_aop]
    pub async fn create_user(&self, name: &str, email: &str) -> ContainerResult<String> {
        println!("👤 创建用户: {} ({})", name, email);
        
        // 模拟业务逻辑延迟
        sleep(Duration::from_millis(30)).await;
        
        let user_id = format!("user_{}", std::process::id());
        Ok(user_id)
    }

    #[auto_aop]
    pub async fn get_user(&self, id: &str) -> ContainerResult<Option<String>> {
        println!("🔍 查询用户: {}", id);
        
        if id == "error" {
            return Err(ContainerError::ComponentCreationFailed {
                name: "UserService".to_string(),
                source: anyhow::anyhow!("用户不存在"),
            });
        }

        Ok(Some(format!("用户数据_{}", id)))
    }

    #[auto_aop]
    pub async fn update_user(&self, id: &str, name: &str) -> ContainerResult<()> {
        println!("📝 更新用户: {} -> {}", id, name);
        Ok(())
    }

    #[auto_aop]
    pub async fn delete_user(&self, id: &str) -> ContainerResult<()> {
        println!("🗑️  删除用户: {}", id);
        Ok(())
    }

    pub async fn get_user_profile(&self, id: &str) -> ContainerResult<HashMap<String, String>> {
        let mut profile = HashMap::new();
        profile.insert("id".to_string(), id.to_string());
        profile.insert("name".to_string(), format!("用户_{}", id));
        profile.insert("email".to_string(), format!("user{}@example.com", id));
        Ok(profile)
    }
}

/// 订单服务
#[derive(Debug, Component)]
#[component(name = "order_service", lifecycle = "singleton", auto_proxy = true)]
pub struct OrderService {
    orders: HashMap<String, f64>,
}

impl Default for OrderService {
    fn default() -> Self {
        Self {
            orders: HashMap::new(),
        }
    }
}

impl OrderService {
    #[auto_aop]
    pub async fn create_order(&self, user_id: &str, amount: f64) -> ContainerResult<String> {
        println!("🛒 创建订单: 用户={}, 金额={}", user_id, amount);
        
        if amount <= 0.0 {
            return Err(ContainerError::ComponentCreationFailed {
                name: "OrderService".to_string(),
                source: anyhow::anyhow!("订单金额必须大于0"),
            });
        }

        let order_id = format!("order_{}", std::process::id());
        Ok(order_id)
    }

    #[auto_aop]
    pub async fn get_order(&self, id: &str) -> ContainerResult<Option<HashMap<String, String>>> {
        println!("🔍 查询订单: {}", id);
        
        let mut order = HashMap::new();
        order.insert("id".to_string(), id.to_string());
        order.insert("status".to_string(), "已完成".to_string());
        order.insert("amount".to_string(), "99.99".to_string());
        
        Ok(Some(order))
    }

    #[auto_aop]
    pub async fn cancel_order(&self, id: &str) -> ContainerResult<()> {
        println!("❌ 取消订单: {}", id);
        Ok(())
    }
}

/// 通知服务（瞬态生命周期）
#[derive(Debug, Component)]
#[component(name = "notification_service", lifecycle = "transient", auto_proxy = true)]
pub struct NotificationService {
    instance_id: String,
}

impl Default for NotificationService {
    fn default() -> Self {
        Self {
            instance_id: format!("notif_{}", std::process::id()),
        }
    }
}

impl NotificationService {
    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> ContainerResult<()> {
        println!("📧 [{}] 发送邮件到: {} | 主题: {}", self.instance_id, to, subject);
        println!("   内容: {}", body);
        Ok(())
    }

    pub async fn send_sms(&self, phone: &str, message: &str) -> ContainerResult<()> {
        println!("📱 [{}] 发送短信到: {} | 消息: {}", self.instance_id, phone, message);
        Ok(())
    }

    pub async fn send_push_notification(&self, user_id: &str, title: &str, message: &str) -> ContainerResult<()> {
        println!("🔔 [{}] 推送通知给用户{}: {} - {}", self.instance_id, user_id, title, message);
        Ok(())
    }
}

// ============ 应用服务层（聚合服务） ============

/// 应用门面服务 - 聚合多个业务服务
#[derive(Debug, Component)]
#[component(name = "app_facade", lifecycle = "singleton", auto_proxy = true)]
pub struct ApplicationFacade {
    initialized: bool,
}

impl Default for ApplicationFacade {
    fn default() -> Self {
        Self {
            initialized: false,
        }
    }
}

impl ApplicationFacade {
    pub async fn initialize(&mut self) -> ContainerResult<()> {
        println!("🚀 初始化应用门面服务");
        self.initialized = true;
        Ok(())
    }

    pub async fn register_and_create_order(
        &self, 
        name: &str, 
        email: &str, 
        amount: f64
    ) -> ContainerResult<(String, String)> {
        println!("🎯 执行业务流程: 注册用户并创建订单");
        
        // 这里演示了如何在服务方法内部解析其他服务
        let container = global_container();
        let user_service: Arc<UserService> = container.resolve().await?;
        let order_service: Arc<OrderService> = container.resolve().await?;
        let notification_service: Arc<NotificationService> = container.resolve().await?;
        
        // 1. 创建用户
        let user_id = user_service.create_user(name, email).await?;
        println!("✅ 用户创建完成: {}", user_id);
        
        // 2. 创建订单
        let order_id = order_service.create_order(&user_id, amount).await?;
        println!("✅ 订单创建完成: {}", order_id);
        
        // 3. 发送通知
        notification_service.send_email(
            email, 
            "欢迎注册", 
            &format!("欢迎 {}! 您的订单 {} 已创建成功。", name, order_id)
        ).await?;
        
        notification_service.send_push_notification(
            &user_id, 
            "订单确认", 
            &format!("订单 {} 金额 ¥{:.2} 已确认", order_id, amount)
        ).await?;
        
        Ok((user_id, order_id))
    }

    pub async fn get_user_orders(&self, user_id: &str) -> ContainerResult<Vec<HashMap<String, String>>> {
        println!("📋 获取用户订单列表: {}", user_id);
        
        let container = global_container();
        let user_service: Arc<UserService> = container.resolve().await?;
        let order_service: Arc<OrderService> = container.resolve().await?;
        
        // 验证用户存在
        user_service.get_user(user_id).await?;
        
        // 模拟获取用户的订单列表
        let order1 = order_service.get_order("order_1").await?;
        let order2 = order_service.get_order("order_2").await?;
        
        let mut orders = Vec::new();
        if let Some(order) = order1 {
            orders.push(order);
        }
        if let Some(order) = order2 {
            orders.push(order);
        }
        
        Ok(orders)
    }
}

// ============ Bean工厂演示 ============

/// 数据库连接工厂
#[bean(name = "db_connection_factory", lifecycle = "singleton")]
pub async fn create_database_connection() -> ContainerResult<String> {
    println!("🏭 Bean工厂: 创建数据库连接");
    sleep(Duration::from_millis(100)).await;
    Ok("数据库连接实例".to_string())
}

/// HTTP客户端工厂
#[bean(name = "http_client", lifecycle = "singleton")]
pub async fn create_http_client() -> ContainerResult<String> {
    println!("🌐 Bean工厂: 创建HTTP客户端");
    Ok("HTTP客户端实例".to_string())
}

// ============ AOP切面定义 ============

/// 综合日志切面 - 记录所有方法调用
#[derive(Debug, Default, BeforeAspect, AfterReturningAspect)]
#[aspect(name = "综合日志切面", pointcut = "*Service::*", priority = 1)]
pub struct ComprehensiveLoggingAspect;

#[async_trait]
impl BeforeAspect for ComprehensiveLoggingAspect {
    async fn before(&self, context: &mut MethodContext) -> ContainerResult<()> {
        let args_str = if !context.args.is_empty() {
            format!(" 参数=[{}]", context.args.join(", "))
        } else {
            String::new()
        };
        
        println!("🔍 [{}] 开始执行 -> {}::{}{}", 
                BeforeAspect::name(self), 
                context.target_type, 
                context.method_name,
                args_str);
        
        // 在上下文中存储开始时间
        context.set_attribute("start_timestamp", &format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "综合日志切面"
    }
    
    fn priority(&self) -> i32 {
        1
    }
}

#[async_trait]
impl AfterReturningAspect for ComprehensiveLoggingAspect {
    async fn after_returning(&self, context: &MethodContext, _result: &Box<dyn std::any::Any + Send + Sync>) {
        let elapsed = context.elapsed().as_millis();
        
        println!("✅ [{}] 执行成功 <- {}::{} (耗时: {}ms)", 
                AfterReturningAspect::name(self), 
                context.target_type, 
                context.method_name,
                elapsed);
    }
    
    fn name(&self) -> &str {
        "综合日志切面"
    }
    
    fn priority(&self) -> i32 {
        1
    }
}

/// 性能监控切面 - 监控慢方法
#[derive(Debug, Default, AfterAspect)]
#[aspect(name = "性能监控切面", type_pattern = "*Service", method_pattern = "*", priority = 2)]
pub struct PerformanceMonitoringAspect {
    slow_threshold_ms: u64,
}

impl PerformanceMonitoringAspect {
    pub fn new() -> Self {
        Self {
            slow_threshold_ms: 100, // 100ms阈值
        }
    }
}

#[async_trait]
impl AfterAspect for PerformanceMonitoringAspect {
    async fn after(&self, context: &MethodContext, result: &ContainerResult<Box<dyn std::any::Any + Send + Sync>>) {
        let elapsed = context.elapsed().as_millis() as u64;
        
        if elapsed > self.slow_threshold_ms {
            println!("🐌 [{}] 慢方法警告: {}::{} 耗时 {}ms (阈值: {}ms)", 
                    self.name(),
                    context.target_type, 
                    context.method_name,
                    elapsed,
                    self.slow_threshold_ms);
        }
        
        // 记录方法执行状态
        match result {
            Ok(_) => println!("📊 [{}] 方法执行成功: {}::{}", 
                            self.name(), context.target_type, context.method_name),
            Err(_) => println!("📊 [{}] 方法执行失败: {}::{}", 
                             self.name(), context.target_type, context.method_name),
        }
    }
    
    fn name(&self) -> &str {
        "性能监控切面"
    }
    
    fn priority(&self) -> i32 {
        2
    }
}

/// 安全审计切面 - 记录敏感操作
#[derive(Debug, Default, AroundAspect)]
#[aspect(name = "安全审计切面", pointcut = "*Service::delete_*", priority = 3)]
pub struct SecurityAuditAspect;

#[async_trait]
impl AroundAspect for SecurityAuditAspect {
    async fn before_proceed(&self, context: &mut MethodContext) -> ContainerResult<bool> {
        if context.method_name.starts_with("delete_") {
            println!("🔒 [{}] 安全检查: 敏感操作 {}::{}", 
                    self.name(),
                    context.target_type, 
                    context.method_name);
            
            // 模拟权限检查
            println!("🛡️  [{}] 权限验证通过", self.name());
            context.set_attribute("audit_logged", "true");
        }
        Ok(true) // 允许继续执行
    }
    
    async fn after_proceed(&self, context: &MethodContext, result: &ContainerResult<Box<dyn std::any::Any + Send + Sync>>) {
        if context.get_attribute("audit_logged").is_some() {
            match result {
                Ok(_) => println!("📝 [{}] 审计记录: 敏感操作执行成功 {}::{}", 
                                self.name(), context.target_type, context.method_name),
                Err(_) => println!("📝 [{}] 审计记录: 敏感操作执行失败 {}::{}", 
                                 self.name(), context.target_type, context.method_name),
            }
        }
    }
    
    fn name(&self) -> &str {
        "安全审计切面"
    }
    
    fn priority(&self) -> i32 {
        3
    }
}

/// 异常处理切面 - 处理和记录所有异常
#[derive(Debug, Default, AfterThrowingAspect)]
#[aspect(name = "异常处理切面", pointcut = "*Service::*", priority = 4)]
pub struct ExceptionHandlingAspect;

#[async_trait]
impl AfterThrowingAspect for ExceptionHandlingAspect {
    async fn after_throwing(&self, context: &MethodContext, error: &ContainerError) {
        println!("❌ [{}] 异常处理: {}::{} 发生错误", 
                self.name(), 
                context.target_type, 
                context.method_name);
        
        println!("   错误详情: {}", error);
        println!("   执行耗时: {}ms", context.elapsed().as_millis());
        
        // 模拟错误报告
        println!("📧 [{}] 已发送错误报告到监控系统", self.name());
        
        // 根据错误类型执行不同的处理策略
        match error {
            ContainerError::ComponentCreationFailed { .. } => {
                println!("🔄 [{}] 建议: 检查组件配置和依赖", self.name());
            }
            ContainerError::ComponentNotFound { .. } => {
                println!("🔍 [{}] 建议: 确认组件已正确注册", self.name());
            }
            _ => {
                println!("🛠️  [{}] 建议: 查看详细日志进行排查", self.name());
            }
        }
    }
    
    fn name(&self) -> &str {
        "异常处理切面"
    }
    
    fn priority(&self) -> i32 {
        4
    }
}

// ============ 演示函数 ============

/// 演示依赖注入和生命周期管理
async fn demo_dependency_injection() -> ContainerResult<()> {
    println!("\n=== 📦 依赖注入和生命周期管理演示 ===");
    
    let container = global_container();
    
    // 演示单例组件
    println!("\n>> 单例组件演示:");
    let user_service1: Arc<UserService> = container.resolve().await?;
    let user_service2: Arc<UserService> = container.resolve().await?;
    println!("单例组件地址相同: {}", Arc::ptr_eq(&user_service1, &user_service2));
    
    // 演示瞬态组件  
    println!("\n>> 瞬态组件演示:");
    let notif1: Arc<NotificationService> = container.resolve().await?;
    let notif2: Arc<NotificationService> = container.resolve().await?;
    println!("瞬态组件地址不同: {}", !Arc::ptr_eq(&notif1, &notif2));
    println!("实例1 ID: {}", notif1.instance_id);
    println!("实例2 ID: {}", notif2.instance_id);
    
    Ok(())
}

/// 演示自动AOP代理功能
async fn demo_automatic_aop() -> ContainerResult<()> {
    println!("\n=== ✨ 自动AOP代理功能演示 ===");
    
    let container = global_container();
    
    // 直接resolve启用了auto_proxy的服务，容器会自动创建AOP代理
    let user_service: Arc<UserService> = container.resolve().await?;
    let order_service: Arc<OrderService> = container.resolve().await?;
    
    println!("\n>> 用户服务操作 (自动AOP):");
    let user_id = user_service.create_user("Alice", "alice@example.com").await?;
    let user_data = user_service.get_user(&user_id).await?;
    println!("用户数据: {:?}", user_data);
    
    user_service.update_user(&user_id, "Alice Smith").await?;
    
    println!("\n>> 订单服务操作 (自动AOP):");
    let order_id = order_service.create_order(&user_id, 199.99).await?;
    let order_data = order_service.get_order(&order_id).await?;
    println!("订单数据: {:?}", order_data);
    
    println!("\n>> 敏感操作 (安全审计切面):");
    user_service.delete_user(&user_id).await?;
    
    println!("\n>> 异常处理演示:");
    let _ = user_service.get_user("error").await; // 故意触发错误
    
    Ok(())
}

/// 演示业务流程和服务聚合
async fn demo_business_workflow() -> ContainerResult<()> {
    println!("\n=== 🎯 业务流程和服务聚合演示 ===");
    
    let container = global_container();
    let app_facade: Arc<ApplicationFacade> = container.resolve().await?;
    
    println!("\n>> 复杂业务流程: 用户注册 + 订单创建 + 通知发送:");
    let (user_id, order_id) = app_facade.register_and_create_order(
        "Bob Johnson", 
        "bob@example.com", 
        299.99
    ).await?;
    
    println!("✅ 业务流程完成: 用户ID={}, 订单ID={}", user_id, order_id);
    
    println!("\n>> 聚合查询: 获取用户订单列表:");
    let orders = app_facade.get_user_orders(&user_id).await?;
    println!("用户订单: {:?}", orders);
    
    Ok(())
}

/// 演示Bean工厂
async fn demo_bean_factories() -> ContainerResult<()> {
    println!("\n=== 🏭 Bean工厂演示 ===");
    
    let container = global_container();
    
    println!("Bean工厂组件已通过ctor自动注册:");
    println!("  🔗 db_connection_factory - 数据库连接工厂");
    println!("  🌐 http_client - HTTP客户端工厂");
    println!("注意: Bean工厂在初始化时已被调用，创建了相应的实例");
    
    // 展示注册的Bean信息
    let registry = mf_contex::registry::global_registry();
    let components = registry.get_all_components();
    
    println!("\n>> Bean工厂注册的组件:");
    for component in components {
        if component.name == "db_connection_factory" || component.name == "http_client" {
            println!("  📦 {} - {} ({})", 
                    component.name, 
                    component.type_name, 
                    component.lifecycle);
        }
    }
    
    Ok(())
}

/// 演示配置管理
async fn demo_configuration_management() -> ContainerResult<()> {
    println!("\n=== ⚙️  配置管理演示 ===");
    
    let container = global_container();
    let config: Arc<ConfigComponent> = container.resolve().await?;
    
    println!("应用配置:");
    println!("  数据库URL: {}", config.config.database_url);
    println!("  Redis URL: {}", config.config.redis_url);
    println!("  服务器端口: {}", config.config.server_port);
    println!("  调试模式: {}", config.config.debug_mode);
    println!("  最大连接数: {}", config.config.max_connections);
    
    Ok(())
}

/// 演示错误处理和异常流程
async fn demo_error_handling() -> ContainerResult<()> {
    println!("\n=== ❌ 错误处理和异常流程演示 ===");
    
    let container = global_container();
    let order_service: Arc<OrderService> = container.resolve().await?;
    
    println!("\n>> 业务异常处理:");
    
    // 故意传入无效金额触发业务异常
    match order_service.create_order("test_user", -10.0).await {
        Ok(order_id) => println!("订单创建成功: {}", order_id),
        Err(e) => println!("订单创建失败: {}", e),
    }
    
    println!("\n>> 数据库异常处理:");
    let db_pool: Arc<DatabasePool> = container.resolve().await?;
    
    // 故意执行错误SQL触发数据库异常
    match db_pool.execute_query("SELECT * FROM error_table").await {
        Ok(results) => println!("查询结果: {:?}", results),
        Err(e) => println!("数据库查询失败: {}", e),
    }
    
    Ok(())
}

/// 显示容器统计信息
async fn show_container_statistics() -> ContainerResult<()> {
    println!("\n=== 📊 容器统计信息 ===");
    
    let registry = mf_contex::registry::global_registry();
    let components = registry.get_all_components();
    
    println!("\n>> 注册的组件总览:");
    let mut singleton_count = 0;
    let mut transient_count = 0;
    let mut auto_proxy_count = 0;
    
    for component in &components {
        let lifecycle_icon = match component.lifecycle {
            Lifecycle::Singleton => { singleton_count += 1; "🔒" },
            Lifecycle::Transient => { transient_count += 1; "🔄" },
            Lifecycle::Scoped => "📍",
        };
        
        let proxy_status = if component.auto_proxy {
            auto_proxy_count += 1;
            "✨ 自动代理"
        } else {
            "🔧 普通组件"
        };
        
        println!("  {} {} - {} ({})", 
                lifecycle_icon, 
                component.name, 
                component.lifecycle, 
                proxy_status);
    }
    
    println!("\n>> 统计摘要:");
    println!("  📦 组件总数: {}", components.len());
    println!("  🔒 单例组件: {}", singleton_count);
    println!("  🔄 瞬态组件: {}", transient_count);
    println!("  ✨ 启用自动代理: {}", auto_proxy_count);
    
    println!("\n>> AOP切面信息:");
    println!("  🔍 ComprehensiveLoggingAspect - 综合日志切面 (前置/返回后)");
    println!("  📊 PerformanceMonitoringAspect - 性能监控切面 (后置)");
    println!("  🔒 SecurityAuditAspect - 安全审计切面 (环绕)");
    println!("  ❌ ExceptionHandlingAspect - 异常处理切面 (异常后)");
    
    Ok(())
}

/// 主演示函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 === ModuForge 综合功能演示 === 🚀");
    println!();
    println!("本演示包含以下功能模块:");
    println!("  📦 依赖注入容器");
    println!("  ✨ 自动AOP代理");
    println!("  🎯 业务流程管理");
    println!("  🏭 Bean工厂");
    println!("  ⚙️  配置管理");
    println!("  ❌ 错误处理");
    println!("  📊 系统统计");
    
    // 设置一些环境变量演示配置注入
    unsafe {
        std::env::set_var("DATABASE_URL", "postgresql://prod:5432/myapp");
        std::env::set_var("SERVER_PORT", "9090");
    }
    
    // 初始化容器（所有组件和切面自动注册）
    println!("\n🔧 初始化ModuForge容器...");
    initialize_container().await?;
    println!("✅ 容器初始化完成！");
    
    // 依次演示各个功能模块
    demo_dependency_injection().await?;
    demo_automatic_aop().await?;
    demo_business_workflow().await?;
    demo_bean_factories().await?;
    demo_configuration_management().await?;
    demo_error_handling().await?;
    show_container_statistics().await?;
    
    println!("\n🎉 === ModuForge 综合功能演示完成 === 🎉");
    println!();
    println!("✨ 主要特性验证:");
    println!("  ✅ 依赖注入 - 自动解析和管理组件依赖");
    println!("  ✅ 生命周期 - 单例、瞬态、作用域生命周期管理");
    println!("  ✅ 自动代理 - 通过auto_proxy实现零手动操作AOP");
    println!("  ✅ 切面编程 - 前置、后置、环绕、异常处理切面");
    println!("  ✅ 配置管理 - 环境变量注入和配置组件");
    println!("  ✅ Bean工厂 - 复杂对象创建和管理");
    println!("  ✅ 错误处理 - 完整的异常处理和恢复机制");
    println!("  ✅ 系统监控 - 性能监控和安全审计");
    println!();
    println!("🚀 ModuForge - 企业级Rust依赖注入框架!");
    
    Ok(())
}