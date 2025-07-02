# ModuForge 数据模型 (moduforge-model)

`moduforge-model` 是 ModuForge 生态系统的核心数据模型模块，提供了完整的文档数据结构和类型系统。该模块基于不可变数据结构设计，支持高性能的文档操作、类型验证和内容匹配。

## 🏗️ 架构概述

ModuForge 数据模型采用分层架构设计，每个组件都有明确的职责：

```
┌─────────────────────────────────────────────────────────────┐
│                        Tree                                 │
│              (文档树 + 分片存储 + 缓存优化)                    │
├─────────────────────────────────────────────────────────────┤
│                    NodePool                                 │
│              (节点池 + 内存管理 + 并发安全)                    │
├─────────────────────────────────────────────────────────────┤
│                      Node                                   │
│              (节点定义 + 属性 + 标记)                         │
├─────────────────────────────────────────────────────────────┤
│                     Schema                                  │
│              (模式定义 + 类型验证 + 约束检查)                  │
├─────────────────────────────────────────────────────────────┤
│                    ContentMatch                             │
│              (内容匹配 + 语法解析 + 状态机)                    │
└─────────────────────────────────────────────────────────────┘
```

## 🧩 核心组件

### 1. Tree
**文件**: `src/tree.rs`  
**职责**: 文档树管理和分片存储

- **分片存储**: 基于哈希分片的高性能节点存储
- **LRU 缓存**: 智能的节点ID到分片索引缓存
- **并发安全**: 使用不可变数据结构确保线程安全
- **批量操作**: 优化的批量节点操作

**核心特性**:
```rust
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Tree {
    pub root_id: NodeId,
    pub nodes: Vector<im::HashMap<NodeId, Arc<Node>>>, // 分片存储
    pub parent_map: im::HashMap<NodeId, NodeId>,       // 父子关系映射
    num_shards: usize,                                 // 分片数量
}
```

**分片优化**:
```rust
impl Tree {
    // 智能分片索引计算
    pub fn get_shard_index(&self, id: &NodeId) -> usize;
    
    // 批量分片索引计算
    pub fn get_shard_index_batch<'a>(&self, ids: &'a [&'a NodeId]) -> Vec<(usize, &'a NodeId)>;
    
    // 节点操作
    pub fn add(&mut self, parent_id: &NodeId, nodes: Vec<NodeEnum>) -> PoolResult<()>;
    pub fn remove_node(&mut self, parent_id: &NodeId, nodes: Vec<NodeId>) -> PoolResult<()>;
    pub fn move_node(&mut self, source_parent_id: &NodeId, target_parent_id: &NodeId, node_id: &NodeId, position: Option<usize>) -> PoolResult<()>;
}
```

### 2. Node
**文件**: `src/node.rs`  
**职责**: 基础节点定义

- **不可变设计**: 基于 `im::Vector` 的不可变数据结构
- **序列化优化**: 紧凑的 JSON 序列化格式
- **类型安全**: 完整的类型定义和验证

