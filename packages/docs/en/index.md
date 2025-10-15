---
layout: home

hero:
  name: "ModuForge-RS"
  text: "Modular Runtime Framework"
  tagline: "Rust-powered runtime covering immutable data, transactions, collaboration, search, and persistence"
  image:
    src: /logo.svg
    alt: ModuForge-RS
  actions:
    - theme: brand
      text: Quick Start
      link: /en/quick-start
    - theme: alt
      text: Plugin Guide
      link: /en/plugin-development-guide
    - theme: alt
      text: GitHub
      link: https://github.com/Cassielxd/moduforge-rs

features:
  - icon: 🏗️
    title: Layered Modules
    details: 11 core crates cover model, state, transform, runtime, persistence, search, and collaboration.
  - icon: 🚀
    title: Adaptive Runtimes
    details: Sync, async, and actor runtimes with built-in scheduling and resource detection.
  - icon: 🔧
    title: Plugin Ecosystem
    details: Unified extension and resource system with middleware, history, and metrics.
  - icon: 💾
    title: Persistence Ready
    details: SQLite event store with snapshots, compression, and tunable durability.
  - icon: 🔍
    title: Search Integration
    details: Tantivy + jieba incremental indexing driven directly from transactions.
  - icon: 🤝
    title: Real-time Collaboration
    details: Yrs (CRDT) plus Warp WebSocket for multi-client document rooms.
---

## What is ModuForge-RS?

ModuForge-RS is a Rust-based modular runtime for applications that manipulate large tree structures, require transactional consistency, and benefit from collaboration or search capabilities. The workspace is composed of `core`, `model`, `state`, `transform`, `file`, `persistence`, `search`, `collaboration`, `collaboration-client`, `macro`, and `derive` crates that can be composed as needed.

### Core capabilities

- 🏗️ **Modular architecture** – separated layers that can be developed and tested independently.
- 🚀 **Flexible runtimes** – choose between sync, async, and actor execution depending on workload.
- 🔧 **Plugin system** – extensions, middleware, and resource tables encapsulate business logic.
- 💾 **Event persistence** – WAL + snapshot pipeline with configurable durability settings.
- 🔍 **Full-text search** – stream transactions to Tantivy for near real-time indexing.
- 🤝 **Collaboration tooling** – CRDT document sync, room lifecycle, metrics, and health probes.
- 🧰 **Macros & derives** – declarative helpers to create nodes, marks, and plugins without boilerplate.

### How the pieces fit together

- **model**: immutable node tree, marks, attributes, schema, and content constraints.
- **state**: plugin lifecycle, transaction scheduling, resource management, and logging.
- **transform**: step/transaction pipeline ensuring atomic updates and replay semantics.
- **core**: runtime orchestration, event bus, extension manager, history, and metrics.
- **file / persistence / search / collaboration**: supporting crates for storage, indexing, and collaboration.
- **macro / derive**: declarative macros for authoring runtime integrations.

### When to use ModuForge

- Building desktop or web editors backed by immutable trees.
- Implementing systems that need undo/redo, snapshots, and reproducible history.
- Synchronising offline edits with event replay or CRDT merges.
- Adding collaboration or search to domain-specific runtimes.

Start with the [Quick Start](./quick-start.md) or dive into the [Architecture Overview](./architecture-overview.md) for deeper details.
