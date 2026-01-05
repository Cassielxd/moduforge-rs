# 核心概念

本文档详细介绍 ModuForge-RS 的核心概念和设计理念。理解这些概念将帮助你更好地使用框架。

## 泛型设计理念

### 抽象与具体

ModuForge-RS 采用泛型设计，提供灵活的抽象层：

```rust
// 泛型数据容器
pub trait DataContainer: Clone + Send + Sync + Debug + 'static {
    type NodeType: Clone + Send + Sync + Debug;
    type IdType: Clone + Send + Sync + Debug + Hash + Eq;

    fn get_node(&self, id: &Self::IdType) -> Option<&Self::NodeType>;
    fn add_node(&mut self, node: Self::NodeType) -> Result<Self::IdType>;
    fn update_node(&mut self, id: &Self::IdType, node: Self::NodeType) -> Result<()>;
    fn remove_node(&mut self, id: &Self::IdType) -> Result<()>;
}

// Schema定义trait
pub trait SchemaDefinition: Clone + Send + Sync + Debug + 'static {
    type Container: DataContainer;

    fn validate_node(&self, node: &Node) -> Result<()>;
    fn factory(&self) -> &NodeFactory;
}
```

这种设计允许：
- 自定义存储后端
- 不同的节点类型
- 灵活的Schema验证

## 节点系统

### 节点（Node）

节点是文档的基本构成单元：

```rust
pub struct Node {
    pub id: Box<str>,                     // 节点唯一标识
    pub r#type: Box<str>,                 // 节点类型
    pub attrs: HashMap<String, Value>,    // 节点属性
    pub text: Option<String>,              // 文本内容
    pub children: Vec<Box<str>>,          // 子节点ID列表
    pub marks: Vec<Mark>,                 // 应用的标记
    pub parent: Option<Box<str>>,         // 父节点ID
}
```

### 节点池（NodePool）

NodePool 是默认的 DataContainer 实现，管理所有节点：

```rust
pub struct NodePool {
    nodes: DashMap<Box<str>, Arc<Node>>,  // 并发安全的节点存储
    id_generator: Arc<AtomicU64>,         // 原子ID生成器
    root_id: Option<Box<str>>,            // 根节点ID
}

impl NodePool {
    // 高效的节点操作
    pub fn get_node(&self, id: &str) -> Option<Arc<Node>>;
    pub fn add_node(&self, node: Node) -> Result<Box<str>>;
    pub fn update_node(&self, id: &str, node: Node) -> Result<()>;
    pub fn remove_node(&self, id: &str) -> Result<()>;

    // 树遍历
    pub fn get_children(&self, parent_id: &str) -> Vec<Arc<Node>>;
    pub fn get_ancestors(&self, node_id: &str) -> Vec<Box<str>>;
}
```

### 实际应用：Price-RS 节点

Price-RS 项目展示了节点系统的实际应用：

```rust
// 项目节点
#[derive(Node)]
pub struct ProjectNode {
    #[node(type = "project")]
    pub r#type: String,

    #[node(attribute)]
    pub code: String,           // 项目编码

    #[node(attribute)]
    pub name: String,           // 项目名称

    #[node(attribute)]
    pub jzxz: String,          // 建筑性质

    #[node(attribute)]
    pub jzmj: f64,             // 建筑面积

    #[node(children)]
    pub units: Vec<UnitProjectNode>,  // 单位工程
}

// 分部节点
#[derive(Node)]
pub struct FbNode {
    #[node(type = "fb")]
    pub r#type: String,

    #[node(attribute)]
    pub code: String,           // 分部编码

    #[node(attribute)]
    pub name: String,           // 分部名称

    #[node(attribute)]
    pub total_price: f64,       // 总价

    #[node(children)]
    pub items: Vec<QdNode>,     // 清单项
}
```

## 标记系统

### 标记（Mark）

标记用于文本格式化和语义标注：