**节点结构**:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    #[serde(rename = "i")]
    pub id: NodeId,                    // 节点ID
    #[serde(rename = "t")]
    pub r#type: String,                // 节点类型
    #[serde(rename = "a")]
    pub attrs: Attrs,                  // 节点属性
    #[serde(rename = "c")]
    pub content: im::Vector<NodeId>,   // 子节点列表
    #[serde(rename = "m")]
    pub marks: im::Vector<Mark>,       // 标记列表
}
```

### 3. NodeType
**文件**: `src/node_type.rs`  
**职责**: 节点类型定义和验证

- **类型规范**: 完整的节点类型定义
- **内容验证**: 基于内容匹配规则的类型验证
- **属性约束**: 属性定义和默认值管理
- **标记支持**: 支持的标记类型定义

**类型定义**:
```rust
#[derive(Clone, PartialEq, Eq)]
pub struct NodeType {
    pub name: String,                          // 类型名称
    pub spec: NodeSpec,                        // 类型规范
    pub desc: String,                          // 描述信息
    pub groups: Vec<String>,                   // 逻辑分组
    pub attrs: HashMap<String, Attribute>,     // 属性定义
    pub default_attrs: HashMap<String, Value>, // 默认属性
    pub content_match: Option<ContentMatch>,   // 内容匹配规则
    pub mark_set: Option<Vec<MarkType>>,       // 支持的标记类型
}
```

**节点规范**:
```rust
pub struct NodeSpec {
    pub content: Option<String>,                           // 内容约束表达式
    pub marks: Option<String>,                             // 标记类型表达式
    pub group: Option<String>,                             // 逻辑分组
    pub desc: Option<String>,                              // 描述信息
    pub attrs: Option<HashMap<String, AttributeSpec>>,    // 属性规范
}
```

### 4. Schema
**文件**: `src/schema.rs`  
**职责**: 文档模式管理

- **模式编译**: 从规范定义编译为可用的模式
- **类型管理**: 节点类型和标记类型的统一管理
- **缓存机制**: 全局缓存支持
- **验证规则**: 完整的文档结构验证

**模式定义**:
```rust
#[derive(Clone, Debug)]
pub struct Schema {
    pub spec: SchemaSpec,                    // 模式规范
    pub top_node_type: Option<NodeType>,     // 顶级节点类型
    pub cached: Arc<Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>>, // 全局缓存
    pub nodes: HashMap<String, NodeType>,    // 节点类型映射
    pub marks: HashMap<String, MarkType>,    // 标记类型映射
}
```

### 5. ContentMatch
**文件**: `src/content.rs`  
**职责**: 内容匹配和语法解析

- **语法解析**: 支持复杂的内容表达式语法
- **状态机**: 基于 NFA/DFA 的高效匹配
- **内容验证**: 实时内容结构验证
- **智能填充**: 根据规则自动填充缺失内容

**匹配规则**:
```rust
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ContentMatch {
    pub next: Vec<MatchEdge>,           // 匹配边
    pub wrap_cache: Vec<Option<NodeType>>, // 包装缓存
    pub valid_end: bool,                // 是否有效结束
}
```

**语法支持**:
- `*` - 零个或多个
- `+` - 一个或多个
- `?` - 零个或一个
- `|` - 选择
- `()` - 分组
- `{n,m}` - 范围

### 6. Attrs
**文件**: `src/attrs.rs`  
**职责**: 属性系统

- **类型安全**: 类型安全的属性访问
- **不可变设计**: 基于 `im::HashMap` 的不可变属性
- **序列化优化**: 高效的 JSON 序列化
- **索引支持**: 支持索引访问和修改

**属性定义**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attrs {
    pub attrs: HashMap<String, Value>,
}
```

**使用示例**:
```rust
let mut attrs = Attrs::default();
attrs["key1"] = json!("value1");
attrs["key2"] = json!(42);

// 类型安全访问
let value: String = attrs.get_value("key1").unwrap();
```

### 7. Mark
**文件**: `src/mark.rs`  
**职责**: 标记系统

- **格式化支持**: 文本格式化和样式标记
- **属性扩展**: 支持标记属性
- **类型定义**: 完整的标记类型系统

**标记定义**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Mark {
    pub r#type: String,  // 标记类型
    pub attrs: Attrs,    // 标记属性
}
```

## 🔧 技术栈

### 核心依赖
```toml
[dependencies]
# 不可变数据结构
im = { workspace = true }

# 序列化
serde = { workspace = true }
serde_json = { workspace = true }

# 并发和同步
parking_lot = { workspace = true }
dashmap = { workspace = true }
crossbeam = { workspace = true }

# 缓存和性能
lru = { workspace = true }
once_cell = "1.19"

# 异步支持
tokio = { workspace = true }
async-trait = { workspace = true }

# 日志和监控
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
tracing-appender = { workspace = true }

