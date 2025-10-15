# ModuForge-RS Quick Start Guide

This guide walks through integrating ModuForge-RS into a fresh project using the APIs that ship in this workspace.

## Requirements
- Rust 1.70 or newer (latest stable recommended)
- OS: Windows 10+, macOS 10.15+, or Ubuntu 20.04+
- Node.js 18+ if you plan to run the documentation site or the sample front-end
- Tooling: `cargo` and optionally `pnpm`/`npm`

## Create a project and add dependencies

```bash
cargo new my-moduforge-app
cd my-moduforge-app
```

Point dependencies to the local workspace while developing (adjust the relative paths to match your checkout):

```toml
[dependencies]
moduforge-core = { path = "../moduforge-rs/crates/core" }
moduforge-model = { path = "../moduforge-rs/crates/model" }
moduforge-state = { path = "../moduforge-rs/crates/state" }
moduforge-transform = { path = "../moduforge-rs/crates/transform" }

# Optional capabilities
moduforge-file = { path = "../moduforge-rs/crates/file" }
moduforge-persistence = { path = "../moduforge-rs/crates/persistence" }
moduforge-search = { path = "../moduforge-rs/crates/search" }
moduforge-collaboration = { path = "../moduforge-rs/crates/collaboration" }

# Shared dependencies
tokio = { version = "1", features = ["full"] }
anyhow = "1"
serde = { version = "1.0", features = ["derive", "rc"] }
serde_json = "1.0"
imbl = { version = "6", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

> Once the crates are published to crates.io you can swap `path` for a `version` requirement.

## Minimal runtime

`src/main.rs`

```rust
use anyhow::Result;
use mf_core::{ForgeAsyncRuntime, ForgeConfig, Environment};
use mf_core::types::{Content, NodePoolFnTrait, RuntimeOptions};
use mf_model::{node_pool::NodePool, Attrs, Node, NodeType};
use mf_state::init_logging;
use mf_transform::node_step::AddNodeStep;
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

You should see `current nodes: 2`, confirming the document tree is ready.

## SQLite event store

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
// Convert transactions into PersistedEvent values before calling store.append_batch(events).await?
```

## Full-text search

```rust
use mf_search::{create_tantivy_service, IndexEvent, RebuildScope};
use std::{path::PathBuf, sync::Arc};

async fn rebuild(pool: Arc<mf_model::node_pool::NodePool>) -> anyhow::Result<()> {
    let dir = PathBuf::from("./data/index");
    let service = create_tantivy_service(&dir)?;
    service
        .handle(IndexEvent::Rebuild { pool, scope: RebuildScope::Full })
        .await
}
```

## Real-time collaboration

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

## Recommended commands

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

## Next steps

1. Read the [architecture overview](./architecture-overview.md) for the big picture.
2. Follow the [plugin development guide](./plugin-development-guide.md) to implement domain logic.
3. Explore the [integration example](./example-integration-project.md) for a complete skeleton.
4. Run the `examples/demo` Tauri + Vue project if you need a UI reference.
