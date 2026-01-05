# ModuForge-RS 项目介绍

## 概述

ModuForge-RS 是一个围绕**不可变树形数据模型**构建的 Rust 工作区，专为构建高并发、可协作、可扩展的业务运行时与编辑器内核而设计。它通过事件驱动架构与事务管线，无缝衔接模型层、状态层和转换层，为复杂业务场景提供了一个完整的领域无关内核。

## 项目愿景

ModuForge-RS 的核心愿景是提供一个**统一的、可扩展的运行时框架**，使开发者能够：

- 🚀 **快速构建**：通过模块化设计和声明式 API，快速搭建复杂的业务系统
- 🔄 **保证一致性**：基于不可变数据结构和事务系统，确保数据的强一致性
- 🤝 **支持协作**：内置 CRDT 协作能力，轻松实现多人实时协同
- 📈 **易于扩展**：通过插件系统和扩展机制，灵活扩展业务能力
- 🎯 **领域无关**：核心框架不绑定特定业务领域，适用于各种场景

## 核心特性

### 1. 模块化运行时

`moduforge-core` 提供三种运行模式，满足不同场景需求：

- **同步运行时**：适合简单、快速的场景
- **异步运行时**：基于 Tokio，支持高并发异步处理
- **Actor 运行时**：基于 Ractor，提供分布式 Actor 模型

```rust
// 自动选择最优运行时
let runtime = ForgeRuntimeBuilder::new()
    .build()
    .await?;

// 或手动指定运行时类型
let runtime = ForgeRuntimeBuilder::new()
    .runtime_type(RuntimeType::Async)
    .build()
    .await?;
```

### 2. 不可变数据模型

`moduforge-model` 基于 `rpds` 持久化数据结构，提供：

- **节点树系统**：树形结构管理，支持高效的结构化数据组织
- **标记系统**：为节点添加语义标记，支持富文本编辑等场景
- **属性系统**：类型安全的属性管理
- **Schema 验证**：编译期和运行时的结构验证

```rust
use mf_model::{Node, NodeType, Attrs};

// 创建文档节点
let doc = Node::new(
    "doc".into(),
    NodeType::block("document"),
    Attrs::new(),
    None,
);

// 创建段落节点
let paragraph = Node::new(
    "p1".into(),
    NodeType::block("paragraph"),
    Attrs::new(),
    Some("Hello ModuForge".into()),
);
```

### 3. 事务式变更流程

`moduforge-transform` 将所有数据变更抽象为 Step，确保：

- **原子性**：所有变更要么全部成功，要么全部失败
- **可回放**：完整的操作历史，支持撤销/重做
- **增量同步**：高效的增量更新机制
- **补丁生成**：自动生成变更补丁，支持协作场景

```rust
use mf_transform::node_step::AddNodeStep;

// 创建事务
let mut tr = runtime.get_tr().await?;

// 添加步骤
tr.add_step(Box::new(AddNodeStep::new_single(doc, None)));
tr.add_step(Box::new(AddNodeStep::new_single(paragraph, Some("doc".into()))));

// 提交事务
tr.commit()?;
runtime.dispatch(tr).await?;
```

### 4. 插件与资源系统

`moduforge-state` 提供完整的插件生态：

- **插件生命周期管理**：初始化、激活、停用、卸载
- **资源表管理**：全局资源的注册与访问
- **事务调度**：协调多个插件的事务处理
- **日志系统**：结构化日志输出

```rust
use mf_state::{Plugin, PluginKey};

// 定义插件
struct MyPlugin;

impl Plugin for MyPlugin {
    fn init(&self, state: &mut State) -> Result<()> {
        // 初始化逻辑
        Ok(())
    }
}

// 注册插件
state.register_plugin(PluginKey::new("my_plugin"), MyPlugin)?;
```

### 5. 自适应执行

运行时内置智能调度能力：

- **系统资源探测**：自动检测 CPU、内存等系统资源
- **自适应调度**：根据资源情况动态调整并发度
- **任务超时保护**：防止任务长时间占用资源
- **度量指标**：实时监控运行时性能

