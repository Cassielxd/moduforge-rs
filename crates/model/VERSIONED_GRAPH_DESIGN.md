# ModuForge-RS 版本化图模型设计

## 概述

ModuForge-RS 版本化图模型是一个结合了 `petgraph` 的 `StableDiGraph` 和 `im` crate 不可变数据结构的混合解决方案，支持复杂的节点关系、版本控制和递归创建功能。

## 核心组件

### 1. VersionedGraph - 版本化图结构

```rust
pub struct VersionedGraph {
    current_graph: StableDiGraph<GraphNode, Relation>,
    node_map: ImHashMap<NodeId, NodeIndex>,
    relation_index: ImHashMap<RelationType, ImVector<EdgeIndex>>,
    root_id: Option<NodeId>,
    metadata: ImHashMap<String, serde_json::Value>,
    snapshots: ImVector<GraphSnapshot>,
    current_version: u64,
    max_snapshots: usize,
}
```

**特性：**
- 结合 `StableDiGraph` 的图算法能力和 `im` 的不可变特性
- 支持版本控制和快照管理
- 自动快照创建策略
- 完整的图操作 API

### 2. GraphSnapshot - 图快照

```rust
pub struct GraphSnapshot {
    id: Uuid,
    version: u64,
    timestamp: chrono::DateTime<chrono::Utc>,
    description: Option<String>,
    nodes: ImHashMap<NodeId, GraphNode>,
    edges: ImVector<(NodeId, NodeId, Relation)>,
    node_indices: ImHashMap<NodeId, usize>,
    root_id: Option<NodeId>,
    metadata: ImHashMap<String, serde_json::Value>,
}
```

**特性：**
- 完整的图状态序列化
- 版本历史追踪
- 时间戳和描述信息
- 支持快照恢复

### 3. GraphNodeType - 图节点类型系统

```rust
pub struct GraphNodeType {
    name: String,
    description: String,
    spec: NodeSpec,
    groups: ImVector<String>,
    attributes: ImHashMap<String, Value>,
    default_attributes: ImHashMap<String, Value>,
    allowed_relations: ImVector<RelationType>,
    forbidden_relations: ImVector<RelationType>,
    child_constraints: ImHashMap<String, ChildConstraint>,
    parent_constraints: ImHashMap<String, ParentConstraint>,
    creation_rules: ImVector<CreationRule>,
    validation_rules: ImVector<ValidationRule>,
    behaviors: ImHashMap<String, NodeBehavior>,
}
```

**特性：**
- 递归创建功能（类似原 NodeType 的 create_and_fill）
- 复杂的约束系统
- 规则驱动的节点创建
- 自动验证机制

## 核心功能

### 1. 递归创建 (create_and_fill)

```rust
pub fn create_and_fill(
    &self,
    graph: &mut VersionedGraph,
    id: Option<NodeId>,
    attrs: Option<&ImHashMap<String, Value>>,
    content: Vec<Node>,
    marks: Option<Vec<Mark>>,
    schema: &Schema,
    node_types: &ImHashMap<String, GraphNodeType>,
) -> PoolResult<NodeId>
```

**功能：**
- 递归创建节点及其子节点
- 自动应用创建规则
- 自动验证节点约束
- 建立节点间关系

### 2. 版本控制

```rust
// 创建快照
pub fn create_snapshot(&mut self, description: Option<String>) -> PoolResult<GraphSnapshot>

// 恢复快照
pub fn restore_snapshot(&mut self, version: u64) -> PoolResult<()>

// 获取版本历史
pub fn get_snapshots(&self) -> &ImVector<GraphSnapshot>
```

**特性：**
- 自动快照创建策略
- 完整的版本历史
- 快照数量限制
- 时间戳和描述信息

### 3. 规则系统

#### 创建规则 (CreationRule)
```rust
pub struct CreationRule {
    name: String,
    condition: CreationCondition,
    action: CreationAction,
    priority: i32,
    enabled: bool,
}
```

**支持的条件：**
- `ParentType(String)` - 父节点类型检查
- `ChildCountLess(usize)` - 子节点数量检查
- `MissingChildType(String)` - 缺少子节点类型
- `AttributeCondition(String, Value)` - 属性条件
- 复合条件：`And`, `Or`, `Not`

**支持的动作：**
- `CreateChild(String)` - 创建子节点
- `CreateParent(String)` - 创建父节点
- `CreateSibling(String)` - 创建兄弟节点
- `SetAttribute(String, Value)` - 设置属性
- `Sequence(Vec<CreationAction>)` - 复合动作

#### 验证规则 (ValidationRule)
```rust
pub struct ValidationRule {
    name: String,
    condition: ValidationCondition,
    error_message: String,
    priority: i32,
    enabled: bool,
}
```

**支持的验证条件：**
- `ChildCount(usize, usize)` - 子节点数量范围
- `RequiredChildType(String)` - 必需子节点类型
- `AttributeValue(String, Value)` - 属性值检查
- `RelationCount(RelationType, usize, usize)` - 关系数量范围

### 4. 约束系统

