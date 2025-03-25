# ModuForge Core

ModuForge Core 是一个基于 Rust 实现的文档编辑器核心库，提供了灵活的文档模型和内容管理系统。

## 核心概念

### 1. 文档模型 (Document Model)

文档模型是 ModuForge 的核心，它定义了文档的结构和行为。主要包含以下几个关键组件：

#### 1.1 节点类型 (NodeType)
```rust
pub struct NodeType {
    pub name: String,           // 节点类型标识符
    pub spec: NodeSpec,         // 节点规范定义
    pub desc: String,           // 节点描述
    pub groups: Vec<String>,    // 节点所属分组
    pub attrs: HashMap<String, Attribute>,  // 节点属性定义
    pub default_attrs: HashMap<String, String>,  // 默认属性值
    pub content_match: Option<ContentMatch>,  // 内容匹配规则
    pub mark_set: Option<Vec<MarkType>>,     // 允许的标记类型
}
```

节点类型定义了：
- 节点的基本属性（名称、描述、分组）
- 节点的属性约束
- 节点的内容结构规则
- 节点支持的标记类型

#### 1.2 节点规范 (NodeSpec)
```rust
pub struct NodeSpec {
    pub content: Option<String>,    // 内容约束表达式
    pub marks: Option<String>,      // 标记类型表达式
    pub group: Option<String>,      // 分组信息
    pub desc: Option<String>,       // 描述信息
    pub attrs: Option<HashMap<String, AttributeSpec>>,  // 属性规范
}
```

节点规范用于配置节点类型的行为和约束。

### 2. 内容匹配系统 (Content Matching System)

内容匹配系统负责验证和构建文档结构，确保文档内容符合预定义的规则。

#### 2.1 内容匹配规则 (ContentMatch)
```rust
pub struct ContentMatch {
    pub next: Vec<MatchEdge>,       // 可能的下一节点类型
    pub wrap_cache: Vec<Option<NodeType>>,  // 包装节点缓存
    pub valid_end: bool,            // 是否为有效的结束状态
}
```

内容匹配规则定义了：
- 允许的节点序列
- 节点的重复规则
- 节点的可选性

#### 2.2 内容表达式语法

支持以下内容表达式：
- `*` - 零个或多个节点
- `+` - 一个或多个节点
- `?` - 零个或一个节点
- `|` - 节点类型选择
- 空格分隔的序列

例如：
```rust
"DW+"      // 一个或多个 DW 节点
"DW*"      // 零个或多个 DW 节点
"DW djgc"  // DW 节点后跟 djgc 节点
```

### 3. 状态流转系统 (State Transition System)

#### 3.1 状态定义
```rust
pub struct State {
    pub config: Arc<Configuration>,           // 编辑器配置
    pub fields_instances: ImHashMap<String, PluginState>,  // 插件状态实例
    pub node_pool: Arc<NodePool>,             // 文档节点池
    pub version: u64,                         // 状态版本号
}
```

#### 3.2 状态转换流程

1. **初始化状态**
```rust
let state = State::create(state_config).await?;
```

2. **状态转换规则**
- 每个操作都会产生新的状态
- 状态转换必须保持文档一致性
- 状态转换必须记录在历史中
- 状态转换必须经过插件验证

3. **状态验证**
- 文档结构验证
- 插件状态验证
- 事务过滤验证

#### 3.3 核心 apply_transaction 方法

`apply_transaction` 是状态转换系统的核心方法，负责处理事务的应用和插件的交互。其执行流程如下：

