---
layout: home

hero:
  name: "ModuForge-RS"
  text: "模块化应用框架"
  tagline: "基于 Rust 的高性能模块化框架，支持状态管理、规则引擎、实时协作和插件扩展"
  image:
    src: /logo.svg
    alt: ModuForge-RS
  actions:
    - theme: brand
      text: 快速开始
      link: /setup-external-project
    - theme: alt
      text: 插件开发指南
      link: /plugin-development-guide
    - theme: alt
      text: 查看 GitHub
      link: https://github.com/Cassielxd/moduforge-rs

features:
  - icon: 🏗️
    title: 模块化架构
    details: 14个独立Crate组成的高度模块化架构，支持按需引入和灵活组合。
  
  - icon: 🚀
    title: 高性能运行时
    details: 基于Tokio的异步运行时，支持不可变数据结构和并发事务处理。
  
  - icon: 🔧
    title: 插件生态系统
    details: 完整的插件开发框架，支持依赖管理、生命周期控制和热插拔。
  
  - icon: 📊
    title: 规则引擎
    details: 内置GoRules JDM标准兼容的规则引擎和高性能表达式语言。
  
  - icon: 🤝
    title: 实时协作
    details: 基于Yrs CRDT的冲突自由协作系统，支持WebSocket实时同步。
  
  - icon: ⚡
    title: 强大宏系统
    details: 提供Node和Mark派生宏，支持自定义类型表达式、JSON默认值和类型安全转换。
  
  - icon: 🎯
    title: 业务中性
    details: 零业务逻辑耦合，通过扩展机制适配编辑器、计价、流程等场景。
---

## 什么是 ModuForge-RS？

ModuForge-RS 是一个基于 Rust 的高性能模块化应用框架，专门为构建复杂的业务应用而设计。框架由 14 个独立的 Crate 组成，提供了从数据模型、状态管理、规则引擎到实时协作的完整解决方案。

### 核心能力

🏗️ **模块化设计** - 14个专业Crate，按需组合，灵活扩展  
⚡ **高性能运行时** - 基于Tokio异步架构，支持高并发处理  
🔧 **插件生态** - 完整的插件开发框架，支持热插拔和依赖管理  
📊 **规则引擎** - 内置业务规则引擎，支持动态决策和表达式计算  
🤝 **实时协作** - 基于CRDT的冲突自由协作，支持多用户实时编辑  
⚡ **强大宏系统** - Node/Mark派生宏，支持自定义类型表达式和JSON默认值  
🎯 **业务中性** - 零业务逻辑耦合，可适配任何领域的应用场景

### ModuForge 是如何工作的?

- **工作方式:** 定义基础节点、标记和约束，然后定义扩展添加行为。

  - **model:** 基础数据的定义，包括节点(Node)，标记(Mark)，约束(Schema)等。

  - **state:** 状态管理，主要负责状态的更新，插件的调度。

  - **transform:** 事务的实现，类似数据库事务。保证操作的原子性，保证数据的一致性。可以扩展最小的操作单元。

  - **core:** 组合 model、state、transform 进一步实现编辑器的核心功能，添加并收集扩展。

  - **rules:** 规则引擎系统，包含表达式解析、后端执行、引擎核心和模板系统。

  - **macro:** 宏系统，通过 `#[derive(Node)]` 和 `#[derive(Mark)]` 自动生成节点和标记相关代码。

### 项目目录结构

```
moduforge-rs/
├── crates/         # 框架核心库集合
│   ├── core/       # mf-core - 核心运行时
│   ├── model/      # mf-model - 数据模型
│   ├── state/      # mf-state - 状态管理
│   ├── transform/  # mf-transform - 数据转换
│   ├── engine/     # mf-engine - 规则引擎
│   ├── expression/ # mf-expression - 表达式语言
│   ├── template/   # mf-template - 模板系统
│   ├── collaboration/    # mf-collaboration - 协作服务
│   ├── collaboration-client/ # mf-collaboration-client
│   ├── file/       # mf-file - 文件处理
│   ├── search/     # mf-search - 搜索引擎
│   ├── persistence/ # mf-persistence - 持久化
│   ├── macro/      # mf-macro - 过程宏
│   └── derive/     # mf-derive - 派生宏
│
├── packages/       # 应用和工具包
│   ├── docs/       # 完整的文档网站
│   │   ├── quick-start.md          # 快速入门指南
│   │   ├── architecture-overview.md # 架构概览
│   │   ├── api-reference.md        # API 参考
│   │   ├── performance-guide.md    # 性能优化指南
│   │   ├── plugin-development-guide.md # 插件开发
│   │   ├── macro-system-guide.md   # 宏系统开发指南
│   │   └── ...                     # 更多文档
│   └── devtool-rules/  # 开发工具
│
├── demo/           # Tauri 演示应用
│   ├── src-tauri/  # Tauri 后端
│   ├── src/        # 前端代码
│   └── package.json
│
├── Cargo.toml      # 工作空间配置
├── CLAUDE.md       # Claude 开发指南
└── README.md       # 项目说明
```

