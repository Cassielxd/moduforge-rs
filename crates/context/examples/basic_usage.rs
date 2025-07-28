use mf_context::*;
use std::{fmt::Debug, sync::Arc};
use async_trait::async_trait;

// 定义一个简单的数据库服务
#[derive(Debug, Default)]
pub struct DatabaseService {
    connection_url: String,
}

impl DatabaseService {
    pub async fn save(&self, data: &str) -> ContainerResult<String> {
        println!("保存到数据库: {}", data);
        Ok(format!("saved_{}", data))
    }
    
    pub async fn find(&self, id: &str) -> ContainerResult<Option<String>> {
        println!("在数据库中查找: {}", id);
        Ok(Some(format!("found_{}", id)))
    }
}

#[async_trait]
impl Component for DatabaseService {
    fn component_name() -> &'static str {
        "database_service"
    }
    
    fn lifecycle() -> Lifecycle {
        Lifecycle::Singleton
    }
}

// 定义用户服务
#[derive(Debug)]
pub struct UserService {
    db: Arc<DatabaseService>,
}

impl UserService {
    pub fn new(db: Arc<DatabaseService>) -> Self {
        Self { db }
    }
    
    pub async fn create_user(&self, name: &str) -> ContainerResult<String> {
        let user_data = format!("User: {}", name);
        self.db.save(&user_data).await
    }
    
    pub async fn get_user(&self, id: &str) -> ContainerResult<Option<String>> {
        self.db.find(id).await
    }
}

#[async_trait]
impl Component for UserService {
    fn component_name() -> &'static str {
        "user_service"
    }
    
    fn lifecycle() -> Lifecycle {
        Lifecycle::Singleton
    }
}

// 定义应用配置
#[derive(Debug, Default)]
pub struct AppConfig {
    pub port: u16,
    pub host: String,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            port: 8080,
            host: "localhost".to_string(),
        }
    }
}

#[async_trait]
impl Component for AppConfig {
    fn component_name() -> &'static str {
        "app_config"
    }
    
    fn lifecycle() -> Lifecycle {
        Lifecycle::Singleton
    }
}

// 手动注册组件
fn register_components() -> ContainerResult<()> {
    use mf_context::registry::global_registry;
    use mf_context::component::ComponentFactory;
    
    // 注册数据库服务
    let db_factory: ComponentFactory = Box::new(|_resolver| {
        Box::pin(async move {
            let service = DatabaseService::default();
            service.initialize().await?;
            Ok(Arc::new(service) as Arc<dyn std::any::Any + Send + Sync>)
        })
    });
    global_registry().register_component::<DatabaseService>(db_factory)?;
    
    // 注册用户服务
    let user_factory: ComponentFactory = Box::new(|resolver| {
        Box::pin(async move {
            let db: Arc<DatabaseService> = resolver.resolve().await?;
            let service = UserService::new(db);
            service.initialize().await?;
            Ok(Arc::new(service) as Arc<dyn std::any::Any + Send + Sync>)
        })
    });
    global_registry().register_component::<UserService>(user_factory)?;
    
    // 注册应用配置
    let config_factory: ComponentFactory = Box::new(|_resolver| {
        Box::pin(async move {
            let config = AppConfig::new();
            config.initialize().await?;
            Ok(Arc::new(config) as Arc<dyn std::any::Any + Send + Sync>)
        })
    });
    global_registry().register_component::<AppConfig>(config_factory)?;
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ModuForge 依赖注入容器演示 ===");
    
    // 注册组件
    register_components()?;
    
    // 初始化容器
    initialize_container().await?;
    
    let container = global_container();
    
    // 解析用户服务
    let user_service: Arc<UserService> = container.resolve().await?;
    
    // 测试创建用户
    let user_id = user_service.create_user("Alice").await?;
    println!("已创建用户，ID: {}", user_id);
    
    // 测试查找用户
    if let Some(user_data) = user_service.get_user(&user_id).await? {
        println!("找到用户: {}", user_data);
    }
    
    // 解析应用配置
    let config: Arc<AppConfig> = container.resolve().await?;
    println!("应用运行在 {}:{}", config.host, config.port);
    
    // 测试作用域功能
    println!("为测试创建作用域...");
    let scope = container.create_scope("request_scope").await;
    println!("在请求作用域中运行...");
    let user_service: Arc<UserService> = container.resolve().await?;
    user_service.create_user("Bob").await?;
    container.exit_scope(&scope).await?;
    println!("作用域已关闭。");
    
    println!("应用成功完成!");
    Ok(())
}