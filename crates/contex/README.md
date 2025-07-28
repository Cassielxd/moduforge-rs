# ModuForge Enterprise IoC Container

一个功能强大、高性能的企业级依赖注入容器，提供Spring Framework风格的IoC功能，同时保持Rust的性能和安全性优势。

## 🚀 核心特性

### 基础功能
- ✅ **依赖注入容器**: 完整的生命周期管理（单例、瞬态、作用域）
- ✅ **组件自动注册**: 基于trait的配置和自动扫描
- ✅ **类型安全解析**: 编译时类型安全，支持异步
- ✅ **循环依赖检测**: 防止无限依赖循环

### 企业级功能
- ✅ **Profile条件注册**: 环境驱动的组件选择
- ✅ **AOP切面编程**: 日志、性能监控、事务等横切关注点
- ✅ **配置管理**: 多源配置、热更新、类型安全
- ✅ **可变组件支持**: 多种锁策略的线程安全访问

## 📦 快速开始

### 添加依赖

```toml
[dependencies]
mf-contex = { path = "../contex" }
tokio = { version = "1.0", features = ["full"] }
```

### 基础使用

```rust
use mf_contex::*;
use std::sync::Arc;

// 1. 定义服务组件
#[derive(Debug, Default)]
pub struct UserService {
    name: String,
}

impl Component for UserService {
    fn component_name() -> &'static str {
        "user_service"
    }
    
    fn lifecycle() -> Lifecycle {
        Lifecycle::Singleton
    }
}

impl UserService {
    pub async fn get_user(&self, id: &str) -> String {
        format!("User: {}", id)
    }
}

// 2. 注册和使用
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 注册组件
    auto_register_component::<UserService>();
    
    // 初始化容器
    initialize_container().await?;
    
    // 获取容器并解析服务
    let container = global_container();
    let user_service: Arc<UserService> = container.resolve().await?;
    
    // 使用服务
    let user = user_service.get_user("123").await;
    println!("{}", user);
    
    Ok(())
}
```

## 🎯 核心概念

### 组件生命周期

```rust
pub enum Lifecycle {
    Singleton,  // 单例模式 - 全局共享一个实例
    Transient,  // 瞬态模式 - 每次请求创建新实例
    Scoped,     // 作用域模式 - 在特定作用域内共享
}
```

### 组件注册方式

#### 1. 自动注册（推荐）
```rust
#[derive(Debug, Default)]
pub struct MyService;

impl Component for MyService {
    fn component_name() -> &'static str { "my_service" }
    fn lifecycle() -> Lifecycle { Lifecycle::Singleton }
}

// 注册
auto_register_component::<MyService>();
```

#### 2. 手动注册
```rust
use mf_contex::registry::global_registry;
use mf_contex::component::ComponentFactory;

let factory: ComponentFactory = Box::new(|_resolver| {
    Box::pin(async move {
        let instance = MyService::new().await;
        Ok(Arc::new(instance) as Arc<dyn std::any::Any + Send + Sync>)
    })
});

global_registry().register_component::<MyService>(factory)?;
```

## 🌍 Profile条件注册

基于环境或条件动态选择组件实现：

```rust
// 生产环境数据库
#[derive(Debug, Default)]
pub struct PostgreSQLService;

impl Component for PostgreSQLService {
    fn component_name() -> &'static str { "database_service" }
    fn lifecycle() -> Lifecycle { Lifecycle::Singleton }
}

// 开发环境数据库
#[derive(Debug, Default)]
pub struct InMemoryDatabase;

impl Component for InMemoryDatabase {
    fn component_name() -> &'static str { "database_service" }
    fn lifecycle() -> Lifecycle { Lifecycle::Singleton }
}

// 使用Profile激活
activate_profile("production");  // 或 "development"

// 环境变量支持
// export MODUFORGE_PROFILES=production,staging
load_profiles_from_env();
```

### Profile条件表达式

```rust
// 任一Profile匹配
let condition = ProfileCondition::any_of(&["production", "staging"]);

// 所有Profile匹配
let condition = ProfileCondition::all_of(&["production", "cache_enabled"]);

// 环境变量条件
let env_condition = EnvironmentCondition::exists("DATABASE_URL");
let value_condition = EnvironmentCondition::equals("ENV", "production");

// 复合条件
let composite = CompositeCondition::and(vec![
    Box::new(ProfileCondition::of("production")),
    Box::new(EnvironmentCondition::exists("DATABASE_URL")),
]);
```

