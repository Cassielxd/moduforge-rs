# ModuForge-RS API 参考

本页汇总工作区内主要 crate 的常用入口及示例代码。所有 API 均来自当前仓库，更多细节请查阅对应 crate 的 README 或源码。

## 📦 核心运行时（moduforge-core）

常用方法：
- `ForgeRuntime::create(options)` / `ForgeAsyncRuntime::create(options)` —— 创建同步或异步运行时。
- `ForgeRuntime::create_with_config(options, config)` —— 使用自定义 `ForgeConfig` 启动运行时。
- `ForgeRuntime::dispatch(transaction)` / `dispatch_with_meta` —— 以事务方式提交 Step 列表。
- `ForgeRuntime::command(command)` —— 调用扩展命令。
- `HistoryManager::undo()` / `redo()` —— 撤销或重做最近的事务。

```rust
use mf_core::{ForgeAsyncRuntime, ForgeConfig};
use mf_core::types::{Content, RuntimeOptions};

let options = RuntimeOptions::default().set_content(Content::None);
let runtime = ForgeAsyncRuntime::create_with_config(options, ForgeConfig::default()).await?;
```

## 🏪 状态管理（moduforge-state）

常用方法：
- `State::create(config)` —— 初始化不可变状态容器。
- `State::tr()` —— 打开与状态绑定的事务对象。
- `Transaction::add_step(step)` / `commit()` —— 组装并提交事务。
- `init_logging(level, file_path)` —— 初始化基于 `tracing` 的日志。
- `ResourceTable` / `Plugin` trait —— 注册业务资源与插件。

```rust
use mf_state::{State, StateConfig};
let state = State::create(StateConfig::default()).await?;
let mut tr = state.tr();
```

## 🔄 事务步骤（moduforge-transform）

常用方法：
- `AddNodeStep::new_single(node, parent)` —— 在事务中插入节点。
- `AttrStep::new(node_id, key, value)` —— 更新节点属性。
- `MarkStep::add_mark(node_id, mark)` —— 管理节点标记。
- `BatchStep::new(steps)` —— 批量执行多个 Step。
- `transform_error(msg)` —— 构造统一的事务错误类型。

```rust
use mf_transform::node_step::AddNodeStep;
transaction.add_step(Box::new(AddNodeStep::new_single(node, Some(parent_id))));
```

## 🧬 数据模型（moduforge-model）

常用方法：
- `Node::new(id, node_type, attrs, content)` —— 创建节点。
- `Mark::new(mark_type, attrs)` —— 创建标记。
- `Attrs::set(key, value)` / `Attrs::merge()` —— 操作属性集合。
- `Schema::new(nodes, marks)` —— 定义结构约束。
- `NodePool::default()` / `NodePool::from_tree()` —— 管理节点池。

```rust
use mf_model::{Attrs, Node, NodeType};
let mut attrs = Attrs::new();
attrs.set("title", "示例");
let node = Node::new("doc".into(), NodeType::block("document"), attrs, None);
```

## 📁 文件格式（moduforge-file）

常用方法：
- `DocumentWriter::begin(path)` → `add_segment(kind, bytes)` → `finalize()` —— 追加式写入文档。
- `DocumentReader::open(path)` → `read_segments(kind, callback)` —— 迭代读取并校验段。
- `ZipDocumentWriter` / `ZipDocumentReader` —— 压缩或解压多段内容。
- `encode_history_frames` / `decode_history_frames` —— 序列化历史快照。

## 💾 事件存储（moduforge-persistence）

常用方法：
- `SqliteEventStore::open(path, CommitMode)` —— 启动 SQLite WAL 事件存储。
- `EventStore::append(events)` / `append_batch(events)` —— 写入事件。
- `EventStore::latest_snapshot()` / `write_snapshot()` / `compact()` —— 管理快照与压缩。
- `PersistOptions`、`CommitMode` —— 配置持久化策略、快照节奏与压缩。

```rust
use mf_persistence::{api::CommitMode, sqlite::SqliteEventStore};
let store = SqliteEventStore::open("./data/app.db", CommitMode::AsyncDurable { group_window_ms: 8 })?;
```

## 🔍 全文检索（moduforge-search）

常用方法：
- `ensure_default_step_indexers()` —— 注册内置 Step 索引器。
- `create_tantivy_service(index_dir)` —— 创建基于 Tantivy 的索引服务。
- `IndexService::handle(IndexEvent)` —— 处理 Step/Transaction 增量或执行重建。
- `IndexEvent::{StepApplied, TransactionCommitted, Rebuild}` —— 表示不同索引意图。

## 🤝 协作系统（moduforge-collaboration）

常用方法：
- `CollaborationServer::new(yrs_manager, port)` —— 配置 WebSocket 与 HTTP 端点。
- `CollaborationServer::start()` —— 启动协作服务。
- `SyncService::get_room_status()` / `get_room_info()` —— 查询房间状态。
- `YrsManager::get_or_create_awareness()` —— 管理 CRDT Awareness。

```rust
use mf_collab::{CollaborationServer, YrsManager};
let server = CollaborationServer::new(Arc::new(YrsManager::new()), 3030);
server.start().await;
```

## 👥 协作客户端（moduforge-collaboration-client）

常用方法：
- `client::CollaborationClient::connect(url)` —— 建立 WebSocket 连接。
- `mapping_v2` 模块 —— 在事务与协作文档之间互相转换。
- `AwarenessRef` —— 安全地共享 Yrs awareness。

## 🧰 宏系统

常用方法：
- `moduforge-macros` 提供 `mf_extension!`、`mf_plugin!`、`#[impl_command]`、`mf_ops!`、`mf_global_attr!` 等宏。
- `moduforge-macros-derive` 提供 `#[derive(Node)]`、`#[derive(Mark)]`、`#[derive(PState)]` 等派生宏。

```rust
use mf_derive::Node;

#[derive(Node)]
#[node_type = "article"]
struct Article {
    #[attr]
    title: String,
}
```

## 📚 参考

- 每个 crate 均附带 README、示例或测试，可获取更完整的 API 说明。
- `examples/` 与 `packages/docs` 中的示例覆盖了集成场景的实践路径。
