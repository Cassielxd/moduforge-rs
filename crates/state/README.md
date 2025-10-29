# ModuForge-RS 状态管理包

[![Crates.io](https://img.shields.io/crates/v/moduforge-state)](https://crates.io/crates/moduforge-state)
[![Documentation](https://docs.rs/moduforge-state/badge.svg)](https://docs.rs/moduforge-state)
[![License](https://img.shields.io/crates/l/moduforge-state)](LICENSE)

ModuForge-RS 状态管理包提供了基于不可变数据结构的现代化状态管理系统，支持事务处理、插件扩展、资源管理和实时协作。该包是 ModuForge-RS 框架的核心组件，为应用程序提供可靠、高效的状态管理能力。

## 🏗️ 架构概述

ModuForge-RS 状态管理采用不可变数据结构范式，确保状态变更的可预测性和可追溯性。系统基于以下核心设计原则：

- **不可变状态**: 使用 `im-rs` 库实现高效的不可变数据结构
- **事务驱动**: 所有状态变更通过事务进行，支持 ACID 特性
- **插件架构**: 可扩展的插件系统，支持动态功能扩展
- **资源管理**: 全局资源表和生命周期管理
- **事件溯源**: 完整的状态变更历史记录和重放能力

### 核心架构组件

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   State         │    │   Transaction   │    │   Plugin        │
│   (状态管理)     │◄──►│   (事务处理)     │◄──►│   (插件系统)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Resource      │    │   ResourceTable │    │   GothamState   │
│   (资源管理)     │    │   (资源表)       │    │   (框架状态)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🚀 核心功能

### 1. 状态管理 (State)
- **不可变状态**: 基于 `im::HashMap` 的不可变状态存储
- **版本控制**: 自动版本号管理，支持状态回滚
- **配置管理**: 灵活的状态配置和初始化
- **序列化支持**: 完整的状态序列化和反序列化

### 2. 事务处理 (Transaction)
- **ACID 事务**: 原子性、一致性、隔离性、持久性
- **批量操作**: 高效的批量状态变更处理
- **元数据支持**: 丰富的元数据存储和检索
- **命令模式**: 可扩展的命令执行接口

### 3. 插件系统 (Plugin)
- **动态加载**: 运行时插件加载和卸载
- **优先级管理**: 基于优先级的插件执行顺序
- **状态隔离**: 插件状态的安全隔离和管理
- **生命周期**: 完整的插件生命周期管理

### 4. 资源管理 (Resource)
- **类型安全**: 基于 `TypeId` 的类型安全资源管理
- **全局资源表**: 集中式资源注册和查找
- **生命周期**: 自动资源清理和内存管理
- **并发安全**: 线程安全的资源访问

### 5. 日志系统 (Logging)
- **结构化日志**: 基于 `tracing` 的结构化日志记录
- **多输出**: 支持控制台和文件双重输出
- **级别控制**: 灵活的日志级别配置
- **性能监控**: 内置性能指标收集

## 📦 技术栈

### 核心依赖
```toml
[dependencies]
# 不可变数据结构
im = { version = "15.1", features = ["serde"] }

# 序列化
serde = { version = "1.0", features = ["derive", "rc"] }
serde_json = "1.0"

# 异步运行时
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"

# 并发和同步
crossbeam = "0.8"
dashmap = "6.1.0"

# 错误处理
anyhow = "1"
thiserror = "2.0.12"

# 日志系统
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"

# 时间处理
time = "0.3"
```

### ModuForge-RS 内部依赖
```toml
# 数据模型
moduforge-model = "0.4.12"

# 数据转换
moduforge-transform = "0.4.12"
```

## 🚀 快速开始

### 基本使用

```rust
use mf_state::{State, StateConfig, Transaction};
use mf_model::{schema::Schema, node_pool::NodePool};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志系统（可选）
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 创建状态配置
    let schema = Arc::new(Schema::default());
    let state_config = StateConfig {
        schema: Some(schema),
        doc: None,
        stored_marks: None,
        plugins: None,
        resource_manager: None,
    };
    
    // 创建状态实例
    let state = State::create(state_config).await?;
    
    // 创建事务
    let mut transaction = Transaction::new(&state);
    
    // 添加节点
    let node_id = "new_node".to_string();
    transaction.add_node(
        node_id.clone(),
        vec![/* 节点数据 */]
    )?;
    
    // 设置元数据
    transaction.set_meta("action", "add_node");
    transaction.set_meta("user_id", "user_123");
    
    // 应用事务
    let result = state.apply(transaction).await?;
    
    println!("事务应用成功，新状态版本: {}", result.state.version);
    Ok(())
}
```

### 插件开发

```rust
use mf_state::{
    plugin::{Plugin, PluginSpec, PluginTrait, StateField},
    resource::Resource,
    State, Transaction, StateResult
};
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug)]
struct MyPluginState {
    counter: i32,
}

impl Resource for MyPluginState {}

#[derive(Debug)]
struct MyPlugin;

#[async_trait]
impl PluginTrait for MyPlugin {
    async fn filter_transaction(
        &self,
        tr: &Transaction,
        _state: &State,
    ) -> bool {
        // 检查事务是否应该被过滤
        !tr.get_meta::<String>("skip_plugin").is_some()
    }
    
    async fn append_transaction(
        &self,
        _trs: &[Transaction],
        _old_state: &State,
        _new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 可以在这里添加额外的事务
        Ok(None)
    }
}

#[derive(Debug)]
struct MyStateField;

#[async_trait]
impl StateField for MyStateField {
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: Option<&State>,
    ) -> Arc<dyn Resource> {
        Arc::new(MyPluginState { counter: 0 })
    }
    
    async fn apply(
        &self,
        _tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        // 更新插件状态
        if let Some(state) = value.downcast_arc::<MyPluginState>() {
            Arc::new(MyPluginState {
                counter: state.counter + 1,
            })
        } else {
            value
        }
    }
}

// 创建插件
let plugin = Plugin::new(PluginSpec {
    key: ("my_plugin".to_string(), "v1".to_string()),
    tr: Some(Arc::new(MyPlugin)),
    state_field: Some(Arc::new(MyStateField)),
    priority: 10,
});
```

### 资源管理

```rust
use mf_state::{
    resource::Resource,
    resource_table::ResourceTable,
    gotham_state::GothamState,
    ops::GlobalResourceManager
};
use std::sync::Arc;

#[derive(Debug, Clone)]
struct MyResource {
    data: String,
}

impl Resource for MyResource {}

// 使用资源表
let resource_table = ResourceTable::default();
resource_table.add("my_resource".to_string(), MyResource {
    data: "Hello World".to_string(),
});

// 获取资源
if let Some(resource) = resource_table.get::<MyResource>("my_resource") {
    println!("资源数据: {}", resource.data);
}

// 使用 Gotham 状态
let gotham_state = GothamState::default();
gotham_state.put(MyResource {
    data: "Gotham Resource".to_string(),
});

if let Some(resource) = gotham_state.try_get::<MyResource>() {
    println!("Gotham 资源: {}", resource.data);
}

// 使用全局资源管理器
let mut manager = GlobalResourceManager::new();
manager.resource_table.add("global_resource".to_string(), MyResource {
    data: "Global Resource".to_string(),
});
```

## 🔧 配置选项

### 状态配置

```rust
use mf_state::StateConfig;
use mf_model::{schema::Schema, node_pool::NodePool, mark::Mark};
use std::sync::Arc;

let config = StateConfig {
    // 文档结构定义
    schema: Some(Arc::new(Schema::default())),
    
    // 初始文档内容
    doc: Some(Arc::new(NodePool::default())),
    
    // 存储的标记
    stored_marks: Some(vec![Mark::default()]),
    
    // 插件列表
    plugins: Some(vec![/* 插件列表 */]),
    
    // 资源管理器
    resource_manager: Some(Arc::new(GlobalResourceManager::new())),
};
```

### 日志配置

> ⚠️ **注意**：`mf_state::init_logging` 已被弃用，请使用 `mf_core::tracing_init::dev_tracing::init_tracing` 代替。

#### 推荐方式（使用 mf_core）

```rust
#[cfg(feature = "dev-tracing")]
use mf_core::tracing_init::dev_tracing::{init_tracing, TraceConfig};

// 控制台输出（开发环境）
#[cfg(feature = "dev-tracing")]
let _guard = init_tracing(TraceConfig::console())?;

// JSON 文件输出
#[cfg(feature = "dev-tracing")]
let _guard = init_tracing(TraceConfig::json("./logs/trace.json"))?;

// Chrome Tracing（性能分析）
#[cfg(feature = "dev-tracing-chrome")]
let _guard = init_tracing(TraceConfig::chrome("./logs/trace.json"))?;

// Perfetto（高级性能分析）
#[cfg(feature = "dev-tracing-perfetto")]
let _guard = init_tracing(TraceConfig::perfetto("./logs/trace.perfetto"))?;
```

#### 简单方式（仅用于示例/测试）

```rust
// 如果只需要简单的控制台日志，可以直接使用 tracing_subscriber
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .with_target(false)
    .init();
```

#### 旧方式（已弃用）

```rust
use mf_state::init_logging;

// ⚠️ 已弃用：只输出到控制台
init_logging("debug", None)?;

// ⚠️ 已弃用：同时输出到文件和控制台
init_logging("info", Some("logs/moduforge.log"))?;
```

## 📊 性能特性

### 不可变数据结构优化
- **结构共享**: 利用 `im-rs` 的结构共享减少内存使用
- **延迟克隆**: 只在必要时进行数据克隆
- **批量操作**: 支持高效的批量状态变更

### 并发性能
- **无锁设计**: 使用不可变数据结构避免锁竞争
- **原子操作**: 基于原子操作的状态版本管理
- **并发安全**: 线程安全的状态访问和修改

### 内存管理
- **智能缓存**: 自动缓存频繁访问的状态
- **资源池**: 高效的资源分配和回收
- **内存监控**: 内置内存使用监控

## 🛠️ 错误处理

ModuForge-RS 状态管理包提供了完善的错误处理机制：

```rust
use mf_state::error::{StateResult, error};

// 自定义错误处理
fn handle_state_error(result: StateResult<State>) -> anyhow::Result<State> {
    match result {
        Ok(state) => Ok(state),
        Err(e) => {
            // 记录错误
            tracing::error!("状态操作失败: {}", e);
            
            // 根据错误类型进行不同处理
            if e.to_string().contains("schema") {
                return Err(error::schema_error("Schema 配置错误").into());
            }
            
            Err(e)
        }
    }
}
```

### 常见错误类型
- **插件错误**: 插件初始化或执行失败
- **事务错误**: 事务应用或验证失败
- **配置错误**: 状态配置无效
- **序列化错误**: 状态序列化或反序列化失败
- **资源错误**: 资源操作失败

## 🧪 测试

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_state_creation() {
        let config = StateConfig::default();
        let state = State::create(config).await.unwrap();
        assert_eq!(state.version, 1);
    }
    
    #[tokio::test]
    async fn test_transaction_application() {
        let state = State::create(StateConfig::default()).await.unwrap();
        let mut transaction = Transaction::new(&state);
        
        // 添加测试步骤
        transaction.set_meta("test", "value");
        
        let result = state.apply(transaction).await.unwrap();
        assert_eq!(result.state.version, 2);
    }
}
```

### 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_plugin_integration() {
        // 创建带插件的状态
        let plugin = create_test_plugin();
        let config = StateConfig {
            plugins: Some(vec![Arc::new(plugin)]),
            ..Default::default()
        };
        
        let state = State::create(config).await.unwrap();
        
        // 测试插件功能
        let mut transaction = Transaction::new(&state);
        transaction.set_meta("plugin_test", "value");
        
        let result = state.apply(transaction).await.unwrap();
        assert!(result.state.has_field("test_plugin"));
    }
}
```

## 🔍 监控和调试

### 性能监控

```rust
use mf_state::{State, Transaction};
use std::time::Instant;

async fn monitor_transaction_performance(state: &State, transaction: Transaction) {
    let start = Instant::now();
    
    let result = state.apply(transaction).await.unwrap();
    
    let duration = start.elapsed();
    tracing::info!(
        "事务处理完成 - 版本: {}, 耗时: {:?}",
        result.state.version,
        duration
    );
}
```

### 状态调试

```rust
use mf_state::State;

fn debug_state(state: &State) {
    tracing::debug!("状态信息:");
    tracing::debug!("  版本: {}", state.version);
    tracing::debug!("  字段数量: {}", state.fields_instances.len());
    tracing::debug!("  插件数量: {}", state.plugins().len());
    tracing::debug!("  文档节点数: {}", state.doc().len());
}
```

## 📚 API 参考

### 核心类型

- **`State`**: 主状态管理结构体
- **`StateConfig`**: 状态配置结构体
- **`Transaction`**: 事务处理结构体
- **`Plugin`**: 插件结构体
- **`Resource`**: 资源特征
- **`ResourceTable`**: 资源表结构体
- **`GothamState`**: Gotham 框架状态

### 主要方法

#### State
- `create(config)`: 创建新状态
- `apply(transaction)`: 应用事务
- `get_field(name)`: 获取字段
- `serialize()`: 序列化状态
- `deserialize(data, config)`: 反序列化状态

#### Transaction
- `new(state)`: 创建新事务
- `add_node(parent_id, nodes)`: 添加节点
- `remove_node(parent_id, node_ids)`: 删除节点
- `set_node_attribute(id, values)`: 设置节点属性
- `add_mark(id, marks)`: 添加标记
- `remove_mark(id, mark_types)`: 删除标记
- `set_meta(key, value)`: 设置元数据
- `get_meta(key)`: 获取元数据

#### Plugin
- `new(spec)`: 创建新插件
- `get_state(state)`: 获取插件状态
- `apply_filter_transaction(tr, state)`: 应用事务过滤
- `apply_append_transaction(trs, old_state, new_state)`: 应用事务追加

## 🤝 贡献指南

我们欢迎社区贡献！请查看以下指南：

1. **代码风格**: 遵循 Rust 标准编码规范
2. **测试覆盖**: 为新功能添加相应的测试
3. **文档更新**: 更新相关文档和示例
4. **性能考虑**: 考虑性能影响和优化

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🔗 相关链接

- [ModuForge-RS 主页](https://github.com/moduforge/moduforge-rs)
- [API 文档](https://docs.rs/moduforge-state)
- [示例项目](https://github.com/moduforge/moduforge-rs/tree/main/demo)
- [问题反馈](https://github.com/moduforge/moduforge-rs/issues) 