## 🔄 AOP切面编程

横切关注点的优雅解决方案：

### 内置切面

```rust
// 日志切面
let logging_aspect = LoggingAspect::new("ServiceLogger")
    .with_args(true)
    .with_result(true)
    .with_time(true);

add_before_aspect(
    Pointcut::new("*Service", "*"),
    Box::new(logging_aspect),
);

// 性能监控切面
let perf_aspect = PerformanceAspect::new("PerfMonitor", 100); // 100ms阈值
add_after_aspect(
    Pointcut::new("*Service", "slow_*"),
    Box::new(perf_aspect),
);
```

### 自定义切面

```rust
#[derive(Debug)]
pub struct TransactionAspect;

#[async_trait]
impl BeforeAspect for TransactionAspect {
    async fn before(&self, context: &mut MethodContext) -> ContainerResult<()> {
        println!("开始事务: {}::{}", context.target_type, context.method_name);
        // 事务开始逻辑
        Ok(())
    }
    
    fn name(&self) -> &str { "TransactionAspect" }
    fn priority(&self) -> i32 { 100 } // 高优先级
}

#[async_trait]
impl AfterReturningAspect for TransactionAspect {
    async fn after_returning(&self, context: &MethodContext, _result: &Box<dyn Any + Send + Sync>) {
        println!("提交事务: {}::{}", context.target_type, context.method_name);
        // 事务提交逻辑
    }
    
    fn name(&self) -> &str { "TransactionAspect" }
}

// 注册切面
add_before_aspect(
    Pointcut::new("*Repository", "save*"),
    Box::new(TransactionAspect),
);
```

### 切点表达式

```rust
// 匹配所有Service类的所有方法
Pointcut::new("*Service", "*")

// 匹配UserService的get开头的方法
Pointcut::new("UserService", "get*")

// 匹配所有Repository的save和update方法
Pointcut::new("*Repository", "save*")
Pointcut::new("*Repository", "update*")
```

## ⚙️ 配置管理

类型安全的配置管理系统：

### 配置结构定义

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub debug: bool,
    pub log_level: String,
}
```

### 配置源管理

```rust
// 添加JSON配置文件
add_config_source(ConfigSource::JsonFile("config.json".to_string()));

// 添加环境变量源（默认已添加）
add_config_source(ConfigSource::Environment);

// 内存配置
let mut memory_config = HashMap::new();
memory_config.insert("debug".to_string(), "true".to_string());
add_config_source(ConfigSource::Memory(memory_config));
```

### 配置使用

```rust
// 获取配置对象
let config_manager = get_config_manager();
let mut manager = config_manager.write().unwrap();
let app_config: AppConfig = manager.get_config()?;

println!("Database: {}:{}", app_config.database.host, app_config.database.port);

// 获取单个配置值
let debug_mode = get_config_bool("debug").unwrap_or(false);
let log_level = get_config_string("log_level").unwrap_or_else(|| "info".to_string());
```

### 环境变量注入宏

```rust
// 获取环境变量，有默认值
let rust_env = inject_env!("RUST_ENV", "development");

// 获取环境变量，返回Option
let database_url = inject_env!("DATABASE_URL");

// 获取配置值
let debug = inject_config!("debug", "false");
```

## 🔒 可变组件支持

线程安全的可变组件访问：

### 包装器使用

```rust
// Mutex包装器
let mutex_service: Arc<MutexWrapper<CounterService>> = container.resolve_mutex().await?;
mutex_service.with_lock(|counter| {
    counter.count += 1;
    println!("Count: {}", counter.count);
}).await;

// RwLock包装器
let rwlock_service: Arc<RwLockWrapper<CounterService>> = container.resolve_rwlock().await?;

// 读取
rwlock_service.with_read_lock(|counter| {
    println!("Current count: {}", counter.count);
}).await;

// 写入
rwlock_service.with_write_lock(|counter| {
    counter.count += 1;
}).await;

// 异步锁
let async_mutex: Arc<AsyncMutexWrapper<CounterService>> = container.resolve().await?;
async_mutex.with_lock(|counter| {
    counter.count += 1;
}).await;
```

## 🏗️ 企业级架构示例

### 分层架构

```rust
// 1. 数据访问层
#[derive(Debug, Default)]
pub struct UserRepository {
    db: Arc<dyn DatabaseService>,
}