#### 子节点约束 (ChildConstraint)
```rust
pub struct ChildConstraint {
    allowed_types: ImVector<String>,
    forbidden_types: ImVector<String>,
    min_count: usize,
    max_count: Option<usize>,
    relation_type: RelationType,
    required: bool,
}
```

#### 父节点约束 (ParentConstraint)
```rust
pub struct ParentConstraint {
    allowed_types: ImVector<String>,
    forbidden_types: ImVector<String>,
    relation_type: RelationType,
    required: bool,
}
```

## 使用示例

### 基本使用

```rust
// 1. 创建版本化图
let mut graph = VersionedGraph::new()
    .with_max_snapshots(50);

// 2. 定义节点类型
let mut document_type = GraphNodeType::new("document".to_string(), NodeSpec::default());
document_type.add_group("block".to_string());

// 3. 添加创建规则
let creation_rule = CreationRule {
    name: "auto_create_paragraph".to_string(),
    condition: CreationCondition::MissingChildType("paragraph".to_string()),
    action: CreationAction::CreateChild("paragraph".to_string()),
    priority: 1,
    enabled: true,
};
document_type.add_creation_rule(creation_rule);

// 4. 创建节点类型映射
let mut node_types = ImHashMap::new();
node_types = node_types.update("document".to_string(), document_type);

// 5. 递归创建节点
let schema = Schema::default();
let document_id = node_types.get("document").unwrap().create_and_fill(
    &mut graph,
    Some(NodeId::from("doc_1")),
    None,
    vec![],
    None,
    &schema,
    &node_types,
)?;

// 6. 创建快照
let snapshot = graph.create_snapshot(Some("Initial creation".to_string()))?;

// 7. 恢复快照
graph.restore_snapshot(snapshot.version())?;
```

### 复杂约束示例

```rust
// 定义复杂的子节点约束
let mut paragraph_constraint = ChildConstraint {
    allowed_types: ImVector::new()
        .push_back("paragraph".to_string())
        .push_back("heading".to_string()),
    forbidden_types: ImVector::new(),
    min_count: 1,
    max_count: Some(10),
    relation_type: RelationType::ParentChild,
    required: true,
};

// 定义父节点约束
let mut parent_constraint = ParentConstraint {
    allowed_types: ImVector::new().push_back("document".to_string()),
    forbidden_types: ImVector::new(),
    relation_type: RelationType::ParentChild,
    required: true,
};

document_type.add_child_constraint("content".to_string(), paragraph_constraint);
document_type.add_parent_constraint("container".to_string(), parent_constraint);
```

## 性能优化

### 1. 不可变数据结构
- 使用 `im` crate 的不可变集合
- 支持结构共享，减少内存使用
- 高效的批量操作

### 2. 索引优化
- 节点映射：`ImHashMap<NodeId, NodeIndex>`
- 关系索引：`ImHashMap<RelationType, ImVector<EdgeIndex>>`
- 快速查找和遍历

### 3. 快照管理
- 自动快照数量限制
- 增量快照创建
- 内存使用优化

## 扩展性

### 1. 自定义关系类型
```rust
pub enum RelationType {
    ParentChild,
    Reference,
    Dependency,
    Association,
    Contains,
    Inherits,
    Implements,
    Composition,
    Aggregation,
    Custom(String),
}
```

### 2. 自定义行为
```rust
pub enum BehaviorType {
    AutoCreateChildren,
    AutoCreateParent,
    AutoSetAttributes,
    AutoCreateRelations,
    Custom(String),
}
```

### 3. 插件系统
- 支持自定义创建规则
- 支持自定义验证规则
- 支持自定义行为定义

## 与原有系统的兼容性

### 1. 与 NodeType 的对比

| 功能 | 原 NodeType | 新 GraphNodeType |
|------|-------------|------------------|
| 递归创建 | ✅ create_and_fill | ✅ create_and_fill |
| 属性管理 | ✅ 基础属性 | ✅ 扩展属性系统 |
| 内容验证 | ✅ check_content | ✅ 规则系统 |
| 版本控制 | ❌ 不支持 | ✅ 完整支持 |
| 关系管理 | ❌ 仅父子关系 | ✅ 多种关系类型 |
| 约束系统 | ❌ 基础约束 | ✅ 复杂约束系统 |

### 2. 迁移路径

```rust
// 从原 NodeType 迁移到 GraphNodeType
let old_node_type = NodeType::new("document".to_string(), spec);
let new_graph_node_type = GraphNodeType::new("document".to_string(), spec);

// 保持原有的 create_and_fill 功能
let node_id = new_graph_node_type.create_and_fill(
    &mut graph,
    Some(id),
    Some(attrs),
    content,
    marks,
    schema,
    node_types,
)?;
```

## 总结

ModuForge-RS 版本化图模型成功结合了：

1. **StableDiGraph 的图算法能力** - 支持复杂的图遍历和算法
2. **im 的不可变特性** - 提供版本控制和内存效率
3. **递归创建功能** - 保持与原 NodeType 的兼容性
4. **版本控制系统** - 支持完整的快照和恢复功能
5. **规则驱动系统** - 支持复杂的节点创建和验证规则
6. **约束系统** - 支持复杂的节点关系约束

这个设计既保持了与原有系统的兼容性，又提供了更强大的功能和更好的性能。