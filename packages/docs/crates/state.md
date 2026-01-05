# moduforge-state 文档

`moduforge-state` 提供 ModuForge-RS 的泛型状态管理、插件系统和资源管理功能。

## 概述

### 核心功能

- **泛型状态管理**：基于 DataContainer 和 SchemaDefinition 的泛型状态系统
- **插件系统**：可扩展的异步插件架构
- **资源管理**：全局资源表和类型安全的资源访问
- **事务处理**：完整的事务应用和回滚机制
- **版本管理**：全局自增的状态版本号
- **持久化**：状态序列化和反序列化支持

## 安装

```toml
[dependencies]
moduforge-state = "0.7.0"
moduforge-model = "0.7.0"
moduforge-transform = "0.7.0"
```

## 泛型状态系统

### StateGeneric 结构

```rust
use mf_model::rpds::HashTrieMapSync;

/// 泛型状态结构，支持自定义容器和模式
pub struct StateGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 配置信息，包含插件管理器和模式定义
    pub config: Arc<ConfigurationGeneric<C, S>>,

    /// 插件状态字段实例
    pub fields_instances: Arc<HashTrieMapSync<String, Arc<dyn Resource>>>,

    /// 节点池（文档容器）
    pub node_pool: Arc<C>,

    /// 状态版本号，用于追踪变更
    pub version: u64,
}

/// 默认的 State 实现（NodePool + Schema）
pub type State = StateGeneric<NodePool, Schema>;
```

### 配置结构

```rust
/// 泛型配置结构
pub struct ConfigurationGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 插件管理器
    pub plugin_manager: PluginManagerGeneric<C, S>,

    /// 文档实例（可选）
    pub doc: Option<Arc<C>>,

    /// 文档结构定义
    pub schema: Arc<S>,

    /// 全局资源管理器
    pub resource_manager: Arc<GlobalResourceManager>,
}
```

### 创建状态

```rust
use mf_state::{State, StateConfig};
use mf_model::{NodePool, Schema};
use std::sync::Arc;

// 使用 StateConfig 创建状态
let state_config = StateConfig {
    schema: Some(Arc::new(schema)),
    doc: Some(Arc::new(node_pool)),
    stored_marks: None,
    plugins: Some(vec![plugin1, plugin2]),
    resource_manager: Some(Arc::new(resource_manager)),
};

// 异步创建状态
let state = State::create(state_config).await?;

// 泛型创建（用于自定义容器）
let generic_state = StateGeneric::new_generic(
    Arc::new(config),
    Arc::new(custom_container)
)?;
```

### 状态操作

```rust
// 获取文档
let doc: Arc<NodePool> = state.doc();

// 获取模式定义
let schema: Arc<Schema> = state.schema();

// 获取资源管理器
let resource_manager = state.resource_manager();

// 创建新事务
let tr = state.tr();

// 异步应用事务，生成新状态
let result = state.apply(transaction).await?;
let new_state = result.state;
let applied_transactions = result.transactions;

// 获取插件列表（按优先级排序）
let plugins = state.sorted_plugins().await;

// 获取版本号
let version = state.version;
```

## 事务系统

### TransactionGeneric 结构

```rust
/// 泛型事务结构
pub struct TransactionGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 事务ID
    pub id: String,

    /// 变换步骤
    pub steps: Vec<StepGeneric<C, S>>,

    /// 反向步骤（用于撤销）
    pub invert_steps: Vec<StepGeneric<C, S>>,

    /// 文档容器
    pub doc: Arc<C>,

    /// 模式定义
    pub schema: Arc<S>,
}

/// 默认的 Transaction 实现
pub type Transaction = TransactionGeneric<NodePool, Schema>;
```

### 事务结果

```rust
/// 事务应用结果
pub struct TransactionResultGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 新的状态
    pub state: Arc<StateGeneric<C, S>>,

    /// 应用的事务列表（可能包含插件追加的事务）
    pub transactions: Vec<Arc<TransactionGeneric<C, S>>>,
}
```

### 事务应用流程

```rust
// 创建事务
let mut tr = state.tr();

// 添加步骤
tr.add_step(step);

// 应用事务
let result = state.apply(tr).await?;

// 内部流程：
// 1. filter_transaction - 所有插件过滤事务
// 2. apply_inner - 应用事务到文档，创建新状态
// 3. append_transaction - 插件可追加新事务
// 4. 循环处理追加的事务直到稳定
// 5. 返回最终状态和所有应用的事务
```

## 插件系统

### PluginGeneric 特征

```rust
use async_trait::async_trait;

/// 泛型插件特征
#[async_trait]
pub trait PluginGeneric<C, S>: Send + Sync
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 插件唯一标识
    fn key(&self) -> String;

    /// 插件优先级（用于排序）
    fn priority(&self) -> i32 { 0 }

    /// 过滤事务（返回 false 阻止事务）
    async fn apply_filter_transaction(
        &self,
        tr: &TransactionGeneric<C, S>,
        state: &StateGeneric<C, S>
    ) -> bool {
        true
    }

    /// 追加事务（在主事务后追加新事务）
    async fn append_transaction(
        &self,
        prev_state: &StateGeneric<C, S>,
        new_state: &StateGeneric<C, S>,
        transactions: &[Arc<TransactionGeneric<C, S>>],
        start: usize
    ) -> Option<Arc<TransactionGeneric<C, S>>> {
        None
    }
}
```

