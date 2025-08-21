# ModuForge-RS API 参考

本文档提供 ModuForge-RS 框架所有公共 API 的详细参考。

## 📦 Crates API 概览

### 核心 API

#### [moduforge-core](#mf-core-api) - 核心运行时
```rust
// 主要 API
ForgeAsyncRuntime::create(config) -> Result<Self>
runtime.apply_transaction(transaction) -> Result<Arc<State>>
runtime.dispatch_flow(transaction) -> Result<()>
```

#### [moduforge-state](#mf-state-api) - 状态管理  
```rust
// 主要 API
State::new(config) -> Self
state.tr() -> Transaction
PluginManager::register_plugin(plugin) -> Result<()>
DependencyManager::add_dependency(dependent, dependency) -> Result<()>
Plugin::new(spec) -> Self
```

#### [moduforge-model](#mf-model-api) - 数据模型
```rust
// 主要 API
Node::new(id, node_type, attrs, content) -> Self
Mark::new(mark_type, attrs) -> Self
Schema::new(nodes, marks) -> Self
```

#### [moduforge-transform](#mf-transform-api) - 数据转换
```rust
// 主要 API
Transaction::new() -> Self
transaction.add_step(step) -> &mut Self
AddNodeStep::new(node, parent) -> Self
```

### 规则引擎 API

#### [moduforge-rules-engine](#mf-engine-api) - 规则引擎
```rust
// 主要 API
Engine::new(loader) -> Self
engine.evaluate(rule_name, input) -> Result<Variable>
Decision::from_json(json) -> Result<Self>
```

#### [moduforge-rules-expression](#mf-expression-api) - 表达式语言
```rust
// 主要 API
Expression::compile(source) -> Result<Self>
expression.execute(variables) -> Result<Variable>
Variable::from(value) -> Self
```

### 协作与数据 API

#### [moduforge-collaboration](#mf-collaboration-api) - 协作系统
```rust
// 主要 API
SyncService::new() -> Self
sync_service.create_room(config) -> Result<()>
YrsManager::new() -> Self
```

#### [moduforge-file](#mf-file-api) - 文件处理
```rust
// 主要 API
ZipDocWriter::new() -> Self
writer.export_document(state, path) -> Result<()>
ZipDocReader::from_file(path) -> Result<Self>
```

---

## mf-core API

### ForgeAsyncRuntime

异步运行时管理器，框架的核心入口点。

#### 创建运行时

```rust
use moduforge_core::runtime::async_runtime::ForgeAsyncRuntime;
use moduforge_core::types::RuntimeOptions;

// 创建运行时
let options = RuntimeOptions::default();
let runtime = ForgeAsyncRuntime::create(options).await?;
```

#### 主要方法

```rust
impl ForgeAsyncRuntime {
    // 创建新的运行时实例
    pub async fn create(options: RuntimeOptions) -> Result<Self>;
    
    // 获取当前状态
    pub fn get_state(&self) -> Arc<State>;
    
    // 应用事务并返回新状态
    pub async fn apply_transaction(&self, tr: Transaction) -> Result<Arc<State>>;
    
    // 执行事务流程（包括中间件和插件处理）
    pub async fn dispatch_flow(&self, tr: Transaction) -> Result<()>;
    
    // 带元数据的事务流程执行
    pub async fn dispatch_flow_with_meta(
        &self, 
        tr: Transaction, 
        title: String, 
        meta: serde_json::Value
    ) -> Result<()>;
}
```

### RuntimeOptions

运行时配置选项。

```rust
#[derive(Debug, Clone)]
pub struct RuntimeOptions {
    pub content: Content,
    pub extensions: Vec<Extensions>,
    pub middlewares: Vec<Box<dyn Middleware>>,
    pub history_limit: Option<usize>,
}

impl RuntimeOptions {
    pub fn new() -> RuntimeOptionsBuilder;
}
```

### RuntimeOptionsBuilder

运行时选项构建器。