```
+------------------+
|      开始        |
+------------------+
         ↓
+------------------+
|    事务过滤      |
+------------------+
         ↓
    +----------+
    |过滤失败?  |
    +----------+
         ↓
    +----------+     +------------------+
    |  是      |     |    初始化事务    |
    +----------+     |    列表和状态    |
         ↓          |    追踪          |
+------------------+     +------------------+
|   返回原始状态    |     ↓
+------------------+     +------------------+
                        |  插件事务处理循环  |
                        +------------------+
                                 ↓
                        +------------------+
                        |   检查所有插件    |
                        +------------------+
                                 ↓
                        +------------------+
                        |   有新事务?      |
                        +------------------+
                                 ↓
                    +----------+     +------------------+
                    |  是      |     |    事务后处理    |
                    +----------+     +------------------+
                         ↓          ↓
                    +------------------+     +------------------+
                    |   应用新事务     |     |  遍历所有插件    |
                    +------------------+     +------------------+
                         ↓          ↓
                    +------------------+     +------------------+
                    |   更新状态追踪   |     |  执行插件后处理  |
                    +------------------+     +------------------+
                         ↓          ↓
                    +------------------+     +------------------+
                    |   添加到事务列表 |     |  更新插件状态    |
                    +------------------+     +------------------+
                         ↓          ↓
                    +------------------+     +------------------+
                    |      结束       |     |      结束       |
                    +------------------+     +------------------+
```

这个方法的主要特点：

1. **事务过滤机制**
   - 在应用事务前进行过滤
   - 支持插件自定义过滤规则
   - 可以阻止不合法的事务

2. **插件事务追加**
   - 支持插件追加新事务
   - 维护事务处理顺序
   - 防止循环依赖

3. **状态追踪**
   - 记录每个插件的处理状态
   - 追踪事务处理进度
   - 确保状态一致性

4. **事务后处理**
   - 允许插件进行清理工作
   - 更新插件状态
   - 维护系统一致性

### 4. 事务系统 (Transaction System)

#### 4.1 事务定义
```rust
pub struct Transaction {
    pub steps: Vec<Step>,          // 事务步骤
    pub doc: Arc<NodePool>,        // 文档节点池
    pub metadata: TransactionMeta, // 事务元数据
}
```

#### 4.2 事务执行流程

1. **事务开始**
```rust
let tr = state.tr();
```

2. **事务应用流程**
```rust
// 1. 事务前处理
state.before_apply_transaction(&mut tr).await?;

// 2. 事务过滤
if !state.filter_transaction(&tr, None).await? {
    return Ok(TransactionResult { 
        state: self.clone(), 
        trs: vec![tr] 
    });
}

// 3. 应用事务
let mut new_state = state.apply_inner(&tr).await?;

// 4. 事务后处理
state.after_apply_transaction(&new_state, &mut tr).await?;
```

3. **事务类型**

a. **简单事务**
- 单个编辑操作
- 直接执行和回滚
- 不包含子事务

b. **复合事务**
- 多个编辑操作
- 可以包含子事务
- 原子性保证

c. **插件事务**
- 由插件产生的事务
- 可以修改或扩展原有事务
- 可以过滤事务执行

#### 4.3 核心 apply 方法分析

`apply` 方法是状态转换的核心，它负责将事务应用到当前状态。主要流程如下：

1. **事务前处理**
```rust
pub async fn before_apply_transaction(&self, tr: &mut Transaction) -> StateResult<()> {
    // 调用所有插件的 before_apply_transaction 钩子
    for plugin in &self.config.plugins {
        plugin.before_apply_transaction(tr, self).await?;
    }
    Ok(())
}
```

2. **事务过滤**
```rust
pub async fn filter_transaction(&self, tr: &Transaction, ignore: Option<usize>) -> StateResult<bool> {
    // 检查所有插件是否允许事务执行
    for (i, plugin) in self.config.plugins.iter().enumerate() {
        if Some(i) != ignore && !plugin.apply_filter_transaction(tr, self).await {
            return Ok(false);
        }
    }
    Ok(true)
}
```

3. **事务应用**
```rust
pub async fn apply_inner(&self, tr: &Transaction) -> StateResult<State> {
    // 1. 创建新的配置
    let mut config = self.config.as_ref().clone();
    config.doc = Some(tr.doc.clone());
    
    // 2. 创建新状态
    let mut new_instance = State::new(Arc::new(config));
    
    // 3. 应用插件状态
    for plugin in &self.config.plugins {
        if let Some(field) = &plugin.spec.state {
            if let Some(old_plugin_state) = self.get_field(&plugin.key) {
                let value = field.apply(tr, old_plugin_state, self, &new_instance).await;
                new_instance.set_field(&plugin.key, value)?;
            }
        }
    }
    
    Ok(new_instance)
}
```

