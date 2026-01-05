# 状态管理

ModuForge-RS 的状态管理系统基于泛型设计，提供灵活、类型安全的文档状态管理机制。

## 核心架构

### 泛型状态定义

```rust
use mf_model::traits::{DataContainer, SchemaDefinition};
use std::sync::Arc;

/// 泛型状态结构
pub struct StateGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 节点池（文档数据）
    pub node_pool: Arc<C>,

    /// Schema定义
    pub schema: Arc<S>,

    /// 选区信息
    pub selection: Selection,

    /// 插件列表
    pub plugins: Vec<Arc<dyn Plugin>>,

    /// 资源表
    pub resources: ResourceTable,

    /// 版本号
    pub version: u64,

    /// 配置
    pub config: Arc<StateConfig>,
}

// 默认类型别名
pub type State = StateGeneric<NodePool, Schema>;
```

## 事务系统

### Transaction 定义

事务封装了对状态的原子性修改：

```rust
use mf_transform::step::StepGeneric;
use std::collections::HashMap;
use serde_json::Value;

pub struct TransactionGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 事务唯一标识
    pub id: Box<str>,

    /// 步骤列表
    pub steps: Vec<Arc<dyn StepGeneric<C, S>>>,

    /// 逆步骤列表（用于撤销）
    pub invert_steps: Vec<Arc<dyn StepGeneric<C, S>>>,

    /// 元数据
    pub metadata: HashMap<String, Value>,

    /// 时间戳
    pub timestamp: i64,
}

// 默认类型别名
pub type Transaction = TransactionGeneric<NodePool, Schema>;
```

### 事务操作方法

```rust
impl Transaction {
    /// 添加节点
    pub fn add_node(
        &mut self,
        parent_id: Box<str>,
        nodes: Vec<Node>
    ) -> Result<()> {
        let step = AddNodeStep {
            parent_id,
            nodes,
            index: None,
        };
        self.steps.push(Arc::new(step));
        Ok(())
    }

    /// 更新节点属性
    pub fn update_node(
        &mut self,
        node_id: Box<str>,
        attrs: HashMap<String, Value>
    ) -> Result<()> {
        let step = UpdateNodeAttrsStep {
            node_id,
            attrs,
            old_attrs: None,
        };
        self.steps.push(Arc::new(step));
        Ok(())
    }

    /// 删除节点
    pub fn remove_node(
        &mut self,
        node_id: Box<str>
    ) -> Result<()> {
        let step = RemoveNodeStep {
            node_id,
            remove_children: true,
        };
        self.steps.push(Arc::new(step));
        Ok(())
    }
}
```

## 插件系统

### Plugin Trait

插件提供状态扩展能力：

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Plugin: Send + Sync {
    /// 插件唯一键
    fn key(&self) -> &str;

    /// 初始化插件
    async fn init(&mut self, state: &State) -> Result<()> {
        Ok(())
    }

    /// 事务应用时调用
    async fn apply(
        &mut self,
        tr: &Transaction,
        old_state: &State
    ) -> Result<()>;

    /// 销毁插件
    fn destroy(&mut self) {
        // 清理资源
    }
}
```

### 实际插件实现示例

```rust
use mf_state::plugin::Plugin;

/// 历史记录插件
pub struct HistoryPlugin {
    max_steps: usize,
    history: Vec<Transaction>,
    current: usize,
}

#[async_trait]
impl Plugin for HistoryPlugin {
    fn key(&self) -> &str {
        "history"
    }

    async fn apply(
        &mut self,
        tr: &Transaction,
        _old_state: &State
    ) -> Result<()> {
        // 截断未来的历史
        self.history.truncate(self.current);

        // 添加新事务
        self.history.push(tr.clone());
        self.current += 1;

        // 限制历史长度
        if self.history.len() > self.max_steps {
            self.history.remove(0);
            self.current -= 1;
        }

        Ok(())
    }
}
```

## 资源管理

### ResourceTable

资源表提供类型安全的依赖注入：

```rust
use std::any::{Any, TypeId};
use dashmap::DashMap;

pub struct ResourceTable {
    resources: DashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ResourceTable {
    /// 插入资源
    pub fn insert<T: Resource>(&self, resource: T) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.resources
            .insert(type_id, Box::new(resource))
            .and_then(|boxed| boxed.downcast().ok())
            .map(|boxed| *boxed)
    }

    /// 获取资源
    pub fn get<T: Resource>(&self) -> Option<impl Deref<Target = T>> {
        let type_id = TypeId::of::<T>();
        self.resources.get(&type_id)
            .and_then(|entry| {
                entry.value()
                    .downcast_ref::<T>()
                    .map(|_| ResourceRef { entry })
            })
    }

    /// 移除资源
    pub fn remove<T: Resource>(&self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.resources
            .remove(&type_id)
            .and_then(|(_, boxed)| boxed.downcast().ok())
            .map(|boxed| *boxed)
    }
}

// Resource trait
pub trait Resource: Any + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