### 状态字段

```rust
/// 插件状态字段
pub trait StateField: Send + Sync {
    type Value: Resource;

    /// 初始化状态字段
    async fn init(
        &self,
        config: &StateConfig,
        state: &State
    ) -> Arc<Self::Value>;

    /// 应用事务到状态字段
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<Self::Value>,
        prev_state: &State,
        new_state: &State
    ) -> Arc<Self::Value>;
}
```

## Price-RS 实例

### PriceAggregationPlugin

```rust
use mf_state::{PluginGeneric, StateGeneric};
use price_rs::{PriceContainer, PriceSchema, PriceCalculator};

/// 价格聚合插件
pub struct PriceAggregationPlugin {
    calculator: Arc<PriceCalculator>,
}

#[async_trait]
impl PluginGeneric<PriceContainer, PriceSchema> for PriceAggregationPlugin {
    fn key(&self) -> String {
        "price_aggregation".to_string()
    }

    fn priority(&self) -> i32 {
        -100 // 低优先级，最后执行
    }

    async fn append_transaction(
        &self,
        prev_state: &StateGeneric<PriceContainer, PriceSchema>,
        new_state: &StateGeneric<PriceContainer, PriceSchema>,
        transactions: &[Arc<TransactionGeneric<PriceContainer, PriceSchema>>],
        start: usize
    ) -> Option<Arc<TransactionGeneric<PriceContainer, PriceSchema>>> {
        // 检查是否有新的价格节点
        if !self.has_price_changes(&transactions[start..]) {
            return None;
        }

        // 创建聚合事务
        let mut tr = new_state.tr_generic();

        // 计算总价
        let total = self.calculator.calculate_total(&new_state.node_pool).await;

        // 更新总价节点
        tr.update_node("total_price", |node| {
            node.attrs.insert("value".into(), total.into());
        });

        Some(Arc::new(tr))
    }
}
```

### 历史管理插件

```rust
use mf_state::{StateField, Resource};

/// 历史状态字段
pub struct HistoryField {
    max_depth: usize,
}

#[async_trait]
impl StateField for HistoryField {
    type Value = HistoryState;

    async fn init(
        &self,
        config: &StateConfig,
        state: &State
    ) -> Arc<Self::Value> {
        Arc::new(HistoryState {
            done: Vec::new(),
            undone: Vec::new(),
            max_depth: self.max_depth,
        })
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<Self::Value>,
        prev_state: &State,
        new_state: &State
    ) -> Arc<Self::Value> {
        let mut history = (*value).clone();

        // 添加到历史记录
        history.done.push(HistoryItem {
            transaction: Arc::new(tr.clone()),
            state_before: Arc::new(prev_state.clone()),
            state_after: Arc::new(new_state.clone()),
        });

        // 清空重做栈
        history.undone.clear();

        // 限制历史深度
        if history.done.len() > self.max_depth {
            history.done.remove(0);
        }

        Arc::new(history)
    }
}
```

## 资源管理

### GlobalResourceManager

```rust
/// 全局资源管理器
pub struct GlobalResourceManager {
    resources: DashMap<String, Arc<dyn Resource>>,
}

impl GlobalResourceManager {
    /// 插入资源
    pub fn insert(&self, key: String, value: Arc<dyn Resource>) {
        self.resources.insert(key, value);
    }

    /// 获取资源
    pub fn get<T: Resource>(&self, key: &str) -> Option<Arc<T>> {
        self.resources
            .get(key)
            .and_then(|r| r.value().downcast_arc::<T>().cloned())
    }

    /// 删除资源
    pub fn remove(&self, key: &str) -> Option<Arc<dyn Resource>> {
        self.resources.remove(key).map(|(_, v)| v)
    }
}
```

### Resource 特征

```rust
/// 资源特征，支持类型擦除和向下转换
pub trait Resource: Send + Sync + 'static {
    /// 向下转换为具体类型
    fn downcast_arc<T: Resource>(self: Arc<Self>) -> Option<Arc<T>>;
}

// 自动实现
impl<T: Send + Sync + 'static> Resource for T {
    fn downcast_arc<U: Resource>(self: Arc<Self>) -> Option<Arc<U>> {
        // 使用 Any trait 进行类型转换
    }
}
```

## 状态持久化

### 序列化

```rust
/// 序列化状态
pub async fn serialize(&self) -> StateResult<StateSerialize>
where
    C: serde::Serialize,
{
    let mut state_fields: HashMap<String, Vec<u8>> = HashMap::new();

    // 序列化所有插件状态
    for plugin in self.plugins().await {
        if let Some(state_field) = &plugin.spec.state_field {
            if let Some(value) = self.get_field(&plugin.key) {
                if let Some(json) = state_field.serialize_erased(&value) {
                    state_fields.insert(plugin.key.clone(), json);
                }
            }
        }
    }

    // 序列化文档容器
    let container_bytes = serde_json::to_vec(&self.doc())?;

    Ok(StateSerialize {
        state_fields,
        node_pool: container_bytes,
    })
}
```