```rust
pub struct Mark {
    pub r#type: Box<str>,                 // 标记类型
    pub attrs: HashMap<String, Value>,    // 标记属性
}

// 常用标记类型
impl Mark {
    pub fn bold() -> Self {
        Mark {
            r#type: "bold".into(),
            attrs: HashMap::new(),
        }
    }

    pub fn link(href: &str) -> Self {
        Mark {
            r#type: "link".into(),
            attrs: HashMap::from([
                ("href".to_string(), json!(href))
            ]),
        }
    }

    pub fn background_color(color: &str) -> Self {
        Mark {
            r#type: "background".into(),
            attrs: HashMap::from([
                ("color".to_string(), json!(color))
            ]),
        }
    }
}
```

### 自定义标记

通过派生宏创建自定义标记：

```rust
#[derive(Mark)]
pub struct AnnotationMark {
    #[mark(type = "annotation")]
    pub r#type: String,

    #[mark(attribute)]
    pub author: String,

    #[mark(attribute)]
    pub timestamp: i64,

    #[mark(attribute)]
    pub comment: String,
}
```

## 转换系统

### Step 抽象

Step 代表一个原子操作：

```rust
pub trait StepGeneric<C, S>: Send + Sync
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 应用步骤到容器
    fn apply(&self, pool: &mut C) -> StepResult<()>;

    /// 生成逆操作（用于撤销）
    fn invert(&self, pool: &C) -> StepResult<Box<dyn StepGeneric<C, S>>>;

    /// 映射位置（用于协作）
    fn map(&self, mapping: &Mapping) -> StepResult<Box<dyn StepGeneric<C, S>>>;

    /// 序列化为JSON
    fn to_json(&self) -> Value;
}
```

### 内置 Steps

```rust
// 添加节点
pub struct AddNodeStep {
    pub parent_id: Box<str>,
    pub nodes: Vec<Node>,
    pub index: Option<usize>,
}

// 删除节点
pub struct RemoveNodeStep {
    pub node_id: Box<str>,
    pub remove_children: bool,
}

// 更新属性
pub struct UpdateNodeAttrsStep {
    pub node_id: Box<str>,
    pub attrs: HashMap<String, Value>,
    pub old_attrs: Option<HashMap<String, Value>>,
}

// 批量操作
pub struct BatchStep {
    pub steps: Vec<Box<dyn StepGeneric<NodePool, Schema>>>,
}
```

### Transform 事务

Transform 管理多个 Step 的执行：

```rust
pub struct TransformGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub doc: C,                              // 文档容器
    pub schema: S,                           // Schema验证
    pub steps: Vec<Box<dyn StepGeneric<C, S>>>, // 步骤列表
    pub docs: Vec<C>,                        // 中间状态
    pub mapping: Mapping,                    // 位置映射
}

impl<C, S> TransformGeneric<C, S> {
    /// 添加步骤
    pub fn add_step(&mut self, step: Box<dyn StepGeneric<C, S>>) -> Result<()> {
        // 验证步骤
        self.schema.validate_step(&step)?;

        // 应用步骤
        step.apply(&mut self.doc)?;

        // 记录步骤
        self.steps.push(step);
        self.docs.push(self.doc.clone());

        Ok(())
    }

    /// 生成逆Transform（用于撤销）
    pub fn invert(&self) -> Result<Self> {
        let mut inverted_steps = Vec::new();

        // 反向生成逆步骤
        for (i, step) in self.steps.iter().enumerate().rev() {
            let doc_before = if i > 0 {
                &self.docs[i - 1]
            } else {
                &self.doc
            };
            inverted_steps.push(step.invert(doc_before)?);
        }

        Ok(Self::new(self.docs.last().cloned().unwrap(), self.schema.clone()))
    }
}
```

## 状态管理

### State 结构

State 维护文档的完整状态：

```rust
pub struct StateGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub node_pool: Arc<C>,              // 节点池
    pub schema: Arc<S>,                  // Schema
    pub selection: Selection,            // 选区
    pub plugins: Vec<Arc<dyn Plugin>>,  // 插件列表
    pub resources: ResourceTable,        // 资源表
    pub version: u64,                    // 版本号
}

// 默认类型别名
pub type State = StateGeneric<NodePool, Schema>;
```

### Transaction

Transaction 封装了状态变更：

