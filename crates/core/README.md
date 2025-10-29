# ModuForge 核心模块 (moduforge-core)

`moduforge-core` 是 ModuForge 生态系统的核心运行时框架，提供了完整的编辑器运行时环境。该模块基于 Rust 构建，采用异步架构设计，支持插件系统、事件驱动、中间件机制等现代化特性。

## 🏗️ 架构概述

ModuForge 核心采用分层架构设计，每个组件都有明确的职责：

```
┌─────────────────────────────────────────────────────────────┐
│                    ForgeRuntime                             │
│              (同步运行时 + 基础功能)                          │
├─────────────────────────────────────────────────────────────┤
│                  ForgeAsyncRuntime                          │
│              (异步运行时 + 高性能处理)                        │
├─────────────────────────────────────────────────────────────┤
│                    AsyncProcessor                           │
│              (异步任务处理 + 队列管理)                        │
├─────────────────────────────────────────────────────────────┤
│                    EventBus                                 │
│              (事件系统 + 发布订阅)                            │
├─────────────────────────────────────────────────────────────┤
│                ExtensionManager                             │
│              (扩展管理 + 插件系统)                            │
├─────────────────────────────────────────────────────────────┤
│                 HistoryManager                              │
│              (历史管理 + 撤销重做)                            │
└─────────────────────────────────────────────────────────────┘
```

## 🧩 核心组件

### 1. ForgeRuntime
**文件**: `src/runtime.rs`  
**职责**: 同步运行时核心实现

- **状态管理**: 文档状态和事务处理
- **事件系统**: 事件分发和处理
- **中间件支持**: 前置和后置中间件链
- **历史记录**: 撤销/重做功能
- **扩展管理**: 插件和扩展集成

**核心方法**:
```rust
impl ForgeRuntime {
    // 创建新的运行时实例
    pub async fn create(options: RuntimeOptions) -> ForgeResult<Self>;
    
    // 执行命令
    pub async fn command(&mut self, command: Arc<dyn Command>) -> ForgeResult<()>;
    
    // 分发事务
    pub async fn dispatch(&mut self, transaction: Transaction) -> ForgeResult<()>;
    
    // 撤销/重做操作
    pub fn undo(&mut self);
    pub fn redo(&mut self);
}
```

### 2. ForgeAsyncRuntime
**文件**: `src/async_runtime.rs`  
**职责**: 异步运行时高性能实现

- **性能监控**: 可配置的性能指标收集
- **超时保护**: 全面的超时机制
- **流式处理**: 基于 FlowEngine 的高性能处理
- **异步优化**: 优化的异步任务处理

**性能配置**:
```rust
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub enable_monitoring: bool,
    pub middleware_timeout_ms: u64,      // 中间件超时
    pub log_threshold_ms: u64,           // 日志阈值
    pub task_receive_timeout_ms: u64,    // 任务接收超时
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: false,
            middleware_timeout_ms: 500,
            log_threshold_ms: 50,
            task_receive_timeout_ms: 5000,
        }
    }
}
```

### 3. AsyncProcessor
**文件**: `src/async_processor.rs`  
**职责**: 异步任务处理器

- **任务队列**: 高性能任务队列管理
- **并发控制**: 可配置的并发任务数
- **重试机制**: 自动重试和错误恢复
- **统计监控**: 详细的性能统计信息