### 核心组件详解

#### State 状态管理

State 是编辑器的核心状态管理组件，负责维护编辑器的整体状态。它包含以下关键特性：

- **配置管理**: 通过 `Configuration` 结构体管理编辑器的配置信息，包括插件列表、文档结构定义等
- **插件状态**: 通过 `fields_instances` 管理所有插件的状态数据
- **文档管理**: 通过 `node_pool` 管理文档的节点池
- **版本控制**: 通过 `version` 字段追踪状态变更
- **资源管理**: 通过 `resource_manager` 管理全局资源

State 提供了以下主要功能：
- 创建和初始化新的编辑器状态
- 管理插件状态
- 处理事务和状态更新
- 重新配置状态以适应新的需求

#### GlobalResourceManager 全局资源管理器

GlobalResourceManager 是编辑器运行时的全局资源管理器，负责管理所有注册的资源和状态。它包含以下关键特性：

- **资源表管理**: 通过 `ResourceTable` 管理所有注册的资源
- **Gotham状态管理**: 通过 `GothamState` 管理特定于Gotham框架的状态
- **线程安全**: 实现了 `Send` 和 `Sync` trait，确保可以在线程间安全传递和共享
- **资源清理**: 提供 `clear` 方法用于清理所有资源

GlobalResourceManager 的主要使用场景：
- 插件间共享资源
- 管理全局状态
- 处理跨插件的数据交换
- 管理编辑器运行时的全局配置

GlobalResourceManager 使用案例

以下是一个使用 GlobalResourceManager 的典型场景：

```rust
// 1. 定义自定义资源类型
#[derive(Debug)]
struct CacheManager {
    data: HashMap<String, String>,
}

impl Resource for CacheManager {
    fn name(&self) -> Cow<str> {
        "CacheManager".into()
    }
}

// 2. 在插件初始化时注册资源
async fn plugin_init(config: &StateConfig, instance: Option<&State>) -> PluginState {
    // 获取资源管理器
    let resource_manager = instance.unwrap().resource_manager();
    let mut resource_manager = resource_manager.write().unwrap();
    
    // 创建并注册缓存管理器
    let cache_manager = CacheManager {
        data: HashMap::new(),
    };
    resource_manager.resource_table.add(cache_manager);
    
    // 返回插件状态
    Arc::new(HashMap::new())
}

// 3. 在插件中使用共享资源
async fn plugin_operation(state: &State) {
    // 获取资源管理器
    let resource_manager = state.resource_manager();
    let resource_manager = resource_manager.read().unwrap();
    
    // 获取缓存管理器
    let cache_manager = resource_manager.resource_table.get::<CacheManager>(0).unwrap();
    
    // 使用缓存管理器
    cache_manager.data.insert("key".to_string(), "value".to_string());
}

// 4. 在另一个插件中访问相同资源
async fn another_plugin_operation(state: &State) {
    let resource_manager = state.resource_manager();
    let resource_manager = resource_manager.read().unwrap();
    
    let cache_manager = resource_manager.resource_table.get::<CacheManager>(0).unwrap();
    let value = cache_manager.data.get("key").unwrap();
    println!("获取到的值: {}", value);
}
```

这个案例展示了：
1. 如何定义自定义资源类型
2. 如何在插件初始化时注册资源
3. 如何在插件中使用共享资源
4. 如何在多个插件间共享和访问同一资源

通过 GlobalResourceManager，不同插件可以安全地共享和访问全局资源，而不需要直接依赖其他插件。

#### Macro 宏系统

ModuForge-RS 提供了强大的过程宏系统，通过 `#[derive(Node)]` 和 `#[derive(Mark)]` 自动生成节点和标记的相关代码。宏系统设计严格遵循 SOLID 原则，提供类型安全、灵活且高性能的代码生成能力。

**核心特性**：
- **设计分离**: `node_definition()` 只包含 `#[attr]` 字段的模式定义，`from()` 处理所有字段的实例创建
- **属性精确性**: 只有标记了 `#[attr]` 的字段才会成为节点的属性定义
- **类型安全**: 支持泛型类型和自定义类型的安全转换，要求自定义类型实现 `Default + Serialize` traits

