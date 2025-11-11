# ModuForge-RS

## 框架简介

ModuForge-RS 是一个围绕不可变树形数据模型构建的 Rust
工作区，用于快速搭建高并发、可协作、可扩展的业务运行时与编辑器内核。核心运行时通过事件驱动与事务管线衔接模型层（节点与标记）、状态层（插件与资源）、转换层（步骤与补丁），再结合可热插拔的扩展、历史管理、指标采集、异步与
Actor 运行时，形成一套完整的领域无关内核。围绕核心还提供持久化、全文检索、实时协作、文件格式、宏系统、基准测试与文档站点，帮助团队在保持严谨一致性的同时快速落地复杂业务场景。

## 框架特性

- 模块化运行时：`moduforge-core` 提供同步/异步/Actor 三套运行模式，内置事件总线、扩展管理、中间件与历史栈，支持按需组合。
- 不可变数据模型：`moduforge-model` 基于 `imbl` 提供持久化节点树、标记系统、属性、模式与内容匹配，保障并发一致性与结构化约束。
- 事务式变更流程：`moduforge-transform` 把所有改动拆解为可组合的 Step（节点、标记、属性、批量、补丁），确保原子提交、可回放与增量同步。
- 插件与资源系统：`moduforge-state` 管理插件生命周期、资源表、事务调度与日志输出，为自定义业务能力提供宿主环境。
- 自适应执行：运行时内置系统资源探测、调度器自适应、任务超时保护与度量指标，适配从离线桌面到服务器的不同部署环境。
- 历史与快照：历史管理器支持撤销/重做、分段快照；`moduforge-file` 与 `moduforge-persistence` 提供追加式文件、压缩打包、SQLite
  WAL、可调一致性的事件存储。
- 搜索与索引：`moduforge-search` 基于 Tantivy + jieba 分词构建增量索引服务，可通过 Step 注册机制把事务变更自动投递到索引。
- 实时协作：`moduforge-collaboration` 使用 Yrs (CRDT) 与 Warp WebSocket 暴露房间、健康检查、广播等服务；
  `moduforge-collaboration-client` 提供 Tokio/Tungstenite 客户端与 Awareness 映射，方便前端整合。
- 宏与派生：`moduforge-macros` 和 `moduforge-macros-derive` 给予声明式 API，快速定义节点、标记、插件、命令及其配置，并在编译期做一致性校验。
- 工具与示例：包含 Tauri + Vue 编辑器示例、协作前端包、基准测试协调器、XML Schema、VitePress 文档站点，覆盖落地到运维的完整配套。

## 架构总览

```text
                +------------------------------+
                |      应用/业务插件层         |
                +---------------+--------------+
                                |
                  +-------------v-------------+
                  |      moduforge-core       |
                  |  Runtime / Events / DI    |
                  +------+-----+------+-------+
                         |     |      |
            +------------+     |      +-------------+
            |                  |                    |
   +--------v-----+   +-------v------+     +-------v-------+
   | moduforge-   |   | moduforge-   |     | moduforge-    |
   |    model     |   |    state     |     |  transform    |
   | 数据模型层   |   | 状态与插件层 |     | 事务与补丁层  |
   +--------+-----+   +-------+------+     +-------+-------+
            |                 |                    |
            +-----------------+--------------------+
                                |
                    +-----------v-----------+
                    |   Persistence / File  |
                    |   Search / Collab     |
                    +-----------------------+
```

### 核心库

- `crates/model` (`moduforge-model`): 定义节点、标记、属性、Schema、内容验证、节点池与 ID 生成器，所有结构均基于不可变数据结构构建。
- `crates/state` (`moduforge-state`): 提供插件注册、资源生命周期、事务与日志体系，封装 State/Transaction API 以及
  `init_logging` 入口。
- `crates/transform` (`moduforge-transform`): 实现 Step 抽象、节点/标记/属性操作、批量事务与补丁机制，封装错误处理与
  `TransformResult` 类型。
- `crates/core` (`moduforge-core`): 汇聚运行时配置、事件总线、扩展管理、历史、度量与 Actor 系统（基于 `ractor`），同时导出
  `ForgeRuntimeBuilder`、`ForgeAsyncRuntime`、`ForgeActorRuntime`、`AdaptiveRuntimeSelector` 等入口。

