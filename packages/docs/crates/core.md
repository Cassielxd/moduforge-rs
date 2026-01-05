# moduforge-core 文档

`moduforge-core` 是 ModuForge-RS 的核心运行时模块，提供事件驱动的运行时环境、扩展管理、历史管理和多种执行模式。

## 概述

### 核心功能

- **多种运行时模式**：同步、异步、Actor 三种运行时
- **自适应选择**：根据系统资源自动选择最优运行时
- **事件系统**：事件总线和事件处理
- **扩展管理**：插件式扩展机制
- **历史管理**：撤销/重做功能
- **中间件支持**：事务处理中间件
- **性能监控**：运行时指标采集

### 设计理念

1. **灵活性**：支持多种运行模式，适应不同场景
2. **可扩展性**：通过扩展和插件机制轻松扩展功能
3. **高性能**：针对不同场景优化的运行时实现
4. **易用性**：统一的 API，简化使用复杂度

## 安装

```toml
[dependencies]
moduforge-core = "0.7.0"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

## 快速开始

### 最简单的使用

```rust
use anyhow::Result;
use mf_core::ForgeRuntimeBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    // 自动选择最优运行时
    let mut runtime = ForgeRuntimeBuilder::new()
        .build()
        .await?;

    println!("运行时已启动: {:?}", runtime.runtime_type());

    Ok(())
}
```

## 运行时系统

### 1. 运行时类型

ModuForge-RS 提供三种运行时类型：

#### 同步运行时 (Sync Runtime)

适合简单、快速的场景：

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType};

let runtime = ForgeRuntimeBuilder::new()
    .runtime_type(RuntimeType::Sync)
    .build()
    .await?;
```

**特点**：
- 单线程执行
- 简单直接
- 最低开销
- 适合原型开发和简单应用

**适用场景**：
- 快速原型
- 简单文档处理
- 单用户应用
- 低资源环境

#### 异步运行时 (Async Runtime)

基于 Tokio 的高并发运行时：

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType};

let runtime = ForgeRuntimeBuilder::new()
    .runtime_type(RuntimeType::Async)
    .build()
    .await?;
```

**特点**：
- 高并发能力
- 非阻塞 I/O
- 资源高效利用
- 适合 Web 应用

**适用场景**：
- Web 服务器
- API 后端
- I/O 密集型应用
- 中等并发需求

#### Actor 运行时 (Actor Runtime)

基于 Ractor 的 Actor 模型：

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType};

let runtime = ForgeRuntimeBuilder::new()
    .runtime_type(RuntimeType::Actor)
    .build()
    .await?;
```

**特点**：
- 消息驱动
- 隔离性强
- 分布式友好
- 高度并发

**适用场景**：
- 大规模并发
- 分布式系统
- 实时协作
- 微服务架构

### 2. 自适应运行时选择

让系统自动选择最优运行时：

```rust
use mf_core::ForgeRuntimeBuilder;

// 自动检测系统资源并选择运行时
let runtime = ForgeRuntimeBuilder::new()
    .build()  // 不指定类型，自动选择
    .await?;
```

**选择逻辑**：

```rust
// 低资源 (<2GB RAM, <4 cores)
if system_resources.is_low() {
    RuntimeType::Sync
}
// 中等资源 (2-8GB RAM, 4-8 cores)
else if system_resources.is_medium() {
    RuntimeType::Async
}
// 高资源 (>8GB RAM, >8 cores)
else {
    RuntimeType::Actor
}
```

### 3. 运行时配置

#### 基础配置

```rust
use mf_core::{ForgeRuntimeBuilder, Environment};

let runtime = ForgeRuntimeBuilder::new()
    // 运行时类型
    .runtime_type(RuntimeType::Async)

    // 运行环境
    .environment(Environment::Production)

    // 任务配置
    .max_concurrent_tasks(20)
    .queue_size(5000)
    .task_timeout_ms(30000)

    .build()
    .await?;
```

#### 历史配置

```rust
let runtime = ForgeRuntimeBuilder::new()
    // 历史记录限制
    .history_limit(1000)

    // 快照间隔（每N个事务保存快照）
    .snapshot_interval(100)

    .build()
    .await?;
```

#### 性能配置

```rust
let runtime = ForgeRuntimeBuilder::new()
    // 启用性能监控
    .enable_monitoring(true)

    // 工作线程数
    .worker_threads(4)

    // 缓存配置
    .cache_size(10000)
    .cache_ttl_secs(3600)

    .build()
    .await?;
```

#### 完整配置示例

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType, Environment};

