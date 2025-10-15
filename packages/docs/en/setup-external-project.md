# ModuForge-RS External Project Setup Guide

This guide explains how to consume the workspace from another repository. All snippets reference crates and APIs that exist today; historical references to `moduforge-rules-*` have been removed.

## 1. Copy .cursorrules templates

Pick the Cursor automation template that fits your team:

- **Full**: `.cursorrules-for-external-projects` — includes workflow guidance and architecture notes.
- **Lightweight**: `.cursorrules-external-simple` — keeps only dependency hints and essential commands.

```bash
cp path/to/moduforge-rs/.cursorrules-for-external-projects ./your-project/.cursorrules
# or
cp path/to/moduforge-rs/.cursorrules-external-simple ./your-project/.cursorrules
```

## 2. Configure Cargo.toml

Prefer path dependencies while developing against the workspace:

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

## 3. Runtime scaffold

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

    println!("runtime ready");
    Ok(())
}
```

## 4. Plugins and resources

```rust
use mf_core::{Extension, ForgeResult};
use mf_core::types::Extensions;
use mf_state::{plugin::Plugin, State};

#[derive(Default)]
struct AuditPlugin;

#[async_trait::async_trait]
impl Plugin for AuditPlugin {
    async fn on_transaction_applied(
        &self,
        state: &State,
        description: &str,
    ) -> ForgeResult<()> {
        tracing::info!(target: "audit", %description, node_count = state.doc().size());
        Ok(())
    }
}

pub fn audit_extension() -> Extensions {
    Extensions::E(Extension::new_plugin("audit", Box::new(AuditPlugin::default())))
}
```

Register extensions when building `RuntimeOptions`:

```rust
let options = RuntimeOptions::default()
    .set_content(Content::NodePoolFn(Arc::new(DefaultPool)))
    .add_extension(audit_extension());
```

## 5. Persistence and search

```rust
use mf_persistence::{
    api::{CommitMode, PersistOptions},
    sqlite::SqliteEventStore,
};

let store = SqliteEventStore::open(
    "./data/app.db",
    CommitMode::AsyncDurable { group_window_ms: 8 },
)?;
let persist_options = PersistOptions {
    commit_mode: CommitMode::AsyncDurable { group_window_ms: 8 },
    snapshot_every_n_events: 200,
    snapshot_every_bytes: 4 * 1024 * 1024,
    snapshot_every_ms: 60_000,
    compression: true,
};
// Convert transactions into PersistedEvent values, then call store.append_batch(events).await?
```

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

## 6. Collaboration server

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

## 7. Suggested project layout

```
your-project/
├── .cursorrules
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── runtime/
│   ├── plugins/
│   └── services/
├── tests/
└── README.md
```

## 8. Common commands

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

Workspace tooling:

```bash
# Tauri + Vue demo
cargo run --manifest-path ../moduforge-rs/examples/demo/Cargo.toml

# Documentation site
cd ../moduforge-rs/packages/docs && pnpm install && pnpm dev
```

## 9. Further reading

1. [architecture-overview.md](../architecture-overview.md) — layered architecture summary.
2. [plugin-development-guide.md](../plugin-development-guide.md) — advanced plugin patterns.
3. [example-integration-project.md](../example-integration-project.md) — complete skeleton with persistence/search/collaboration.
4. `packages/docs` — VitePress site that can be reused as a project knowledge base.

Following these steps you can confidently integrate ModuForge-RS into your own project.