```rust
impl RuntimeOptionsBuilder {
    pub fn new() -> Self;
    pub fn content(mut self, content: Content) -> Self;
    pub fn extensions(mut self, extensions: Vec<Extensions>) -> Self;
    pub fn add_middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self;
    pub fn history_limit(mut self, limit: usize) -> Self;
    pub fn build(self) -> RuntimeOptions;
}
```

### Extension

扩展系统，管理节点、标记和插件。

```rust
impl Extension {
    pub fn new() -> Self;
    pub fn add_node(&mut self, node: Node) -> &mut Self;
    pub fn add_mark(&mut self, mark: Mark) -> &mut Self;
    pub fn add_plugin(&mut self, plugin: Arc<Plugin>) -> &mut Self;
    pub fn get_nodes(&self) -> &Vec<Node>;
    pub fn get_marks(&self) -> &Vec<Mark>;
    pub fn get_plugins(&self) -> &Vec<Arc<Plugin>>;
}
```

### Middleware

中间件系统，用于拦截和处理事务。

```rust
#[async_trait]
pub trait Middleware: Send + Sync + Debug {
    // 中间件名称
    fn name(&self) -> String;
    
    // 在核心处理之前执行
    async fn before_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> ForgeResult<Option<Transaction>> {
        Ok(None)
    }
    
    // 在核心处理之后执行
    async fn after_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> ForgeResult<Option<Transaction>> {
        Ok(None)
    }
}
```

---

## mf-state API

### State

全局状态管理器。

```rust
impl State {
    // 创建新状态
    pub fn new(config: StateConfig) -> Self;
    
    // 创建新事务
    pub fn tr(&self) -> Transaction;
    
    // 获取文档
    pub fn doc(&self) -> &Arc<Tree>;
    
    // 获取状态ID
    pub fn id(&self) -> &str;
    
    // 获取版本号
    pub fn version(&self) -> u64;
    
    // 获取插件状态
    pub fn get_field(&self, key: &str) -> Option<Arc<dyn Resource>>;
    
    // 获取资源管理器
    pub fn resource_manager(&self) -> &Arc<RwLock<GlobalResourceManager>>;
    
    // 重新配置状态
    pub fn reconfigure(&self, config: StateConfig) -> Result<State>;
}
```

### Transaction

事务管理器，保证操作的原子性。

```rust
impl Transaction {
    // 创建新事务
    pub fn new(state: &State) -> Self;
    
    // 添加操作步骤
    pub fn add_step(&mut self, step: Box<dyn Step>) -> &mut Self;
    
    // 设置元数据
    pub fn set_meta<T: Serialize>(&mut self, key: &str, value: T) -> &mut Self;
    
    // 获取元数据
    pub fn get_meta<T: DeserializeOwned>(&self, key: &str) -> Option<T>;
    
    // 检查是否包含元数据
    pub fn has_meta(&self, key: &str) -> bool;
    
    // 提交事务
    pub fn commit(&mut self) -> &mut Self;
    
    // 获取文档
    pub fn doc(&self) -> &Arc<Tree>;
}
```

### PluginManager

插件管理器，提供完整的插件生命周期管理。

```rust
impl PluginManager {
    // 创建新的插件管理器
    pub fn new() -> Self;
    
    // 注册插件
    pub async fn register_plugin(&self, plugin: Arc<Plugin>) -> Result<()>;
    
    // 完成注册并验证
    pub async fn finalize_registration(&self) -> Result<()>;
    
    // 获取排序后的插件列表
    pub async fn get_sorted_plugins(&self) -> Vec<Arc<Plugin>>;
    
    // 检查是否已初始化
    pub async fn is_initialized(&self) -> bool;
}
```

### Plugin

插件定义。

```rust
impl Plugin {
    // 创建新插件
    pub fn new(spec: PluginSpec) -> Self;
    
    // 获取插件名称
    pub fn get_name(&self) -> &str;
    
    // 获取插件元数据
    pub fn get_metadata(&self) -> PluginMetadata;
    
    // 获取插件配置
    pub fn get_config(&self) -> PluginConfig;
    
    // 获取插件状态
    pub fn get_state(&self, state: &State) -> Option<Arc<dyn Resource>>;
}
```

### PluginTrait

插件行为定义。