**支持的功能特性**：
- 基本类型：`String`, `i32`, `f64`, `bool` 等
- 泛型类型：`Option<T>`, `Vec<T>`, `HashMap<K,V>` 等
- 自定义类型：支持构造函数表达式，如 `CustomStruct::new()`
- JSON 默认值：支持复杂的 JSON 结构作为默认值
- 双向转换：自动生成 `From` trait 实现
- 错误处理：类型验证与优雅降级

**使用示例**：

```rust
use mf_derive::Node;

#[derive(Node)]
#[node_type = "document"]
#[marks = "bold italic"]
#[content = "block+"]
struct DocumentNode {
    // 基本属性字段
    #[attr]
    title: String,
    
    #[attr(default="未命名文档")]
    description: String,
    
    #[attr(default=1)]
    version: i32,
    
    // 可选类型字段
    #[attr]
    subtitle: Option<String>,
    
    // 自定义类型表达式
    #[attr(default="DocumentConfig::new()")]
    config: DocumentConfig,
    
    // JSON 默认值
    #[attr(default={"theme": "light", "auto_save": true})]
    ui_config: serde_json::Value,
    
    // 非属性字段（不会出现在 node_definition 中）
    computed_hash: String,
}
```

**生成的方法**：
- `node_definition()`: 获取节点定义（只包含 `#[attr]` 字段的模式定义）
- `from()`: 从 `mf_model::node::Node` 创建结构体实例（处理所有字段）
- `default_instance()`: 创建默认实例（失败时的降级方法）
- `From` trait 实现：支持双向转换

详细的宏系统使用指南请参考：[宏系统开发指南](./macro-system-guide.md)

#### State 与 GlobalResourceManager 的区别

虽然 State 和 GlobalResourceManager 都涉及状态管理，但它们在职责和使用场景上有明显区别：

1. **管理范围不同**
   - State 管理编辑器的整体状态，包括文档内容、插件状态、配置等
   - GlobalResourceManager 专注于管理运行时资源和全局共享状态

2. **生命周期不同**
   - State 的生命周期与编辑器实例绑定，随编辑器的创建而创建，销毁而销毁
   - GlobalResourceManager 的生命周期更灵活，可以在不同 State 实例间共享

3. **访问方式不同**
   - State 通过事务（Transaction）进行状态更新，保证操作的原子性和一致性
   - GlobalResourceManager 提供直接的资源访问接口，适合快速读写共享资源

4. **使用场景不同**
   - State 用于管理编辑器的核心状态，如文档内容、插件配置等
   - GlobalResourceManager 用于管理跨插件共享的资源，如缓存、配置等

5. **线程安全性不同**
   - State 的状态更新是单线程的，通过事务保证一致性
   - GlobalResourceManager 是线程安全的，可以在多线程环境下安全访问

6. **扩展性不同**
   - State 的状态结构相对固定，主要围绕文档和插件
   - GlobalResourceManager 可以动态注册和管理任意类型的资源，扩展性更强

### ModuForge 使用了哪些技术框架？

- **im-rs:**  ModuForge 使用 im-rs 进行基础数据定义，保证数据的不可变性。

- **tokio:** 异步运行时，支持高并发的异步操作。

- **serde:** 序列化和反序列化支持，用于数据的持久化和传输。

- **thiserror/anyhow:** 错误处理框架，提供类型安全的错误管理。

- **zen:** 规则引擎管理，解除业务硬编码耦合（如果使用）。

### ModuForge 框架设计思路

- **可扩展性:** ModuForge 设计为高度可扩展，允许开发者根据需求自定义编辑器的功能和行为。这包括插件系统，使得添加新功能变得简单，任何功能都可以扩展。例如：历史记录，撤销，重做。

- **模块化:** 整个框架被分解成多个独立的模块，每个模块负责编辑器的一个特定方面，如模型、状态管理、命令执行等。这种设计使得开发者可以根据项目需求选择性地引入所需模块。

- **不可变数据:** 使用 im-rs 确保数据结构的不可变性，提供安全的并发访问和高效的结构共享。

- **事件驱动:** 基于事件驱动架构，所有状态变更都通过事件系统进行，确保系统的响应性和可预测性。

- **命令模式:** 使用命令模式来处理编辑操作。每个编辑操作都被封装成一个命令对象，这样可以方便地撤销或重做操作，同时也有助于实现复杂的编辑逻辑。

