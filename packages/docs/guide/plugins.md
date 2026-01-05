# 插件系统

ModuForge-RS 的插件系统提供了强大的扩展机制，让您可以自定义编辑器的各个方面。

## 插件基础

### 插件结构

每个插件包含：

- **元数据（Metadata）**：插件信息
- **状态字段（StateField）**：插件私有状态
- **钩子函数（Hooks）**：生命周期回调
- **命令（Commands）**：插件提供的操作

### 快速创建插件

使用 `mf_plugin!` 宏快速创建插件：

```rust
use mf_macro::{mf_plugin, mf_meta};
use mf_state::{State, Transaction};

mf_plugin!(
    my_plugin,

    // 插件元数据
    metadata = mf_meta!(
        version = "1.0.0",
        description = "我的第一个插件",
        author = "开发者",
        tags = ["example", "demo"]
    ),

    // 事务过滤钩子（事务应用前）
    filter_transaction = async |trs: Vec<Arc<Transaction>>,
                                old_state: &Arc<State>,
                                new_state: &Arc<State>| {
        // 可以修改、过滤或拒绝事务
        Ok(trs)
    },

    // 事务追加钩子（事务应用后）
    append_transaction = async |trs: &[Arc<Transaction>],
                                old_state: &Arc<State>,
                                new_state: &Arc<State>| {
        // 可以返回额外的事务
        Ok(None)
    },

    docs = "插件的详细文档"
);
```

## 基于 price-rs 的实际插件示例

### 项目结构插件

来自 price-rs 的项目结构管理插件：

```rust
use mf_macro::{mf_plugin, mf_meta};
use mf_model::{node_pool::NodePool, schema::Schema};
use mf_state::state::StateGeneric;
use mf_state::Transaction;
use std::sync::Arc;

mf_plugin!(
    project_structure,

    metadata = mf_meta!(
        version = "1.0.0",
        description = "项目结构插件",
        author = "moduforge",
        tags = ["project_structure", "hierarchy"]
    ),

    // 在事务应用后验证项目结构
    append_transaction = async |trs: &[Arc<Transaction>],
                                _old_state: &Arc<StateGeneric<NodePool, Schema>>,
                                new_state: &Arc<StateGeneric<NodePool, Schema>>| {
        // 检查是否有结构变化
        for tr in trs {
            for step in tr.steps() {
                if let Step::AddNode { parent, nodes } = step {
                    // 验证新节点符合项目结构规则
                    for node in nodes {
                        if !validate_project_node(parent, node, new_state).await {
                            return Err(StateError::ValidationFailed(
                                format!("节点 {} 不符合项目结构规则", node.id)
                            ));
                        }
                    }
                }
            }
        }

        Ok(None)
    },

    docs = "管理和验证项目的树形结构，确保节点层级关系正确"
);

async fn validate_project_node(
    parent_id: &NodeId,
    node: &Node,
    state: &State
) -> bool {
    let parent = state.doc().get_node(parent_id);

    match (parent.node_type.as_str(), node.node_type.as_str()) {
        ("GCXM", "DXGC") => true,  // 工程项目可包含单项工程
        ("GCXM", "DWGC") => true,  // 工程项目可包含单位工程
        ("DXGC", "DWGC") => true,  // 单项工程可包含单位工程
        ("DXGC", "DXGC") => true,  // 单项工程可递归包含
        _ => false,
    }
}
```

### 带状态的插件

管理复杂状态的插件示例：