**任务状态**:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,        // 等待处理
    Processing,     // 处理中
    Completed,      // 已完成
    Failed(String), // 失败
    Timeout,        // 超时
    Cancelled,      // 已取消
}
```

**处理器配置**:
```rust
#[derive(Clone, Debug)]
pub struct ProcessorConfig {
    pub max_queue_size: usize,        // 最大队列大小
    pub max_concurrent_tasks: usize,  // 最大并发任务数
    pub task_timeout: Duration,       // 任务超时时间
    pub max_retries: u32,             // 最大重试次数
    pub retry_delay: Duration,        // 重试延迟
}
```

### 4. EventBus
**文件**: `src/event.rs`  
**职责**: 事件总线系统

- **发布订阅**: 异步事件分发
- **并发处理**: 并发事件处理器
- **优雅关闭**: 支持优雅关闭和信号处理
- **事件类型**: 支持多种事件类型

**事件类型**:
```rust
#[derive(Clone)]
pub enum Event {
    Create(Arc<State>),                                    // 创建事件
    TrApply(u64, Arc<Vec<Transaction>>, Arc<State>),      // 事务应用事件
    Destroy,                                               // 销毁事件
    Stop,                                                  // 停止事件
}
```

**事件处理器**:
```rust
#[async_trait::async_trait]
pub trait EventHandler<T>: Send + Sync + Debug {
    async fn handle(&self, event: &T) -> ForgeResult<()>;
}
```

### 5. ExtensionManager
**文件**: `src/extension_manager.rs`  
**职责**: 扩展和插件管理

- **插件加载**: 动态插件加载和卸载
- **模式解析**: 自动解析扩展模式
- **资源管理**: 全局资源管理器集成
- **性能监控**: 扩展加载性能指标

**扩展类型**:
```rust
#[derive(Clone)]
pub enum Extensions {
    N(Node),      // 节点扩展
    M(Mark),      // 标记扩展
    E(Extension), // 扩展对象
}
```

### 6. HistoryManager
**文件**: `src/history_manager.rs`  
**职责**: 历史记录管理

- **状态快照**: 状态历史记录
- **撤销重做**: 完整的撤销/重做功能
- **历史限制**: 可配置的历史记录限制
- **时间旅行**: 支持历史状态跳转

**历史操作**:
```rust
impl<T: Clone> HistoryManager<T> {
    // 插入新状态
    pub fn insert(&mut self, state: T);
    
    // 跳转到过去状态
    pub fn jump_to_past(&mut self, index: usize);
    
    // 跳转到未来状态
    pub fn jump_to_future(&mut self, index: usize);
    
    // 通用跳转方法
    pub fn jump(&mut self, n: isize);
}
```

### 7. Middleware
**文件**: `src/middleware.rs`  
**职责**: 中间件系统

- **前置处理**: 事务分发前的处理
- **后置处理**: 事务分发后的处理
- **中间件栈**: 可组合的中间件栈
- **超时保护**: 中间件执行超时保护

**中间件特征**:
```rust
#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    fn name(&self) -> String;
    
    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> ForgeResult<()>;
    
    async fn after_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> ForgeResult<Option<Transaction>>;
}
```

## 🔧 技术栈

### 核心依赖
```toml
[dependencies]
# 异步运行时
tokio = { version = "1.36.0", features = ["full"] }
tokio-util = { workspace = true }
async-channel = { workspace = true }

# 并发和同步
im = { workspace = true }
lazy_static = { workspace = true }

# 序列化
serde = { workspace = true }
serde_json = { workspace = true }

# 错误处理
anyhow = { workspace = true }
thiserror = { workspace = true }

# 异步特征
async-trait = { workspace = true }
futures = { workspace = true }

# 性能监控
metrics = "0.22.0"

# ModuForge 生态系统
moduforge-model = { version = "0.4.12", path = "../model" }
moduforge-state = { version = "0.4.12", path = "../state" }
moduforge-transform = { version = "0.4.12", path = "../transform" }
```

### 核心技术
- **异步编程**: 基于 Tokio 的高性能异步运行时
- **事件驱动**: 发布-订阅模式的事件系统
- **插件架构**: 可扩展的插件和扩展系统
- **中间件模式**: 可组合的中间件处理链
- **性能监控**: 全面的性能指标收集

## 🚀 快速开始

### 1. 最简单的用法（推荐）

```rust
use mf_core::ForgeRuntimeBuilder;

#[tokio::main]
async fn main() -> mf_core::ForgeResult<()> {
    // 自动检测系统资源，选择最优运行时
    let mut runtime = ForgeRuntimeBuilder::new()
        .build()
        .await?;

    // 获取当前状态
    let state = runtime.get_state().await?;
    println!("文档节点数: {}", state.doc().size());

    Ok(())
}
```

### 2. 指定运行时类型

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType};
use std::sync::Arc;

#[tokio::main]
async fn main() -> mf_core::ForgeResult<()> {
    // 明确使用 Async 运行时
    let mut runtime = ForgeRuntimeBuilder::new()
        .runtime_type(RuntimeType::Async)
        .build()
        .await?;

    // 执行命令
    let command = Arc::new(MyCommand);
    runtime.command(command).await?;

    // 获取文档
    let doc = runtime.doc().await?;
    println!("文档节点数: {}", doc.size());

    Ok(())
}
```

### 3. 完全自定义配置

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType, Environment};

