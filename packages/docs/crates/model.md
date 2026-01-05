# moduforge-model 文档

`moduforge-model` 是 ModuForge-RS 的数据模型层,提供基于不可变数据结构的树形文档模型、标记系统和 Schema 验证。

## 概述

### 核心功能

- **节点系统**：树形结构的基本单元
- **节点池**：高效的节点存储和管理
- **标记系统**：为文本添加格式和语义
- **Schema 定义**：结构验证和约束
- **ID 生成器**：唯一标识符生成
- **属性系统**：类型安全的节点属性

### 设计理念

1. **不可变性**：所有数据结构都是不可变的，修改会返回新实例
2. **结构共享**：基于 `rpds` 的持久化数据结构，高效共享未修改部分
3. **线程安全**：天然支持并发访问
4. **类型安全**：充分利用 Rust 的类型系统

## 安装

```toml
[dependencies]
moduforge-model = "0.7.0"
```

## 核心组件

### 1. 节点 (Node)

节点是文档树的基本单元。

#### 节点结构

```rust
pub struct Node {
    id: NodeId,                              // 唯一标识符
    node_type: NodeType,                     // 节点类型
    attrs: rpds::HashTrieMap<String, Value>, // 属性
    text: Option<Text>,                      // 文本内容
    children: rpds::Vector<NodeId>,          // 子节点ID列表
    marks: Vec<Mark>,                        // 标记列表
}
```

#### 节点类型

```rust
pub enum NodeType {
    Block(String),   // 块级节点：document, paragraph, heading 等
    Inline(String),  // 内联节点：link, image, mention 等
    Text,            // 纯文本节点
}
```

#### 创建节点

```rust
use mf_model::{Node, NodeType, Attrs};

// 创建文档根节点
let doc = Node::new(
    "doc".into(),
    NodeType::block("document"),
    Attrs::new(),
    None
);

// 创建段落节点
let paragraph = Node::new(
    "p1".into(),
    NodeType::block("paragraph"),
    Attrs::new(),
    Some("这是段落内容".into())
);

// 创建标题节点（带属性）
let mut attrs = Attrs::new();
attrs.insert("level".into(), 1.into());

let heading = Node::new(
    "h1".into(),
    NodeType::block("heading"),
    attrs,
    Some("标题文本".into())
);

// 创建链接节点
let mut link_attrs = Attrs::new();
link_attrs.insert("href".into(), "https://example.com".into());

let link = Node::new(
    "link1".into(),
    NodeType::inline("link"),
    link_attrs,
    Some("链接文本".into())
);
```

#### 节点操作

```rust
// 获取节点信息
let id = node.id();
let node_type = node.node_type();
let text = node.text();
let children = node.children();

// 检查节点类型
if node.is_block() {
    println!("这是块级节点");
}

if node.is_inline() {
    println!("这是内联节点");
}

if node.is_text() {
    println!("这是文本节点");
}

// 修改节点（返回新实例）
let updated_node = node.with_text("新文本".into());
let updated_node = node.with_attr("key", "value");
let updated_node = node.add_child("child_id".into());

// 获取属性
if let Some(level) = node.attrs().get("level") {
    println!("标题级别: {}", level);
}
```

### 2. 节点池 (NodePool)

节点池管理所有节点，提供高效的存储和查询。

#### 创建节点池

```rust
use mf_model::NodePool;

let pool = NodePool::new();
```

#### 节点管理

```rust
// 添加节点
pool.add_node(node)?;

// 批量添加
pool.add_nodes(vec![node1, node2, node3])?;

// 获取节点
let node = pool.get_node("node_id")?;

// 检查节点是否存在
if pool.contains("node_id") {
    println!("节点存在");
}

// 更新节点
pool.update_node("node_id", updated_node)?;

// 删除节点
pool.remove_node("node_id")?;

// 获取所有节点
let all_nodes = pool.all_nodes();

// 获取节点数量
let count = pool.len();
```