let runtime = ForgeRuntimeBuilder::new()
    // 基础配置
    .runtime_type(RuntimeType::Async)
    .environment(Environment::Production)

    // 性能配置
    .max_concurrent_tasks(20)
    .queue_size(5000)
    .task_timeout_ms(30000)
    .worker_threads(4)

    // 历史配置
    .history_limit(1000)
    .snapshot_interval(100)

    // 监控配置
    .enable_monitoring(true)

    // 缓存配置
    .cache_size(10000)
    .cache_ttl_secs(3600)

    .build()
    .await?;

println!("运行时配置:");
println!("  类型: {:?}", runtime.runtime_type());
println!("  环境: {:?}", runtime.environment());
```

## 运行时 API

### 状态管理

```rust
// 获取当前状态
let state = runtime.get_state().await?;

// 获取文档
let doc = state.doc();

// 获取选区
let selection = state.selection();
```

### 事务处理

```rust
use mf_transform::node_step::AddNodeStep;
use mf_model::{Node, NodeType, Attrs};

// 创建事务
let mut tr = runtime.get_tr().await?;

// 添加步骤
let node = Node::new("n1".into(), NodeType::block("p"), Attrs::new(), Some("文本"));
tr.add_step(Box::new(AddNodeStep::new_single(node, None)));

// 提交事务
tr.commit()?;

// 分发事务到运行时
runtime.dispatch(tr).await?;
```

### 历史操作

```rust
// 检查是否可以撤销
if runtime.can_undo() {
    runtime.undo().await?;
    println!("已撤销");
}

// 检查是否可以重做
if runtime.can_redo() {
    runtime.redo().await?;
    println!("已重做");
}

// 获取历史信息
let history = runtime.history_info().await?;
println!("可撤销步数: {}", history.undo_depth);
println!("可重做步数: {}", history.redo_depth);
```

### 快照管理

```rust
// 保存快照
runtime.save_snapshot("checkpoint.mf").await?;

// 加载快照
runtime.load_snapshot("checkpoint.mf").await?;

// 自动快照（每100个事务）
let runtime = ForgeRuntimeBuilder::new()
    .snapshot_interval(100)
    .build()
    .await?;
```

### 关闭运行时

```rust
// 优雅关闭
runtime.shutdown().await?;
```

## 事件系统

### 事件总线

事件总线用于组件间通信：

```rust
use mf_core::event::{Event, EventBus, EventHandler};

// 获取事件总线
let event_bus = runtime.event_bus();

// 订阅事件
event_bus.subscribe("transaction_applied", Box::new(|event| {
    println!("事务已应用: {:?}", event.data());
    Ok(())
})).await?;

// 发布事件
event_bus.emit(Event::new("custom_event", some_data)).await?;
```

### 内置事件

```rust
// 运行时事件
"runtime_started"        // 运行时启动
"runtime_shutdown"       // 运行时关闭

// 事务事件
"transaction_started"    // 事务开始
"transaction_applying"   // 事务应用中
"transaction_applied"    // 事务已应用
"transaction_failed"     // 事务失败

// 文档事件
"document_changed"       // 文档已更改
"node_added"            // 节点已添加
"node_updated"          // 节点已更新
"node_removed"          // 节点已删除

// 历史事件
"history_undo"          // 撤销操作
"history_redo"          // 重做操作
"snapshot_saved"        // 快照已保存
"snapshot_loaded"       // 快照已加载

// 状态事件
"state_updated"         // 状态已更新
"selection_changed"     // 选区已改变
```

### 自定义事件处理器

```rust
struct MyEventHandler;

#[async_trait]
impl EventHandler for MyEventHandler {
    async fn handle(&self, event: &Event) -> Result<()> {
        match event.name() {
            "transaction_applied" => {
                println!("处理事务应用事件");
                // 自定义处理逻辑
            }
            "document_changed" => {
                println!("文档已更改");
                // 触发保存、同步等操作
            }
            _ => {}
        }
        Ok(())
    }
}

// 注册处理器
event_bus.register_handler(Box::new(MyEventHandler)).await?;
```

## 扩展系统

### 扩展管理器

```rust
use mf_core::{Extension, ExtensionManager};

// 获取扩展管理器
let ext_manager = runtime.extension_manager();

// 注册扩展
ext_manager.register("my_extension", Box::new(MyExtension)).await?;

// 获取扩展
let ext = ext_manager.get("my_extension")?;

// 移除扩展
ext_manager.unregister("my_extension").await?;

// 列出所有扩展
let extensions = ext_manager.list();
```

### 自定义扩展

```rust
use mf_core::Extension;
use mf_transform::Transaction;
use mf_model::Document;

struct ValidationExtension;

#[async_trait]
impl Extension for ValidationExtension {
    fn name(&self) -> &str {
        "validation"
    }