4. **事务后处理**
```rust
async fn after_apply_transaction(&self, new_state: &State, tr: &mut Transaction) -> StateResult<()> {
    // 调用所有插件的 after_apply_transaction 钩子
    for plugin in &self.config.plugins {
        plugin.after_apply_transaction(new_state, tr, self).await?;
    }
    Ok(())
}
```

#### 4.4 插件系统集成

插件系统通过以下方式与状态转换集成：

1. **插件特征**
```rust
pub trait PluginTrait: Send + Sync + Debug {
    async fn append_transaction(&self, tr: &Transaction, old_state: &State, new_state: &State) -> Option<Transaction>;
    async fn filter_transaction(&self, tr: &Transaction, state: &State) -> bool;
    async fn before_apply_transaction(&self, tr: &mut Transaction, state: &State) -> Result<(), Box<dyn std::error::Error>>;
    async fn after_apply_transaction(&self, new_state: &State, tr: &mut Transaction, old_state: &State) -> Result<(), Box<dyn std::error::Error>>;
}
```

2. **状态字段特征**
```rust
pub trait StateField: Send + Sync + Debug {
    async fn init(&self, config: &StateConfig, instance: Option<&State>) -> PluginState;
    async fn apply(&self, tr: &Transaction, value: PluginState, old_state: &State, new_state: &State) -> PluginState;
}
```

这种设计允许：
- 插件可以修改事务内容
- 插件可以过滤事务执行
- 插件可以在事务前后执行自定义逻辑
- 插件可以维护自己的状态

### 5. 执行流程

#### 5.1 文档创建流程

1. **初始化 Schema**
```rust
let schema = Schema::compile(schema_spec)?;
```

2. **创建根节点**
```rust
let root = schema.top_node_type.create_and_fill(
    Some(id),
    None,
    vec![],
    None,
    &schema,
);
```

3. **内容填充过程**
- 检查内容匹配规则
- 创建缺失的必需节点
- 递归创建子节点
- 建立节点间的引用关系

#### 5.2 内容验证流程

1. **节点内容验证**
```rust
node_type.check_content(content, schema)
```

2. **属性验证**
```rust
node_type.check_attrs(attrs)
```

3. **内容匹配验证**
```rust
content_match.match_fragment(fragment, schema)
```

### 6. 示例

#### 6.1 定义文档结构
```rust
let schema_spec = SchemaSpec {
    nodes: {
        let mut nodes = HashMap::new();
        nodes.insert(
            "doc".to_string(),
            NodeSpec {
                content: Some("DW+".to_string()),
                marks: None,
                group: None,
                desc: None,
                attrs: None,
            },
        );
        // ... 其他节点定义
        nodes
    },
    marks: HashMap::new(),
    top_node: Some("doc".to_string()),
};
```

#### 6.2 创建文档
```rust
let schema = Schema::compile(schema_spec)?;
let id = IdGenerator::get_id();
let nodes = schema.top_node_type.create_and_fill(
    Some(id),
    None,
    vec![],
    None,
    &schema,
);
```

## 使用说明

1. 首先定义文档的 Schema，包括：
   - 节点类型定义
   - 内容规则
   - 属性约束
   - 标记类型

2. 使用 Schema 创建文档：
   - 创建根节点
   - 添加子节点
   - 设置节点属性
   - 应用标记

3. 验证文档结构：
   - 检查内容规则
   - 验证属性
   - 确保标记正确

## 注意事项

1. 内容匹配规则必须是无歧义的
2. 节点属性必须满足规范要求
3. 标记类型必须在节点类型允许的范围内
4. 文档结构必须符合预定义的规则
5. 状态转换必须保持一致性
6. 事务必须正确处理回滚

## 性能考虑

1. 内容匹配使用 DFA 实现，确保高效的匹配过程
2. 节点引用使用 ID 而不是直接引用，减少内存占用
3. 使用缓存优化重复操作
4. 延迟创建子节点，避免不必要的开销
5. 状态转换使用不可变数据结构
6. 事务使用增量更新策略 