```rust
#[async_trait]
pub trait PluginTrait: Send + Sync + Debug {
    // 获取插件元数据（提供默认实现）
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "default_plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "默认插件".to_string(),
            author: "系统".to_string(),
            dependencies: vec![],
            conflicts: vec![],
            state_fields: vec![],
            tags: vec![],
        }
    }
    
    // 获取插件配置（提供默认实现）
    fn config(&self) -> PluginConfig {
        PluginConfig {
            enabled: true,
            priority: 0,
            settings: std::collections::HashMap::new(),
        }
    }
    
    // 事务过滤（提供默认实现）
    async fn filter_transaction(&self, _tr: &Transaction, _state: &State) -> bool {
        true
    }
    
    // 追加事务处理（提供默认实现）
    async fn append_transaction(
        &self,
        _transactions: &[Transaction],
        _old_state: &State,
        _new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        Ok(None)
    }
}
```

### StateField

插件状态字段管理。

```rust
#[async_trait]
pub trait StateField: Send + Sync + Debug {
    // 初始化插件状态
    async fn init(&self, config: &StateConfig, instance: &State) -> Arc<dyn Resource>;
    
    // 应用状态变更
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        old_state: &State,
        new_state: &State,
    ) -> Arc<dyn Resource>;
    
    // 序列化状态（提供默认实现）
    fn serialize(&self, _value: Arc<dyn Resource>) -> Option<Vec<u8>> {
        None
    }
    
    // 反序列化状态（提供默认实现）
    fn deserialize(&self, _data: &Vec<u8>) -> Option<Arc<dyn Resource>> {
        None
    }
}
```

### PluginSpec

插件规范结构体，定义插件的配置和行为。

```rust
#[derive(Clone, Debug)]
pub struct PluginSpec {
    pub state_field: Option<Arc<dyn StateField>>,
    pub tr: Arc<dyn PluginTrait>,
}

impl PluginSpec {
    // 插件状态管理器
    async fn filter_transaction(&self, tr: &Transaction, state: &State) -> bool;
    
    // 执行事务追加
    async fn append_transaction(
        &self,
        trs: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>>;
}
```

### PluginMetadata

插件元数据结构体。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,              // 插件名称
    pub version: String,           // 插件版本
    pub description: String,       // 插件描述
    pub author: String,            // 插件作者
    pub dependencies: Vec<String>, // 插件依赖
    pub conflicts: Vec<String>,    // 插件冲突
    pub state_fields: Vec<String>, // 插件状态字段
    pub tags: Vec<String>,         // 插件标签
}
```

### PluginConfig

插件配置结构体。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,  // 插件是否启用
    pub priority: i32,  // 插件优先级
    pub settings: std::collections::HashMap<String, serde_json::Value>, // 插件配置
}
```

### DependencyManager

插件依赖管理器，提供依赖关系管理和验证。

```rust
impl DependencyManager {
    // 创建新的依赖管理器
    pub fn new() -> Self;
    
    // 添加插件节点
    pub fn add_plugin(&mut self, plugin_name: &str);
    
    // 添加依赖关系
    pub fn add_dependency(&mut self, dependent: &str, dependency: &str) -> Result<()>;
    
    // 检查缺失的依赖
    pub fn check_missing_dependencies(&self) -> MissingDependencyReport;
    
    // 检查循环依赖
    pub fn has_circular_dependencies(&self) -> bool;
    
    // 获取循环依赖
    pub fn get_circular_dependencies(&self) -> Vec<Vec<String>>;
    
    // 获取拓扑排序
    pub fn get_topological_order(&self) -> Result<Vec<String>>;
    
    // 获取插件的直接依赖
    pub fn get_direct_dependencies(&self, plugin_name: &str) -> Vec<String>;
    
    // 获取插件的所有依赖（包括间接依赖）
    pub fn get_all_dependencies(&self, plugin_name: &str) -> HashSet<String>;
    
    // 获取循环依赖的详细报告
    pub fn get_circular_dependency_report(&self) -> CircularDependencyReport;
}
```

### MissingDependencyReport

缺失依赖报告。