```rust
pub struct TransactionGeneric<C, S> {
    pub id: Box<str>,                              // 事务ID
    pub steps: Vec<Arc<dyn StepGeneric<C, S>>>,   // 步骤列表
    pub invert_steps: Vec<Arc<dyn StepGeneric<C, S>>>, // 逆步骤
    pub metadata: HashMap<String, Value>,          // 元数据
    pub timestamp: i64,                             // 时间戳
}

impl<C, S> TransactionGeneric<C, S> {
    /// 添加节点操作
    pub fn add_node(&mut self, parent_id: Box<str>, nodes: Vec<Node>) -> Result<()> {
        let step = AddNodeStep { parent_id, nodes, index: None };
        self.steps.push(Arc::new(step));
        Ok(())
    }

    /// 更新节点属性
    pub fn update_node(&mut self, node_id: Box<str>, attrs: HashMap<String, Value>) -> Result<()> {
        let step = UpdateNodeAttrsStep { node_id, attrs, old_attrs: None };
        self.steps.push(Arc::new(step));
        Ok(())
    }

    /// 删除节点
    pub fn remove_node(&mut self, node_id: Box<str>) -> Result<()> {
        let step = RemoveNodeStep { node_id, remove_children: true };
        self.steps.push(Arc::new(step));
        Ok(())
    }
}
```

## 运行时系统

### 三种运行时模式

ModuForge-RS 提供三种运行时模式，适应不同场景：

#### 1. 异步运行时（ForgeAsyncRuntime）

适用于高并发场景：

```rust
pub struct ForgeAsyncRuntime {
    state: Arc<RwLock<State>>,
    history: Arc<RwLock<HistoryManager>>,
    event_bus: Arc<EventBus<Event>>,
    processor: Arc<AsyncProcessor>,
    middleware_stack: MiddlewareStack,
}

// 使用示例
let runtime = ForgeAsyncRuntime::new(config).await?;
runtime.dispatch(transaction).await?;
```

#### 2. 同步运行时（ForgeSyncRuntime）

适用于简单场景，无异步依赖：

```rust
pub struct ForgeSyncRuntime {
    state: Arc<Mutex<State>>,
    history: Arc<Mutex<HistoryManager>>,
    processor: Arc<SyncProcessor>,
}

// 使用示例
let runtime = ForgeSyncRuntime::new(config)?;
runtime.dispatch_sync(transaction)?;
```

#### 3. Actor运行时（ForgeActorRuntime）

基于Actor模型，适用于复杂状态管理：

```rust
pub struct ForgeActorRuntime {
    system: ForgeActorSystem,
    state_actor: ActorRef<StateMessage>,
    transaction_processor: ActorRef<TransactionMessage>,
}

// 使用示例
let runtime = ForgeActorRuntime::new(config).await?;
runtime.send_transaction(transaction).await?;
```

### 自适应选择

运行时可以根据系统资源自动选择：

```rust
pub struct AdaptiveRuntimeSelector;

impl AdaptiveRuntimeSelector {
    pub fn select_runtime(config: &ForgeConfig) -> RuntimeType {
        let resources = SystemResources::detect();

        match resources.tier {
            ResourceTier::High => RuntimeType::Async,    // 高配置：异步
            ResourceTier::Medium => RuntimeType::Actor,  // 中配置：Actor
            ResourceTier::Low => RuntimeType::Sync,      // 低配置：同步
        }
    }
}
```

## 事件系统

### 事件类型

```rust
pub enum EventGeneric<C, S> {
    /// 文档创建
    Create(Arc<StateGeneric<C, S>>),

    /// 事务应用
    TrApply {
        old_state: Arc<StateGeneric<C, S>>,
        new_state: Arc<StateGeneric<C, S>>,
        transactions: Vec<Arc<TransactionGeneric<C, S>>>,
    },

    /// 撤销操作
    Undo {
        old_state: Arc<StateGeneric<C, S>>,
        new_state: Arc<StateGeneric<C, S>>,
        transactions: Vec<Arc<TransactionGeneric<C, S>>>,
    },

    /// 重做操作
    Redo {
        old_state: Arc<StateGeneric<C, S>>,
        new_state: Arc<StateGeneric<C, S>>,
        transactions: Vec<Arc<TransactionGeneric<C, S>>>,
    },

    /// 事务失败
    TrFailed {
        state: Arc<StateGeneric<C, S>>,
        transaction: TransactionGeneric<C, S>,
        error: String,
    },
}
```

