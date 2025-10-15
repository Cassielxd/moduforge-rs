# ModuForge-RS 场景指南

本文列举几个常见业务场景，并说明如何围绕 ModuForge-RS 组合各个模块。

## 1. 文档/表格编辑器
- 利用 `moduforge-model` 定义节点/标记结构，配合 `Schema` 保证文档合法性。
- `moduforge-transform` 的 Step 体系用于插入、更新、移动节点。
- 历史与撤销：开启 `HistoryManager`，必要时将快照写入 `moduforge-file`。
- 多用户协作：接入 `moduforge-collaboration`，前端结合 `moduforge-collaboration-client`。

## 2. 配置与参数中心
- 使用 `State` 管理不可变版本，事务用于分支/合并。
- 通过资源表共享服务发现、权限等配置信息。
- 借助 `moduforge-search` 构建全文检索或标签过滤。

## 3. 离线优先工作台
- `moduforge-file` 提供追加式文件与历史帧编码，支持断网编辑。
- 在线时回放本地事务并持久化到 `SqliteEventStore` 或其他后端。
- 根据场景选择 `CommitMode::MemoryOnly`（演示）、`AsyncDurable`（桌面）、`SyncDurable`（关键数据）。

## 4. 外部规则与流程协同（可选）
- 可在插件中封装外部规则服务或工作流引擎，将执行结果写回事务 meta。
-对实时性要求较高的校验可以直接在 Plugin 中实现。

## 5. 数据分析与报表
- Step 中增量信息可通过 `IndexEvent` 投射到索引或 OLAP 系统。
- 使用 `moduforge-transform` 的批量 Step 整理节点树，再导出到 `moduforge-file` 供离线处理。

## 实施建议清单
- [ ] 明确文档/数据模型，并准备节点初始化脚本。
- [ ] 为业务逻辑抽象出插件，避免直接操作运行时内部。
- [ ] 根据部署形态选取合适的运行时和持久化策略。
- [ ] 需要协作或检索时，提前规划目录、端口、索引位置。
- [ ] 使用基准测试验证 Step 或插件对性能的影响。