- **状态管理:** 编辑器的状态被集中管理，所有对文档的修改都会触发状态的变化。这种设计有助于保持数据的一致性和可预测性。

## 大型树形编辑器适用性分析

ModuForge 框架特别适合用于大型树形编辑器的开发，以下是详细分析：

### 1. 树形结构支持

框架提供了完整的树形结构支持：

- **节点定义**：
  - 每个节点都有唯一的ID
  - 支持节点类型（type）
  - 支持节点属性（attrs）
  - 支持子节点列表（content）
  - 支持节点标记（marks）

- **树操作API**：
  - 获取子节点列表
  - 递归获取所有子节点（深度优先）
  - 获取父节点
  - 获取节点深度
  - 获取节点路径
  - 检查叶子节点
  - 获取兄弟节点
  - 获取子树大小

### 2. 编辑功能支持

框架提供了完整的编辑功能：

- **节点操作**：
  - 添加节点
  - 替换节点
  - 移动节点
  - 删除节点
  - 递归删除子树

- **事务支持**：
  - 所有操作都在事务中执行
  - 支持操作的原子性
  - 支持撤销/重做
  - 支持补丁记录

### 3. 统计功能支持

框架提供了丰富的统计功能：

- **节点统计**：
  - 获取节点总数
  - 获取子树大小
  - 获取节点深度
  - 支持自定义过滤和查找

- **性能优化**：
  - 使用不可变数据结构（im-rs）
  - 使用 Arc 进行引用计数
  - 支持并发访问（Send + Sync）

### 4. 特别适合的场景

1. **大型树形数据编辑**：
   - 支持深度嵌套的树结构
   - 高效的节点查找和遍历
   - 支持大规模数据操作

2. **复杂数据统计**：
   - 支持自定义统计规则
   - 支持节点过滤和查找
   - 支持子树统计

3. **实时编辑和更新**：
   - 支持事务性操作
   - 支持撤销/重做
   - 支持增量更新

### 5. 性能考虑

1. **内存效率**：
   - 使用不可变数据结构
   - 使用引用计数
   - 支持共享节点

2. **操作效率**：
   - 高效的节点查找
   - 优化的树遍历
   - 批量操作支持

3. **并发支持**：
   - 线程安全设计
   - 支持并发访问
  - 支持资源管理

### 6. 扩展性

1. **自定义节点类型**：
   - 支持自定义节点属性
   - 支持自定义节点标记
   - 支持自定义节点内容

2. **插件系统**：
   - 支持自定义编辑操作
   - 支持自定义统计规则
   - 支持自定义验证规则

### 结论

ModuForge 框架非常适合用于大型树形编辑器的开发，特别是在以下场景：

1. 需要处理大量树形数据的编辑器
2. 需要复杂编辑操作的应用
3. 需要实时统计和更新的系统
4. 需要高性能和并发支持的应用
5. 需要高度可定制的编辑器

框架的设计充分考虑了性能、可扩展性和易用性，能够很好地支持大型树形编辑器的开发需求。

## 关于 ModuForge

ModuForge 是基于当前计价软件延伸出来的通用编辑器框架，因此它与具体的计价业务无关，它只是一个大型的、通用的编辑器框架。

## License

计价软件内部团队使用请勿泄露。

## 📚 相关文档

本项目包含多个详细的分析和设计文档，涵盖架构分析、业务应用、设计模式等多个方面：

### 🎋 业务模型映射

#### [Node模型到建筑预算的精确映射](./node-budget-mapping.md)
**详细介绍了如何将ModuForge的Node模型精确映射到建筑预算业务**

- **核心内容**：
  - ModuForge Node模型与建筑预算业务的详细映射关系
  - 预算项目层级结构的完整定义（预算文档→工程项目→单位工程→分部工程→分项工程→清单项目）
  - NodeSpec业务类型规范的具体实现代码
  - Mark标记系统在业务状态管理中的应用
  - 实际业务查询和统计功能的代码示例

- **技术亮点**：
  - 层级结构天然支持工程预算的组织方式
  - 属性系统完美匹配造价数据（数量、单价、金额等）
  - 标记系统支持业务状态管理（已计算、已锁定、已套定额等）
  - 提供完整的业务分析器实现代码

### 🚀 架构应用场景

#### [架构适用业务场景分析](./architecture_use_cases.md)
**深入分析ModuForge架构在不同业务场景中的适用性**