```rust
#[derive(Debug, Clone)]
pub struct MissingDependencyReport {
    pub has_missing_dependencies: bool,
    pub total_missing_count: usize,
    pub missing_dependencies: HashMap<String, Vec<String>>,
    pub available_plugins: HashSet<String>,
}

impl MissingDependencyReport {
    // 生成人类可读的报告
    pub fn to_string(&self) -> String;
    
    // 获取所有缺失的依赖名称
    pub fn get_all_missing_dependency_names(&self) -> HashSet<String>;
}
```

### CircularDependencyReport

循环依赖报告。

```rust
#[derive(Debug, Clone)]
pub struct CircularDependencyReport {
    pub has_circular_dependencies: bool,
    pub cycle_count: usize,
    pub cycles: Vec<Vec<String>>,
    pub affected_plugins: HashSet<String>,
}

impl CircularDependencyReport {
    // 生成人类可读的报告
    pub fn to_string(&self) -> String;
}
```

---

## mf-model API

### Node

树形节点结构。

```rust
impl Node {
    // 创建新节点
    pub fn new(id: String, node_type: NodeType, attrs: Attrs, content: Option<String>) -> Self;
    
    // 获取节点ID
    pub fn id(&self) -> &str;
    
    // 获取节点类型
    pub fn node_type(&self) -> &NodeType;
    
    // 获取节点属性
    pub fn attrs(&self) -> &Attrs;
    
    // 获取/设置节点内容
    pub fn content(&self) -> Option<&str>;
    pub fn set_content(&mut self, content: String);
    
    // 获取节点标记
    pub fn marks(&self) -> &Vector<Mark>;
    
    // 添加/移除标记
    pub fn add_mark(&mut self, mark: Mark);
    pub fn remove_mark(&mut self, mark_type: &str);
    
    // 检查是否包含标记
    pub fn has_mark(&self, mark_type: &str) -> bool;
}
```

### NodeType

节点类型定义。

```rust
impl NodeType {
    // 创建文本节点类型
    pub fn text(name: &str) -> Self;
    
    // 创建块节点类型
    pub fn block(name: &str) -> Self;
    
    // 创建内联节点类型
    pub fn inline(name: &str) -> Self;
    
    // 获取节点类型名称
    pub fn name(&self) -> &str;
    
    // 检查节点类型
    pub fn is_text(&self) -> bool;
    pub fn is_block(&self) -> bool;
    pub fn is_inline(&self) -> bool;
}
```

### Attrs

属性系统。

```rust
impl Attrs {
    // 创建空属性
    pub fn new() -> Self;
    
    // 从映射创建
    pub fn from<I, K, V>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<AttrValue>;
    
    // 获取属性值
    pub fn get(&self, key: &str) -> Option<&AttrValue>;
    
    // 设置属性值
    pub fn set<V: Into<AttrValue>>(&mut self, key: String, value: V);
    
    // 移除属性
    pub fn remove(&mut self, key: &str) -> Option<AttrValue>;
    
    // 检查是否包含属性
    pub fn contains_key(&self, key: &str) -> bool;
    
    // 获取所有键
    pub fn keys(&self) -> impl Iterator<Item = &String>;
    
    // 获取所有值
    pub fn values(&self) -> impl Iterator<Item = &AttrValue>;
}
```

### AttrValue

属性值类型。

```rust
impl AttrValue {
    // 创建不同类型的属性值
    pub fn string(s: String) -> Self;
    pub fn number(n: f64) -> Self;
    pub fn boolean(b: bool) -> Self;
    pub fn null() -> Self;
    
    // 类型检查
    pub fn is_string(&self) -> bool;
    pub fn is_number(&self) -> bool;
    pub fn is_boolean(&self) -> bool;
    pub fn is_null(&self) -> bool;
    
    // 值提取
    pub fn as_string(&self) -> Option<&str>;
    pub fn as_f64(&self) -> Option<f64>;
    pub fn as_bool(&self) -> Option<bool>;
}
```

### Mark

节点标记系统。