## Price-RS 实际应用

### 复杂业务状态管理

Price-RS 展示了如何在实际项目中管理复杂业务状态：

```rust
use mf_state::{Resource, Plugin};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 工程造价状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCalculationState {
    /// 价格缓存
    pub price_cache: HashMap<Box<str>, PriceInfo>,

    /// 依赖关系图
    pub dependencies: HashMap<Box<str>, Vec<Box<str>>>,

    /// 需要重新计算的节点
    pub dirty_nodes: HashSet<Box<str>>,

    /// 计算配置
    pub config: CalculationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceInfo {
    pub base_price: f64,
    pub total_price: f64,
    pub breakdown: PriceBreakdown,
    pub last_updated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceBreakdown {
    pub labor: f64,        // 人工费
    pub material: f64,     // 材料费
    pub machinery: f64,    // 机械费
    pub management: f64,   // 管理费
    pub profit: f64,       // 利润
    pub tax: f64,          // 税金
}

impl Resource for PriceCalculationState {}

/// 价格计算插件
pub struct PriceCalculationPlugin {
    state: Arc<RwLock<PriceCalculationState>>,
}

#[async_trait]
impl Plugin for PriceCalculationPlugin {
    fn key(&self) -> &str {
        "price_calculation"
    }

    async fn apply(
        &mut self,
        tr: &Transaction,
        old_state: &State
    ) -> Result<()> {
        let mut calc_state = self.state.write().await;

        // 分析事务，找出受影响的节点
        for step in &tr.steps {
            self.analyze_step(step, &mut calc_state)?;
        }

        // 获取新状态的文档
        let new_doc = tr.apply_to(&old_state.node_pool)?;

        // 重新计算脏节点
        self.recalculate_prices(&mut calc_state, &new_doc).await?;

        Ok(())
    }
}

impl PriceCalculationPlugin {
    fn analyze_step(
        &self,
        step: &Arc<dyn StepGeneric<NodePool, Schema>>,
        state: &mut PriceCalculationState
    ) -> Result<()> {
        // 根据步骤类型标记受影响的节点
        if let Some(update) = step.as_any().downcast_ref::<UpdateNodeAttrsStep>() {
            // 属性更新，标记节点为脏
            state.dirty_nodes.insert(update.node_id.clone());

            // 标记所有依赖此节点的节点
            if let Some(deps) = state.dependencies.get(&update.node_id) {
                for dep in deps {
                    state.dirty_nodes.insert(dep.clone());
                }
            }
        }

        Ok(())
    }

    async fn recalculate_prices(
        &self,
        state: &mut PriceCalculationState,
        doc: &NodePool
    ) -> Result<()> {
        // 拓扑排序，确保依赖顺序
        let sorted_nodes = self.topological_sort(&state.dirty_nodes, &state.dependencies);

        for node_id in sorted_nodes {
            if let Some(node) = doc.get_node(&node_id) {
                let price_info = self.calculate_node_price(&node, state, doc).await?;
                state.price_cache.insert(node_id, price_info);
            }
        }

        // 清空脏节点集合
        state.dirty_nodes.clear();

        Ok(())
    }

    async fn calculate_node_price(
        &self,
        node: &Node,
        state: &PriceCalculationState,
        doc: &NodePool
    ) -> Result<PriceInfo> {
        let mut breakdown = PriceBreakdown::default();

        // 根据节点类型执行不同的计算逻辑
        match node.r#type.as_str() {
            "QdNode" => {
                // 清单节点：计算基础价格
                let quantity = node.attrs.get("quantity")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                let unit_price = node.attrs.get("unit_price")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                breakdown.material = quantity * unit_price * 0.5;
                breakdown.labor = quantity * unit_price * 0.3;
                breakdown.machinery = quantity * unit_price * 0.1;
            }

            "FbNode" => {
                // 分部节点：汇总子节点价格
                for child_id in &node.children {
                    if let Some(child_price) = state.price_cache.get(child_id) {
                        breakdown.material += child_price.breakdown.material;
                        breakdown.labor += child_price.breakdown.labor;
                        breakdown.machinery += child_price.breakdown.machinery;
                    }
                }
            }

            _ => {}
        }

        // 计算管理费、利润和税金
        breakdown.management = (breakdown.labor + breakdown.machinery) * state.config.management_rate;
        breakdown.profit = (breakdown.material + breakdown.labor) * state.config.profit_rate;
        breakdown.tax = (breakdown.material + breakdown.labor + breakdown.machinery
                        + breakdown.management + breakdown.profit) * state.config.tax_rate;

        let total = breakdown.material + breakdown.labor + breakdown.machinery
                   + breakdown.management + breakdown.profit + breakdown.tax;

        Ok(PriceInfo {
            base_price: breakdown.material + breakdown.labor + breakdown.machinery,
            total_price: total,
            breakdown,
            last_updated: chrono::Utc::now().timestamp(),
        })
    }
}
```

## 选区管理

### Selection 结构