#[tokio::main]
async fn main() -> mf_core::ForgeResult<()> {
    // 生产环境配置
    let mut runtime = ForgeRuntimeBuilder::new()
        .runtime_type(RuntimeType::Actor)
        .environment(Environment::Production)
        .max_concurrent_tasks(20)
        .queue_size(5000)
        .enable_monitoring(true)
        .history_limit(1000)
        .build()
        .await?;

    println!("运行时类型: {:?}", runtime.runtime_type());
    Ok(())
}
```

### 4. 运行时类型匹配

```rust
use mf_core::{ForgeRuntimeBuilder, AnyRuntime};

#[tokio::main]
async fn main() -> mf_core::ForgeResult<()> {
    let runtime = ForgeRuntimeBuilder::new().build().await?;

    // 根据运行时类型执行不同操作
    match &runtime {
        AnyRuntime::Sync(rt) => {
            println!("✅ 使用同步运行时 - 适合简单场景");
        },
        AnyRuntime::Async(rt) => {
            println!("✅ 使用异步运行时 - 适合中等并发");
        },
        AnyRuntime::Actor(rt) => {
            println!("✅ 使用 Actor 运行时 - 适合高并发");
        },
    }

    // 或者使用辅助方法
    if let Some(async_rt) = runtime.as_async() {
        println!("这是异步运行时的特定操作");
    }

    Ok(())
}
```

### 5. 事件系统使用

```rust
use mf_core::{ForgeRuntimeBuilder, Event, EventHandler};
use std::sync::Arc;

#[derive(Debug)]
struct MyEventHandler;