### 事件处理器

```rust
#[async_trait]
pub trait EventHandler<T>: Send + Sync + Debug {
    async fn handle(&self, event: &T) -> ForgeResult<()>;
}

// 实际应用示例：搜索索引更新
pub struct SearchIndexHandler {
    service: Arc<IndexService>,
}

#[async_trait]
impl EventHandler<Event> for SearchIndexHandler {
    async fn handle(&self, event: &Event) -> ForgeResult<()> {
        match event {
            Event::TrApply { new_state, .. } => {
                // 更新搜索索引
                self.service.update_index(new_state).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

## 中间件系统

### 中间件 Trait

```rust
#[async_trait]
pub trait MiddlewareGeneric<C, S>: Send + Sync
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 中间件名称
    fn name(&self) -> String;

    /// 事务处理前
    async fn before_dispatch(
        &self,
        transaction: &mut TransactionGeneric<C, S>,
    ) -> ForgeResult<()> {
        Ok(())
    }

    /// 事务处理后
    async fn after_dispatch(
        &self,
        state: Option<Arc<StateGeneric<C, S>>>,
        transactions: &[Arc<TransactionGeneric<C, S>>],
    ) -> ForgeResult<Option<TransactionGeneric<C, S>>> {
        Ok(None)
    }
}
```

### 实际应用示例

```rust
// 日志中间件
pub struct LoggingMiddleware;

#[async_trait]
impl Middleware for LoggingMiddleware {
    fn name(&self) -> String {
        "LoggingMiddleware".to_string()
    }

    async fn before_dispatch(&self, transaction: &mut Transaction) -> ForgeResult<()> {
        info!("执行事务: {}", transaction.id);
        Ok(())
    }
}

// 性能监控中间件
pub struct PerformanceMiddleware {
    threshold_ms: u64,
}

#[async_trait]
impl Middleware for PerformanceMiddleware {
    async fn before_dispatch(&self, transaction: &mut Transaction) -> ForgeResult<()> {
        transaction.metadata.insert(
            "start_time".to_string(),
            json!(Instant::now()),
        );
        Ok(())
    }

    async fn after_dispatch(
        &self,
        _state: Option<Arc<State>>,
        transactions: &[Arc<Transaction>],
    ) -> ForgeResult<Option<Transaction>> {
        for tx in transactions {
            if let Some(start) = tx.metadata.get("start_time") {
                let duration = start.elapsed();
                if duration.as_millis() > self.threshold_ms {
                    warn!("事务执行时间过长: {}ms", duration.as_millis());
                }
            }
        }
        Ok(None)
    }
}
```

## 扩展机制

### Extension Trait

```rust
pub trait Extension: Send + Sync {
    /// 扩展名称
    fn name(&self) -> &str;

    /// 提供的节点类型
    fn nodes(&self) -> Vec<NodeDefinition> {
        vec![]
    }

    /// 提供的标记类型
    fn marks(&self) -> Vec<MarkDefinition> {
        vec![]
    }

    /// 提供的命令
    fn commands(&self) -> Vec<Command> {
        vec![]
    }
}
```

### 使用宏定义扩展

```rust
use moduforge_macro::mf_extension;

mf_extension!(MyExtension, {
    commands: {
        "bold": BoldCommand,
        "italic": ItalicCommand,
    },
    nodes: {
        "custom_block": CustomBlockNode,
    },
    marks: {
        "highlight": HighlightMark,
    }
});
```

## 总结

ModuForge-RS 的核心概念包括：

1. **泛型设计**：灵活的抽象层，支持自定义实现
2. **节点系统**：树形文档结构，高性能节点管理
3. **标记系统**：富文本支持，可扩展的格式化
4. **转换系统**：原子操作Step，事务性Transform
5. **状态管理**：不可变State，事务性修改
6. **运行时系统**：三种模式，自适应选择
7. **事件系统**：响应式架构，解耦设计
8. **中间件系统**：横切关注点，灵活扩展
9. **扩展机制**：插件化设计，易于定制

这些概念共同构成了一个强大、灵活、高性能的文档编辑框架。