#### 节点查询

```rust
// 查找特定类型的节点
let paragraphs: Vec<&Node> = pool.all_nodes()
    .iter()
    .filter(|n| n.node_type().name() == "paragraph")
    .collect();

// 查找包含特定文本的节点
let nodes_with_text: Vec<&Node> = pool.all_nodes()
    .iter()
    .filter(|n| n.text().map_or(false, |t| t.contains("关键词")))
    .collect();

// 查找特定属性的节点
let level_1_headings: Vec<&Node> = pool.all_nodes()
    .iter()
    .filter(|n| {
        n.node_type().name() == "heading" &&
        n.attrs().get("level") == Some(&1.into())
    })
    .collect();
```

### 3. 文档树 (Tree)

文档树表示完整的节点层次结构。

#### 创建文档树

```rust
use mf_model::Tree;

// 从节点池创建文档树
let tree = Tree::new(pool, "root_id".into());

// 空文档树
let empty_tree = Tree::empty();
```

#### 树操作

```rust
// 获取根节点
let root = tree.root_node()?;

// 获取节点的父节点
let parent = tree.parent("child_id")?;

// 获取节点的子节点
let children = tree.children("parent_id")?;

// 获取节点的兄弟节点
let siblings = tree.siblings("node_id")?;

// 获取节点的祖先
let ancestors = tree.ancestors("node_id")?;

// 获取节点的后代
let descendants = tree.descendants("node_id")?;
```

#### 树遍历

```rust
// 深度优先遍历
tree.traverse_dfs("root_id", |node| {
    println!("访问节点: {} - {}", node.id(), node.node_type().name());
    Ok(())
})?;

// 广度优先遍历
tree.traverse_bfs("root_id", |node| {
    println!("访问节点: {} - {}", node.id(), node.node_type().name());
    Ok(())
})?;

// 获取树的深度
let depth = tree.depth("root_id")?;

// 获取节点在树中的路径
let path = tree.path_to("node_id")?;
```

### 4. 标记系统 (Mark)

标记用于为文本添加格式和语义信息。

#### 标记定义

```rust
use mf_model::{Mark, MarkType, Attrs};

// 创建加粗标记
let strong = Mark::new(MarkType::Strong, Attrs::new());

// 创建斜体标记
let em = Mark::new(MarkType::Em, Attrs::new());

// 创建代码标记
let code = Mark::new(MarkType::Code, Attrs::new());

// 创建链接标记（带属性）
let mut link_attrs = Attrs::new();
link_attrs.insert("href".into(), "https://example.com".into());
link_attrs.insert("title".into(), "示例链接".into());
let link = Mark::new(MarkType::Link, link_attrs);

// 创建自定义标记
let custom = Mark::new(MarkType::Custom("highlight".into()), Attrs::new());
```

#### 标记类型

```rust
pub enum MarkType {
    Strong,           // 加粗
    Em,               // 斜体
    Code,             // 代码
    Link,             // 链接
    Underline,        // 下划线
    Strike,           // 删除线
    Subscript,        // 下标
    Superscript,      // 上标
    Custom(String),   // 自定义标记
}
```

#### 使用标记

```rust
// 为节点添加标记
let marked_node = node.add_mark(Mark::new(MarkType::Strong, Attrs::new()));

// 移除标记
let unmarked_node = node.remove_mark(&MarkType::Strong);

// 检查节点是否有特定标记
if node.has_mark(&MarkType::Strong) {
    println!("节点有加粗标记");
}

// 获取所有标记
let marks = node.marks();

// 组合多个标记
let text_with_marks = node
    .add_mark(Mark::new(MarkType::Strong, Attrs::new()))
    .add_mark(Mark::new(MarkType::Em, Attrs::new()));
```

### 5. 属性系统 (Attrs)

属性系统提供类型安全的节点属性管理。

#### 属性类型