```rust
use mf_state::plugin::{StateFieldGeneric, PluginTraitGeneric};
use mf_state::resource::Resource;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// 定义插件状态
#[derive(Clone, Serialize, Deserialize)]
pub struct ManifestOrderState {
    // 全局排序
    pub global_order: Vec<NodeId>,
    // 排序索引
    pub rank_map: HashMap<NodeId, usize>,
    // 脏标记
    pub needs_reorder: bool,
}

impl Resource for ManifestOrderState {}

// StateField 实现
pub struct ManifestOrderField;

#[async_trait]
impl StateFieldGeneric<NodePool, Schema> for ManifestOrderField {
    type Value = ManifestOrderState;

    async fn init(
        &self,
        _config: &StateConfigGeneric<NodePool, Schema>,
        instance: &StateGeneric<NodePool, Schema>,
    ) -> Arc<Self::Value> {
        let mut state = ManifestOrderState {
            global_order: Vec::new(),
            rank_map: HashMap::new(),
            needs_reorder: false,
        };

        // 初始化排序
        self.build_initial_order(instance.doc(), &mut state);

        Arc::new(state)
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<Self::Value>,
        _old_state: &StateGeneric<NodePool, Schema>,
        new_state: &StateGeneric<NodePool, Schema>,
    ) -> Arc<Self::Value> {
        let mut state = (*value).clone();

        // 检查是否需要重新排序
        for step in tr.steps() {
            match step {
                Step::AddNode { .. } | Step::RemoveNode { .. } | Step::MoveNode { .. } => {
                    state.needs_reorder = true;
                    break;
                }
                _ => {}
            }
        }

        if state.needs_reorder {
            self.reorder(&mut state, new_state.doc()).await;
            state.needs_reorder = false;
        }

        Arc::new(state)
    }

    fn serialize(&self, value: &Arc<Self::Value>) -> Option<Vec<u8>> {
        bincode::serialize(&**value).ok()
    }

    fn deserialize(&self, data: &[u8]) -> Option<Arc<Self::Value>> {
        bincode::deserialize(data).ok().map(Arc::new)
    }
}

impl ManifestOrderField {
    fn build_initial_order(&self, doc: &Document, state: &mut ManifestOrderState) {
        let mut order = Vec::new();

        // 深度优先遍历收集所有清单节点
        fn collect_manifests(node: &Node, order: &mut Vec<NodeId>) {
            if node.node_type == "qd" {
                order.push(node.id.clone());
            }
            for child in node.children() {
                collect_manifests(child, order);
            }
        }

        collect_manifests(doc.root(), &mut order);

        // 构建排序索引
        for (rank, node_id) in order.iter().enumerate() {
            state.rank_map.insert(node_id.clone(), rank);
        }

        state.global_order = order;
    }

    async fn reorder(&self, state: &mut ManifestOrderState, doc: &Document) {
        // 重新构建排序
        state.global_order.clear();
        state.rank_map.clear();
        self.build_initial_order(doc, state);
    }
}

// 注册插件
mf_plugin!(
    manifest_order_plugin,

    metadata = mf_meta!(
        version = "1.0.0",
        description = "清单全局排序管理"
    ),

    state_field = ManifestOrderField {},

    // 将排序信息写回文档
    append_transaction = async |trs: &[Arc<Transaction>],
                                _old_state: &Arc<State>,
                                new_state: &Arc<State>| {
        let order_state = new_state.get_field::<ManifestOrderField>().await;

        if order_state.needs_reorder {
            let mut tr = new_state.tr();

            // 更新所有节点的 rank 属性
            for (node_id, rank) in &order_state.rank_map {
                tr.set_node_attribute(
                    node_id.clone(),
                    hashmap!{ "rank" => json!(rank) }
                )?;
            }

            return Ok(Some(tr));
        }

        Ok(None)
    }
);
```

## 插件生命周期

### 初始化阶段

```rust
pub struct LifecyclePlugin;

impl PluginTrait for LifecyclePlugin {
    fn init(&self, state: &State) -> Result<()> {
        println!("插件初始化");

        // 验证环境
        self.validate_environment()?;

        // 注册命令
        self.register_commands(state)?;

        // 初始化资源
        self.setup_resources()?;

        Ok(())
    }

    fn destroy(&self) -> Result<()> {
        println!("插件销毁");

        // 清理资源
        self.cleanup_resources()?;

        Ok(())
    }
}
```

### 事务处理流程

```rust
impl PluginTrait for TransactionPlugin {
    // 1. 事务过滤（应用前）
    async fn filter_transaction(
        &self,
        trs: Vec<Arc<Transaction>>,
        old_state: &Arc<State>,
        new_state: &Arc<State>,
    ) -> Result<Vec<Arc<Transaction>>> {
        let mut filtered = Vec::new();

        for tr in trs {
            // 验证事务
            if self.validate_transaction(&tr, old_state).await? {
                // 可以修改事务
                let modified = self.modify_transaction(tr)?;
                filtered.push(modified);
            }
            // 被过滤掉的事务不会被应用
        }

        Ok(filtered)
    }

    // 2. 状态更新（应用时）
    async fn apply_state_field(
        &self,
        tr: &Transaction,
        old_value: Arc<PluginState>,
        old_state: &State,
        new_state: &State,
    ) -> Arc<PluginState> {
        // 更新插件状态
        let mut new_value = (*old_value).clone();
        new_value.process_transaction(tr);
        Arc::new(new_value)
    }

    // 3. 事务追加（应用后）
    async fn append_transaction(
        &self,
        trs: &[Arc<Transaction>],
        old_state: &Arc<State>,
        new_state: &Arc<State>,
    ) -> Result<Option<Transaction>> {
        // 检查是否需要额外的事务
        if self.needs_post_processing(trs) {
            let mut tr = new_state.tr();

            // 添加衍生操作
            self.add_derived_operations(&mut tr)?;

            return Ok(Some(tr));
        }

        Ok(None)
    }
}
```

## 插件间协作

### 共享资源