    async fn on_before_transaction(&self, tr: &Transaction) -> Result<()> {
        println!("验证事务: {} steps", tr.steps().len());
        // 验证逻辑
        Ok(())
    }

    async fn on_after_transaction(&self, tr: &Transaction) -> Result<()> {
        println!("事务已应用");
        // 后处理逻辑
        Ok(())
    }

    async fn on_validate_document(&self, doc: &Document) -> Result<()> {
        // 文档验证逻辑
        Ok(())
    }
}

// 注册扩展
runtime.extension_manager()
    .register("validation", Box::new(ValidationExtension))
    .await?;
```

### 扩展点

```rust
pub trait Extension: Send + Sync {
    fn name(&self) -> &str;

    // 事务生命周期
    async fn on_before_transaction(&self, tr: &Transaction) -> Result<()> { Ok(()) }
    async fn on_after_transaction(&self, tr: &Transaction) -> Result<()> { Ok(()) }

    // 步骤生命周期
    async fn on_before_step(&self, step: &dyn Step) -> Result<()> { Ok(()) }
    async fn on_after_step(&self, step: &dyn Step) -> Result<()> { Ok(()) }

    // 验证
    async fn on_validate_document(&self, doc: &Document) -> Result<()> { Ok(()) }
    async fn on_validate_node(&self, node: &Node) -> Result<()> { Ok(()) }

    // 状态变化
    async fn on_state_changed(&self, old: &State, new: &State) -> Result<()> { Ok(()) }
}
```

## 中间件系统

### 中间件定义

```rust
use mf_core::middleware::{Middleware, NextFn};

struct LoggingMiddleware;

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn process(&self, tr: Transaction, next: NextFn) -> Result<Transaction> {
        println!("事务开始: {} steps", tr.steps().len());

        // 调用下一个中间件
        let result = next(tr).await?;

        println!("事务完成");
        Ok(result)
    }
}
```

### 注册中间件

```rust
// 在构建时注册
let runtime = ForgeRuntimeBuilder::new()
    .middleware(Box::new(LoggingMiddleware))
    .middleware(Box::new(ValidationMiddleware))
    .build()
    .await?;
```

### 内置中间件

```rust
// 日志中间件
use mf_core::middleware::LoggingMiddleware;
runtime.use_middleware(Box::new(LoggingMiddleware::new()));

// 验证中间件
use mf_core::middleware::ValidationMiddleware;
runtime.use_middleware(Box::new(ValidationMiddleware::new(schema)));

// 性能监控中间件
use mf_core::middleware::MetricsMiddleware;
runtime.use_middleware(Box::new(MetricsMiddleware::new()));
```

## 性能监控

### 启用监控

```rust
let runtime = ForgeRuntimeBuilder::new()
    .enable_monitoring(true)
    .build()
    .await?;
```

### 获取指标

```rust
// 获取运行时指标
let metrics = runtime.metrics().await?;

println!("事务统计:");
println!("  总数: {}", metrics.transaction_count);
println!("  成功: {}", metrics.transaction_success);
println!("  失败: {}", metrics.transaction_failed);
println!("  平均耗时: {}ms", metrics.avg_transaction_time);

println!("\n性能指标:");
println!("  CPU 使用率: {}%", metrics.cpu_usage);
println!("  内存使用: {}MB", metrics.memory_usage);
println!("  活跃任务: {}", metrics.active_tasks);
```

### 自定义指标

```rust
// 记录自定义指标
runtime.record_metric("custom_operation", 123.45).await?;

// 增加计数器
runtime.increment_counter("page_views").await?;

// 记录时长
runtime.record_duration("api_call", duration).await?;
```

## Actor 系统

### Actor 消息

```rust
use mf_core::actors::{StateMessage, TransactionMessage};

// 向状态 Actor 发送消息
runtime.send_state_message(StateMessage::GetState {
    reply: tx,
}).await?;

// 向事务处理 Actor 发送消息
runtime.send_transaction_message(TransactionMessage::Apply(tr)).await?;
```

### Actor 配置

```rust
use mf_core::ActorSystemConfig;

let config = ActorSystemConfig::new()
    .mailbox_size(10000)
    .supervisor_strategy(SupervisorStrategy::OneForOne)
    .max_restarts(3);

let runtime = ForgeRuntimeBuilder::new()
    .runtime_type(RuntimeType::Actor)
    .actor_config(config)
    .build()
    .await?;
```

## 错误处理

### 错误类型

```rust
pub enum ForgeError {
    // 运行时错误
    RuntimeError(String),
    RuntimeNotInitialized,
    RuntimeAlreadyShutdown,

    // 事务错误
    TransactionFailed(String),
    TransactionTimeout,

