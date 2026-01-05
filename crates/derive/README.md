# ModuForge-RS Derive Macros

[![Crates.io](https://img.shields.io/crates/v/moduforge-macros-derive)](https://crates.io/crates/moduforge-macros-derive)
[![Documentation](https://docs.rs/moduforge-macros-derive/badge.svg)](https://docs.rs/moduforge-macros-derive)
[![License](https://img.shields.io/crates/l/moduforge-macros-derive)](LICENSE)

ModuForge-RS 的派生宏库，提供声明式的节点（Node）和标记（Mark）定义功能，极大简化了 ModuForge-RS 框架的使用。

## 🚀 特性

- **声明式定义**：通过属性宏配置节点和标记的行为
- **类型安全**：编译时验证类型兼容性和配置有效性
- **自动代码生成**：自动生成转换方法和辅助函数
- **友好的错误提示**：详细的编译错误信息和修复建议
- **零运行时开销**：所有转换在编译期完成
- **丰富的类型支持**：支持基本类型、可选类型、默认值等

## 📦 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
moduforge-macros-derive = "0.1.0"
moduforge-core = "0.1.0"
moduforge-model = "0.1.0"
```

## 🎯 快速开始

### Node 派生宏

```rust
use mf_derive::Node;

#[derive(Node)]
#[node_type = "paragraph"]
#[desc = "段落节点"]
#[content = "text*"]
#[marks = "bold italic"]
pub struct ParagraphNode {
    #[id]
    node_id: String,

    #[attr]
    content: String,

    #[attr(default="left")]
    alignment: String,
}

// 生成的方法说明：

// 1. node_definition() - 静态方法，返回节点类型定义
let node_def = ParagraphNode::node_definition();  // 返回 mf_core::node::Node

// 2. to_node() - 实例方法，将结构体转换为 Node
let paragraph = ParagraphNode {
    node_id: "p1".to_string(),
    content: "Hello World".to_string(),
    alignment: "center".to_string(),
};
let node = paragraph.to_node();  // 返回 mf_model::node::Node

// 3. from() - 静态方法，从 Node 创建结构体
let paragraph_back = ParagraphNode::from(&node)?;  // 返回 Result<ParagraphNode, String>
```

**为什么这些方法有返回值？**

- `node_definition()` → 返回节点类型定义，用于注册到框架
- `to_node()` → 返回转换后的 Node 对象，用于存储和处理
- `from()` → 返回转换结果，可能成功（返回结构体）或失败（返回错误）

### Mark 派生宏

```rust
use mf_derive::Mark;

#[derive(Mark)]
#[mark_type = "bold"]
pub struct BoldMark {
    #[attr(default=700)]
    weight: i32,

    #[attr]
    color: Option<String>,
}

// 使用示例
let bold = BoldMark {
    weight: 900,
    color: Some("#000000".to_string()),
};

let mark = bold.to_mark();
```

## 🏗️ 架构设计

```
moduforge-macros-derive/
├── src/
│   ├── lib.rs              # 宏入口点
│   ├── node/               # Node 派生宏实现
│   │   ├── mod.rs
│   │   └── derive_impl.rs  # 核心处理逻辑
│   ├── mark/               # Mark 派生宏实现
│   │   ├── mod.rs
│   │   └── derive_impl.rs
│   ├── parser/             # 属性解析器
│   │   ├── attribute_parser.rs
│   │   ├── field_analyzer.rs
│   │   └── default_value.rs
│   ├── generator/          # 代码生成器
│   │   ├── node_generator.rs
│   │   └── mark_generator.rs
│   ├── converter/          # 类型转换器
│   │   ├── type_converter.rs
│   │   └── builtin_converters.rs
│   └── common/             # 通用工具
│       ├── error.rs
│       ├── utils.rs
│       └── constants.rs
```

### 模块职责

| 模块 | 职责 | 设计原则 |
|------|------|----------|
| **parser** | 解析宏属性和字段配置 | 单一职责原则 |
| **generator** | 生成 Rust 代码 | 开闭原则 |
| **converter** | 处理类型转换逻辑 | 里氏替换原则 |
| **common** | 提供共享工具和错误处理 | 依赖倒置原则 |

## 📋 属性参考

### Node 结构体级属性

| 属性 | 必需 | 描述 | 示例 |
|------|------|------|------|
| `node_type` | ✅ | 节点类型标识符 | `#[node_type = "paragraph"]` |
| `desc` | ❌ | 节点描述信息 | `#[desc = "段落节点"]` |
| `content` | ❌ | 内容约束表达式 | `#[content = "text*"]` |
| `marks` | ❌ | 支持的标记列表 | `#[marks = "bold italic"]` |

### Mark 结构体级属性

| 属性 | 必需 | 描述 | 示例 |
|------|------|------|------|
| `mark_type` | ✅ | 标记类型标识符 | `#[mark_type = "bold"]` |

### 字段级属性

| 属性 | 描述 | 适用于 | 示例 |
|------|------|--------|------|
| `#[id]` | 标记为节点 ID | Node | `#[id] node_id: String` |
| `#[attr]` | 标记为属性字段 | Node/Mark | `#[attr] content: String` |
| `#[attr(default=...)]` | 指定默认值 | Node/Mark | `#[attr(default=42)] count: i32` |

## 🔧 支持的类型

### 基本类型
- `String` - 字符串
- `i32`, `i64`, `u32`, `u64` - 整数
- `f32`, `f64` - 浮点数
- `bool` - 布尔值

### 可选类型
- `Option<T>` - 其中 T 为支持的基本类型

### 默认值
- 字符串: `#[attr(default="text")]`
- 数字: `#[attr(default=42)]`
- 布尔: `#[attr(default=true)]`

## 📚 高级用法

### 复杂节点定义

```rust
#[derive(Node)]
#[node_type = "task"]
#[desc = "任务节点"]
#[content = "(subtask|comment)*"]
#[marks = "priority status deadline"]
pub struct TaskNode {
    #[id]
    task_id: String,

    #[attr]
    title: String,

    #[attr]
    description: Option<String>,

    #[attr(default="pending")]
    status: String,

    #[attr(default=0)]
    priority: i32,

    #[attr]
    assignee: Option<String>,

    #[attr]
    due_date: Option<String>,

    // 非属性字段不会被序列化
    internal_state: TaskState,
}
```

### 与 lazy_static 集成

```rust
use lazy_static::lazy_static;
use mf_derive::Node;

#[derive(Node)]
#[node_type = "document"]
pub struct DocumentNode {
    #[attr] title: String,
}

lazy_static! {
    // 创建全局节点定义
    pub static ref DOCUMENT: mf_core::node::Node = DocumentNode::node_definition();
}

// 注册节点
pub fn register_nodes() -> Vec<mf_core::node::Node> {
    vec![DOCUMENT.clone()]
}
```

## ⚠️ 错误处理

### 编译时错误

宏提供详细的编译时错误信息：

```rust
// ❌ 错误：缺少必需的 node_type 属性
#[derive(Node)]
pub struct BadNode {
    content: String,
}
// 编译错误：Node 派生宏需要 #[node_type = "..."] 属性

// ❌ 错误：不支持的字段类型
#[derive(Node)]
#[node_type = "bad"]
pub struct BadNode {
    #[attr]
    items: Vec<String>, // Vec 类型不被支持
}
// 编译错误：属性字段不支持 Vec<String> 类型
```

### 运行时错误处理

```rust
// 安全的类型转换
match TaskNode::from(&node) {
    Ok(task) => {
        println!("Task: {}", task.title);
    },
    Err(e) => {
        eprintln!("转换失败: {}", e);
    }
}
```

## 🎯 最佳实践

### 1. 结构化字段组织

```rust
#[derive(Node)]
#[node_type = "article"]
pub struct ArticleNode {
    // ID 字段优先
    #[id]
    article_id: String,

    // 必需属性
    #[attr]
    title: String,
    #[attr]
    author: String,

    // 可选属性
    #[attr]
    subtitle: Option<String>,
    #[attr]
    tags: Option<String>,

    // 带默认值的属性
    #[attr(default="draft")]
    status: String,
    #[attr(default=0)]
    view_count: i32,

    // 内部状态（非属性）
    cached_html: Option<String>,
}
```

### 2. 文档注释

为生成的代码添加文档注释：

```rust
/// 文章节点
///
/// 表示博客系统中的一篇文章
#[derive(Node)]
#[node_type = "article"]
#[desc = "博客文章节点"]
pub struct ArticleNode {
    /// 文章唯一标识符
    #[id]
    article_id: String,

    /// 文章标题
    #[attr]
    title: String,
}
```

### 3. 类型安全设计

使用枚举和强类型：

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

#[derive(Node)]
#[node_type = "task"]
pub struct TaskNode {
    #[attr]
    title: String,

    // 存储为字符串但内部使用枚举
    #[attr(default="pending")]
    status_str: String,

    // 内部强类型表示
    status: TaskStatus,
}

impl TaskNode {
    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.status_str = format!("{:?}", status).to_lowercase();
    }
}
```

## 🔍 调试技巧

### 查看生成的代码

使用 `cargo expand` 查看宏展开后的代码：

```bash
cargo install cargo-expand
cargo expand --package moduforge-macros-derive
```

### 启用详细错误信息

在 `Cargo.toml` 中：

```toml
[profile.dev]
debug = true
debug-assertions = true
```

## 🤝 贡献指南

欢迎贡献！请查看 [CONTRIBUTING.md](../../CONTRIBUTING.md) 了解详情。

### 开发环境设置

```bash
# 克隆仓库
git clone https://github.com/your-org/moduforge-rs.git
cd moduforge-rs/crates/derive

# 运行测试
cargo test

# 运行示例
cargo run --example basic_usage
```

## 📄 许可证

本项目采用 MIT 或 Apache-2.0 双重许可。详见 [LICENSE-MIT](../../LICENSE-MIT) 和 [LICENSE-APACHE](../../LICENSE-APACHE)。

## 🔗 相关链接

- [ModuForge-RS 主项目](https://github.com/your-org/moduforge-rs)
- [API 文档](https://docs.rs/moduforge-macros-derive)
- [示例项目](https://github.com/your-org/moduforge-examples)
- [问题反馈](https://github.com/your-org/moduforge-rs/issues)

## 📝 更新日志

查看 [CHANGELOG.md](CHANGELOG.md) 了解版本更新信息。