```rust
pub enum AttrValue {
    String(String),
    Number(i64),
    Float(f64),
    Bool(bool),
    Null,
}

// 类型转换
impl From<&str> for AttrValue {
    fn from(s: &str) -> Self {
        AttrValue::String(s.to_string())
    }
}

impl From<i64> for AttrValue {
    fn from(n: i64) -> Self {
        AttrValue::Number(n)
    }
}

impl From<bool> for AttrValue {
    fn from(b: bool) -> Self {
        AttrValue::Bool(b)
    }
}
```

#### 属性操作

```rust
use mf_model::Attrs;

// 创建空属性集
let mut attrs = Attrs::new();

// 添加属性
attrs.insert("name".into(), "value".into());
attrs.insert("count".into(), 42.into());
attrs.insert("enabled".into(), true.into());

// 获取属性
if let Some(name) = attrs.get("name") {
    println!("名称: {}", name);
}

// 检查属性是否存在
if attrs.contains_key("name") {
    println!("name 属性存在");
}

// 移除属性
attrs.remove("name");

// 合并属性
let merged = attrs.merge(&other_attrs);

// 迭代属性
for (key, value) in attrs.iter() {
    println!("{} = {:?}", key, value);
}
```

### 6. Schema 系统

Schema 定义了文档结构的约束和验证规则。

#### 定义 Schema

```rust
use mf_model::schema::{Schema, NodeSpec, MarkSpec};

let mut schema = Schema::new();

// 定义文档节点
schema.add_node(NodeSpec::new("document")
    .content("block+")           // 内容规则：一个或多个块级节点
    .block()                     // 块级节点
);

// 定义段落节点
schema.add_node(NodeSpec::new("paragraph")
    .content("inline*")          // 内容：零个或多个内联节点
    .marks("strong em code link") // 允许的标记
    .block()
);

// 定义标题节点
schema.add_node(NodeSpec::new("heading")
    .content("text*")
    .attrs(vec!["level"])        // 必需属性
    .attr_default("level", 1.into())
    .block()
);

// 定义列表节点
schema.add_node(NodeSpec::new("bullet_list")
    .content("list_item+")
    .block()
);

schema.add_node(NodeSpec::new("list_item")
    .content("paragraph block*")
    .block()
);

// 定义内联节点
schema.add_node(NodeSpec::new("link")
    .inline()
    .attrs(vec!["href"])
    .content("text*")
);

// 定义标记
schema.add_mark(MarkSpec::new("strong"));
schema.add_mark(MarkSpec::new("em"));
schema.add_mark(MarkSpec::new("code").excludes("link"));
```

#### 内容表达式

Schema 支持强大的内容表达式：

```rust
// 基本表达式
"text"           // 纯文本
"paragraph"      // 段落节点
"block"          // 任意块级节点
"inline"         // 任意内联节点

// 重复表达式
"paragraph+"     // 一个或多个段落
"paragraph*"     // 零个或多个段落
"paragraph?"     // 零个或一个段落
"paragraph{2,5}" // 2到5个段落

// 组合表达式
"heading paragraph+" // 标题后跟一个或多个段落
"block | inline"     // 块级或内联节点
"(paragraph | list)+" // 段落或列表的组合
```

#### 验证文档

```rust
// 验证节点
match schema.validate_node(&node) {
    Ok(_) => println!("节点有效"),
    Err(e) => println!("验证失败: {}", e),
}

// 验证内容
match schema.validate_content("paragraph", &children) {
    Ok(_) => println!("内容有效"),
    Err(e) => println!("内容无效: {}", e),
}

// 检查节点是否可以包含特定子节点
if schema.can_contain("paragraph", "text") {
    println!("段落可以包含文本");
}

// 检查标记是否可以应用到节点
if schema.can_apply_mark("paragraph", &MarkType::Strong) {
    println!("段落可以应用加粗标记");
}
```

### 7. ID 生成器

