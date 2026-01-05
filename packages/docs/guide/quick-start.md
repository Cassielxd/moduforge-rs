# 快速开始

本指南将帮助你快速上手 ModuForge-RS，从安装到创建第一个编辑器应用。

## 环境准备

### 系统要求

- **Rust 工具链**：1.70 或更高版本
- **操作系统**：Windows、macOS 或 Linux
- **内存**：建议 4GB 以上
- **磁盘空间**：至少 2GB 可用空间

### 安装 Rust

如果还没有安装 Rust，请访问 [rust-lang.org](https://www.rust-lang.org/) 安装最新的稳定版：

```bash
# Unix/Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# 下载并运行 rustup-init.exe
```

验证安装：

```bash
rustc --version
cargo --version
```

### 创建新项目

使用 Cargo 创建一个新的 Rust 项目：

```bash
cargo new my-forge-app
cd my-forge-app
```

### 添加依赖

编辑 `Cargo.toml`，添加 ModuForge-RS 依赖：

```toml
[package]
name = "my-forge-app"
version = "0.1.0"
edition = "2021"

[dependencies]
# 核心运行时
moduforge-core = "0.7.0"
# 数据模型
moduforge-model = "0.7.0"
# 状态管理
moduforge-state = "0.7.0"
# 变更管理
moduforge-transform = "0.7.0"

# 异步运行时
tokio = { version = "1", features = ["full"] }
# 错误处理
anyhow = "1"
```

## 第一个应用

### 方式 1：最简单的使用

创建一个最基本的运行时实例：

```rust
// src/main.rs
use anyhow::Result;
use mf_core::ForgeRuntimeBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt::init();

    // 创建运行时 - 自动选择最优配置
    let mut runtime = ForgeRuntimeBuilder::new()
        .build()
        .await?;

    println!("ModuForge 运行时已启动！");

    // 获取当前状态
    let state = runtime.get_state().await?;
    println!("文档节点数: {}", state.doc().size());
    println!("运行时类型: {:?}", runtime.runtime_type());

    Ok(())
}
```

运行程序：

```bash
cargo run
```

### 方式 2：创建文档结构

创建一个包含节点的简单文档：

```rust
use anyhow::Result;
use mf_core::ForgeRuntimeBuilder;
use mf_model::{Node, NodeType, Attrs};
use mf_transform::node_step::AddNodeStep;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // 创建运行时
    let mut runtime = ForgeRuntimeBuilder::new()
        .build()
        .await?;

    // 创建文档节点
    let doc = Node::new(
        "doc".into(),
        NodeType::block("document"),
        Attrs::new(),
        None,
    );

    // 创建段落节点
    let paragraph = Node::new(
        "p1".into(),
        NodeType::block("paragraph"),
        Attrs::new(),
        Some("Hello ModuForge!".into()),
    );

    // 创建标题节点
    let heading = Node::new(
        "h1".into(),
        NodeType::block("heading"),
        {
            let mut attrs = Attrs::new();
            attrs.insert("level".into(), 1.into());
            attrs
        },
        Some("欢迎使用 ModuForge-RS".into()),
    );

    // 创建事务
    let mut tr = runtime.get_tr().await?;

    // 添加节点步骤
    tr.add_step(Box::new(AddNodeStep::new_single(doc, None)));
    tr.add_step(Box::new(AddNodeStep::new_single(heading, Some("doc".into()))));
    tr.add_step(Box::new(AddNodeStep::new_single(paragraph, Some("doc".into()))));

    // 提交事务
    tr.commit()?;
    runtime.dispatch(tr).await?;

    // 查看结果
    let state = runtime.get_state().await?;
    println!("文档结构创建完成！");
    println!("节点总数: {}", state.doc().size());

    // 遍历文档节点
    if let Some(doc_node) = state.doc().get_node("doc") {
        println!("\n文档内容:");
        for child_id in doc_node.children() {
            if let Some(child) = state.doc().get_node(child_id) {
                println!("  - {}: {}",
                    child.node_type().name(),
                    child.text().unwrap_or("(无内容)")
                );
            }
        }
    }

    Ok(())
}
```

### 方式 3：带有撤销/重做功能

添加历史管理功能：

```rust
use anyhow::Result;
use mf_core::{ForgeRuntimeBuilder, RuntimeTrait};
use mf_model::{Node, NodeType, Attrs};
use mf_transform::node_step::{AddNodeStep, UpdateNodeStep};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // 创建带历史管理的运行时
    let mut runtime = ForgeRuntimeBuilder::new()
        .history_limit(100)  // 保留100步历史
        .build()
        .await?;

    // 创建初始文档
    let doc = Node::new("doc".into(), NodeType::block("document"), Attrs::new(), None);
    let mut tr = runtime.get_tr().await?;
    tr.add_step(Box::new(AddNodeStep::new_single(doc, None)));
    tr.commit()?;
    runtime.dispatch(tr).await?;

    // 添加第一个段落
    let p1 = Node::new("p1".into(), NodeType::block("paragraph"), Attrs::new(), Some("第一段".into()));
    let mut tr = runtime.get_tr().await?;
    tr.add_step(Box::new(AddNodeStep::new_single(p1, Some("doc".into()))));
    tr.commit()?;
    runtime.dispatch(tr).await?;
    println!("添加第一段");

    // 添加第二个段落
    let p2 = Node::new("p2".into(), NodeType::block("paragraph"), Attrs::new(), Some("第二段".into()));
    let mut tr = runtime.get_tr().await?;
    tr.add_step(Box::new(AddNodeStep::new_single(p2, Some("doc".into()))));
    tr.commit()?;
    runtime.dispatch(tr).await?;
    println!("添加第二段");

    // 撤销最后一次操作
    runtime.undo().await?;
    println!("撤销操作 - 第二段被移除");

    // 重做操作
    runtime.redo().await?;
    println!("重做操作 - 第二段恢复");

    // 查看最终状态
    let state = runtime.get_state().await?;
    println!("\n最终文档有 {} 个节点", state.doc().size());

    Ok(())
}
```

## 运行时配置

### 选择运行时类型

ModuForge-RS 提供三种运行时类型：

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType};

