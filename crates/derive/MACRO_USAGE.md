# ModuForge-RS 派生宏使用指南

本文档详细说明了 ModuForge-RS 框架中 `#[derive(Node)]` 和 `#[derive(Mark)]` 派生宏的使用方法。

## 目录

- [概述](#概述)
- [Node 派生宏](#node-派生宏)
- [Mark 派生宏](#mark-派生宏)
- [属性说明](#属性说明)
- [字段类型支持](#字段类型支持)
- [完整示例](#完整示例)
- [常见错误](#常见错误)
- [最佳实践](#最佳实践)

## 概述

ModuForge-RS 提供了两个主要的派生宏：

- `#[derive(Node)]` - 用于定义节点类型
- `#[derive(Mark)]` - 用于定义标记类型

这些宏自动生成必要的转换方法，简化了节点和标记的定义过程。

## Node 派生宏

### 基本用法

```rust
use mf_derive::Node;

#[derive(Node)]
#[node_type = "paragraph"]
#[desc = "段落节点"]
pub struct ParagraphNode {
    #[attr]
    content: String,
}
```

### 生成的方法

`#[derive(Node)]` 会为结构体生成以下方法：

1. **`node_definition() -> mf_core::node::Node`** - 静态方法，返回节点定义
2. **`to_node(&self) -> mf_model::node::Node`** - 实例方法，将结构体转换为节点实例
3. **`from(node: &mf_model::node::Node) -> Result<Self, String>`** - 静态方法，从节点创建结构体
4. **`default_instance() -> Self`** - 私有方法，创建默认实例（错误恢复用）

### 使用示例

```rust
// 创建节点定义（用于 lazy_static）
lazy_static! {
    pub static ref PARAGRAPH: mf_core::node::Node = ParagraphNode::node_definition();
}

// 从结构体创建节点实例
let paragraph = ParagraphNode {
    content: "Hello World".to_string(),
};
let node_instance = paragraph.to_node();

// 从节点实例创建结构体
let paragraph_back = ParagraphNode::from(&node_instance)?;
```

## Mark 派生宏

### 基本用法

```rust
use mf_derive::Mark;

#[derive(Mark)]
#[mark_type = "bold"]
pub struct BoldMark {
    #[attr]
    weight: String,
}
```

### 生成的方法

`#[derive(Mark)]` 会为结构体生成以下方法：

1. **`mark_definition() -> mf_core::mark::Mark`** - 静态方法，返回标记定义
2. **`to_mark(&self) -> mf_model::mark::Mark`** - 实例方法，将结构体转换为标记实例
3. **`from(mark: &mf_model::mark::Mark) -> Result<Self, String>`** - 静态方法，从标记创建结构体

## 属性说明

### 结构体级属性

#### Node 属性

| 属性 | 必需 | 描述 | 示例 |
|------|------|------|------|
| `node_type` | ✅ | 节点类型标识符 | `#[node_type = "paragraph"]` |
| `desc` | ❌ | 节点描述 | `#[desc = "段落节点"]` |
| `content` | ❌ | 内容约束表达式 | `#[content = "text*"]` |
| `marks` | ❌ | 支持的标记列表 | `#[marks = "bold italic"]` |

#### Mark 属性

| 属性 | 必需 | 描述 | 示例 |
|------|------|------|------|
| `mark_type` | ✅ | 标记类型标识符 | `#[mark_type = "bold"]` |

### 字段级属性

| 属性 | 描述 | 示例 |
|------|------|------|
| `#[attr]` | 标记字段作为节点/标记属性 | `#[attr] content: String` |
| `#[attr(default="value")]` | 指定默认值 | `#[attr(default="Hello")] text: String` |
| `#[id]` | 标记字段作为节点ID | `#[id] node_id: String` |

## 字段类型支持

### 基本类型

```rust
#[derive(Node)]
#[node_type = "example"]
pub struct ExampleNode {
    #[attr] text: String,           // 字符串
    #[attr] count: i32,             // 整数
    #[attr] price: f64,             // 浮点数
    #[attr] enabled: bool,          // 布尔值
}
```

### 可选类型

```rust
#[derive(Node)]
#[node_type = "example"]
pub struct ExampleNode {
    #[attr] title: Option<String>,   // 可选字符串
    #[attr] size: Option<i32>,       // 可选整数
    #[attr] ratio: Option<f64>,      // 可选浮点数
}
```

### 默认值支持

```rust
#[derive(Node)]
#[node_type = "example"]
pub struct ExampleNode {
    #[attr(default="默认标题")]
    title: String,
    
    #[attr(default=42)]
    count: i32,
    
    #[attr(default=3.14)]
    ratio: f64,
    
    #[attr(default=true)]
    enabled: bool,
}
```

### ID 字段

```rust
#[derive(Node)]
#[node_type = "example"]
pub struct ExampleNode {
    #[id]
    node_id: String,  // 映射到 Node.id
    
    #[attr]
    content: String,  // 映射到 Node.attrs
}
```

## 完整示例

### 复杂节点定义

```rust
use mf_derive::Node;

/// 项目节点 - 支持多种标记和内容约束
#[derive(Node)]
#[node_type = "project"]
#[desc = "项目管理节点"]
#[content = "(task|milestone)*"]
#[marks = "priority status"]
pub struct ProjectNode {
    /// 项目ID - 映射到节点ID
    #[id]
    project_id: String,
    
    /// 项目名称
    #[attr]
    name: String,
    
    /// 项目描述（可选）
    #[attr]
    description: Option<String>,
    
    /// 项目状态，默认为"active"
    #[attr(default="active")]
    status: String,
    
    /// 优先级，默认为1
    #[attr(default=1)]
    priority: i32,
    
    /// 完成百分比，默认为0.0
    #[attr(default=0.0)]
    progress: f64,
    
    /// 是否启用，默认为true
    #[attr(default=true)]
    enabled: bool,
    
    // 非属性字段 - 不会映射到节点属性
    created_at: std::time::SystemTime,
    internal_data: Vec<u8>,
}

impl Default for ProjectNode {
    fn default() -> Self {
        Self {
            project_id: "default_project".to_string(),
            name: String::new(),
            description: None,
            status: "active".to_string(),
            priority: 1,
            progress: 0.0,
            enabled: true,
            created_at: std::time::SystemTime::now(),
            internal_data: Vec::new(),
        }
    }
}

// 使用示例
lazy_static! {
    pub static ref PROJECT: mf_core::node::Node = ProjectNode::node_definition();
}

pub fn create_project_nodes() -> Vec<mf_core::node::Node> {
    vec![PROJECT.clone()]
}
```

### 标记定义示例

```rust
use mf_derive::Mark;

/// 优先级标记
#[derive(Mark)]
#[mark_type = "priority"]
pub struct PriorityMark {
    #[attr(default=1)]
    level: i32,
    
    #[attr(default="normal")]
    label: String,
    
    #[attr]
    color: Option<String>,
}

impl Default for PriorityMark {
    fn default() -> Self {
        Self {
            level: 1,
            label: "normal".to_string(),
            color: None,
        }
    }
}
```

## 常见错误

### 1. 缺少必需属性

```rust
// ❌ 错误：缺少 node_type
#[derive(Node)]
pub struct BadNode {
    content: String,
}

// ✅ 正确：包含必需属性
#[derive(Node)]
#[node_type = "good"]
pub struct GoodNode {
    #[attr]
    content: String,
}
```

### 2. 不支持的字段类型

```rust
// ❌ 错误：不支持 Vec<String> 作为属性
#[derive(Node)]
#[node_type = "bad"]
pub struct BadNode {
    #[attr]
    items: Vec<String>,
}

// ✅ 正确：使用支持的类型
#[derive(Node)]
#[node_type = "good"]
pub struct GoodNode {
    #[attr]
    item_count: i32,
    
    // 或者不标记为属性
    items: Vec<String>,
}
```

### 3. 无效的默认值

```rust
// ❌ 错误：类型不匹配
#[derive(Node)]
#[node_type = "bad"]
pub struct BadNode {
    #[attr(default="not_a_number")]
    count: i32,
}

// ✅ 正确：匹配的类型
#[derive(Node)]
#[node_type = "good"]
pub struct GoodNode {
    #[attr(default=42)]
    count: i32,
}
```

## 最佳实践

### 1. 结构体组织

```rust
// 推荐：将相关字段分组
#[derive(Node)]
#[node_type = "document"]
pub struct DocumentNode {
    // ID 字段放在最前面
    #[id]
    doc_id: String,
    
    // 必需的属性
    #[attr]
    title: String,
    
    // 可选的属性
    #[attr]
    author: Option<String>,
    
    #[attr]
    tags: Option<String>,
    
    // 带默认值的属性
    #[attr(default="draft")]
    status: String,
    
    #[attr(default=0)]
    version: i32,
    
    // 非属性字段放在最后
    internal_metadata: serde_json::Value,
}
```

### 2. 文档注释

```rust
/// 文档节点 - 表示一个可编辑的文档
/// 
/// 支持的标记：bold, italic, underline
/// 内容约束：可包含段落和标题
#[derive(Node)]
#[node_type = "document"]
#[desc = "可编辑文档"]
#[content = "(paragraph|heading)*"]
#[marks = "bold italic underline"]
pub struct DocumentNode {
    /// 文档唯一标识符
    #[id]
    doc_id: String,
    
    /// 文档标题
    #[attr]
    title: String,
    
    /// 文档作者（可选）
    #[attr]
    author: Option<String>,
}
```

### 3. 错误处理

```rust
pub fn create_document_from_node(node: &mf_model::node::Node) -> Result<DocumentNode, String> {
    DocumentNode::from(node).map_err(|e| {
        format!("创建文档失败: {}", e)
    })
}
```

### 4. 类型安全

```rust
// 推荐：使用枚举而不是字符串表示状态
#[derive(Debug, Clone)]
pub enum DocumentStatus {
    Draft,
    Review,
    Published,
}

impl Default for DocumentStatus {
    fn default() -> Self {
        DocumentStatus::Draft
    }
}

#[derive(Node)]
#[node_type = "document"]
pub struct DocumentNode {
    #[attr]
    title: String,
    
    // 非属性字段，使用强类型
    status: DocumentStatus,
    
    // 如果需要序列化为属性，使用字符串
    #[attr(default="draft")]
    status_str: String,
}
```

## 总结

ModuForge-RS 的派生宏提供了一种声明式、类型安全的方式来定义节点和标记。通过正确使用这些宏，可以：

1. **简化代码** - 自动生成转换方法
2. **提高类型安全** - 编译时检查类型兼容性
3. **改善可维护性** - 清晰的结构体定义
4. **减少错误** - 自动处理复杂的转换逻辑

遵循本文档中的最佳实践，可以充分发挥派生宏的优势，构建高质量的 ModuForge-RS 应用。