### 支撑能力

- `crates/file` (`moduforge-file`): 附带 append-only 文件格式、CRC 校验、Blake3 哈希、Zip 封装与内存映射读取，支持增量历史帧编解码。
- `crates/persistence` (`moduforge-persistence`): 定义 `EventStore`/`Persistence` trait，内置 SQLite WAL
  实现，可配置提交模式（MemoryOnly、AsyncDurable、SyncDurable）、快照节奏与压缩策略。
- `crates/search` (`moduforge-search`): 以 Tantivy 为核心实现索引后端、索引服务、State 插件与 Step 注册器，支持目录化部署与临时索引。
- `crates/collaboration` (`moduforge-collaboration`): 提供 `YrsManager`、`SyncService`、`CollaborationServer`
  ，实现房间生命周期、WebSocket 广播、离线处理与健康检查。
- `crates/collaboration_client` (`moduforge-collaboration-client`): 面向客户端的连接、事务映射、Awareness
  状态同步工具，帮助前端/桌面应用复用 ModuForge 事务。
- `crates/macro` (`moduforge-macros`) 与 `crates/derive` (`moduforge-macros-derive`): 暴露 `mf_extension!`、`mf_plugin!`、
  `#[impl_command]` 等宏，以及 `#[derive(Node)]`、`#[derive(Mark)]`、`#[derive(PState)]` 等派生，减少大量模板代码。
- `schema/`: 存放 XML Schema (`moduforge-schema.xsd`) 与示例 `main.xml`，配合 `XmlSchemaParser` 把外部结构定义注入运行时。
- `examples/`: 提供 `demo` (Tauri + Vue 协作编辑器)、`demo2`、`snapshot_demo` 等完整应用示例。
- `tools/benchmark-coordinator`: 基准测试调度器，按层级/单 crate 运行 Criterion，并收集与对比结果。
- `packages/`: 包含协作前端包与 VitePress 文档（`packages/docs`），覆盖快速入门、节点映射、性能分析、故障排查等主题。

## 快速开始

### 环境要求

- Rust 1.70 及以上（建议使用最新稳定版，工作区 edition=2024）。
- 推荐安装 `just` 或 `cargo` 常规工具，需启用 `tokio` 全功能特性。
- 若要体验示例应用，还需 Node.js 18+、pnpm 或 npm，以及 Tauri 必要依赖。

### 构建与测试

```bash
# 拉取依赖后构建全工作区
cargo build --workspace

# 运行所有测试（包含协作、持久化等模块）
cargo test --workspace

# 针对核心模块运行基准测试
cargo bench -p moduforge-model

# 执行基准测试协调器示例
cargo run -p benchmark-coordinator -- run-all --parallel 2
```

### 最小运行时代码

#### 方式 1：最简单的用法（推荐）

```rust
use anyhow::Result;
use mf_core::ForgeRuntimeBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    // 自动检测系统资源，选择最优运行时
    let mut runtime = ForgeRuntimeBuilder::new()
        .build()
        .await?;

    // 获取当前状态
    let state = runtime.get_state().await?;
    println!("文档节点数: {}", state.doc().size());

    Ok(())
}
```

#### 方式 2：指定运行时类型

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType};
use mf_model::{Node, NodeType, Attrs};
use mf_transform::node_step::AddNodeStep;

#[tokio::main]
async fn main() -> Result<()> {
    // 明确使用 Async 运行时
    let mut runtime = ForgeRuntimeBuilder::new()
        .runtime_type(RuntimeType::Async)
        .build()
        .await?;

    // 构建文档与段落节点
    let doc = Node::new("doc".into(), NodeType::block("document"), Attrs::new(), None);
    let paragraph = Node::new(
        "p1".into(),
        NodeType::block("paragraph"),
        Attrs::new(),
        Some("Hello ModuForge".into()),
    );

    let mut tr = runtime.get_tr().await?;
    tr.add_step(Box::new(AddNodeStep::new_single(doc, None)));
    tr.add_step(Box::new(AddNodeStep::new_single(paragraph, Some("doc".into()))));
    tr.commit()?;
    runtime.dispatch(tr).await?;

    let state = runtime.get_state().await?;
    println!("当前节点数: {}", state.doc().size());
    Ok(())
}
```

#### 方式 3：完全自定义配置

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType, Environment};

#[tokio::main]
async fn main() -> Result<()> {
    // 生产环境配置
    let mut runtime = ForgeRuntimeBuilder::new()
        .runtime_type(RuntimeType::Actor)
        .environment(Environment::Production)
        .max_concurrent_tasks(20)
        .queue_size(5000)
        .enable_monitoring(true)
        .history_limit(1000)
        .build()
        .await?;

    println!("运行时类型: {:?}", runtime.runtime_type());
    Ok(())
}
```

