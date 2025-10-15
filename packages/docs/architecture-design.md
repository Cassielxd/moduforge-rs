# ModuForge-RS 架构设计

本文概述 ModuForge-RS 的设计目标、分层原则以及在真实项目中的推荐实践。与旧版本不同，以下内容仅讨论当前工作区保留的模块。

## 设计目标
1. **不可变数据**：使用 `imbl` 持久化结构保障多线程与回放安全。
2. **事务驱动**：所有状态变更以 Step/Transaction 形式记录并可撤销。
3. **扩展友好**：扩展、插件、资源统一通过 `moduforge-core` 管理。
4. **运行时自适应**：同步、异步、Actor 三种执行方式按场景选择。
5. **生态配套**：文件格式、事件存储、检索、协作与宏系统开箱即用。

## 分层结构
- **界面/插件层**：业务插件、命令、扩展负责封装业务规则。
- **运行时层**：`moduforge-core` 提供事件、历史、中间件、资源探测。
- **状态层**：`moduforge-state` 管理不可变状态和插件生命周期。
- **事务层**：`moduforge-transform` 定义 Step 并保证事务一致性。
- **模型层**：`moduforge-model` 提供节点、标记、属性、Schema。
- **协作/存储层**：`moduforge-collaboration`、`moduforge-file`、`moduforge-persistence`、`moduforge-search`。

```
用户界面 / 插件  ──>  运行时 (core)  ──>  状态 (state)  ──>  事务 (transform)  ──>  模型 (model)
                                       │             │
                                       └─────┬───────┘
                                             │
                        协作 (collaboration) / 存储 (file + persistence) / 检索 (search)
```

## 运行时与任务模型
- `ForgeRuntime`：同步运行，适合桌面、脚本化场景。
- `ForgeAsyncRuntime`：基于 Tokio 的高并发运行时，支持异步插件与事务流。
- `ForgeActorRuntime`：在 `ractor` 之上构建的 Actor 模式，适合大量并发任务和隔离需求。
- `ForgeRuntimeBuilder`：统一的运行时构建入口，可根据系统资源自动选择。

**推荐做法**：
- 对外暴露一个封装层，集中管理 runtime 的生命周期、扩展注册和资源注入。
- 将插件初始化、索引服务、协作服务等异步任务放到 Tokio runtime 中统一托管。

## 插件与资源
- 插件需实现 `mf_state::plugin::Plugin`，可监听事务、执行校验、写入日志。
- `ResourceTable` 用于跨插件共享只读/可写资源。
- `init_logging(level, path)` 建议在 main 中调用一次，用于初始化 `tracing`。

**示例**：
```rust
pub fn register_extensions(options: RuntimeOptions) -> RuntimeOptions {
    options.add_extension(Extensions::E(Extension::new_plugin(
        "audit-plugin",
        Box::new(AuditPlugin::default()),
    )))
    // 其余业务插件可按此方式继续注册。
}
```

## 持久化策略
- `SqliteEventStore::open(path, CommitMode)`：默认事件存储。
- `CommitMode` 支持 MemoryOnly、AsyncDurable（推荐桌面/开发环境）、SyncDurable。
- `PersistOptions` 可按事件数/字节数/时间间隔触发快照。

> 在业务项目中可实现 `Persistence` trait，将事务 meta 与事件存储整合到统一管线。

## 全文检索流程
1. 在 `moduforge-search` 初始化时调用 `ensure_default_step_indexers()`。
2. 事务提交后通过 `IndexEvent::TransactionCommitted` 推送增量。
3. 需要全量重建时触发 `IndexEvent::Rebuild { scope: RebuildScope::Full }`。

## 协作与前端集成
- `CollaborationServer::start()` 提供 `/collaboration/{room}` WebSocket 与健康检查等 HTTP 接口。
- `moduforge-collaboration-client` 封装了事务 <-> Yrs 操作的映射，可在桌面或 Web 前端复用。
- 建议在协作房间管理中集成持久化/快照逻辑，实现断线重连和数据恢复。

## 配套工具
- `moduforge-file`：历史帧编码、Zip 打包、自定义格式策略。
- `tools/benchmark-coordinator`：批量运行 Criterion，输出 JSON/HTML 报告。
- `packages/docs`：VitePress 文档站点，可作为团队知识库模板。

## 典型部署建议
- **桌面/单机**：`ForgeAsyncRuntime` + SQLite 事件存储 + 本地索引。
- **服务端**：Actor 运行时 + 后台异步任务 + 协作服务部署在独立 Tokio runtime 上。
- **离线优先**：结合 `moduforge-file` 做快照，在线后回放事件并触发索引/协作同步。

## 实践清单
- [ ] 确认 Schema 与节点池初始化逻辑。
- [ ] 使用 `RuntimeOptions` 注册必要扩展与中间件。
- [ ] 配置日志、指标、历史保留策略。
- [ ] 按需接入持久化、搜索、协作模块。
- [ ] 为关键功能编写集成测试和基准测试。

通过以上步骤可以在保证一致性与可扩展性的前提下，构建可靠的业务运行时平台。