# 工具库
nanoid = "0.4.0"
regex = "1.0"
rand = "0.8"
rayon = { workspace = true }
```

### 核心技术
- **不可变数据结构**: 基于 im-rs 的高性能不可变集合
- **分片存储**: 智能的哈希分片存储策略
- **LRU 缓存**: 高效的缓存管理
- **类型系统**: 完整的类型定义和验证
- **内容匹配**: 基于状态机的内容验证

## 🚀 快速开始

### 基本使用

```rust
use mf_model::{
    Node, Attrs, Mark, Tree, NodeEnum,
    node_type::{NodeType, NodeSpec},
    schema::{Schema, SchemaSpec}
};
use serde_json::json;

// 1. 创建节点
let attrs = Attrs::from([("level".to_string(), json!(1))].into_iter().collect());
let node = Node::new(
    "node1",
    "paragraph".to_string(),
    attrs,
    vec![],
    vec![]
);

// 2. 创建文档树
let tree = Tree::new(node);

// 3. 添加子节点
let child_node = Node::new(
    "node2",
    "text".to_string(),
    Attrs::default(),
    vec![],
    vec![]
);

let node_enum = NodeEnum(child_node, vec![]);
tree.add(&"node1".to_string(), vec![node_enum])?;

// 4. 查询节点
let node = tree.get_node(&"node1".to_string());
let children = tree.children(&"node1".to_string());
```

### 模式定义

```rust
use mf_model::schema::{Schema, SchemaSpec, NodeSpec, MarkSpec};
use std::collections::HashMap;

// 1. 定义节点规范
let mut nodes = HashMap::new();
nodes.insert("paragraph".to_string(), NodeSpec {
    content: Some("inline*".to_string()),
    marks: Some("_".to_string()),
    group: Some("block".to_string()),
    desc: Some("段落节点".to_string()),
    attrs: None,
});

// 2. 定义标记规范
let mut marks = HashMap::new();
marks.insert("strong".to_string(), MarkSpec {
    attrs: None,
    inclusive: true,
    spanning: true,
    code: false,
});

// 3. 创建模式规范
let schema_spec = SchemaSpec {
    nodes,
    marks,
    top_node: Some("doc".to_string()),
};

// 4. 编译模式
let schema = Schema::compile(schema_spec)?;
```

### 内容匹配

```rust
use mf_model::content::ContentMatch;
use mf_model::node_type::NodeType;

// 1. 解析内容表达式
let content_match = ContentMatch::parse(
    "paragraph+".to_string(),
    &node_types
);

// 2. 验证内容
let nodes = vec![paragraph_node, text_node];
if let Some(result) = content_match.match_fragment(&nodes, &schema) {
    if result.valid_end {
        println!("内容验证通过");
    }
}

// 3. 智能填充
if let Some(needed_types) = content_match.fill(&existing_nodes, true, &schema) {
    println!("需要添加的节点类型: {:?}", needed_types);
}
```

### 属性操作

```rust
use mf_model::attrs::Attrs;
use serde_json::json;

// 1. 创建属性
let mut attrs = Attrs::default();
attrs["color"] = json!("red");
attrs["size"] = json!(12);

// 2. 类型安全访问
let color: String = attrs.get_value("color").unwrap();
let size: i32 = attrs.get_value("size").unwrap();

// 3. 更新属性
let new_values = [("bold".to_string(), json!(true))].into_iter().collect();
let updated_attrs = attrs.update(new_values);
```

## 📊 性能特性

### 分片存储优化
```rust
// 自动分片计算
let shard_index = tree.get_shard_index(&node_id);

// 批量分片操作
let shard_indices = tree.get_shard_index_batch(&node_ids);

