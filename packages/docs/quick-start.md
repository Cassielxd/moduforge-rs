# ModuForge-RS 快速入门指南

本文介绍如何在全新工程中集成 ModuForge-RS，示例全部基于当前工作区提供的 API 编写。

## 环境要求
- Rust 1.70 及以上（推荐使用最新稳定版）
- 操作系统：Windows 10+/macOS 10.15+/Ubuntu 20.04+
- Node.js 18+（仅在需要运行示例前端或文档站点时）
- 常用工具：`cargo`，任选 `pnpm`/`npm`

## 创建项目并引入依赖

```bash
cargo new my-moduforge-app
cd my-moduforge-app
```

在 `Cargo.toml` 中通过 `path` 方式引用工作区的 crate（假设仓库与项目同级目录）：

```toml
[dependencies]
moduforge-core = { path = "../moduforge-rs/crates/core" }
moduforge-model = { path = "../moduforge-rs/crates/model" }
moduforge-state = { path = "../moduforge-rs/crates/state" }
moduforge-transform = { path = "../moduforge-rs/crates/transform" }

# 可选组件（按需引入）
moduforge-file = { path = "../moduforge-rs/crates/file" }
moduforge-persistence = { path = "../moduforge-rs/crates/persistence" }
moduforge-search = { path = "../moduforge-rs/crates/search" }
moduforge-collaboration = { path = "../moduforge-rs/crates/collaboration" }

# 通用依赖
tokio = { version = "1", features = ["full"] }
anyhow = "1"
serde = { version = "1.0", features = ["derive", "rc"] }
serde_json = "1.0"
imbl = { version = "6", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

> 等到 crate 发布到 crates.io 后，可将 `path` 换成对应的 `version`。

## 第一个运行时示例

`src/main.rs`

```rust
use anyhow::Result;
use mf_core::{ForgeAsyncRuntime, ForgeConfig, Environment};
use mf_core::types::{Content, NodePoolFnTrait, RuntimeOptions};
use mf_model::{node_pool::NodePool, Attrs, Node, NodeType};
use mf_transform::node_step::AddNodeStep;
use mf_state::init_logging;
use std::sync::Arc;

#[derive(Debug)]
struct DefaultPool;

#[async_trait::async_trait]
impl NodePoolFnTrait for DefaultPool {
    async fn create(
        &self,
        _config: &mf_state::StateConfig,
    ) -> mf_core::ForgeResult<NodePool> {
        Ok(NodePool::default())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging("info", None)?;

    let config = ForgeConfig::builder()
        .environment(Environment::Development)
        .build()?;

    let options = RuntimeOptions::default()
        .set_content(Content::NodePoolFn(Arc::new(DefaultPool)));

    let mut runtime = ForgeAsyncRuntime::create_with_config(options, config).await?;

    let doc = Node::new("doc".into(), NodeType::block("document"), Attrs::new(), None);
    let paragraph = Node::new(
        "p1".into(),
        NodeType::block("paragraph"),
        Attrs::new(),
        Some("Hello ModuForge".into()),
    );

    let mut tr = runtime.get_state().tr();
    tr.add_step(Box::new(AddNodeStep::new_single(doc, None)));
    tr.add_step(Box::new(AddNodeStep::new_single(paragraph, Some("doc".into()))));
    runtime.dispatch_flow(tr).await?;

    println!("current nodes: {}", runtime.get_state().doc().size());
    Ok(())
}
```

```bash
cargo run
```

出现 `current nodes: 2` 即表示文档树创建成功。

## 事件持久化（SQLite）

```rust
use mf_persistence::{
    api::{CommitMode, PersistOptions},
    sqlite::SqliteEventStore,
};

let store = SqliteEventStore::open(
    "./data/app.db",
    CommitMode::AsyncDurable { group_window_ms: 8 },
)?;

let options = PersistOptions {
    commit_mode: CommitMode::AsyncDurable { group_window_ms: 8 },
    snapshot_every_n_events: 200,
    snapshot_every_bytes: 4 * 1024 * 1024,
    snapshot_every_ms: 60_000,
    compression: true,
};
// 将 Transaction 转换成 PersistedEvent 后调用 store.append_batch(events).await?
```

## 全文检索

```rust
use mf_search::{create_tantivy_service, IndexEvent, RebuildScope};
use std::{path::PathBuf, sync::Arc};

async fn rebuild_index(pool: Arc<mf_model::node_pool::NodePool>) -> anyhow::Result<()> {
    let dir = PathBuf::from("./data/index");
    let service = create_tantivy_service(&dir)?;
    service
        .handle(IndexEvent::Rebuild { pool, scope: RebuildScope::Full })
        .await
}
```

## 实时协作

```rust
use mf_collab::{CollaborationServer, YrsManager};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    CollaborationServer::new(Arc::new(YrsManager::new()), 3030)
        .start()
        .await;
    Ok(())
}
```

## 推荐命令

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

## 下一步

1. 阅读 [`architecture-overview.md`](./architecture-overview.md) 了解整体架构。
2. 按需参考 [`plugin-development-guide.md`](./plugin-development-guide.md) 编写业务插件。
3. 查看 [`example-integration-project.md`](./example-integration-project.md) 获取完整骨架。
4. 运行 `examples/demo`（Tauri + Vue）体验协作示例。