// 1. 同步运行时（最简单）
let runtime = ForgeRuntimeBuilder::new()
    .runtime_type(RuntimeType::Sync)
    .build()
    .await?;

// 2. 异步运行时（推荐）
let runtime = ForgeRuntimeBuilder::new()
    .runtime_type(RuntimeType::Async)
    .build()
    .await?;

// 3. Actor 运行时（高并发场景）
let runtime = ForgeRuntimeBuilder::new()
    .runtime_type(RuntimeType::Actor)
    .build()
    .await?;
```

### 自适应配置

让运行时自动选择最优配置：

```rust
use mf_core::{ForgeRuntimeBuilder, Environment};

let runtime = ForgeRuntimeBuilder::new()
    .environment(Environment::Production)  // 生产环境优化
    .build()  // 自动检测系统资源
    .await?;
```

### 完整配置示例

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType, Environment};

let runtime = ForgeRuntimeBuilder::new()
    // 基本配置
    .runtime_type(RuntimeType::Async)
    .environment(Environment::Production)

    // 性能配置
    .max_concurrent_tasks(20)        // 最大并发任务数
    .queue_size(5000)                // 任务队列大小
    .task_timeout_ms(30000)          // 任务超时时间

    // 历史配置
    .history_limit(1000)             // 历史记录限制
    .snapshot_interval(100)          // 快照间隔

    // 监控配置
    .enable_monitoring(true)         // 启用性能监控

    .build()
    .await?;
```

## 使用宏简化开发

### 定义自定义节点

使用派生宏快速定义节点类型：

```rust
use mf_derive::Node;

#[derive(Node)]
#[node_type = "task"]
#[marks = "important urgent"]
struct TaskNode {
    #[attr]
    title: String,

    #[attr]
    completed: bool,

    #[attr]
    priority: i32,
}

// 使用自定义节点
let task = TaskNode {
    title: "完成文档".to_string(),
    completed: false,
    priority: 1,
};

let node = task.to_node("task1".into(), Some("任务详情".into()));
```

## 常见问题

### Q: 如何选择合适的运行时类型？

**A:** 选择建议：
- **Sync**：简单应用、快速原型
- **Async**：一般 Web 应用、I/O 密集型
- **Actor**：高并发、分布式场景

### Q: 运行时会自动保存状态吗？

**A:** 不会。需要手动调用持久化 API 或集成 `moduforge-persistence`。

### Q: 如何处理错误？

**A:** ModuForge-RS 使用 `Result` 类型返回错误。建议使用 `anyhow` 或 `thiserror` 处理：

```rust
use anyhow::{Result, Context};

async fn my_function() -> Result<()> {
    let runtime = ForgeRuntimeBuilder::new()
        .build()
        .await
        .context("创建运行时失败")?;

    Ok(())
}
```

### Q: 可以在多线程环境使用吗？

**A:** 可以。ModuForge-RS 的核心类型都是线程安全的，但建议使用 Actor 运行时处理并发。

## 下一步

恭喜！你已经创建了第一个 ModuForge-RS 应用。接下来可以：

- 📖 深入了解[核心概念](./core-concepts.md)
- 🏗️ 学习[架构设计](./architecture.md)
- 💡 查看[示例代码](../examples/)
- 🔧 探索各个[crate 的详细文档](../crates/)

## 获取帮助

遇到问题？可以通过以下方式获取帮助：

- 查看 [GitHub Issues](https://github.com/Cassielxd/moduforge-rs/issues)
- 阅读 [DeepWiki 文档](https://deepwiki.com/Cassielxd/moduforge-rs)
- 查看代码中的测试用例和示例