```rust
impl Mark {
    // 创建新标记
    pub fn new(mark_type: MarkType, attrs: Attrs) -> Self;
    
    // 获取标记类型
    pub fn mark_type(&self) -> &MarkType;
    
    // 获取标记属性
    pub fn attrs(&self) -> &Attrs;
    
    // 检查标记是否匹配
    pub fn matches(&self, mark_type: &str) -> bool;
}
```

### Tree

树形结构操作。

```rust
impl Tree {
    // 创建新树
    pub fn new(root: Node) -> Self;
    
    // 获取根节点
    pub fn root(&self) -> &Arc<Node>;
    
    // 获取节点
    pub fn get_node(&self, id: &str) -> Option<Arc<Node>>;
    
    // 添加节点
    pub fn add_node(&mut self, node: Node, parent_id: Option<String>) -> Result<()>;
    
    // 移除节点
    pub fn remove_node(&mut self, id: &str) -> Result<Option<Arc<Node>>>;
    
    // 获取子节点
    pub fn get_children(&self, id: &str) -> Vec<Arc<Node>>;
    
    // 获取父节点
    pub fn get_parent(&self, id: &str) -> Option<Arc<Node>>;
    
    // 遍历所有节点
    pub fn traverse<F>(&self, visitor: F) 
    where F: FnMut(&Arc<Node>);
    
    // 查找节点
    pub fn find<F>(&self, predicate: F) -> Option<Arc<Node>>
    where F: Fn(&Arc<Node>) -> bool;
    
    // 获取节点数量
    pub fn size(&self) -> usize;
    
    // 获取树的深度
    pub fn depth(&self) -> usize;
}
```

---

## mf-transform API

### Step

变换步骤的基础 trait。

```rust
pub trait Step: Send + Sync + Debug {
    // 应用步骤
    fn apply(&self, doc: &mut Tree) -> Result<()>;
    
    // 获取逆向步骤
    fn invert(&self, doc: &Tree) -> Result<Box<dyn Step>>;
    
    // 合并步骤
    fn merge(&self, other: &dyn Step) -> Option<Box<dyn Step>>;
    
    // 获取步骤类型
    fn step_type(&self) -> &'static str;
}
```

### AddNodeStep

添加节点步骤。

```rust
impl AddNodeStep {
    // 创建单个节点添加步骤
    pub fn new_single(node: Node, parent_id: Option<String>) -> Self;
    
    // 创建批量节点添加步骤
    pub fn new_batch(nodes: Vec<NodeToAdd>) -> Self;
    
    // 收集节点ID
    pub fn collect_node_ids(node_enum: &NodeToAdd) -> Vec<String>;
}
```

### AttrStep

属性修改步骤。

```rust
impl AttrStep {
    // 创建属性修改步骤
    pub fn new(node_id: String, attrs: Attrs) -> Self;
    
    // 创建单个属性设置步骤
    pub fn set_attr<V: Into<AttrValue>>(node_id: String, key: String, value: V) -> Self;
    
    // 创建属性删除步骤
    pub fn remove_attr(node_id: String, key: String) -> Self;
}
```

### MarkStep

标记操作步骤。

```rust
// 添加标记步骤
impl AddMarkStep {
    pub fn new(node_id: String, marks: Vec<Mark>) -> Self;
}

// 移除标记步骤
impl RemoveMarkStep {
    pub fn new(node_id: String, mark_types: Vec<String>) -> Self;
}
```

### BatchStep

批量操作步骤。

```rust
impl BatchStep {
    // 创建批量步骤
    pub fn new(steps: Vec<Box<dyn Step>>) -> Self;
    
    // 添加步骤
    pub fn add_step(&mut self, step: Box<dyn Step>);
    
    // 获取步骤数量
    pub fn step_count(&self) -> usize;
}
```

---

## mf-engine API

### Engine

规则引擎核心。

```rust
impl Engine {
    // 创建新引擎
    pub fn new<L: Loader + 'static>(loader: L) -> Self;
    
    // 评估规则
    pub async fn evaluate(&self, rule_name: &str, input: &Variable) -> Result<Variable>;
    
    // 批量评估
    pub async fn evaluate_batch(&self, rules: Vec<(&str, &Variable)>) -> Result<Vec<Variable>>;
    
    // 预加载规则
    pub async fn preload_rule(&self, rule_name: &str) -> Result<()>;
    
    // 清除缓存
    pub fn clear_cache(&self);
    
    // 设置配置
    pub fn set_config(&mut self, config: EngineConfig);
}
```