ID 生成器负责生成唯一的节点标识符。

#### 使用 ID 生成器

```rust
use mf_model::IdGenerator;

// 创建 ID 生成器
let generator = IdGenerator::new();

// 生成 UUID 风格的 ID
let id1 = generator.generate();

// 生成带前缀的 ID
let id2 = generator.generate_with_prefix("node");

// 生成 Base62 编码的短 ID
let id3 = generator.generate_short();

// 自定义 ID 生成策略
let custom_id = generator.generate_custom(|| {
    format!("custom_{}", uuid::Uuid::new_v4())
});
```

## 完整示例

### 示例 1：构建简单文档

```rust
use mf_model::{Node, NodeType, Attrs, NodePool, Tree};

// 创建节点池
let pool = NodePool::new();

// 创建文档节点
let doc = Node::new(
    "doc".into(),
    NodeType::block("document"),
    Attrs::new(),
    None
);
pool.add_node(doc)?;

// 创建标题
let mut h_attrs = Attrs::new();
h_attrs.insert("level".into(), 1.into());
let heading = Node::new(
    "h1".into(),
    NodeType::block("heading"),
    h_attrs,
    Some("欢迎使用 ModuForge".into())
);
pool.add_node(heading)?;

// 创建段落
let p1 = Node::new(
    "p1".into(),
    NodeType::block("paragraph"),
    Attrs::new(),
    Some("这是第一段内容".into())
);
pool.add_node(p1)?;

let p2 = Node::new(
    "p2".into(),
    NodeType::block("paragraph"),
    Attrs::new(),
    Some("这是第二段内容".into())
);
pool.add_node(p2)?;

// 建立父子关系
let doc = pool.get_node("doc")?.add_child("h1".into());
let doc = doc.add_child("p1".into());
let doc = doc.add_child("p2".into());
pool.update_node("doc", doc)?;

// 创建文档树
let tree = Tree::new(pool, "doc".into());

// 遍历文档
tree.traverse_dfs("doc", |node| {
    println!("{}: {}",
        node.node_type().name(),
        node.text().unwrap_or("(无内容)")
    );
    Ok(())
})?;
```

### 示例 2：富文本节点

```rust
use mf_model::{Node, NodeType, Mark, MarkType, Attrs};

// 创建带标记的文本节点
let text = Node::new(
    "text1".into(),
    NodeType::text(),
    Attrs::new(),
    Some("重要的文本内容".into())
)
.add_mark(Mark::new(MarkType::Strong, Attrs::new()))
.add_mark(Mark::new(MarkType::Em, Attrs::new()));

// 创建链接
let mut link_attrs = Attrs::new();
link_attrs.insert("href".into(), "https://example.com".into());

let link = Node::new(
    "link1".into(),
    NodeType::inline("link"),
    link_attrs,
    Some("点击这里".into())
);

// 创建包含链接的段落
let paragraph = Node::new(
    "p1".into(),
    NodeType::block("paragraph"),
    Attrs::new(),
    None
)
.add_child("text1".into())
.add_child("link1".into());
```

### 示例 3：列表结构

```rust
use mf_model::{Node, NodeType, Attrs, NodePool};

let pool = NodePool::new();

// 创建列表容器
let list = Node::new(
    "list1".into(),
    NodeType::block("bullet_list"),
    Attrs::new(),
    None
);
pool.add_node(list)?;

// 创建列表项
for i in 1..=3 {
    let item = Node::new(
        format!("li{}", i).into(),
        NodeType::block("list_item"),
        Attrs::new(),
        None
    );

    let p = Node::new(
        format!("p{}", i).into(),
        NodeType::block("paragraph"),
        Attrs::new(),
        Some(format!("列表项 {}", i).into())
    );

    pool.add_node(p)?;
    pool.add_node(item.add_child(format!("p{}", i).into()))?;

    // 添加到列表
    let list = pool.get_node("list1")?.add_child(format!("li{}", i).into());
    pool.update_node("list1", list)?;
}
```

