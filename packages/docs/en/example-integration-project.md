# ModuForge-RS Integration Example

This walkthrough shows how to embed the workspace in an external project, wiring together the runtime, plugins, persistence, search, and collaboration layers. All references match crates that ship with the repository.

## 1. Suggested layout

```
example-project/
├── Cargo.toml
├── .cursorrules
├── src/
│   ├── main.rs
│   ├── runtime/
│   │   ├── mod.rs
│   │   └── plugins.rs
│   ├── schema/
│   └── services/
├── tests/
└── README.md
```

## 2. Dependencies

Reuse the Cargo.toml snippet from the quick start and point path dependencies to the cloned workspace. When the crates are published you can switch to crates.io versions.

## 3. Runtime builder (`src/runtime/mod.rs`)

```rust
use anyhow::Result;
use mf_core::{ForgeAsyncRuntime, ForgeConfig};
use mf_core::types::{Content, NodePoolFnTrait, RuntimeOptions};
use mf_model::node_pool::NodePool;
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

pub async fn build_runtime() -> Result<ForgeAsyncRuntime> {
    let config = ForgeConfig::default();
    let options = RuntimeOptions::default()
        .set_content(Content::NodePoolFn(Arc::new(DefaultPool)));

    Ok(ForgeAsyncRuntime::create_with_config(options, config).await?)
}
```

## 4. Plugin registration (`src/runtime/plugins.rs`)

```rust
use mf_core::{Extension, ForgeResult};
use mf_core::types::Extensions;
use mf_state::{plugin::Plugin, State};

#[derive(Default)]
pub struct AuditPlugin;

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

pub fn builtin_extensions() -> Vec<Extensions> {
    vec![Extensions::E(Extension::new_plugin(
        "audit-plugin",
        Box::new(AuditPlugin::default()),
    ))]
}
```

Hook the extensions into the runtime:

```rust
let mut options = RuntimeOptions::default()
    .set_content(Content::NodePoolFn(Arc::new(DefaultPool)));
for extension in builtin_extensions() {
    options = options.add_extension(extension);
}
```

## 5. Persistence and search

```rust
use anyhow::Result;
use mf_persistence::{
    api::{CommitMode, PersistOptions},
    sqlite::SqliteEventStore,
};
use std::sync::Arc;

pub fn create_event_store() -> Result<Arc<SqliteEventStore>> {
    let store = SqliteEventStore::open(
        "./data/app.db",
        CommitMode::AsyncDurable { group_window_ms: 8 },
    )?;
    let _options = PersistOptions {
        commit_mode: CommitMode::AsyncDurable { group_window_ms: 8 },
        snapshot_every_n_events: 200,
        snapshot_every_bytes: 4 * 1024 * 1024,
        snapshot_every_ms: 60_000,
        compression: true,
    };
    Ok(store)
}
```

```rust
use mf_search::{create_tantivy_service, IndexEvent, RebuildScope};
use std::{path::PathBuf, sync::Arc};

pub async fn rebuild_index(pool: Arc<mf_model::node_pool::NodePool>) -> anyhow::Result<()> {
    let dir = PathBuf::from("./data/index");
    let service = create_tantivy_service(&dir)?;
    service
        .handle(IndexEvent::Rebuild { pool, scope: RebuildScope::Full })
        .await
}
```

> Production systems typically implement the persistence orchestrator to convert transactions into `PersistedEvent` values before calling the event store.

## 6. Collaboration service

```rust
use anyhow::Result;
use mf_collab::{CollaborationServer, YrsManager};
use std::sync::Arc;

pub async fn start_collaboration(port: u16) -> Result<()> {
    CollaborationServer::new(Arc::new(YrsManager::new()), port)
        .start()
        .await;
    Ok(())
}
```

## 7. Automated tests

```rust
#[tokio::test]
async fn runtime_boots() {
    let runtime = build_runtime().await.expect("runtime boot");
    assert_eq!(runtime.get_state().doc().size(), 0);
}
```

## 8. Troubleshooting

- **Dependency conflicts**: keep `Cargo.lock` aligned with the workspace, run `cargo update`, inspect `cargo tree -d`.
- **Runtime errors**: ensure logging is initialised via `init_logging`, double-check plugin hooks, review `ForgeConfig`.
- **Search or collaboration issues**: verify directories are writable and ports are available; enable optional crates as needed.

## 9. Advanced tips

- Switch between `ForgeAsyncRuntime` and `ForgeActorRuntime` depending on workload.
- Use `moduforge-file` history codecs to support offline replay or snapshot export.
- Run `tools/benchmark-coordinator` to track performance regressions.

With these pieces in place you can reproduce the workspace runtime inside your own project and extend it with persistence, search, and collaboration features.
