# 编译错误解决指南

## Node 宏编译错误："有返回值"问题

### 问题描述

当在结构体上添加 `#[derive(Node)]` 宏后，编译时可能会遇到关于"返回值"的错误提示。

### 常见错误原因和解决方案

#### 1. 结构体没有实现 Default trait

**错误信息**：
```
error[E0277]: the trait bound `YourStruct: Default` is not satisfied
```

**原因**：Node 宏生成的 `from()` 方法在错误恢复时需要默认值。

**解决方案**：

```rust
// ❌ 错误
#[derive(Node)]
#[node_type = "example"]
pub struct ExampleNode {
    #[attr]
    name: String,
}

// ✅ 正确 - 实现 Default trait
#[derive(Node)]
#[node_type = "example"]
pub struct ExampleNode {
    #[attr]
    name: String,
}

impl Default for ExampleNode {
    fn default() -> Self {
        Self {
            name: String::new(),
        }
    }
}

// 或者使用 derive
#[derive(Node, Default)]
#[node_type = "example"]
pub struct ExampleNode {
    #[attr]
    #[default = ""]
    name: String,
}
```

#### 2. 字段类型不兼容

**错误信息**：
```
error: cannot return value of type `Vec<String>`
```

**原因**：某些字段类型不能直接序列化为节点属性。

**解决方案**：

```rust
// ❌ 错误 - Vec 不能作为属性
#[derive(Node)]
#[node_type = "example"]
pub struct ExampleNode {
    #[attr]
    items: Vec<String>,  // 错误：Vec 类型不支持
}

// ✅ 正确 - 使用 String 存储 JSON
#[derive(Node)]
#[node_type = "example"]
pub struct ExampleNode {
    #[attr]
    items_json: String,  // 存储序列化的 JSON

    // 内部使用的 Vec（不标记为 attr）
    items: Vec<String>,
}

impl ExampleNode {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items_json: serde_json::to_string(&items).unwrap_or_default(),
            items,
        }
    }
}
```

#### 3. 缺少必需的依赖

**错误信息**：
```
error: cannot find crate `mf_core`
error: cannot find crate `mf_model`
```

**解决方案**：

在 `Cargo.toml` 中添加必需的依赖：

```toml
[dependencies]
moduforge-macros-derive = "0.1.0"
moduforge-core = { workspace = true }  # 或指定版本
moduforge-model = { workspace = true }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

#### 4. 自定义类型没有实现 Serialize

**错误信息**：
```
error: the trait `Serialize` is not implemented for `CustomType`
```

**原因**：使用自定义类型作为默认值时，需要实现 Serialize。

**解决方案**：

```rust
use serde::Serialize;

// ❌ 错误
struct CustomData {
    value: String,
}

#[derive(Node)]
#[node_type = "example"]
pub struct ExampleNode {
    #[attr(default="CustomData::new()")]
    data: CustomData,
}

// ✅ 正确 - 实现 Serialize
#[derive(Serialize, Default)]
struct CustomData {
    value: String,
}

#[derive(Node)]
#[node_type = "example"]
pub struct ExampleNode {
    #[attr(default="CustomData::default()")]
    data: CustomData,
}
```

#### 5. 生成的方法签名冲突

**错误信息**：
```
error: method `to_node` is already defined
```

**原因**：结构体已经有同名方法。

**解决方案**：

```rust
// ❌ 错误 - 方法名冲突
#[derive(Node)]
#[node_type = "example"]
pub struct ExampleNode {
    name: String,
}

impl ExampleNode {
    pub fn to_node(&self) -> String {  // 冲突！
        self.name.clone()
    }
}

// ✅ 正确 - 使用不同的方法名
impl ExampleNode {
    pub fn to_string(&self) -> String {
        self.name.clone()
    }
}
```

### 调试步骤

1. **查看完整的错误信息**：
```bash
cargo build --verbose 2>&1 | less
```

2. **查看宏展开的代码**：
```bash
cargo expand --package your-package
```

3. **简化测试**：
创建一个最小示例来隔离问题：

```rust
// test_macro.rs
use mf_derive::Node;

#[derive(Node, Default)]
#[node_type = "test"]
pub struct TestNode {
    #[attr]
    value: String,
}

fn main() {
    let node = TestNode::default();
    let _ = node.to_node();
}
```

4. **检查版本兼容性**：
```bash
cargo tree -i moduforge-macros-derive
```

### 完整的正确示例

```rust
use mf_derive::Node;
use serde::{Serialize, Deserialize};

// 自定义类型需要实现 Serialize + Default
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    version: i32,
    author: String,
}

// 主结构体
#[derive(Node)]
#[node_type = "document"]
#[desc = "文档节点"]
pub struct DocumentNode {
    // ID 字段
    #[id]
    doc_id: String,

    // 基本类型属性
    #[attr]
    title: String,

    // 可选属性
    #[attr]
    subtitle: Option<String>,

    // 带默认值的属性
    #[attr(default="draft")]
    status: String,

    #[attr(default=0)]
    views: i32,

    // 自定义类型（需要 Serialize）
    #[attr(default="Metadata::default()")]
    metadata: Metadata,

    // 非属性字段（不需要特殊处理）
    internal_cache: Vec<u8>,
}

// 必须实现 Default
impl Default for DocumentNode {
    fn default() -> Self {
        Self {
            doc_id: uuid::Uuid::new_v4().to_string(),
            title: "未命名文档".to_string(),
            subtitle: None,
            status: "draft".to_string(),
            views: 0,
            metadata: Metadata::default(),
            internal_cache: Vec::new(),
        }
    }
}

// 使用
fn example() {
    let doc = DocumentNode::default();

    // to_node() 返回 mf_model::node::Node
    let node = doc.to_node();

    // from() 返回 Result<DocumentNode, String>
    match DocumentNode::from(&node) {
        Ok(restored) => println!("恢复成功"),
        Err(e) => eprintln!("恢复失败: {}", e),
    }
}
```

### 仍有问题？

如果上述解决方案都不能解决问题，请：

1. 提供完整的错误信息
2. 提供最小可复现代码
3. 提供 Rust 版本信息 (`rustc --version`)
4. 在项目 Issue 中报告：[GitHub Issues]

### 相关文档

- [README.md](README.md) - 基础使用说明
- [EXAMPLES.md](EXAMPLES.md) - 更多示例代码
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - 其他故障排除