    // 状态错误
    StateError(String),
    InvalidState,

    // Actor 错误
    ActorError(String),
    MessageSendFailed,

    // 扩展错误
    ExtensionError(String),
    ExtensionNotFound(String),
}
```

### 错误处理示例

```rust
use anyhow::{Result, Context};

async fn handle_transaction() -> Result<()> {
    let runtime = ForgeRuntimeBuilder::new()
        .build()
        .await
        .context("创建运行时失败")?;

    let tr = runtime.get_tr()
        .await
        .context("获取事务失败")?;

    runtime.dispatch(tr)
        .await
        .context("分发事务失败")?;

    Ok(())
}
```

## 完整示例

### 示例 1：基础文档编辑器

```rust
use anyhow::Result;
use mf_core::{ForgeRuntimeBuilder, RuntimeTrait};
use mf_model::{Node, NodeType, Attrs};
use mf_transform::node_step::AddNodeStep;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 创建运行时
    let mut runtime = ForgeRuntimeBuilder::new()
        .history_limit(100)
        .build()
        .await?;

    // 创建文档
    let doc = Node::new("doc".into(), NodeType::block("document"), Attrs::new(), None);
    let mut tr = runtime.get_tr().await?;
    tr.add_step(Box::new(AddNodeStep::new_single(doc, None)));
    tr.commit()?;
    runtime.dispatch(tr).await?;

    // 添加内容
    for i in 1..=5 {
        let p = Node::new(
            format!("p{}", i).into(),
            NodeType::block("paragraph"),
            Attrs::new(),
            Some(format!("第 {} 段", i).into())
        );

        let mut tr = runtime.get_tr().await?;
        tr.add_step(Box::new(AddNodeStep::new_single(p, Some("doc".into()))));
        tr.commit()?;
        runtime.dispatch(tr).await?;
    }

    // 查看结果
    let state = runtime.get_state().await?;
    println!("文档节点数: {}", state.doc().size());

    // 撤销最后一次操作
    runtime.undo().await?;
    println!("撤销后节点数: {}", runtime.get_state().await?.doc().size());

    // 重做
    runtime.redo().await?;
    println!("重做后节点数: {}", runtime.get_state().await?.doc().size());

    // 保存快照
    runtime.save_snapshot("checkpoint.mf").await?;

    // 关闭
    runtime.shutdown().await?;

    Ok(())
}
```

### 示例 2：事件驱动应用

```rust
use mf_core::event::{Event, EventHandler};

struct DocumentWatcher;

#[async_trait]
impl EventHandler for DocumentWatcher {
    async fn handle(&self, event: &Event) -> Result<()> {
        match event.name() {
            "document_changed" => {
                println!("文档已更改，触发自动保存");
                // 自动保存逻辑
            }
            "transaction_applied" => {
                println!("事务已应用，更新UI");
                // UI 更新逻辑
            }
            _ => {}
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut runtime = ForgeRuntimeBuilder::new().build().await?;

    // 注册事件处理器
    runtime.event_bus()
        .register_handler(Box::new(DocumentWatcher))
        .await?;

    // 执行操作会自动触发事件
    let mut tr = runtime.get_tr().await?;
    // ... 添加步骤
    runtime.dispatch(tr).await?;  // 触发 document_changed 事件

    Ok(())
}
```

## 最佳实践

### 1. 选择合适的运行时

```rust
// ✅ 简单应用
let runtime = ForgeRuntimeBuilder::new()
    .runtime_type(RuntimeType::Sync)
    .build().await?;

// ✅ Web 应用
let runtime = ForgeRuntimeBuilder::new()
    .runtime_type(RuntimeType::Async)
    .build().await?;

// ✅ 高并发/分布式
let runtime = ForgeRuntimeBuilder::new()
    .runtime_type(RuntimeType::Actor)
    .build().await?;
```

### 2. 合理配置资源

```rust
// ✅ 根据实际需求配置
let runtime = ForgeRuntimeBuilder::new()
    .max_concurrent_tasks(num_cpus::get() * 2)
    .queue_size(1000)
    .build().await?;

// ❌ 过度配置
let runtime = ForgeRuntimeBuilder::new()
    .max_concurrent_tasks(1000)  // 太大
    .queue_size(1000000)         // 太大
    .build().await?;
```

### 3. 使用事件解耦

```rust
// ✅ 通过事件通信
event_bus.emit(Event::new("save_needed", data)).await?;

// ❌ 直接调用
save_document()?;  // 紧耦合
```

## 下一步

- 查看 [moduforge-state](./state.md) 了解状态管理
- 查看 [moduforge-transform](./transform.md) 了解事务处理
- 浏览 [示例代码](../examples/) 学习实际应用