- **业务场景分类**：
  - **业务流程编排**：工作流引擎、数据处理管道(ETL)
  - **计算编排**：计价引擎系统、风控决策引擎  
  - **内容管理**：协同编辑器、内容发布系统
  - **规则引擎**：业务规则引擎、A/B测试框架
  - **智能计算**：推荐系统、机器学习Pipeline

- **实际应用示例**：
  - 保险计价引擎、出行计费系统
  - 在线文档协作、代码协同编辑
  - 大数据处理平台、实时数据流处理
  - 风控系统、推荐算法平台

### 🔗 业务依赖设计

#### [A业务依赖B业务的可插拔架构设计](./business_dependency_design.md)
**传统的业务依赖管理器方案**

- **设计特点**：
  - 通过专门的BusinessDependencyManager管理业务间依赖关系
  - 支持依赖类型分类（计算依赖、数据依赖、事件依赖）
  - 实现完整的依赖检查和执行顺序管理
  - 提供拓扑排序确保依赖执行顺序正确

- **核心组件**：
  - BusinessDependencyManager：统一依赖管理
  - BusinessDependency：依赖关系描述
  - 完整的A业务和B业务插件实现示例

#### [基于Transaction Meta的业务依赖解耦设计](./meta_based_dependency_design.md)
**推荐的轻量级业务依赖方案**

- **设计优势**：
  - 利用Transaction的meta字段传递业务依赖信息
  - 更轻量级，无需额外的依赖管理器组件
  - 完全基于现有事务系统实现
  - 支持业务降级和容错处理

- **技术实现**：
  - Meta字段结构化设计（业务类型、状态、依赖关系）
  - 业务执行上下文传递
  - 依赖满足检查和等待机制
  - 完整的插件实现代码示例

### 📈 架构分析

#### [架构限制性分析](./architecture_limitations_analysis.md)
**客观分析ModuForge架构的优势与限制**

- **分析维度**：
  - 性能特性分析（内存使用、并发能力、响应时间）
  - 扩展性分析（插件系统、业务适配能力）
  - 复杂度分析（开发难度、学习曲线、维护成本）
  - 适用性边界（适合与不适合的业务场景）

#### [简化版历史管理增强](./simple_enhanced_history.md)
**历史管理和撤销重做功能的设计实现**

- **核心功能**：
  - 基于快照的历史管理策略
  - 高效的撤销重做操作实现
  - 历史记录的压缩和清理机制
  - 与事务系统的深度集成

---

## 📚 完整文档导航

### 🚀 新手入门路径
1. **[快速入门指南](./quick-start.md)** - 从安装到第一个应用的完整教程
2. **[架构概览](./architecture-overview.md)** - 了解框架的整体设计和14个核心组件
3. **[API 参考](./api-reference.md)** - 详细的 API 文档和代码示例

### 🔧 开发者深入学习
4. **[插件开发指南](./plugin-development-guide.md)** - 学习插件开发的完整流程
5. **[宏系统开发指南](./macro-system-guide.md)** - 掌握Node和Mark派生宏的使用
6. **[性能优化指南](./performance-guide.md)** - 掌握高性能应用开发技巧
7. **[外部项目集成](./setup-external-project.md)** - 将框架集成到现有项目

### 🏗️ 架构设计参考
8. **[应用场景分析](./architecture_use_cases.md)** - 了解框架适用的业务场景
9. **[架构限制分析](./architecture_limitations_analysis.md)** - 客观了解框架的优势与限制
10. **[业务依赖设计](./business_dependency_design.md)** - 复杂业务间依赖的设计方案

### 💼 实际业务应用
11. **[节点预算映射](./node-budget-mapping.md)** - 框架到实际业务的映射示例
12. **[元数据依赖设计](./meta_based_dependency_design.md)** - 轻量级业务依赖方案

### 📖 文档使用建议

**根据您的角色和需求选择合适的学习路径**：

- **🆕 完全新手**: index.md → quick-start.md → architecture-overview.md → api-reference.md
- **🔧 开发者**: quick-start.md → plugin-development-guide.md → macro-system-guide.md → performance-guide.md
- **🏗️ 架构师**: architecture-overview.md → architecture_use_cases.md → architecture_limitations_analysis.md
- **💼 业务分析师**: node-budget-mapping.md → business_dependency_design.md
- **⚡ 性能工程师**: performance-guide.md → api-reference.md
- **📝 宏系统开发者**: macro-system-guide.md → plugin-development-guide.md

这些文档共同构成了ModuForge项目的完整技术体系，为不同层次的读者提供了从概念理解到具体实现的全方位指导。