impl Component for UserRepository {
    fn component_name() -> &'static str { "user_repository" }
    fn lifecycle() -> Lifecycle { Lifecycle::Singleton }
}

// 2. 业务逻辑层
#[derive(Debug)]
pub struct UserService {
    repository: Arc<UserRepository>,
    cache: Arc<CacheService>,
}

impl Component for UserService {
    fn component_name() -> &'static str { "user_service" }
    fn lifecycle() -> Lifecycle { Lifecycle::Singleton }
}

// 3. 控制器层
#[derive(Debug)]
pub struct UserController {
    service: Arc<UserService>,
}

impl Component for UserController {
    fn component_name() -> &'static str { "user_controller" }
    fn lifecycle() -> Lifecycle { Lifecycle::Singleton }
}
```

### 完整应用启动

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 启动企业级应用 ===");
    
    // 1. 配置初始化
    setup_configuration();
    
    // 2. Profile加载
    load_profiles_from_env();
    activate_profile("production");
    
    // 3. AOP配置
    setup_aspects();
    
    // 4. 组件注册
    register_components();
    
    // 5. 容器初始化
    initialize_container().await?;
    
    // 6. 应用启动
    start_application().await?;
    
    Ok(())
}

fn setup_configuration() {
    add_config_source(ConfigSource::JsonFile("config.json".to_string()));
    set_config("database.host", ConfigValue::String("localhost".to_string()));
    set_config("database.port", ConfigValue::Integer(5432));
}

fn setup_aspects() {
    // 日志切面
    add_before_aspect(
        Pointcut::new("*Service", "*"),
        Box::new(LoggingAspect::new("AppLogger")),
    );
    
    // 性能监控
    add_after_aspect(
        Pointcut::new("*Repository", "*"),
        Box::new(PerformanceAspect::new("DBMonitor", 50)),
    );
}

fn register_components() {
    auto_register_component::<DatabaseService>();
    auto_register_component::<CacheService>();
    auto_register_component::<UserRepository>();
    auto_register_component::<UserService>();
    auto_register_component::<UserController>();
}
```

## 📊 性能特性

### 内存效率
- **零拷贝解析**: 使用Arc共享实例，避免不必要的克隆
- **延迟初始化**: 组件按需创建，减少启动时间
- **缓存优化**: 配置和组件信息缓存，减少重复计算

### 并发性能
- **无锁数据结构**: 使用DashMap等高性能并发容器
- **异步支持**: 全面的async/await支持，避免线程阻塞
- **细粒度锁**: 最小化锁竞争，提高并发吞吐量

## 🚨 最佳实践

### 1. 组件设计
```rust
// ✅ 好的设计 - 单一职责
#[derive(Debug, Default)]
pub struct EmailService {
    smtp_config: SmtpConfig,
}

// ❌ 避免 - 职责过多
#[derive(Debug, Default)]  
pub struct MegaService {
    email: EmailSender,
    sms: SmsSender,
    push: PushSender,
    db: Database,
    cache: Cache,
}
```

### 2. 依赖管理
```rust
// ✅ 好的设计 - 依赖接口而非实现
pub struct UserService {
    repository: Arc<dyn UserRepository>,
    notifier: Arc<dyn NotificationService>,
}

// ❌ 避免 - 直接依赖具体实现
pub struct UserService {
    repository: Arc<MySQLUserRepository>,
    notifier: Arc<EmailNotificationService>,
}
```

### 3. 生命周期选择
```rust
// Singleton - 无状态服务、配置、连接池
impl Component for DatabasePool {
    fn lifecycle() -> Lifecycle { Lifecycle::Singleton }
}

// Transient - 有状态、轻量级对象
impl Component for RequestHandler {
    fn lifecycle() -> Lifecycle { Lifecycle::Transient }
}

// Scoped - 请求级别的状态
impl Component for UserSession {
    fn lifecycle() -> Lifecycle { Lifecycle::Scoped }
}
```

## 📚 API参考

### 核心Traits

#### Component
```rust
pub trait Component: Send + Sync + Debug + Any {
    fn component_name() -> &'static str;
    fn lifecycle() -> Lifecycle;
    async fn initialize(&self) -> ContainerResult<()> { Ok(()) }
    async fn destroy(&self) -> ContainerResult<()> { Ok(()) }
}
```

