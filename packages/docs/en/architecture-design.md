# ModuForge-RS Architecture Design

This document outlines the design goals, layering principles, and recommended practices for ModuForge-RS. Unlike older revisions, it only covers modules that remain in the current workspace.

## Design goals
1. **Immutable data**: rely on `imbl` persistent structures to guarantee thread safety and history replay.
2. **Transaction driven**: capture every mutation as Steps inside Transactions so changes can be undone.
3. **Extension friendly**: manage extensions, plugins, and resources through `moduforge-core` uniformly.
4. **Adaptive runtimes**: offer sync, async, and actor execution so deployments can pick the right model.
5. **Complete ecosystem**: ship file formats, persistence, search, collaboration, and macro tooling out of the box.

## Layered structure
- **Interface / plugin layer**: domain plugins, commands, and extensions encapsulate business rules.
- **Runtime layer**: `moduforge-core` handles events, history, middleware, and resource probes.
- **State layer**: `moduforge-state` manages immutable state and the plugin lifecycle.
- **Transaction layer**: `moduforge-transform` defines Steps and enforces transactional consistency.
- **Model layer**: `moduforge-model` exposes nodes, marks, attributes, and schema helpers.
- **Collaboration / storage layer**: `moduforge-collaboration`, `moduforge-file`, `moduforge-persistence`, `moduforge-search`.

```
UI / Plugins ──> Runtime (core) ──> State (state) ──> Transactions (transform) ──> Model (model)
                                   │            │
                                   └──────┬─────┘
                                          │
                Collaboration (collaboration) / Storage (file + persistence) / Search (search)
```

## Runtime and task model
- `ForgeRuntime`: synchronous execution for desktop tools or scripts.
- `ForgeAsyncRuntime`: Tokio-based runtime for high-concurrency workloads and async plugins.
- `ForgeActorRuntime`: actor mode built on `ractor`, ideal for isolating many concurrent tasks.
- `ForgeRuntimeBuilder`: unified entry to construct runtimes and auto-select configurations based on system resources.

**Recommendations**
- Expose a thin façade that owns the runtime lifecycle, extension registration, and resource injection.
- Host plugin initialisation, indexing services, and collaboration services on the Tokio runtime used by the application.

## Plugins and resources
- Implement `mf_state::plugin::Plugin` to listen to transactions, validate state, or emit logs.
- `ResourceTable` lets plugins share read/write resources safely.
- Call `init_logging(level, path)` once in `main` to bootstrap `tracing`.

**Example**
```rust
pub fn register_extensions(options: RuntimeOptions) -> RuntimeOptions {
    options.add_extension(Extensions::E(Extension::new_plugin(
        "audit-plugin",
        Box::new(AuditPlugin::default()),
    )))
    // Add more extensions here as your domain requires.
}
```

## Persistence strategy
- `SqliteEventStore::open(path, CommitMode)` offers the default event store.
- `CommitMode` options: `MemoryOnly`, `AsyncDurable` (recommended for desktop/dev), `SyncDurable`.
- `PersistOptions` can trigger snapshots based on event count, byte size, or elapsed time.

> Production systems typically implement the `Persistence` trait to push transaction metadata through a single pipeline before hitting the store.

## Full-text indexing flow
1. Call `ensure_default_step_indexers()` when initialising `moduforge-search`.
2. After a transaction commits, send `IndexEvent::TransactionCommitted` with the incremental payload.
3. When a rebuild is required, emit `IndexEvent::Rebuild { scope: RebuildScope::Full }`.

## Collaboration and front-end integration
- `CollaborationServer::start()` exposes `/collaboration/{room}` WebSocket endpoints plus health probes.
- `moduforge-collaboration-client` maps runtime transactions to Yrs operations and can be reused in desktop or web front-ends.
- Integrate persistence/snapshot logic with room management to support reconnection and recovery.

## Supporting tools
- `moduforge-file`: history frame codecs, zip packaging, custom format strategies.
- `tools/benchmark-coordinator`: orchestrate Criterion runs and produce JSON/HTML reports.
- `packages/docs`: VitePress documentation site that can act as a team knowledge base template.

## Deployment suggestions
- **Desktop / single host**: `ForgeAsyncRuntime` + SQLite event store + local index.
- **Server-side**: actor runtime plus background tasks, deploy collaboration services on dedicated Tokio runtimes.
- **Offline first**: pair `moduforge-file` snapshots with event replay to resynchronise when connectivity returns.

## Practice checklist
- [ ] Confirm schema definitions and node pool initialisation.
- [ ] Register required extensions and middleware through `RuntimeOptions`.
- [ ] Configure logging, metrics, and history retention policies.
- [ ] Integrate persistence, search, and collaboration modules as needed.
- [ ] Add integration and benchmark tests for critical functionality.

Following these steps ensures you can build a reliable business runtime platform that stays consistent and extendable.