// 缓存管理
Tree::clear_shard_cache();
```

### 内存管理
- **不可变数据结构**: 减少内存分配和复制
- **智能缓存**: LRU 缓存优化访问性能
- **分片存储**: 减少锁竞争和内存碎片
- **对象池**: 复用昂贵的对象实例

### 并发安全
- **无锁设计**: 基于不可变数据结构的无锁操作
- **原子操作**: 支持原子性的批量操作
- **线程安全**: 完整的线程安全保证

## 🔒 错误处理

### 错误类型
```rust
pub mod error_messages {
    pub const DUPLICATE_NODE: &str = "重复的节点 ID";
    pub const PARENT_NOT_FOUND: &str = "父节点不存在";
    pub const CHILD_NOT_FOUND: &str = "子节点不存在";
    pub const NODE_NOT_FOUND: &str = "节点不存在";
    pub const ORPHAN_NODE: &str = "检测到孤立节点";
    pub const INVALID_PARENTING: &str = "无效的父子关系";
    pub const CYCLIC_REFERENCE: &str = "检测到循环引用";
    pub const CANNOT_REMOVE_ROOT: &str = "无法删除根节点";
}
```

### 错误处理示例
```rust
use mf_model::error::{PoolResult, error_helpers};

fn add_node_safely(tree: &mut Tree, parent_id: &NodeId, node: Node) -> PoolResult<()> {
    // 检查父节点是否存在
    if !tree.contains_node(parent_id) {
        return Err(error_helpers::parent_not_found(parent_id.clone()));
    }
    
    // 检查节点ID是否重复
    if tree.contains_node(&node.id) {
        return Err(error_helpers::duplicate_node(node.id.clone()));
    }
    
    // 安全添加节点
    tree.add_node(parent_id, &vec![node])
}
```

## 🧪 测试

### 测试覆盖
```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test tree
cargo test content
cargo test schema

# 运行性能测试
cargo test --release
```

### 测试示例
```rust
#[test]
fn test_tree_operations() {
    let mut tree = Tree::new(create_test_node("root"));
    
    // 测试添加节点
    let child = create_test_node("child");
    tree.add_node(&"root".to_string(), &vec![child]).unwrap();
    
    // 测试查询节点
    assert!(tree.contains_node(&"child".to_string()));
    assert_eq!(tree.children_count(&"root".to_string()), 1);
    
    // 测试移动节点
    tree.move_node(&"root".to_string(), &"root".to_string(), &"child".to_string(), Some(0)).unwrap();
    
    // 测试删除节点
    tree.remove_node(&"root".to_string(), vec!["child".to_string()]).unwrap();
    assert!(!tree.contains_node(&"child".to_string()));
}
```

## 🔧 配置选项

### 分片配置
```rust
// 自定义分片数量
let num_shards = std::cmp::max(
    std::thread::available_parallelism()
        .map(NonZeroUsize::get)
        .unwrap_or(2),
    2,
);

// 缓存大小配置
static SHARD_INDEX_CACHE: Lazy<RwLock<LruCache<String, usize>>> =
    Lazy::new(|| RwLock::new(LruCache::new(NonZeroUsize::new(10000).unwrap())));
```

### 性能调优
```rust
// 批量操作优化
let shard_indices = tree.get_shard_index_batch(&node_ids);

// 缓存清理
Tree::clear_shard_cache();

// 内存优化
let compact_tree = tree.compact();
```

## 📈 性能优化

### 内存优化
- **不可变数据结构**: 减少内存分配
- **分片存储**: 减少内存碎片
- **智能缓存**: 优化访问模式
- **对象复用**: 减少 GC 压力

### 并发优化
- **无锁设计**: 基于不可变数据结构
- **分片隔离**: 减少锁竞争
- **批量操作**: 提高吞吐量
- **异步支持**: 非阻塞操作

### 算法优化
- **哈希分片**: O(1) 平均访问时间
- **LRU 缓存**: 优化热点数据访问
- **状态机**: 高效的内容匹配
- **批量处理**: 减少系统调用

## 📚 相关文档

- [ModuForge 核心模块](../core/README.md)
- [ModuForge 状态管理](../state/README.md)
- [ModuForge 转换系统](../transform/README.md)
- [ModuForge 协作系统](../collaboration/README.md)

## 🤝 贡献指南

欢迎贡献代码！请确保：

1. 遵循 Rust 编码规范
2. 添加适当的测试
3. 更新相关文档
4. 通过所有 CI 检查
5. 性能测试通过

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](../../LICENSE) 文件。 