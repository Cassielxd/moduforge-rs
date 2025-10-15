# ModuForge-RS 外部项目接入指南

本指南说明如何把当前工作区嵌入到现有项目。从 `.cursorrules`、依赖配置，到运行时、插件、持久化、协作都给出示例。

## 1. 准备 `.cursorrules`
根据需要复制仓库根目录下的规则模板：
- `.cursorrules-for-external-projects`：提供完整的提交与目录约束说明。
- `.cursorrules-external-simple`：仅保留常用命令和依赖提示。

```bash
cp path/to/moduforge-rs/.cursorrules-for-external-projects ./your-project/.cursorrules
# 或
cp path/to/moduforge-rs/.cursorrules-external-simple ./your-project/.cursorrules
```

## 2. 配置依赖

```toml
[dependencies]
moduforge-core = { path = "../moduforge-rs/crates/core" }
moduforge-model = { path = "../moduforge-rs/crates/model" }
moduforge-state = { path = "../moduforge-rs/crates/state" }
moduforge-transform = { path = "../moduforge-rs/crates/transform" }

# 可选配套能力
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

## 3. 初始化运行时
与快速入门示例类似，可以在项目入口处封装一个 `runtime` 模块集中构建和托管运行时：
```rust
let mut options = RuntimeOptions::default()
    .set_content(Content::NodePoolFn(Arc::new(DefaultPool)));
for ext in extensions() {
    options = options.add_extension(ext);
}
let runtime = ForgeAsyncRuntime::create_with_config(options, ForgeConfig::default()).await?;
```

## 4. 插件与扩展示例
```rust
use mf_core::{Extension, ForgeResult};
use mf_core::types::{Content, Extensions, NodePoolFnTrait, RuntimeOptions};
use mf_model::{node_pool::NodePool, Attrs, Node, NodeType};
use mf_state::{plugin::Plugin, State, init_logging};
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

fn extensions() -> Vec<Extensions> {
    vec![Extensions::E(Extension::new_plugin(
        "audit-plugin",
        Box::new(AuditPlugin::default()),
    ))]
}
```

## 5. 持久化 / 检索 / 协作
- 事件存储：`SqliteEventStore::open(path, CommitMode)` 配合 `PersistOptions` 设置快照与压缩策略。
- 全文检索：`create_tantivy_service` 创建服务端，结合 `IndexEvent::{StepApplied, TransactionCommitted, Rebuild}` 同步索引。
- 实时协作：`CollaborationServer::new(Arc::new(YrsManager::new()), port).start().await` 即可启动 WebSocket 和健康检查接口。

## 6. 推荐命令
```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

## 7. 常见问题
- 依赖冲突：执行 `cargo update` 并检查 `cargo tree -d`。
- 运行时异常：确认 `init_logging` 已初始化日志，并根据需要调整 `ForgeConfig`。
- 协作/检索异常：检查端口和目录是否可用，确保已加入所需 crate。

## 8. 后续步骤
1. 阅读 `architecture-overview.md` 了解整体架构。
2. 参考 `plugin-development-guide.md` 编写业务插件。
3. 查看 `example-integration-project.md` 获取完整骨架示例。
4. 如需团队知识库，可直接复用 `packages/docs`（VitePress）。