### 声明式节点定义示例

```rust
use mf_derive::Node;

#[derive(Node)]
#[node_type = "project"]
#[marks = "strong highlight"]
struct ProjectNode {
    #[attr]
    name: String,
    #[attr]
    budget: f64,
}
```

以上代码由宏自动生成 `to_node()`，并在编译期校验属性配置，可直接注入运行时 Schema。

## 典型使用场景

- 复杂树形业务建模：如工程造价、财务科目、物料清单、知识树等，需要严谨层次结构、版本对比、撤销重做与批量变更。
- 富文档或配置编辑器：自定义节点/标记体系、插件式校验、增量补丁、可回放历史，适合搭建桌面/网页编辑器内核。
- 协同工作台：多端实时协同、权限控制、房间监控与断线恢复，结合 `moduforge-collaboration` 与前端客户端即可快速上线。
- 离线优先与同步：借助 append-only 文件格式与快照机制，在弱网或断网环境落地离线编辑，恢复后通过事务回放与 CRDT 合并。
- 搜索与分析：基于 `moduforge-search` 的增量索引，将运行时事务实时投射到全文检索或报表系统中，支持中文场景。

## 工具与生态

- 桌面示例：`examples/demo` 集成 Tauri、Vue、Yrs，演示协作编辑、插件注入、UI 交互。
- 快照演示：`examples/snapshot_demo` 展示历史帧生成、回放与文件格式互操作。
- 前端包：`packages/collaboration-client` 提供 TypeScript 客户端、示例与说明。
- 基准体系：`tools/benchmark-coordinator` 根据层级 orchestrate Criterion，输出 JSON/HTML 结果并支持回归检测。
- 文档站点：`packages/docs` 以 VitePress 构建，覆盖快速入门、节点映射、性能调优、协作排错等专题，可通过 `pnpm dev` 启动本地文档。

## 文档与进一步阅读

### deepwiki

#### https://deepwiki.com/Cassielxd/moduforge-rs

### 核心文档

- 核心模块文档：`crates/core/README.md`、`crates/state/README.md` 等子 README 详细拆解每个组件的 API 与设计。
- XML Schema 指南：`schema/README.md` 介绍如何通过 XSD/示例 XML 定义业务结构。
- 快速入门与深度分析：`packages/docs/quick-start.md`、`packages/docs/README.en.md`、`packages/docs/node-budget-mapping.md`
  等提供从入门到场景分析的完整资料。
- 协作排障：`packages/docs/websocket-error-troubleshooting.md` 汇总常见错误与解决路径。

### 开发调试工具

- **[tokio-console 实时监控指南](docs/TOKIO_CONSOLE_GUIDE.md)**：实时监控异步任务状态、检测性能问题
- **[开发追踪指南](docs/DEV_TRACING_GUIDE.md)**：Chrome Tracing、Perfetto 等性能分析工具使用说明
- **[追踪工具对比](docs/TRACING_TOOLS_COMPARISON.md)**：各种追踪工具的对比和选择建议

## 贡献与路线

- 工作区遵循 Rustfmt (见 `rustfmt.toml`) 与 Criterion 基准，建议在提交前执行 `cargo fmt`,
  `cargo clippy --workspace --all-targets`, `cargo test --workspace`。
- 如需扩展 Step、插件或持久化后端，可参考对应 crate 中的 `README.md`、测试与示例，实现 trait 并通过扩展管理器挂载。
- 欢迎在 GitHub Issues 中提交需求、缺陷或性能反馈，也可以围绕文档站点补充更多业务案例。