### Decision

决策处理器。

```rust
impl Decision {
    // 从 JSON 创建决策
    pub fn from_json(json: &str) -> Result<Self>;
    
    // 从文件加载决策
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>;
    
    // 执行决策
    pub async fn execute(&self, input: &Variable) -> Result<Variable>;
    
    // 获取决策类型
    pub fn decision_type(&self) -> &str;
    
    // 验证决策
    pub fn validate(&self) -> Result<()>;
}
```

### Loader

规则加载器 trait。

```rust
#[async_trait]
pub trait Loader: Send + Sync {
    // 加载规则
    async fn load(&self, rule_name: &str) -> Result<String>;
    
    // 检查规则是否存在
    async fn exists(&self, rule_name: &str) -> bool;
    
    // 列出所有规则
    async fn list_rules(&self) -> Result<Vec<String>>;
}
```

### MemoryLoader

内存规则加载器。

```rust
impl MemoryLoader {
    // 创建新的内存加载器
    pub fn new() -> Self;
    
    // 添加规则
    pub fn add_rule(&mut self, name: String, content: String);
    
    // 移除规则
    pub fn remove_rule(&mut self, name: &str) -> Option<String>;
    
    // 批量添加规则
    pub fn add_rules<I>(&mut self, rules: I)
    where I: IntoIterator<Item = (String, String)>;
}
```

### FilesystemLoader

文件系统规则加载器。

```rust
impl FilesystemLoader {
    // 创建文件系统加载器
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self;
    
    // 设置文件扩展名
    pub fn with_extension(mut self, ext: &str) -> Self;
    
    // 设置是否递归搜索
    pub fn recursive(mut self, recursive: bool) -> Self;
}
```

---

## mf-expression API

### Expression

表达式编译器和执行器。

```rust
impl Expression {
    // 编译表达式
    pub fn compile(source: &str) -> Result<Self>;
    
    // 执行表达式
    pub fn execute(&self, variables: &Variable) -> Result<Variable>;
    
    // 验证表达式语法
    pub fn validate(source: &str) -> Result<()>;
    
    // 获取表达式的依赖变量
    pub fn dependencies(&self) -> Vec<String>;
    
    // 获取表达式的返回类型
    pub fn return_type(&self) -> Option<TypeInfo>;
}
```

### Variable

变量系统。

```rust
impl Variable {
    // 创建不同类型的变量
    pub fn from<T: Into<Variable>>(value: T) -> Self;
    pub fn null() -> Self;
    pub fn boolean(b: bool) -> Self;
    pub fn number(n: f64) -> Self;
    pub fn string(s: String) -> Self;
    pub fn array(items: Vec<Variable>) -> Self;
    pub fn object(map: HashMap<String, Variable>) -> Self;
    
    // 类型检查
    pub fn is_null(&self) -> bool;
    pub fn is_boolean(&self) -> bool;
    pub fn is_number(&self) -> bool;
    pub fn is_string(&self) -> bool;
    pub fn is_array(&self) -> bool;
    pub fn is_object(&self) -> bool;
    
    // 值提取
    pub fn to_bool(&self) -> bool;
    pub fn to_f64(&self) -> Option<f64>;
    pub fn to_string(&self) -> String;
    pub fn as_array(&self) -> Option<&Vec<Variable>>;
    pub fn as_object(&self) -> Option<&HashMap<String, Variable>>;
    
    // 对象/数组操作
    pub fn get(&self, key: &str) -> Option<&Variable>;
    pub fn get_index(&self, index: usize) -> Option<&Variable>;
    pub fn set(&mut self, key: String, value: Variable);
    pub fn push(&mut self, value: Variable);
}
```

### Functions

函数注册和管理。

