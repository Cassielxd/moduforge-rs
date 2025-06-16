# ModuForge-RS

[Read this in English](./README.en.md)

ModuForge 是一个基于 Rust 的状态管理和数据转换框架，专注于不可变数据结构和事件驱动架构。它提供了无业务绑定的编辑器核心实现，可以通过扩展进行定制和扩展，支持任何业务场景的需求。

### ModuForge 是如何工作的?

- **工作方式:** 定义基础节点、标记和约束，然后定义扩展添加行为。

  - **model:** 基础数据的定义，包括节点(Node)，标记(Mark)，约束(Schema)等。

  - **state:** 状态管理，主要负责状态的更新，插件的调度。

  - **transform:** 事务的实现，类似数据库事务。保证操作的原子性，保证数据的一致性。可以扩展最小的操作单元。

  - **core:** 组合 model、state、transform 进一步实现编辑器的核心功能，添加并收集扩展。

  - **rules:** 规则引擎系统，包含表达式解析、后端执行、引擎核心和模板系统。

### 项目目录结构

```
moduforge-rs/
├── core/           # 核心功能模块
│   ├── src/
│   │   ├── lib.rs                 # 核心库入口
│   │   ├── async_processor.rs     # 异步任务处理器
│   │   ├── async_runtime.rs       # 异步运行时环境
│   │   ├── error.rs               # 错误类型和处理
│   │   ├── event.rs               # 事件系统
│   │   ├── extension.rs           # 扩展机制
│   │   ├── extension_manager.rs   # 扩展管理器
│   │   ├── flow.rs                # 流程控制
│   │   ├── helpers/               # 辅助函数
│   │   ├── history_manager.rs     # 历史记录管理
│   │   ├── mark.rs                # 标记系统
│   │   ├── metrics.rs             # 指标系统
│   │   ├── middleware.rs          # 中间件支持
│   │   ├── node.rs                # 节点系统
│   │   ├── runtime.rs             # 运行时环境
│   │   └── types.rs               # 核心类型定义
│   └── Cargo.toml                 # 核心模块依赖配置
│
├── model/          # 数据模型模块
│   ├── src/
│   │   ├── lib.rs                 # 模型定义入口
│   │   ├── node.rs                # 节点定义
│   │   ├── mark.rs                # 标记定义
│   │   ├── attrs.rs               # 属性定义
│   │   ├── mark_type.rs           # 标记类型定义
│   │   ├── node_type.rs           # 节点类型定义
│   │   ├── schema.rs              # 模式定义
│   │   ├── content.rs             # 内容匹配定义
│   │   ├── error.rs               # 错误类型和处理
│   │   ├── id_generator.rs        # ID 生成器
│   │   ├── node_pool.rs           # 节点池管理
│   │   └── types.rs               # 通用类型定义
│   └── Cargo.toml                 # 模型模块依赖配置
│
├── transform/      # 数据转换模块
│   ├── src/
│   │   ├── lib.rs                 # 转换功能入口
│   │   ├── attr_step.rs           # 属性步骤
│   │   ├── draft.rs               # 草稿系统
│   │   ├── mark_step.rs           # 标记步骤
│   │   ├── node_step.rs           # 节点步骤
│   │   ├── patch.rs               # 补丁系统
│   │   ├── step.rs                # 步骤定义
│   │   └── transform.rs           # 转换系统
│   └── Cargo.toml                 # 转换模块依赖配置
│
├── state/          # 状态管理模块
│   ├── src/
│   │   ├── lib.rs                 # 状态管理入口
│   │   ├── error.rs               # 错误类型和处理
│   │   ├── gotham_state.rs        # Gotham 状态管理
│   │   ├── logging.rs             # 日志系统
│   │   ├── ops.rs                 # 操作定义
│   │   ├── plugin.rs              # 插件系统
│   │   ├── resource.rs            # 资源管理
│   │   ├── resource_table.rs      # 资源表
│   │   ├── state.rs               # 状态管理
│   │   └── transaction.rs         # 事务处理
│   └── Cargo.toml                 # 状态模块依赖配置
│
├── rules/          # 规则引擎模块
│   ├── expression/  # 表达式解析和处理
│   ├── backend/     # 规则引擎后端
│   ├── engine/      # 规则引擎核心
│   └── template/    # 模板系统
│
├── macros/         # 宏定义模块
│   ├── src/
│   │   ├── lib.rs                 # 宏定义入口
│   │   ├── command.rs             # 命令宏
│   │   ├── extension.rs           # 扩展宏
│   │   ├── mark.rs                # 标记宏
│   │   ├── node.rs                # 节点宏
│   │   └── plugin.rs              # 插件宏
│   └── Cargo.toml                 # 宏模块依赖配置
│
├── demo/           # 示例和演示代码
│   ├── src/
│   ├── Cargo.toml
│   └── README.md
│
├── docs/           # 项目文档
│   ├── node-budget-mapping.md    # Node模型到业务映射
│   ├── architecture_use_cases.md # 架构应用场景分析
│   ├── plugin-development-guide.md # 插件开发指南
│   └── ...                       # 其他分析文档
│
├── test-data/      # 测试数据
├── Cargo.toml      # 工作空间配置文件
├── Cargo.lock      # 依赖锁定文件
├── rustfmt.toml    # Rust 代码格式化配置
├── release.toml    # 发布配置
└── .gitignore      # Git 忽略文件配置
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

### 📖 文档使用建议

1. **新手入门**：建议先阅读本README.md了解整体架构，再查看`architecture_use_cases.md`了解适用场景

2. **业务建模**：如需将ModuForge应用到具体业务，重点参考`node-budget-mapping.md`的映射方法

3. **复杂依赖**：如果业务间存在复杂依赖关系，优先考虑`meta_based_dependency_design.md`的轻量级方案

4. **架构决策**：在项目技术选型时，参考`architecture_limitations_analysis.md`的客观分析

5. **功能扩展**：需要添加历史管理功能时，参考`simple_enhanced_history.md`的实现方案

这些文档共同构成了ModuForge项目的完整技术体系，为不同层次的读者提供了从概念理解到具体实现的全方位指导。
