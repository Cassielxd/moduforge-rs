use mf_contex::*;
use std::{fmt::Debug, sync::Arc, collections::HashMap};

// 使用Component derive宏定义一个简单组件
#[derive(Debug, Default, Component)]
#[component(name = "logger_service", lifecycle = "singleton")]
pub struct LoggerService {
    prefix: String,
}

impl LoggerService {
    pub fn log(&self, message: &str) {
        println!("[{}] {}", self.prefix.as_str(), message);
    }
    
    pub fn set_prefix(&mut self, prefix: String) {
        self.prefix = prefix;
    }
}

// 使用service宏定义一个可变组件
#[service(name = "counter_service", lifecycle = "singleton", concurrent_read = true)]
#[derive(Debug, Default)]
pub struct CounterService {
    count: i32,
    data: HashMap<String, i32>,
}

impl CounterService {
    pub fn increment(&mut self) -> i32 {
        self.count += 1;
        self.count
    }
    
    pub fn get_count(&self) -> i32 {
        self.count
    }
    
    pub fn set_data(&mut self, key: String, value: i32) {
        self.data.insert(key, value);
    }
    
    pub fn get_data(&self, key: &str) -> Option<i32> {
        self.data.get(key).copied()
    }
}

// 使用Injectable derive宏定义一个有依赖的组件
#[derive(Debug, Injectable, Component)]
#[component(name = "application_service", lifecycle = "singleton")]
pub struct ApplicationService {
    #[inject]
    logger: Arc<LoggerService>,
    app_name: String,
}

impl Default for ApplicationService {
    fn default() -> Self {
        Self {
            logger: Arc::new(LoggerService::default()),
            app_name: "Default App".to_string(),
        }
    }
}

impl ApplicationService {
    pub async fn start(&self) -> ContainerResult<()> {
        self.logger.log(&format!("Starting application: {}", self.app_name));
        Ok(())
    }
    
    pub async fn stop(&self) -> ContainerResult<()> {
        self.logger.log(&format!("Stopping application: {}", self.app_name));
        Ok(())
    }
}

// 使用bean宏定义一个工厂方法
#[bean(name = "config", lifecycle = "singleton")]
pub async fn create_config() -> AppConfig {
    AppConfig {
        host: "0.0.0.0".to_string(),
        port: 3000,
        debug: true,
    }
}

#[derive(Debug, Clone, Component, Default)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub debug: bool,
}

// ApplicationService现在通过宏自动注册，不需要手动注册了

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ModuForge 依赖注入容器宏演示 ===");
    
    // 初始化容器（自动注册的组件会被处理）
    initialize_container().await?;
    
    let container = global_container();
    
    // 测试普通组件
    println!("\n--- 测试日志服务 ---");
    let logger: Arc<LoggerService> = container.resolve().await?;
    logger.log("你好，来自依赖注入!");
    
    // 测试可变组件（注意：这里展示的是可变组件的概念，实际使用中需要特殊的注册方式）
    println!("\n--- 测试可变计数器服务概念 ---");
    println!("可变组件需要特殊注册以正确处理 Arc。");
    println!("在此演示中，我们将展示正常的不可变组件访问:");
    
    let counter_service: Arc<CounterService> = container.resolve().await?;
    println!("计数器服务解析成功（不可变访问）");
    println!("当前计数: {}", counter_service.get_count());
    
    // 在实际应用中，可变组件需要特殊的工厂方法来创建包装器
    
    // 测试有依赖的组件
    println!("\n--- 测试带依赖的应用服务 ---");
    let app_service: Arc<ApplicationService> = container.resolve().await?;
    app_service.start().await?;
    
    // 测试Bean工厂创建的组件
    println!("\n--- 测试 Bean 工厂 ---");
    let config: Arc<AppConfig> = container.resolve().await?;
    println!("应用配置: 调试模式={}, 服务器={}:{}", 
             config.debug, config.host, config.port);
    
    // 异步可变组件在这个示例中跳过，因为需要特殊的注册方式
    println!("\n--- 异步可变组件 ---");
    println!("异步可变组件也需要特殊的工厂注册。");
    
    // 测试作用域
    println!("\n--- 测试作用域组件 ---");
    let scope = container.create_scope("demo_scope").await;
    println!("已创建作用域: {}", scope.name);
    
    // 在作用域内解析组件（单例组件会重用）
    let scoped_logger: Arc<LoggerService> = container.resolve().await?;
    scoped_logger.log("来自作用域上下文的消息");
    
    container.exit_scope(&scope).await?;
    println!("作用域已退出");
    
    app_service.stop().await?;
    println!("\n=== 演示成功完成! ===");
    
    Ok(())
}