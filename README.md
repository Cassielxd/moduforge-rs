# Moduforge

Moduforge是一个不含任何业务绑定的 计价编辑器默认实现，它可以通过扩展进行定制和扩展。 它的无业务绑定本质意味着它没有固定的业务约束，提供了完全的设计自由。可以根据业务需求进行定制。 设计思想是，通过扩展的方式与业务解耦，让编辑器可以支持任何业务，并且可以支持任何业务需求。

### Moduforge是如何工作的?

- **工作方式:** 定义基础节点和标记和约束，然后定义扩展添加行为。

  - **model:** 基础数据的定义，包括节点(Node)，标记(Mark)，约束(Schema)等。

  - **state:** 状态管理，主要负责状态的更新，插件的调度。

  - **transform:** 事务的实现，类似java 的DB事务。保证操作的原子性，保证数据的一致性。可以扩展最小的操作单元。

  - **core:** 组合model,state,transform 进一步实现编辑器的核心功能，添加并收集扩展.

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

#### GlobalResourceManager 使用案例

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

### Moduforge使用了哪些技术框架？

- **im:**  Moduforge编辑器使用im-rs进行基础数据定义，保证数据的不可变。

- **zen:** Moduforge编辑器使用zen进行规则引擎管理，解除业务硬编码耦合。

### Moduforge 框架设计思路

- **可扩展性:** Moduforge设计为高度可扩展，允许开发者根据需求自定义编辑器的功能和行为。这包括插件系统，使得添加新功能变得简单，任何功能都可以扩展.例如：历史记录，撤销，重做。

- **模块化:** 整个框架被分解成多个独立的模块，每个模块负责编辑器的一个特定方面，如模型、状态管理、命令执行等。这种设计使得开发者可以根据项目需求选择性地引入所需模块。

- **命令模式:** 使用命令模式来处理编辑操作。每个编辑操作都被封装成一个命令对象，这样可以方便地撤销或重做操作，同时也有助于实现复杂的编辑逻辑。

- **状态管理:** 编辑器的状态被集中管理，所有对文档的修改都会触发状态的变化。这种设计有助于保持数据的一致性和可预测性。

## 大型树形编辑器适用性分析

Moduforge 框架特别适合用于大型树形编辑器的开发，以下是详细分析：

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

Moduforge 框架非常适合用于大型树形编辑器的开发，特别是在以下场景：

1. 需要处理大量树形数据的编辑器
2. 需要复杂编辑操作的应用
3. 需要实时统计和更新的系统
4. 需要高性能和并发支持的应用
5. 需要高度可定制的编辑器

框架的设计充分考虑了性能、可扩展性和易用性，能够很好地支持大型树形编辑器的开发需求。

## 关于Moduforge

Moduforge,是基于当前计价软件延伸出来的，因此它与 计价 的业务无关，它只是 个大型的 的编辑器。

## License

计价软件内部团队使用请勿泄露。