```rust
// 注册自定义函数
pub fn register_function<F>(name: &str, func: F)
where F: Fn(&[Variable]) -> Result<Variable> + Send + Sync + 'static;

// 获取内置函数列表
pub fn builtin_functions() -> Vec<&'static str>;

// 检查函数是否存在
pub fn function_exists(name: &str) -> bool;
```

---

## mf-collaboration API

### SyncService

协作同步服务。

```rust
impl SyncService {
    // 创建新的同步服务
    pub fn new() -> Self;
    
    // 创建协作房间
    pub async fn create_room(&mut self, config: RoomConfig) -> Result<()>;
    
    // 删除房间
    pub async fn remove_room(&mut self, room_id: &str) -> Result<()>;
    
    // 处理客户端消息
    pub async fn handle_message(
        &self, 
        room_id: &str, 
        client_id: &str, 
        message: Message
    ) -> Result<()>;
    
    // 获取房间状态
    pub async fn get_room_state(&self, room_id: &str) -> Option<RoomState>;
}
```

### YrsManager

Yrs CRDT 管理器。

```rust
impl YrsManager {
    // 创建新的 Yrs 管理器
    pub fn new() -> Self;
    
    // 创建新文档
    pub fn create_doc(&mut self, doc_id: String) -> Result<()>;
    
    // 应用更新
    pub fn apply_update(&mut self, doc_id: &str, update: &[u8]) -> Result<()>;
    
    // 获取文档状态
    pub fn get_state(&self, doc_id: &str) -> Option<Vec<u8>>;
    
    // 获取状态差异
    pub fn get_state_diff(&self, doc_id: &str, state_vector: &[u8]) -> Option<Vec<u8>>;
}
```

---

## mf-file API

### ZipDocWriter

ZIP 文档写入器。

```rust
impl ZipDocWriter {
    // 创建新的写入器
    pub fn new() -> Self;
    
    // 设置序列化格式
    pub fn set_format(&mut self, format: Box<dyn FormatStrategy>);
    
    // 导出文档
    pub async fn export_document<P: AsRef<Path>>(
        &self, 
        state: &State, 
        path: P
    ) -> Result<()>;
    
    // 导出到字节流
    pub async fn export_to_bytes(&self, state: &State) -> Result<Vec<u8>>;
}
```

### ZipDocReader

ZIP 文档读取器。

```rust
impl ZipDocReader {
    // 从文件创建读取器
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>;
    
    // 从字节流创建读取器
    pub fn from_bytes(data: &[u8]) -> Result<Self>;
    
    // 导入文档
    pub async fn import_document(&self) -> Result<State>;
    
    // 获取文档元数据
    pub fn get_metadata(&self) -> Result<DocumentMetadata>;
    
    // 列出文档内容
    pub fn list_entries(&self) -> Result<Vec<String>>;
}
```

### FormatStrategy

序列化格式策略。

```rust
pub trait FormatStrategy: Send + Sync {
    // 序列化
    fn serialize(&self, data: &impl Serialize) -> Result<Vec<u8>>;
    
    // 反序列化
    fn deserialize<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T>;
    
    // 获取格式名称
    fn format_name(&self) -> &str;
    
    // 获取文件扩展名
    fn file_extension(&self) -> &str;
}

// 内置格式
pub struct JsonFormat;
pub struct CborFormat;
pub struct MessagePackFormat;
```

---

## 错误处理

### 常用错误类型

```rust
// 核心错误
pub type ForgeResult<T> = Result<T, ForgeError>;

// 状态错误
pub type StateResult<T> = Result<T, StateError>;

// 引擎错误
pub type EngineResult<T> = Result<T, EngineError>;

// 表达式错误
pub type ExpressionResult<T> = Result<T, ExpressionError>;
```

### 错误处理最佳实践

```rust
use anyhow::{Result, Context};

// 使用 context 添加错误上下文
let result = operation()
    .context("Failed to perform operation")?;

// 使用 map_err 转换错误类型
let result = operation()
    .map_err(|e| CustomError::from(e))?;
```

---

## 使用示例

### 基础使用流程