```rust
use mf_state::resource::{Resource, ResourceTable};

// 定义共享资源
#[derive(Clone)]
pub struct SharedMetrics {
    pub word_count: Arc<AtomicUsize>,
    pub node_count: Arc<AtomicUsize>,
    pub last_update: Arc<RwLock<DateTime<Utc>>>,
}

impl Resource for SharedMetrics {}

// 插件 A：生产者
mf_plugin!(
    metrics_producer,

    append_transaction = async |trs, old_state, new_state| {
        // 获取或创建共享资源
        let metrics = new_state.resources()
            .get_or_insert_with::<SharedMetrics>(|| SharedMetrics {
                word_count: Arc::new(AtomicUsize::new(0)),
                node_count: Arc::new(AtomicUsize::new(0)),
                last_update: Arc::new(RwLock::new(Utc::now())),
            });

        // 更新指标
        let text = new_state.doc().to_text();
        let words = text.split_whitespace().count();
        metrics.word_count.store(words, Ordering::Relaxed);

        let nodes = new_state.doc().node_count();
        metrics.node_count.store(nodes, Ordering::Relaxed);

        *metrics.last_update.write().await = Utc::now();

        Ok(None)
    }
);

// 插件 B：消费者
mf_plugin!(
    metrics_consumer,

    append_transaction = async |trs, old_state, new_state| {
        // 读取共享资源
        if let Some(metrics) = new_state.resources().get::<SharedMetrics>() {
            let words = metrics.word_count.load(Ordering::Relaxed);
            let nodes = metrics.node_count.load(Ordering::Relaxed);

            if words > 10000 {
                // 文档过长，发出警告
                let mut tr = new_state.tr();
                tr.set_meta("warning", json!({
                    "type": "document_too_long",
                    "word_count": words,
                    "node_count": nodes
                }));
                return Ok(Some(tr));
            }
        }

        Ok(None)
    }
);
```

### 事件总线

```rust
use tokio::sync::broadcast;

// 定义事件
#[derive(Clone, Debug)]
pub enum PluginEvent {
    NodeAdded { node_id: NodeId, node_type: String },
    NodeRemoved { node_id: NodeId },
    CalculationComplete { node_id: NodeId, result: f64 },
}

// 事件总线插件
pub struct EventBusPlugin {
    sender: broadcast::Sender<PluginEvent>,
    receiver: broadcast::Receiver<PluginEvent>,
}

impl EventBusPlugin {
    pub fn new() -> Self {
        let (sender, receiver) = broadcast::channel(100);
        Self { sender, receiver }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PluginEvent> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event: PluginEvent) -> Result<()> {
        self.sender.send(event)?;
        Ok(())
    }
}

// 发布者插件
mf_plugin!(
    event_publisher,

    append_transaction = async |trs, old_state, new_state| {
        let event_bus = new_state.resources().get::<EventBusPlugin>()?;

        for tr in trs {
            for step in tr.steps() {
                if let Step::AddNode { nodes, .. } = step {
                    for node in nodes {
                        event_bus.publish(PluginEvent::NodeAdded {
                            node_id: node.id.clone(),
                            node_type: node.node_type.clone(),
                        })?;
                    }
                }
            }
        }

        Ok(None)
    }
);

// 订阅者插件
pub struct SubscriberPlugin {
    receiver: broadcast::Receiver<PluginEvent>,
}

impl SubscriberPlugin {
    pub async fn start_listening(&mut self) {
        while let Ok(event) = self.receiver.recv().await {
            match event {
                PluginEvent::NodeAdded { node_id, node_type } => {
                    println!("节点添加: {} ({})", node_id, node_type);
                }
                PluginEvent::CalculationComplete { node_id, result } => {
                    println!("计算完成: {} = {}", node_id, result);
                }
                _ => {}
            }
        }
    }
}
```

## 插件开发最佳实践

### 1. 性能优化

```rust
// 避免在每次事务都执行昂贵操作
pub struct OptimizedPlugin {
    cache: Arc<RwLock<HashMap<NodeId, CachedData>>>,
    last_update: Arc<RwLock<Instant>>,
}

impl OptimizedPlugin {
    async fn should_update(&self) -> bool {
        let last = self.last_update.read().await;

        // 限制更新频率
        last.elapsed() > Duration::from_millis(100)
    }

    async fn process_with_cache(&self, node_id: &NodeId) -> Result<f64> {
        // 先检查缓存
        if let Some(cached) = self.cache.read().await.get(node_id) {
            if cached.is_valid() {
                return Ok(cached.value);
            }
        }

        // 缓存未命中，执行计算
        let result = self.expensive_calculation(node_id).await?;

        // 更新缓存
        self.cache.write().await.insert(
            node_id.clone(),
            CachedData {
                value: result,
                timestamp: Instant::now(),
            }
        );

        Ok(result)
    }
}
```

