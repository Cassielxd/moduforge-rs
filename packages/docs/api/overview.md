# API 概览

ModuForge-RS 提供了一套完整的 API 用于构建高性能文档编辑器。本节提供 API 的整体概览和快速参考。

## 核心模块

### mf_model - 数据模型

核心的不可变数据结构和文档模型。

```rust
use mf_model::{Node, Tree, Schema, Mark, Attrs};

// 主要类型
pub struct Node { /* ... */ }
pub struct Tree { /* ... */ }
pub struct Schema { /* ... */ }
pub struct Mark { /* ... */ }
pub type Attrs = HashTrieMapSync<String, Value>;
```

**主要 API**：
- `Node::new()` - 创建节点
- `Tree::add()` - 添加节点到树
- `Schema::compile()` - 编译 Schema
- `Mark::new()` - 创建标记

[详细文档 →](./node.md)

### mf_state - 状态管理

事务化的状态管理系统。

```rust
use mf_state::{State, Transaction, Plugin, Selection};

// 主要类型
pub struct State { /* ... */ }
pub struct Transaction { /* ... */ }
pub trait Plugin { /* ... */ }
pub struct Selection { /* ... */ }
```

**主要 API**：
- `State::create()` - 创建状态
- `State::apply()` - 应用事务
- `Transaction::new()` - 创建事务
- `Plugin::new()` - 创建插件

[详细文档 →](./state.md)

### mf_transform - 文档转换

基于步骤的文档转换系统。

```rust
use mf_transform::{Transform, Step, AddNodeStep, RemoveNodeStep};

// 主要类型
pub struct Transform { /* ... */ }
pub enum Step { /* ... */ }
pub struct AddNodeStep { /* ... */ }
pub struct RemoveNodeStep { /* ... */ }
```

**主要 API**：
- `Transform::new()` - 创建转换
- `Transform::add_step()` - 添加步骤
- `Step::apply()` - 应用步骤
- `Step::invert()` - 反转步骤

[详细文档 →](./transform.md)

### mf_core - 运行时框架

编辑器运行时和扩展系统。

```rust
use mf_core::{ForgeRuntime, Extension, Command, Middleware};

// 主要类型
pub enum ForgeRuntime {
    Sync(SyncRuntime),
    Async(ForgeAsyncRuntime),
    Actor(ForgeActorRuntime),
}
```

**主要 API**：
- `ForgeRuntimeBuilder::new()` - 创建运行时
- `Runtime::dispatch()` - 分发命令
- `Extension::register()` - 注册扩展

[详细文档 →](./runtime.md)

## 快速参考

### 创建编辑器

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType};
use mf_model::{Schema, Node};
use mf_state::State;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 创建运行时
    let runtime = ForgeRuntimeBuilder::new()
        .runtime_type(RuntimeType::Async)
        .build()
        .await?;

    // 2. 创建 Schema
    let schema = Schema::default();

    // 3. 创建文档
    let doc = Node::new("doc", vec![
        Node::new("paragraph", vec![
            Node::text("Hello, World!")
        ])
    ]);

    // 4. 创建状态
    let state = State::create(doc, schema, vec![]);

    Ok(())
}
```

### 文档操作

```rust
// 添加节点
let mut tr = state.tr();
tr.add_node("parent_id", vec![new_node])?;
state = state.apply(tr)?;

// 更新属性
let mut tr = state.tr();
tr.set_node_attribute("node_id", hashmap!{
    "key" => json!("value")
})?;
state = state.apply(tr)?;

// 删除节点
let mut tr = state.tr();
tr.remove_node("parent_id", vec!["child_id"])?;
state = state.apply(tr)?;

// 移动节点
let mut tr = state.tr();
tr.move_node("node_id", "old_parent", "new_parent", Some(0))?;
state = state.apply(tr)?;
```

### 插件开发

```rust
use mf_macro::{mf_plugin, mf_meta};

mf_plugin!(
    my_plugin,

    metadata = mf_meta!(
        version = "1.0.0",
        description = "示例插件"
    ),

    append_transaction = async |trs, old_state, new_state| {
        // 插件逻辑
        Ok(None)
    }
);
```

### 自定义节点

```rust
use mf_derive::Node;
use serde::{Serialize, Deserialize};

#[derive(Node, Debug, Clone, Serialize, Deserialize)]
#[node_type = "custom"]
#[content = "text*"]
pub struct CustomNode {
    #[attr]
    pub title: String,