#[async_trait::async_trait]
impl EventHandler<Event> for MyEventHandler {
    async fn handle(&self, event: &Event) -> mf_core::ForgeResult<()> {
        match event {
            Event::Create(state) => {
                println!("🎉 编辑器创建: 版本 {}", state.version);
            }
            Event::TrApply(tr_id, transactions, state) => {
                println!("📝 事务应用: ID {}, 版本 {}", tr_id, state.version);
            }
            Event::Destroy => {
                println!("🗑️ 编辑器销毁");
            }
            Event::Stop => {
                println!("⏹️ 编辑器停止");
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> mf_core::ForgeResult<()> {
    // 在构建时添加事件处理器
    let mut runtime = ForgeRuntimeBuilder::new()
        .event_handler(Arc::new(MyEventHandler))
        .build()
        .await?;

    // 事件会自动触发
    Ok(())
}
```

### 中间件使用

```rust
use mf_core::{Middleware, MiddlewareStack};
use mf_state::{State, Transaction};
use std::sync::Arc;

#[derive(Debug)]
struct LoggingMiddleware {
    name: String,
}

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> mf_core::ForgeResult<()> {
        println!("🔍 [{}] 事务处理开始 - ID: {}", self.name, transaction.id);
        Ok(())
    }

    async fn after_dispatch(
        &self,
        _state: Option<Arc<State>>,
        _transactions: &[Transaction],
    ) -> mf_core::ForgeResult<Option<Transaction>> {
        println!("✅ [{}] 事务处理完成", self.name);
        Ok(None)
    }
}

// 创建中间件栈
let mut middleware_stack = MiddlewareStack::new();
middleware_stack.add(LoggingMiddleware {
    name: "LoggingMiddleware".to_string(),
});

// 注意：中间件栈目前需要通过 RuntimeOptions 配置
// 未来版本会添加到 ForgeRuntimeBuilder
```

## 📊 性能监控

### 内置指标
```rust
// 任务处理指标
pub const TASKS_SUBMITTED_TOTAL: &str = "core.tasks.submitted.total";
pub const TASKS_PROCESSED_TOTAL: &str = "core.tasks.processed.total";
pub const TASK_PROCESSING_DURATION_SECONDS: &str = "core.task.processing.duration.seconds";

// 编辑器指标
pub const EDITOR_CREATION_DURATION_SECONDS: &str = "core.editor.creation.duration.seconds";
pub const COMMANDS_EXECUTED_TOTAL: &str = "core.commands.executed.total";
pub const TRANSACTIONS_DISPATCHED_TOTAL: &str = "core.transactions.dispatched.total";

// 中间件指标
pub const MIDDLEWARE_EXECUTION_DURATION_SECONDS: &str = "core.middleware.execution.duration.seconds";

// 扩展指标
pub const EXTENSIONS_LOADED_TOTAL: &str = "core.extensions.loaded.total";
pub const PLUGINS_LOADED_TOTAL: &str = "core.plugins.loaded.total";
```

### 性能配置建议
```rust
// 开发环境配置
let dev_config = PerformanceConfig {
    enable_monitoring: true,
    middleware_timeout_ms: 10000,    // 10秒
    log_threshold_ms: 100,           // 100ms
    task_receive_timeout_ms: 30000,  // 30秒
};

// 生产环境配置
let prod_config = PerformanceConfig {
    enable_monitoring: true,
    middleware_timeout_ms: 1000,     // 1秒
    log_threshold_ms: 50,            // 50ms
    task_receive_timeout_ms: 5000,   // 5秒
};
```

## 🔍 开发调试工具

### tokio-console 实时监控

`tokio-console` 是一个强大的实时异步任务监控工具，可以帮助你：
- 📊 实时查看所有异步任务的状态
- ⏱️ 监控任务执行时间和唤醒次数
- 🐛 检测任务阻塞和性能问题
- 📈 查看资源使用情况

#### 安装 tokio-console 客户端

```bash
cargo install tokio-console
```

#### 启用 tokio-console 监控

**1. 在 Cargo.toml 中启用 feature：**

```toml
[dependencies]
moduforge-core = { version = "0.6.2", features = ["dev-console"] }
```

**2. 在代码中初始化：**

```rust
#[cfg(feature = "dev-console")]
use mf_core::tracing_init::tokio_console;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化 tokio-console（仅开发环境）
    #[cfg(feature = "dev-console")]
    tokio_console::init()?;

    // 你的应用代码
    let runtime = ForgeRuntimeBuilder::new().build().await?;

    // ...

    Ok(())
}
```

**3. 运行应用：**

```bash
# 启用 dev-console feature 运行
cargo run --features dev-console
```

**4. 在另一个终端连接监控：**

```bash
# 连接到默认地址 127.0.0.1:6669
tokio-console
```

#### tokio-console 界面操作

在 tokio-console 界面中：
- **`t`** - 切换到任务视图（查看所有异步任务）
- **`r`** - 切换到资源视图（查看锁、通道等资源）
- **`h`** - 显示帮助信息
- **`q`** - 退出
- **`↑/↓`** - 上下选择任务
- **`Enter`** - 查看任务详情

#### 运行示例

```bash
# 运行 tokio-console 演示示例
cargo run --example tokio_console_demo --features dev-console

# 在另一个终端运行
tokio-console
```

#### 自定义配置

```rust
#[cfg(feature = "dev-console")]
{
    // 使用自定义地址
    tokio_console::init_with_config("0.0.0.0:6669")?;
}
```

#### 注意事项

⚠️ **重要提示**：
- tokio-console 会有一定的性能开销，**不要在生产环境启用**
- 与其他 tracing 初始化函数（如 `init_tracing`）互斥，只能选择一个
- 需要 tokio 启用 `tracing` feature（已在 `dev-console` feature 中自动启用）

#### 对比：tokio-console vs tracing-chrome

| 特性 | tokio-console | tracing-chrome |
|------|---------------|----------------|
| **实时监控** | ✅ 是 | ❌ 否（事后分析） |
| **需要注解** | ❌ 否（自动） | ✅ 是（`#[instrument]`） |
| **监控范围** | 所有 tokio 任务 | 标记的函数 |
| **任务状态** | ✅ 显示 | ❌ 不显示 |
| **性能开销** | 较低 | 中等 |
| **使用场景** | 实时调试、监控 | 详细性能分析 |
| **可视化** | TUI 界面 | Chrome DevTools |

#### 推荐使用场景

- **开发调试时**：使用 `tokio-console` 实时监控任务状态
- **性能分析时**：使用 `tracing-chrome` 或 `tracing-perfetto` 进行详细分析
- **生产环境**：不启用任何追踪 feature，保持零开销

### 其他追踪工具

除了 tokio-console，还支持以下追踪工具：

```rust
#[cfg(feature = "dev-tracing")]
use mf_core::tracing_init::dev_tracing::{init_tracing, TraceConfig};

// Chrome Tracing（性能分析）
#[cfg(feature = "dev-tracing-chrome")]
let _guard = init_tracing(TraceConfig::chrome("./logs/trace.json"))?;

// Perfetto（高级性能分析）
#[cfg(feature = "dev-tracing-perfetto")]
let _guard = init_tracing(TraceConfig::perfetto("./logs/trace.perfetto"))?;
```

详见 [开发追踪指南](../../docs/DEV_TRACING_GUIDE.md)。

## 🔒 错误处理

### 错误类型
```rust
#[derive(Error, Debug)]
pub enum ForgeError {
    #[error("运行时错误: {0}")]
    Runtime(String),
    
    #[error("事件错误: {0}")]
    Event(String),
    
    #[error("中间件错误: {0}")]
    Middleware(String),
    
    #[error("扩展错误: {0}")]
    Extension(String),
    
    #[error("状态错误: {0}")]
    State(#[from] mf_state::StateError),
    
    #[error("其他错误: {0}")]
    Other(#[from] anyhow::Error),
}
```

### 错误恢复策略
- **自动重试**: 任务处理器支持自动重试机制
- **超时保护**: 全面的超时保护防止死锁
- **优雅降级**: 部分功能失效时的降级处理
- **错误传播**: 完整的错误传播链

## 🧪 测试

### 测试覆盖
```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test async_processor
cargo test event
cargo test middleware

# 运行性能测试
cargo test --release
```

### 测试示例
```rust
#[tokio::test]
async fn test_runtime_creation() -> mf_core::ForgeResult<()> {
    let mut runtime = ForgeRuntimeBuilder::new().build().await?;

    let state = runtime.get_state().await?;
    assert!(state.doc().size() >= 0);
    Ok(())
}

#[tokio::test]
async fn test_specific_runtime_type() -> mf_core::ForgeResult<()> {
    let mut runtime = ForgeRuntimeBuilder::new()
        .runtime_type(RuntimeType::Async)
        .build()
        .await?;

    assert_eq!(runtime.runtime_type(), RuntimeType::Async);
    Ok(())
}

#[tokio::test]
async fn test_async_processor() {
    let config = ProcessorConfig::default();
    let processor = TestProcessor;
    let mut async_processor = AsyncProcessor::new(config, processor);
    
    async_processor.start();
    
    let (task_id, mut rx) = async_processor
        .submit_task(42, 1)
        .await
        .expect("提交任务失败");
    
    let result = rx.recv().await.expect("接收结果失败");
    assert_eq!(result.status, TaskStatus::Completed);
}
```

## 🔧 配置选项

### 运行时配置
```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType, Environment, Content, Extensions};

// 完整配置示例
let runtime = ForgeRuntimeBuilder::new()
    // 运行时类型
    .runtime_type(RuntimeType::Actor)

    // 环境配置
    .environment(Environment::Production)

    // 内容和扩展
    .content(Content::NodePool(node_pool))
    .extension(Extensions::N(node))
    .extension(Extensions::M(mark))

    // 性能配置
    .max_concurrent_tasks(20)
    .queue_size(5000)
    .enable_monitoring(true)
    .middleware_timeout_ms(1000)

    // 历史配置
    .history_limit(100)

    // 事件处理
    .event_handler(Arc::new(MyEventHandler))

    // Schema 配置
    .schema_path("schema/main.xml")

    .build()
    .await?;
```

### 处理器配置
```rust
// 高性能配置
let config = ProcessorConfig {
    max_queue_size: 10000,
    max_concurrent_tasks: 50,
    task_timeout: Duration::from_secs(60),
    max_retries: 5,
    retry_delay: Duration::from_millis(100),
};
```

## 📈 性能优化

### 内存管理
- **智能清理**: 自动清理不活跃的资源
- **对象池**: 复用昂贵的对象实例
- **内存映射**: 高效的内存使用模式

### 并发优化
- **异步 I/O**: 基于 Tokio 的高性能异步处理
- **任务调度**: 智能的任务调度算法
- **锁优化**: 最小化锁竞争

### 缓存策略
- **LRU 缓存**: 最近最少使用的缓存策略
- **预加载**: 智能的预加载机制
- **缓存失效**: 高效的缓存失效策略

## 📚 相关文档

- [ModuForge 状态管理](../state/README.md)
- [ModuForge 数据模型](../model/README.md)
- [ModuForge 转换系统](../transform/README.md)
- [ModuForge 协作系统](../collaboration/README.md)

## 🤝 贡献指南

欢迎贡献代码！请确保：

1. 遵循 Rust 编码规范
2. 添加适当的测试
3. 更新相关文档
4. 通过所有 CI 检查
5. 性能测试通过

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](../../LICENSE) 文件。 