#### MutableComponent
```rust
pub trait MutableComponent: Component {
    async fn with_mutable<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R + Send,
        R: Send;
}
```

### 容器接口

#### Container
```rust
impl Container {
    pub async fn resolve<T: Component + 'static>(&self) -> ContainerResult<Arc<T>>;
    pub async fn resolve_mutex<T: Component + Default + 'static>(&self) -> ContainerResult<Arc<MutexWrapper<T>>>;
    pub async fn resolve_rwlock<T: Component + Default + 'static>(&self) -> ContainerResult<Arc<RwLockWrapper<T>>>;
    pub async fn with_scope<F, Fut, R>(&self, scope_name: &str, f: F) -> ContainerResult<R>;
}
```

### 配置接口

#### ConfigManager
```rust
impl ConfigManager {
    pub fn get_config<T: DeserializeOwned + Clone + Send + Sync + 'static>(&mut self) -> ContainerResult<T>;
    pub fn get_string(&self, key: &str) -> Option<String>;
    pub fn get_i64(&self, key: &str) -> Option<i64>;
    pub fn get_bool(&self, key: &str) -> Option<bool>;
    pub fn set(&mut self, key: &str, value: ConfigValue);
    pub fn add_source(&mut self, source: ConfigSource);
}
```

## 🔧 宏支持

关于宏的使用，请参考 `../macro/MACRO_USAGE.md` 文件了解详情。

目前支持的过程宏：
- `#[derive(Component)]` - 自动实现Component trait
- `#[derive(Injectable)]` - 标记依赖注入字段
- `#[service]` - 标记服务组件
- `#[bean]` - 标记Bean方法

声明式宏需要单独配置使用。

## 🤝 贡献指南

欢迎贡献代码！请遵循以下步骤：

1. Fork 项目
2. 创建特性分支: `git checkout -b feature/amazing-feature`
3. 提交更改: `git commit -m 'Add amazing feature'`
4. 推送分支: `git push origin feature/amazing-feature`
5. 提交Pull Request

### 开发环境设置

```bash
# 克隆项目
git clone https://github.com/your-org/moduforge-rs
cd moduforge-rs

# 运行测试
cargo test

# 运行示例
cargo run --example enterprise_app

# 格式化代码
cargo fmt

# 静态检查
cargo clippy
```

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