### 反序列化

```rust
/// 反序列化状态
pub async fn deserialize(
    s: &StateSerialize,
    configuration: &Configuration
) -> StateResult<State>
where
    C: serde::de::DeserializeOwned,
{
    // 反序列化文档容器
    let container: Arc<NodePool> = serde_json::from_slice(&s.node_pool)?;

    // 创建状态
    let mut state = State::new_generic(
        Arc::new(configuration.clone()),
        container
    )?;

    // 恢复插件状态
    let mut fields = HashTrieMapSync::new_sync();
    for plugin in configuration.plugin_manager.get_sorted_plugins().await {
        if let Some(state_field) = &plugin.spec.state_field {
            if let Some(value) = s.state_fields.get(&plugin.key) {
                if let Some(p_state) = state_field.deserialize_erased(value) {
                    fields.insert_mut(plugin.key.clone(), p_state);
                }
            }
        }
    }

    state.fields_instances = Arc::new(fields);
    Ok(state)
}
```

## 最佳实践

### 1. 插件设计原则

```rust
// ✅ 好：使用泛型保持灵活性
impl<C, S> PluginGeneric<C, S> for MyPlugin
where
    C: DataContainer,
    S: SchemaDefinition<Container = C>,
{
    // 实现
}

// ✅ 好：合理设置优先级
fn priority(&self) -> i32 {
    match self.plugin_type {
        PluginType::Validation => 100,  // 高优先级，先验证
        PluginType::History => 0,       // 中等优先级
        PluginType::Analytics => -100,  // 低优先级，最后执行
    }
}
```

### 2. 事务处理

```rust
// ✅ 好：批量操作使用单个事务
let mut tr = state.tr();
for node in nodes {
    tr.add_step(create_step(node));
}
let result = state.apply(tr).await?;

// ❌ 差：每个操作单独事务
for node in nodes {
    let tr = state.tr();
    tr.add_step(create_step(node));
    state = state.apply(tr).await?.state;  // 低效
}
```

### 3. 资源管理

```rust
// ✅ 好：使用 Arc 避免克隆
let resource = Arc::new(HeavyResource::new());
state.resource_manager().insert("heavy", resource);

// ✅ 好：类型安全的访问
if let Some(resource) = state.resource_manager().get::<HeavyResource>("heavy") {
    resource.process();
}
```

### 4. 状态不可变性

```rust
// ✅ 好：始终通过事务修改状态
let new_state = state.apply(transaction).await?.state;

// ❌ 差：尝试直接修改状态字段
// state.node_pool = new_pool;  // 编译错误！状态是不可变的
```

## 完整示例

### Price-RS 状态管理

```rust
use mf_state::{State, StateConfig, PluginGeneric};
use price_rs::{PriceSchema, PriceContainer, PricePlugin};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建模式和容器
    let schema = Arc::new(PriceSchema::new());
    let container = Arc::new(PriceContainer::new());

    // 配置插件
    let plugins = vec![
        Arc::new(PricePlugin::new()) as Arc<dyn PluginGeneric<_, _>>,
        Arc::new(HistoryPlugin::new()),
        Arc::new(ValidationPlugin::new()),
    ];

    // 创建状态配置
    let config = StateConfig {
        schema: Some(schema),
        doc: Some(container),
        stored_marks: None,
        plugins: Some(plugins),
        resource_manager: None,
    };

    // 创建状态
    let state = State::create(config).await?;

    // 创建并应用事务
    let mut tr = state.tr();

    // 插入价格节点
    tr.insert_node("price_1", PriceNode {
        value: 100.0,
        unit: "元/平米".into(),
    });

    // 应用事务
    let result = state.apply(tr).await?;
    let new_state = result.state;

    // 插件会自动聚合价格
    println!("状态版本: {}", new_state.version);
    println!("应用了 {} 个事务", result.transactions.len());

    // 序列化状态
    let serialized = new_state.serialize().await?;

    // 保存到文件
    std::fs::write("state.json", serde_json::to_vec(&serialized)?)?;

    Ok(())
}
```

## 性能优化

### 1. 使用持久化数据结构

```rust
// HashTrieMapSync 提供高效的不可变更新
pub fields_instances: Arc<HashTrieMapSync<String, Arc<dyn Resource>>>,
```

### 2. 异步插件处理

```rust
// 插件方法都是异步的，支持并发处理
async fn apply_filter_transaction(...) -> bool
async fn append_transaction(...) -> Option<...>
```

### 3. Arc 引用计数

```rust
// 避免深拷贝，使用 Arc 共享不可变数据
pub node_pool: Arc<C>,
pub schema: Arc<S>,
```

## 下一步

- 查看 [moduforge-transform](./transform.md) 了解事务和变换系统
- 查看 [moduforge-model](./model.md) 了解文档模型和节点系统
- 查看 [moduforge-core](./core.md) 了解运行时集成
- 浏览 [Price-RS 项目](https://github.com/LiRenTech/price-rs) 查看实际应用