### 示例 4：Schema 验证

```rust
use mf_model::schema::{Schema, NodeSpec};

// 定义 Schema
let mut schema = Schema::new();

schema.add_node(NodeSpec::new("document")
    .content("heading? paragraph+ list?")
    .block()
);

schema.add_node(NodeSpec::new("paragraph")
    .content("text* | inline*")
    .marks("strong em code")
    .block()
);

schema.add_node(NodeSpec::new("heading")
    .content("text+")
    .attrs(vec!["level"])
    .block()
);

// 验证节点
let node = create_paragraph_node();
match schema.validate_node(&node) {
    Ok(_) => println!("节点符合 Schema"),
    Err(e) => println!("验证失败: {}", e),
}

// 检查内容规则
if schema.can_contain("paragraph", "text") {
    println!("段落可以包含文本");
}
```

## 性能优化

### 1. 使用结构共享

```rust
// 原始节点
let node1 = Node::new("n1", NodeType::block("p"), Attrs::new(), Some("文本"));

// 修改后的节点共享未修改的部分
let node2 = node1.with_text("新文本".into());

// node1 和 node2 共享 attrs 和其他未修改字段
```

### 2. 批量操作

```rust
// ✅ 好：批量添加节点
pool.add_nodes(nodes)?;

// ❌ 差：逐个添加
for node in nodes {
    pool.add_node(node)?;
}
```

### 3. 延迟计算

```rust
// 使用 OnceCell 缓存计算结果
use once_cell::sync::OnceCell;

struct CachedTree {
    tree: Tree,
    size: OnceCell<usize>,
}

impl CachedTree {
    fn size(&self) -> usize {
        *self.size.get_or_init(|| {
            self.tree.count_nodes()
        })
    }
}
```

## 最佳实践

### 1. 节点 ID 管理

```rust
// ✅ 好：使用 ID 生成器
let id = generator.generate();

// ❌ 差：手动创建 ID
let id = "node1".into();  // 可能冲突
```

### 2. 属性类型安全

```rust
// ✅ 好：使用强类型
attrs.insert("level".into(), 1.into());

// ❌ 差：使用字符串
attrs.insert("level".into(), "1".into());  // 类型不匹配
```

### 3. Schema 验证

```rust
// ✅ 好：始终验证节点
schema.validate_node(&node)?;

// ❌ 差：跳过验证
// 可能导致无效的文档结构
```

### 4. 错误处理

```rust
// ✅ 好：正确处理错误
match pool.get_node("id") {
    Ok(node) => process_node(node),
    Err(e) => handle_error(e),
}

// ❌ 差：使用 unwrap
let node = pool.get_node("id").unwrap();  // 可能 panic
```

## 错误处理

```rust
pub enum ModelError {
    NodeNotFound(NodeId),
    InvalidNodeType(String),
    InvalidContent(String),
    SchemaViolation(String),
    AttributeError(String),
}

// 使用 Result 类型
type ModelResult<T> = Result<T, ModelError>;
```

## 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_immutability() {
        let node1 = Node::new("n1", NodeType::block("p"), Attrs::new(), None);
        let node2 = node1.with_text("Hello");

        assert!(node1.text().is_none());
        assert_eq!(node2.text(), Some("Hello"));
    }

    #[test]
    fn test_tree_traversal() {
        let pool = create_test_pool();
        let tree = Tree::new(pool, "root".into());

        let mut count = 0;
        tree.traverse_dfs("root", |_| {
            count += 1;
            Ok(())
        }).unwrap();

        assert_eq!(count, 5);
    }
}
```

## 下一步

- 查看 [moduforge-transform](./transform.md) 了解如何修改文档
- 查看 [moduforge-state](./state.md) 了解状态管理
- 浏览 [示例代码](../examples/) 学习实际应用
