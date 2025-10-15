# ModuForge-RS Use Case Guide

This page highlights common scenarios and shows how ModuForge-RS modules can be combined to deliver them.

## 1. Document or spreadsheet editors
- Use `moduforge-model` to define node/mark types and enforce structure with `Schema`.
- Apply mutations through `moduforge-transform` Steps such as `AddNodeStep`, `AttrStep`, and `MarkStep`.
- Enable history via `HistoryManager`; persist snapshots to `moduforge-file` when needed.
- Add multi-user collaboration through `moduforge-collaboration` and the `moduforge-collaboration-client` front-end package.

## 2. Configuration and parameter hubs
- Manage immutable versions with `State`; branch/merge via transactions.
- Share service discovery or permission data through the resource table.
- Feed transaction deltas into `moduforge-search` to build tag or text-based filters.

## 3. Offline-first workstations
- `moduforge-file` supplies append-only files and history frame codecs for offline editing.
- When reconnected, replay local transactions and persist them to `SqliteEventStore` or another backend.
- Choose `CommitMode::MemoryOnly` for demos, `AsyncDurable` for desktop, `SyncDurable` for critical data.

## 4. External rules or workflow integration (optional)
- Encapsulate external rule engines or workflow systems inside plugins; write results back into transaction metadata.
- For latency-sensitive validations, implement the logic directly inside the plugin layer.

## 5. Data analytics and reporting
- Stream incremental Step information to indexing or OLAP systems through `IndexEvent` values.
- Use batch Steps to reshape the node tree, then export snapshots with `moduforge-file` for offline processing.

## Implementation checklist
- [ ] Finalise your document/data model and prepare node initialisation scripts.
- [ ] Wrap business logic in plugins instead of touching runtime internals directly.
- [ ] Select runtimes and persistence strategies that match the deployment environment.
- [ ] Plan directories, ports, and index locations early when enabling collaboration or search.
- [ ] Run benchmarks to quantify the impact of custom Steps or plugins.