```rust
// 运行时会自动检测系统资源并优化配置
let runtime = ForgeRuntimeBuilder::new()
    .build()  // 自动适配
    .await?;
```

### 6. 历史与快照

完善的历史管理机制：

- **撤销/重做**：完整的操作历史栈
- **分段快照**：高效的快照存储策略
- **追加式文件**：只追加的文件格式，支持增量存储
- **压缩打包**：ZIP 格式的历史归档

```rust
// 撤销操作
runtime.undo().await?;

// 重做操作
runtime.redo().await?;

// 保存快照
runtime.save_snapshot("checkpoint.mf").await?;
```

### 7. 搜索与索引

`moduforge-search` 基于 Tantivy 提供全文搜索：

- **增量索引**：与事务系统集成，自动更新索引
- **中文分词**：基于 jieba 的中文分词支持
- **多字段搜索**：支持复杂的搜索查询
- **高性能**：基于 Tantivy 的高性能搜索引擎

```rust
use mf_search::SearchEngine;

// 创建搜索引擎
let engine = SearchEngine::new("./index")?;

// 搜索
let results = engine.search("关键词", 10)?;
```

### 8. 实时协作

`moduforge-collaboration` 基于 Yrs (CRDT) 提供协作能力：

- **房间管理**：多房间隔离
- **WebSocket 服务**：基于 Warp 的 WebSocket 服务器
- **Awareness**：用户状态感知
- **断线恢复**：自动重连和状态同步

```rust
use mf_collaboration::CollaborationServer;

// 启动协作服务器
let server = CollaborationServer::new(config)?;
server.start().await?;
```

### 9. 宏与派生

`moduforge-macros` 提供声明式 API：

- **节点定义**：`#[derive(Node)]` 自动实现节点转换
- **标记定义**：`#[derive(Mark)]` 自动实现标记系统
- **插件定义**：`#[derive(PState)]` 自动实现插件状态
- **编译期验证**：在编译期检查配置一致性

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

## 技术亮点

### 🔒 类型安全

- 充分利用 Rust 的类型系统
- 编译期错误检测
- 零成本抽象

### ⚡ 高性能

- 基于不可变数据结构，支持结构共享
- 并行处理能力
- 高效的内存管理

### 🛡️ 可靠性

- 事务保证数据一致性
- 完整的错误处理
- 测试覆盖率高

### 🔧 可扩展

- 插件系统
- 扩展管理器
- 中间件支持

## 适用场景

ModuForge-RS 特别适合以下场景：

### 1. 复杂树形业务建模

- 工程造价系统
- 财务科目管理
- 物料清单（BOM）
- 知识树/思维导图

### 2. 富文档编辑器

- 富文本编辑器
- Markdown 编辑器
- 代码编辑器
- 配置文件编辑器

### 3. 协同工作台

- 多人协作编辑
- 实时同步
- 权限控制
- 版本管理

### 4. 离线优先应用

- 离线编辑
- 增量同步
- 冲突解决
- 数据恢复

### 5. 配置管理系统

- 配置版本管理
- 变更追踪
- 审计日志
- 回滚能力

## 系统要求

- **Rust**：1.70 或更高版本（推荐使用最新稳定版）
- **Edition**：2024
- **依赖**：Tokio 异步运行时
- **可选**：Node.js 18+ (用于前端示例)

## 下一步

- 📖 阅读[快速开始指南](./quick-start.md)开始使用
- 🧩 了解[核心概念](./core-concepts.md)深入理解
- 🏗️ 查看[架构设计](./architecture.md)了解系统设计
- 💡 浏览[示例代码](../examples/)学习最佳实践

## 社区与支持

- **源代码**：[GitHub Repository](https://github.com/Cassielxd/moduforge-rs)
- **文档**：[DeepWiki](https://deepwiki.com/Cassielxd/moduforge-rs)
- **问题反馈**：[GitHub Issues](https://github.com/Cassielxd/moduforge-rs/issues)

## 许可证

ModuForge-RS 采用 MIT 许可证开源。
