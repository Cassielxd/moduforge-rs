# ModuForge-RS

[Read this in Chinese](../index.md)

ModuForge-RS is a Rust workspace that delivers an immutable data runtime, transactional pipelines, and collaboration-ready infrastructure. The documentation now reflects the current repository and no longer references the legacy rules engine packages.

## Layered architecture

- **model** – immutable node tree, marks, attributes, schema validation, and node pool utilities.
- **state** – plugin container, resource tables, transaction orchestration, and logging bootstrap.
- **transform** – step/transaction abstractions with node, mark, attribute, and batch operations plus patch synthesis.
- **core** – runtime orchestration, event bus, extension manager, history, metrics, async/actor runtimes, and configuration helpers.
- **file** – append-only file format with history frame codecs, zip import/export, CRC/Blake3 validation.
- **persistence** – durable event store and snapshot APIs with a SQLite implementation.
- **search** – Tantivy-based indexing service and step registry.
- **collaboration / collaboration-client** – Yrs (CRDT) collaboration server and client utilities.
- **macro / derive** – declarative macros and derive macros for nodes, marks, and plugins.

## Repository structure

```
moduforge-rs/
├── crates/
│   ├── core/                 # Runtime, events, extensions, history, metrics
│   ├── model/                # Immutable model layer
│   ├── state/                # Plugin lifecycle and transaction scheduling
│   ├── transform/            # Step + transaction engine
│   ├── file/                 # File format & history codecs
│   ├── persistence/          # Event store + snapshot implementation
│   ├── search/               # Tantivy indexing service
│   ├── collaboration/        # Warp + Yrs collaboration server
│   ├── collaboration_client/ # Client tooling for CRDT sync
│   ├── macro/                # Declarative macro helpers
│   └── derive/               # #[derive(Node/Mark/PState)] macros
├── examples/                 # Tauri + Vue editor, snapshot demo, etc.
├── packages/
│   ├── docs/                 # VitePress documentation site
│   └── collaboration-client/ # Front-end collaboration SDK
├── schema/                   # XML schema & samples
├── tools/benchmark-coordinator/
├── Cargo.toml
└── rustfmt.toml
```

## Highlights

- **Multiple runtimes** – sync, async, and actor execution with adaptive system diagnostics.
- **Events & history** – undo/redo, snapshots, and incremental history codecs out of the box.
- **Extension ecosystem** – middleware, metrics, and extension manager for packaging domain logic.
- **Persistence** – SQLite WAL storage with configurable durability and compaction.
- **Search** – push transaction deltas to Tantivy via the indexing service.
- **Collaboration** – Warp-based WebSocket server with CRDT rooms and health endpoints.
- **Macro tooling** – declarative APIs to author nodes, marks, commands, and plugins quickly.

## Getting started

1. Follow the [Quick Start](./quick-start.md) to spin up a minimal runtime.
2. Inspect `crates/core/README.md` and `crates/state/README.md` for runtime lifecycle details.
3. Explore module-specific READMEs for persistence, search, or collaboration needs.
4. Use the VitePress site in `packages/docs` as the team knowledge base.

## Contributing

- Run `cargo fmt`, `cargo clippy --workspace --all-targets`, and `cargo test --workspace` before opening a PR.
- Use `tools/benchmark-coordinator` to execute and compare Criterion benchmarks when performance changes.
- Keep Chinese and English documentation in sync and update navigation inside `.vitepress/config.ts` when new pages are added.

The project is released under the MIT license—feel free to adapt the runtime to your domain.