### 2. 错误处理

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("插件未初始化")]
    NotInitialized,

    #[error("状态不一致: {0}")]
    InconsistentState(String),

    #[error("资源不可用: {0}")]
    ResourceUnavailable(String),

    #[error("验证失败: {0}")]
    ValidationFailed(String),
}

impl PluginTrait for RobustPlugin {
    async fn filter_transaction(
        &self,
        trs: Vec<Arc<Transaction>>,
        old_state: &Arc<State>,
        new_state: &Arc<State>,
    ) -> Result<Vec<Arc<Transaction>>, PluginError> {
        // 优雅的错误处理
        let validated = trs.into_iter()
            .filter_map(|tr| {
                match self.validate_transaction(&tr) {
                    Ok(true) => Some(tr),
                    Ok(false) => None,
                    Err(e) => {
                        // 记录错误但不中断流程
                        tracing::error!("事务验证失败: {}", e);
                        None
                    }
                }
            })
            .collect();

        Ok(validated)
    }
}
```

### 3. 测试插件

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mf_state::testing::*;

    #[tokio::test]
    async fn test_plugin_initialization() {
        let plugin = create_test_plugin();
        let state = create_test_state();

        // 测试初始化
        let result = plugin.init(&state);
        assert!(result.is_ok());

        // 验证状态字段
        let plugin_state = state.get_field::<TestPluginField>().await;
        assert_eq!(plugin_state.initialized, true);
    }

    #[tokio::test]
    async fn test_transaction_filtering() {
        let plugin = create_test_plugin();
        let state = create_test_state();

        // 创建测试事务
        let mut tr = state.tr();
        tr.add_node("root", vec![create_test_node()])?;

        // 测试过滤
        let result = plugin.filter_transaction(
            vec![Arc::new(tr)],
            &Arc::new(state.clone()),
            &Arc::new(state.clone())
        ).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_plugin_interaction() {
        let producer = metrics_producer::new();
        let consumer = metrics_consumer::new();

        let mut state = create_test_state();
        state.add_plugin(producer);
        state.add_plugin(consumer);

        // 触发插件交互
        let mut tr = state.tr();
        tr.add_node("root", vec![create_large_document()])?;

        let new_state = state.apply(tr).await?;

        // 验证插件间通信
        assert!(new_state.meta().contains_key("warning"));
    }
}
```

## 实际案例：price-rs 的插件系统

### 单价构成计算插件

```rust
// 完整的单价构成计算插件
mf_plugin!(
    djgc_calculation,

    metadata = mf_meta!(
        version = "2.0.0",
        description = "单价构成自动计算插件",
        author = "price-rs team"
    ),

    state_field = DjgcCalculationField {},

    filter_transaction = async |trs, old_state, new_state| {
        // 验证数值合法性
        for tr in &trs {
            for step in tr.steps() {
                if let Step::SetAttribute { attrs, .. } = step {
                    for (key, value) in attrs {
                        if key.ends_with("_price") || key.ends_with("_cost") {
                            if let Some(num) = value.as_f64() {
                                if num < 0.0 {
                                    return Err(StateError::ValidationFailed(
                                        format!("{} 不能为负数", key)
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(trs)
    },

    append_transaction = async |trs, old_state, new_state| {
        let calc_state = new_state.get_field::<DjgcCalculationField>().await;

        if !calc_state.dirty_nodes.is_empty() {
            let mut tr = new_state.tr();

            // 重新计算所有脏节点
            for node_id in &calc_state.dirty_nodes {
                if let Some(node) = new_state.doc().get_node(node_id) {
                    let result = calculate_node_cost(&node).await?;

                    tr.set_node_attribute(
                        node_id.clone(),
                        hashmap!{
                            "total_cost" => json!(result.total),
                            "labor_cost" => json!(result.labor),
                            "material_cost" => json!(result.material),
                            "machinery_cost" => json!(result.machinery),
                            "calculated_at" => json!(Utc::now())
                        }
                    )?;
                }
            }

            return Ok(Some(tr));
        }

        Ok(None)
    }
);

async fn calculate_node_cost(node: &Node) -> Result<CostResult> {
    // 复杂的成本计算逻辑
    let quantity = node.attrs.get("quantity").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let unit_price = node.attrs.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let base_cost = quantity * unit_price;

    CostResult {
        total: base_cost * 1.15,  // 加上管理费和利润
        labor: base_cost * 0.3,
        material: base_cost * 0.5,
        machinery: base_cost * 0.2,
    }
}
```

## 总结

ModuForge-RS 的插件系统提供了：

1. **灵活的扩展机制**：通过钩子函数控制各个阶段
2. **状态管理**：每个插件可以维护自己的状态
3. **插件间协作**：通过共享资源和事件总线通信
4. **类型安全**：编译时保证插件正确性
5. **高性能**：支持异步和并发操作

下一章：[自定义节点](./custom-nodes.md)