选区表示文档中的编辑位置：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    /// 锚点位置
    pub anchor: Position,

    /// 头部位置
    pub head: Position,

    /// 选区类型
    pub r#type: SelectionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// 节点ID
    pub node_id: Box<str>,

    /// 节点内偏移
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionType {
    /// 文本选区
    Text,

    /// 节点选区
    Node,

    /// 单元格选区（表格）
    Cell,
}

impl Selection {
    /// 映射选区通过事务
    pub fn map_through_transaction(&self, tr: &Transaction) -> Selection {
        let mut selection = self.clone();

        for step in &tr.steps {
            selection = selection.map_through_step(step);
        }

        selection
    }

    /// 映射选区通过单个步骤
    pub fn map_through_step(&self, step: &dyn StepGeneric<NodePool, Schema>) -> Selection {
        // 根据步骤类型调整选区位置
        // 例如：如果在选区前插入内容，需要调整偏移量
        self.clone() // 简化实现
    }
}
```

## 状态配置

### StateConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    /// 启用的插件
    pub plugins: Vec<String>,

    /// 历史记录配置
    pub history: HistoryConfig,

    /// 性能配置
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// 最大历史步骤数
    pub max_steps: usize,

    /// 是否追踪远程变更
    pub track_remote: bool,

    /// 历史压缩策略
    pub compression: CompressionStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 批量操作阈值
    pub batch_threshold: usize,

    /// 缓存大小
    pub cache_size: usize,

    /// 异步操作超时
    pub async_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionStrategy {
    /// 不压缩
    None,

    /// 合并相邻的相似操作
    MergeSimilar,

    /// 定期创建快照
    Snapshot { interval: usize },
}
```

## 状态管理最佳实践

### 1. 插件设计原则

```rust
// ✅ 好的插件设计
pub struct WellDesignedPlugin {
    // 使用 Arc 共享状态
    state: Arc<RwLock<PluginState>>,

    // 配置不可变
    config: PluginConfig,
}

impl Plugin for WellDesignedPlugin {
    async fn apply(&mut self, tr: &Transaction, _old_state: &State) -> Result<()> {
        // 1. 快速过滤不相关的事务
        if !self.should_process(tr) {
            return Ok(());
        }

        // 2. 异步处理，不阻塞主线程
        let state = self.state.clone();
        tokio::spawn(async move {
            // 执行耗时操作
            process_transaction(state, tr).await;
        });

        Ok(())
    }
}
```

### 2. 资源管理

```rust
// 注册资源
state.resources.insert(DatabaseConnection::new());
state.resources.insert(CacheManager::new());

// 在插件中使用资源
impl Plugin for DataPlugin {
    async fn apply(&mut self, tr: &Transaction, old_state: &State) -> Result<()> {
        // 获取资源
        let db = old_state.resources.get::<DatabaseConnection>()
            .ok_or_else(|| anyhow!("Database not available"))?;

        // 使用资源
        db.save_transaction(tr).await?;

        Ok(())
    }
}
```

### 3. 事务优化

```rust
// 批量操作优化
pub fn batch_update_nodes(
    tr: &mut Transaction,
    updates: Vec<(Box<str>, HashMap<String, Value>)>
) -> Result<()> {
    // 使用批量步骤而不是多个单独步骤
    let batch_step = BatchStep {
        steps: updates.into_iter()
            .map(|(id, attrs)| {
                Box::new(UpdateNodeAttrsStep {
                    node_id: id,
                    attrs,
                    old_attrs: None,
                }) as Box<dyn StepGeneric<NodePool, Schema>>
            })
            .collect(),
    };

    tr.steps.push(Arc::new(batch_step));
    Ok(())
}
```

### 4. 状态快照

```rust
/// 创建状态快照用于恢复
pub struct StateSnapshot {
    pub node_pool: NodePool,
    pub version: u64,
    pub timestamp: i64,
    pub metadata: HashMap<String, Value>,
}

impl State {
    pub fn create_snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            node_pool: (*self.node_pool).clone(),
            version: self.version,
            timestamp: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        }
    }

    pub fn restore_from_snapshot(snapshot: StateSnapshot, config: StateConfig) -> Self {
        State {
            node_pool: Arc::new(snapshot.node_pool),
            schema: Arc::new(Schema::default()),
            selection: Selection::default(),
            plugins: Vec::new(),
            resources: ResourceTable::new(),
            version: snapshot.version,
            config: Arc::new(config),
        }
    }
}
```

## 总结

ModuForge-RS 的状态管理系统提供了：

1. **泛型设计**：支持自定义容器和Schema
2. **事务机制**：原子性状态修改
3. **插件系统**：可扩展的状态处理
4. **资源管理**：类型安全的依赖注入
5. **选区管理**：精确的编辑位置跟踪
6. **灵活配置**：可调节的性能和行为

通过这些机制，ModuForge-RS 能够支持复杂的业务场景，如 Price-RS 项目中的工程造价计算，同时保持代码的可维护性和扩展性。