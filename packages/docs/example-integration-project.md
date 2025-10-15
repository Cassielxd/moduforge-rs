# ModuForge-RS 集成示例

本示例展示如何在独立项目中串联运行时、插件、持久化、检索与协作能力，帮助你快速搭建与工作区一致的脚手架。

## 1. 目录结构建议
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

## 2. 依赖配置
按快速入门章节添加 `path` 依赖即可。

## 3. 运行时构建 (`src/runtime/mod.rs`)
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
    let mut options = RuntimeOptions::default()
        .set_content(Content::NodePoolFn(Arc::new(DefaultPool)));
    for ext in crate::runtime::plugins::extensions() {
        options = options.add_extension(ext);
    }
    Ok(ForgeAsyncRuntime::create_with_config(options, ForgeConfig::default()).await?)
}
```

## 4. 插件注册 (`src/runtime/plugins.rs`)
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

pub fn extensions() -> Vec<Extensions> {
    vec![Extensions::E(Extension::new_plugin(
        "audit-plugin",
        Box::new(AuditPlugin::default()),
    ))]
}
```

## 5. 持久化与全文检索
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

## 6. 协作服务
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

## 7. 自动化测试
```rust
#[tokio::test]
async fn runtime_boots() {
    let runtime = build_runtime().await.expect("runtime boot");
    assert_eq!(runtime.get_state().doc().size(), 0);
}
```

## 8. 常见问题
- 依赖冲突：保持 `Cargo.lock` 与工作区一致，执行 `cargo update` 并查看 `cargo tree -d`。
- 运行时报错：确认 `init_logging` 已调用，插件实现未 panic，必要时调整 `ForgeConfig`。
- 检索/协作异常：检查索引目录权限、端口占用情况，并确保可选 crate 已加入依赖。

## 9. 进阶建议
- 根据负载切换 `ForgeAsyncRuntime` 与 `ForgeActorRuntime`。
- 结合 `moduforge-file` 的历史帧编解码实现离线回放或快照导出。
- 使用 `tools/benchmark-coordinator` 统一执行性能基准。
