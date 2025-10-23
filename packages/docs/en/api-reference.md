# ModuForge-RS API Reference

This page summarises the most frequently used entry points in each crate. All symbols exist in the current workspace; consult the corresponding README or source files for full details.

## 📦 Core runtime (`moduforge-core`)

- `ForgeRuntime::create(options)` / `ForgeAsyncRuntime::create(options)` — start sync or async runtimes.
- `ForgeRuntime::create_with_config(options, config)` — boot with a custom `ForgeConfig`.
- `ForgeRuntime::dispatch(transaction)` / `dispatch_with_meta` — submit a transaction of Steps.
- `ForgeRuntime::command(command)` — trigger extension commands.
- `HistoryManager::undo()` / `redo()` — walk the history stack.

```rust
use mf_core::{ForgeAsyncRuntime, ForgeConfig};
use mf_core::types::{Content, RuntimeOptions};

let options = RuntimeOptions::default().set_content(Content::None);
let runtime = ForgeAsyncRuntime::create_with_config(options, ForgeConfig::default()).await?;
```

## 🏪 State management (`moduforge-state`)

- `State::create(config)` — initialise the immutable state container.
- `State::tr()` — open a transaction bound to the state.
- `Transaction::add_step(step)` / `commit()` — assemble and apply changes.
- `init_logging(level, file_path)` — bootstrap `tracing`-based logging.
- `ResourceTable` / `Plugin` trait — register business resources and plugins.

```rust
use mf_state::{State, StateConfig};
let state = State::create(StateConfig::default()).await?;
let mut tr = state.tr();
```

## 🔄 Transactions (`moduforge-transform`)

- `AddNodeStep::new_single(node, parent)` — insert a node.
- `AttrStep::new(node_id, key, value)` — mutate attributes.
- `MarkStep::add_mark(node_id, mark)` — manage marks.
- `BatchStep::new(steps)` — batch steps for efficiency.
- `transform_error(msg)` — raise a unified transformation error.

```rust
use mf_transform::node_step::AddNodeStep;
transaction.add_step(Box::new(AddNodeStep::new_single(node, Some(parent_id))));
```

## 🧬 Data model (`moduforge-model`)

- `Node::new(id, node_type, attrs, content)` — build nodes.
- `Mark::new(mark_type, attrs)` — build marks.
- `Attrs::set(key, value)` / `Attrs::merge()` — manipulate attribute sets.
- `Schema::new(nodes, marks)` — define structural constraints.
- `NodePool::default()` / `NodePool::from_tree()` — manage pooled nodes.

```rust
use mf_model::{Attrs, Node, NodeType};
let mut attrs = Attrs::new();
attrs.set("title", "Example");
let node = Node::new("doc".into(), NodeType::block("document"), attrs, None);
```

## 📁 File format (`moduforge-file`)

- `DocumentWriter::begin(path)` → `add_segment(kind, bytes)` → `finalize()` — append-only storage.
- `DocumentReader::open(path)` → `read_segments(kind, callback)` — iterate and validate segments.
- `ZipDocumentWriter` / `ZipDocumentReader` — compress or extract grouped payloads.
- `encode_history_frames` / `decode_history_frames` — persist history checkpoints.

## 💾 Event storage (`moduforge-persistence`)

- `SqliteEventStore::open(path, CommitMode)` — bring up a SQLite WAL event store.
- `EventStore::append(events)` / `append_batch(events)` — write events.
- `EventStore::latest_snapshot()` / `write_snapshot()` / `compact()` — manage snapshots and compaction.
- `PersistOptions`, `CommitMode` — tune durability, snapshot cadence, and compression.

```rust
use mf_persistence::{api::CommitMode, sqlite::SqliteEventStore};
let store = SqliteEventStore::open("./data/app.db", CommitMode::AsyncDurable { group_window_ms: 8 })?;
```

## 🔍 Search (`moduforge-search`)

- `ensure_default_step_indexers()` — register built-in Step indexers.
- `create_tantivy_service(index_dir)` — create a Tantivy-backed service.
- `IndexService::handle(IndexEvent)` — apply Step/Transaction deltas or rebuild indices.
- `IndexEvent::{StepApplied, TransactionCommitted, Rebuild}` — describe indexing intents.

## 🤝 Collaboration (`moduforge-collaboration`)

- `CollaborationServer::new(yrs_manager, port)` — configure WebSocket + HTTP endpoints.
- `CollaborationServer::start()` — launch the server.
- `SyncService::get_room_status()` / `get_room_info()` — inspect rooms.
- `YrsManager::get_or_create_awareness()` — manage CRDT awareness state.

```rust
use mf_collab::{CollaborationServer, YrsManager};
let server = CollaborationServer::new(Arc::new(YrsManager::new()), 3030);
server.start().await;
```

## 👥 Collaboration client (`moduforge-collaboration-client`)

- `client::CollaborationClient::connect(url)` — open a WebSocket session.
- `mapping_v2` module — map runtime transactions to CRDT patches and back.
- `AwarenessRef` — share Yrs awareness across tasks safely.

## 🧰 Macro crates

- `moduforge-macros` offers `mf_extension!`, `mf_plugin!`, `#[impl_command]`, `mf_ops!`, `mf_global_attr!`, and related helpers.
- `moduforge-macros-derive` exposes `#[derive(Node)]`, `#[derive(Mark)]`, and `#[derive(PState)]` for declarative modelling.

```rust
use mf_derive::Node;

#[derive(Node)]
#[node_type = "article"]
struct Article {
    #[attr]
    title: String,
}
```

## 📚 Further reading

- Each crate ships with a README, examples, or tests that describe advanced scenarios.
- The `examples/` directory and `packages/docs` site include end-to-end demos that showcase integration patterns.