```rust
use moduforge_core::runtime::async_runtime::ForgeAsyncRuntime;
use moduforge_core::types::{RuntimeOptions, Content};
use moduforge_core::model::{Node, NodeType, Attrs};
use moduforge_core::transform::node_step::AddNodeStep;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 创建运行时
    let options = RuntimeOptions::new()
        .content(Content::NodePoolFn(Arc::new(|| NodePool::default())))
        .build();
    
    let runtime = ForgeAsyncRuntime::create(options).await?;
    
    // 2. 创建节点
    let node = Node::new(
        "node_1".to_string(),
        NodeType::text("paragraph"),
        Attrs::new(),
        Some("Hello, World!".to_string())
    );
    
    // 3. 创建事务
    let mut transaction = runtime.get_state().tr();
    transaction.add_step(Box::new(AddNodeStep::new_single(node, None)));
    
    // 4. 执行事务
    runtime.dispatch_flow(transaction).await?;
    
    Ok(())
}
```

### 完整插件实现示例

展示最新插件设计的完整实现：

```rust
use moduforge_core::state::{
    plugin::{Plugin, PluginSpec, PluginTrait, StateField, PluginMetadata, PluginConfig},
    resource::Resource,
    state::{State, StateConfig},
    transaction::Transaction,
    error::StateResult,
};
use async_trait::async_trait;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

// 1. 定义插件资源数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyPluginData {
    pub counter: usize,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

impl Resource for MyPluginData {}

// 2. 实现状态字段管理器
#[derive(Debug)]
pub struct MyStateField;

#[async_trait]
impl StateField for MyStateField {
    async fn init(&self, _config: &StateConfig, _instance: &State) -> Arc<dyn Resource> {
        Arc::new(MyPluginData {
            counter: 0,
            last_update: chrono::Utc::now(),
        })
    }
    
    async fn apply(
        &self,
        _tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        let mut data = value.downcast_ref::<MyPluginData>()
            .expect("状态类型错误")
            .clone();
        
        data.counter += 1;
        data.last_update = chrono::Utc::now();
        
        Arc::new(data)
    }
}

// 3. 实现插件行为
#[derive(Debug)]
pub struct MyPlugin;

#[async_trait]
impl PluginTrait for MyPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "my_plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "我的示例插件".to_string(),
            author: "开发者".to_string(),
            dependencies: vec![], // 可以指定依赖的其他插件
            conflicts: vec![],    // 可以指定冲突的插件
            state_fields: vec!["my_plugin_data".to_string()],
            tags: vec!["example".to_string()],
        }
    }
    
    fn config(&self) -> PluginConfig {
        PluginConfig {
            enabled: true,
            priority: 10,
            settings: std::collections::HashMap::new(),
        }
    }
    
    async fn append_transaction(
        &self,
        _transactions: &[Transaction],
        _old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 获取插件状态
        if let Some(plugin_data) = new_state.get_field("my_plugin")
            .and_then(|state| state.downcast_ref::<MyPluginData>()) {
            
            println!("插件状态 - 计数器: {}, 更新时间: {}", 
                plugin_data.counter, 
                plugin_data.last_update
            );
        }
        
        Ok(None)
    }
}

// 4. 创建插件实例
pub fn create_my_plugin() -> Arc<Plugin> {
    let spec = PluginSpec {
        state_field: Some(Arc::new(MyStateField)),
        tr: Arc::new(MyPlugin),
    };
    Arc::new(Plugin::new(spec))
}

// 5. 在运行时中注册插件
async fn setup_with_plugin() -> Result<()> {
    use moduforge_core::runtime::async_runtime::ForgeAsyncRuntime;
    use moduforge_core::types::RuntimeOptions;
    
    // 创建运行时
    let options = RuntimeOptions::default();
    let runtime = ForgeAsyncRuntime::create(options).await?;
    
    // 注册插件
    let plugin = create_my_plugin();
    runtime.get_plugin_manager().register_plugin(plugin).await?;
    
    // 完成插件注册验证
    runtime.get_plugin_manager().finalize_registration().await?;
    
    println!("插件注册完成！");
    Ok(())
}
```

这个 API 参考提供了 ModuForge-RS 框架所有主要组件的详细接口文档，包括最新的插件系统设计，帮助开发者快速上手和深入使用框架。