    #[attr(default = 0)]
    pub priority: i32,
}
```

## API 设计原则

### 1. 不可变性

所有核心数据结构都是不可变的：

```rust
// 不可变操作返回新版本
let new_tree = tree.add_node(node);
let new_state = state.apply(transaction)?;
```

### 2. 类型安全

使用 Rust 的类型系统保证安全：

```rust
// 编译时类型检查
let node: Node = Node::new("paragraph", vec![]);
let state: State = State::create(doc, schema, plugins);
```

### 3. 错误处理

使用 `Result` 类型处理错误：

```rust
// 明确的错误处理
match state.apply(transaction) {
    Ok(new_state) => { /* 成功 */ },
    Err(e) => { /* 处理错误 */ },
}
```

### 4. 异步支持

原生支持异步操作：

```rust
// 异步 API
let result = runtime.dispatch(command).await?;
let new_state = plugin.process(state).await?;
```

## 版本兼容性

### 语义版本控制

ModuForge-RS 遵循语义版本控制：

- **主版本（Major）**：不兼容的 API 变更
- **次版本（Minor）**：向后兼容的功能添加
- **补丁版本（Patch）**：向后兼容的错误修复

### 最低支持版本

- **Rust**: 1.70.0+
- **Tokio**: 1.0+
- **Serde**: 1.0+

## 性能特征

### 时间复杂度

| 操作 | 复杂度 | 说明 |
|-----|-------|------|
| 节点访问 | O(1) | 通过 ID 直接访问 |
| 节点添加 | O(log n) | 不可变树操作 |
| 属性更新 | O(log m) | m 为属性数量 |
| 文档遍历 | O(n) | n 为节点数量 |
| 事务应用 | O(s) | s 为步骤数量 |

### 空间复杂度

- **结构共享**：未修改的部分在版本间共享
- **懒加载**：大文档支持按需加载
- **增量更新**：只传输和存储变化的部分

## 错误类型

### 常见错误

```rust
use mf_state::error::StateError;
use mf_transform::error::TransformError;

// 状态错误
pub enum StateError {
    ValidationFailed(String),
    NodeNotFound(NodeId),
    InvalidOperation(String),
    // ...
}

// 转换错误
pub enum TransformError {
    InvalidStep(String),
    ConflictingSteps,
    SchemaViolation(String),
    // ...
}
```

### 错误处理模式

```rust
// 使用 ? 操作符
fn process() -> Result<State> {
    let tr = state.tr();
    tr.add_node("parent", vec![node])?;
    state.apply(tr)?
}

// 使用 match
match operation() {
    Ok(result) => handle_success(result),
    Err(StateError::ValidationFailed(msg)) => {
        eprintln!("验证失败: {}", msg);
    }
    Err(e) => {
        eprintln!("其他错误: {}", e);
    }
}
```

## 调试支持

### 日志记录

```rust
use tracing::{debug, info, warn, error};

// 启用日志
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();

// 记录日志
debug!("节点添加: {:?}", node);
info!("事务应用成功");
warn!("性能警告: 大事务");
error!("操作失败: {}", e);
```

### 开发工具

```rust
// 启用开发特性
#[cfg(feature = "dev-tracing")]
use mf_core::dev::TracingMiddleware;

#[cfg(feature = "dev-console")]
use mf_core::dev::ConsoleMonitor;
```

## 扩展点

### 自定义命令

```rust
use mf_state::Command;

#[derive(Debug)]
struct CustomCommand;

impl Command for CustomCommand {
    fn name(&self) -> &str { "custom" }

    async fn execute(&self, tr: &mut Transaction) -> Result<()> {
        // 命令实现
        Ok(())
    }
}
```

### 自定义中间件

```rust
use mf_core::Middleware;

struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    async fn before(&self, cmd: &Command) {
        println!("执行命令: {}", cmd.name());
    }

    async fn after(&self, cmd: &Command, result: &Result<()>) {
        println!("命令完成: {:?}", result);
    }
}
```

### 自定义插件

```rust
use mf_state::Plugin;

struct CustomPlugin;

impl Plugin for CustomPlugin {
    fn name(&self) -> &str { "custom" }

    async fn process(&self, state: State) -> Result<State> {
        // 插件处理
        Ok(state)
    }
}
```

## 最佳实践

### 1. 使用 Builder 模式

```rust
let runtime = ForgeRuntimeBuilder::new()
    .runtime_type(RuntimeType::Async)
    .max_concurrent_tasks(10)
    .plugin(my_plugin)
    .middleware(logging_middleware)
    .build()
    .await?;
```

### 2. 批量操作

```rust
// 好：批量操作
let mut tr = state.tr();
for node in nodes {
    tr.add_node("parent", vec![node])?;
}
state = state.apply(tr)?;

// 不好：多次单独操作
for node in nodes {
    let tr = state.tr().add_node("parent", vec![node])?;
    state = state.apply(tr)?;
}
```

### 3. 资源管理

```rust
// 使用 Arc 共享大型数据
let large_data = Arc::new(load_large_data());
let plugin = MyPlugin::new(large_data.clone());
```

## 相关资源

- [GitHub 仓库](https://github.com/moduforge/moduforge-rs)
- [示例项目](../examples/basic-editor.md)
- [price-rs 实际案例](../examples/real-world-nodes.md)
- [性能基准测试](../guide/benchmarks.md)

## 下一步

- [ForgeRuntime API](./runtime.md) - 运行时详细文档
- [State API](./state.md) - 状态管理详细文档
- [Node API](./node.md) - 节点系统详细文档
- [Plugin API](./plugin.md) - 插件系统详细文档