感谢以下项目的灵感和参考：
- [Spring Framework](https://spring.io/) - Java生态的IoC容器标杆
- [tokio](https://tokio.rs/) - Rust异步运行时
- [serde](https://serde.rs/) - Rust序列化框架

---

**ModuForge Enterprise IoC Container** - 让Rust企业级开发更简单 🚀

ModuForge 依赖注入容器是一个高性能、类型安全的 Rust 依赖注入框架，支持异步组件、生命周期管理和循环依赖检测。

## 特性

- 🚀 **异步支持**: 完全异步的组件创建和依赖解析
- 🔒 **类型安全**: 编译时类型检查，运行时零开销
- 🔄 **生命周期管理**: 支持单例、瞬态和作用域生命周期
- 🔍 **循环依赖检测**: 启动时自动检测并报告循环依赖
- 🎯 **零配置**: 支持自动组件扫描和注册
- 📦 **轻量级**: 最小化的运行时开销

## 快速开始

### 1. 添加依赖

```toml
[dependencies]
mf_contex = "0.4.12"
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
```

### 2. 定义组件

```rust
use mf_contex::*;
use async_trait::async_trait;
use std::sync::Arc;

// 定义一个服务组件
#[derive(Debug, Default)]
pub struct DatabaseService {
    connection_url: String,
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

impl DatabaseService {
    pub async fn save(&self, data: &str) -> ContainerResult<String> {
        println!("Saving to database: {}", data);
        Ok(format!("saved_{}", data))
    }
}
```

### 3. 注册和使用组件

```rust
use mf_contex::{*, component::ComponentFactory};

fn register_components() -> ContainerResult<()> {
    let factory: ComponentFactory = Box::new(|_resolver| {
        Box::pin(async move {
            let service = DatabaseService::default();
            service.initialize().await?;
            Ok(Arc::new(service) as Arc<dyn std::any::Any + Send + Sync>)
        })
    });
    
    mf_contex::registry::global_registry()
        .register_component::<DatabaseService>(factory)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 注册组件
    register_components()?;
    
    // 初始化容器
    initialize_container().await?;
    
    // 获取容器并解析组件
    let container = global_container();
    let db_service: Arc<DatabaseService> = container.resolve().await?;
    
    // 使用组件
    let result = db_service.save("test data").await?;
    println!("Result: {}", result);
    
    Ok(())
}
```

## 生命周期类型

### Singleton (单例)
```rust
impl Component for MyService {
    fn lifecycle() -> Lifecycle {
        Lifecycle::Singleton
    }
}
```
全局唯一实例，在首次请求时创建，后续请求返回相同实例。

### Transient (瞬态)
```rust
impl Component for MyService {
    fn lifecycle() -> Lifecycle {
        Lifecycle::Transient
    }
}
```
每次请求都创建新实例。

### Scoped (作用域)
```rust
impl Component for MyService {
    fn lifecycle() -> Lifecycle {
        Lifecycle::Scoped
    }
}
```
在特定作用域内单例，作用域结束时自动清理。

## 作用域管理

```rust
// 创建作用域
let scope = container.create_scope("request_scope").await;

// 在作用域内解析组件
let service: Arc<MyService> = container.resolve().await?;

// 手动退出作用域
container.exit_scope(&scope).await?;

// 或者使用便捷方法
container.with_scope("request_scope", |container| {
    Box::pin(async move {
        let service: Arc<MyService> = container.resolve().await?;
        // 使用服务...
        Ok(())
    })
}).await?;
```

## 依赖注入

```rust
#[derive(Debug)]
pub struct UserService {
    db: Arc<DatabaseService>,
}

impl UserService {
    pub fn new(db: Arc<DatabaseService>) -> Self {
        Self { db }
    }
}

// 注册时解析依赖
let user_factory: ComponentFactory = Box::new(|resolver| {
    Box::pin(async move {
        let db: Arc<DatabaseService> = resolver.resolve().await?;
        let service = UserService::new(db);
        service.initialize().await?;
        Ok(Arc::new(service) as Arc<dyn std::any::Any + Send + Sync>)
    })
});
```

## 错误处理

框架提供了详细的错误类型：

```rust
pub enum ContainerError {
    ComponentNotFound { name: String },
    CircularDependency { components: Vec<String> },
    ComponentCreationFailed { name: String, source: anyhow::Error },
    DependencyResolutionFailed { name: String, dependency: String },
    LifecycleError { message: String },
    // ...
}
```

## 最佳实践

1. **组件设计**: 保持组件无状态或最小状态
2. **依赖管理**: 避免循环依赖，使用接口抽象
3. **生命周期**: 根据使用场景选择合适的生命周期
4. **错误处理**: 合理处理组件创建和依赖解析错误
5. **作用域管理**: 及时清理作用域，避免内存泄漏

## 完整示例

查看 `examples/basic_usage.rs` 获取完整的使用示例。

## 许可证

MIT License

## 自动代理机制说明

> **自动代理机制说明**
>
> 当前 IoC 容器的 `auto_proxy` 配置项和 `create_aop_proxy_if_needed` 方法，主要用于兼容 Spring 风格的自动代理配置。由于 Rust 类型系统的限制，**无法像 Java/Spring 那样在运行时为任意类型自动生成动态代理**。因此，`create_aop_proxy_if_needed` 目前仅作为标记和日志用途，实际返回原始实例，不会自动包裹代理。
>
> **如何实现AOP拦截？**
>
> - 推荐使用 `aop_proxy!`、`proxy_method!` 等宏，或在服务方法中手动调用 `apply_aspects`，实现方法级别的切面拦截。
> - 只有通过这些宏或手动调用，才能真正实现方法的AOP拦截。
> - `auto_proxy` 仅作为配置兼容和未来扩展的钩子，不会自动生效。
>
> **示例：**
>
> ```rust
> // 推荐用法：使用宏生成AOP代理
> aop_proxy!(
>     MyService,
>     MyServiceProxy,
>     {
>         async fn do_something(&self, arg: i32) -> Result<()> {
>             // 方法体
>         }
>     }
> );
> ```
>
> > ⚠️ 注意：如果你希望所有方法都自动被AOP拦截，必须在注册/实现服务时主动使用上述宏或手动调用切面逻辑。