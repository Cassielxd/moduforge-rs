# ModuForge-RS 宏系统开发指南

## 概述

ModuForge-RS 提供了强大的过程宏系统，通过 `#[derive(Node)]` 和 `#[derive(Mark)]` 自动生成节点和标记的相关代码。宏系统设计严格遵循 SOLID 原则，提供类型安全、灵活且高性能的代码生成能力。

## 核心特性

### 🎯 设计分离原则
- **节点定义 vs 实例创建**: `node_definition()` 只包含 `#[attr]` 字段的模式定义，`from()` 处理所有字段的实例创建
- **属性精确性**: 只有标记了 `#[attr]` 的字段才会成为节点的属性定义
- **类型安全**: 支持泛型类型和自定义类型的安全转换，要求自定义类型实现 `Default + Serialize` traits

### 🚀 支持的功能特性
- **基本类型**: `String`, `i32`, `f64`, `bool` 等
- **泛型类型**: `Option<T>`, `Vec<T>`, `HashMap<K,V>` 等
- **自定义类型**: 支持构造函数表达式，如 `CustomStruct::new()`
- **JSON 默认值**: 支持复杂的 JSON 结构作为默认值
- **双向转换**: 自动生成 `From` trait 实现
- **错误处理**: 类型验证与优雅降级

## Node 宏使用指南

### 基本用法

```rust
use mf_derive::Node;

#[derive(Node)]
#[node_type = "paragraph"]
#[marks = "bold italic"]
#[content = "text*"]
struct ParagraphNode {
    #[attr]
    text: String,
    
    #[attr(default=1)]
    level: i32,
    
    // 非属性字段，不会出现在 node_definition() 中
    cache: String,
}
```

### 高级功能示例

#### 1. 泛型类型支持

```rust
#[derive(Node)]
#[node_type = "document"]
struct DocumentNode {
    #[attr]
    title: String,
    
    // Option 类型
    #[attr]
    subtitle: Option<String>,
    
    // Vec 类型
    #[attr]
    tags: Vec<String>,
    
    // 复杂泛型类型
    #[attr]
    metadata: HashMap<String, String>,
}
```

#### 2. 自定义类型表达式

```rust
use serde::{Serialize, Deserialize};

#[derive(Default, Serialize, Clone)]
struct DocumentConfig {
    pub auto_backup: bool,
    pub sync_enabled: bool,
}

impl DocumentConfig {
    pub fn new() -> Self {
        Self {
            auto_backup: true,
            sync_enabled: false,
        }
    }
}

#[derive(Node)]
#[node_type = "document"]
struct AdvancedDocumentNode {
    // 使用自定义构造函数
    #[attr(default="DocumentConfig::new()")]
    config: DocumentConfig,
    
    // 使用带参数的构造函数
    #[attr(default="HashMap::with_capacity(10)")]
    cache: HashMap<String, String>,
    
    // 使用链式调用
    #[attr(default="SettingsBuilder::new().with_defaults().build()")]
    settings: DocumentSettings,
}
```

#### 3. JSON 默认值

```rust
#[derive(Node)]
#[node_type = "ui_component"]
struct UIComponentNode {
    // JSON 对象默认值
    #[attr(default={"theme": "light", "auto_save": true, "max_history": 50})]
    ui_config: serde_json::Value,
    
    // JSON 数组默认值
    #[attr(default=["draft", "review", "published"])]
    workflow_states: serde_json::Value,
}
```

### 生成的方法

每个 `#[derive(Node)]` 结构体会自动生成以下方法：

#### 1. `node_definition()` - 静态方法
```rust
/// 获取节点定义（模式定义）
/// 只包含 #[attr] 标记字段的 AttributeSpec
pub fn node_definition() -> mf_core::node::Node {
    // 自动生成的实现
}
```

#### 2. `from()` - 实例创建方法
```rust
/// 从 mf_model::node::Node 创建结构体实例
/// 处理所有字段（包括非 #[attr] 字段）
pub fn from(node: &mf_model::node::Node) -> Result<Self, String> {
    // 自动生成的实现
}
```

#### 3. `default_instance()` - 默认实例方法
```rust
/// 创建默认实例（失败时的降级方法）
fn default_instance() -> Self {
    // 自动生成的实现
}
```

#### 4. `From` trait 实现
```rust
// 双向转换支持
impl From<MyStruct> for mf_core::node::Node { ... }
impl From<mf_model::node::Node> for MyStruct { ... }
```

## Mark 宏使用指南

### 基本用法

```rust
use mf_derive::Mark;

#[derive(Mark)]
#[mark_type = "emphasis"]
struct EmphasisMark {
    #[attr]
    level: String,
    
    #[attr]
    color: Option<String>,
}
```

### 生成的方法

```rust
impl EmphasisMark {
    /// 将结构体转换为 mf_core::mark::Mark 实例
    pub fn to_mark(&self) -> mf_core::mark::Mark {
        // 自动生成的实现
    }
}
```

## 完整示例：文档节点

以下是一个展示所有功能的完整示例：

```rust
use mf_derive::Node;
use uuid::Uuid;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// 支持类型定义
#[derive(Default, Serialize, Clone)]
struct DocumentConfig {
    pub auto_backup: bool,
    pub sync_enabled: bool,
}

impl DocumentConfig {
    pub fn new() -> Self {
        Self {
            auto_backup: true,
            sync_enabled: false,
        }
    }
}

#[derive(Default, Serialize, Clone)]
struct DocumentSettings {
    pub theme: String,
    pub font_size: i32,
    pub line_height: f32,
}

struct SettingsBuilder {
    settings: DocumentSettings,
}

impl SettingsBuilder {
    pub fn new() -> Self {
        Self {
            settings: DocumentSettings::default(),
        }
    }
    
    pub fn with_defaults(mut self) -> Self {
        self.settings.theme = "system".to_string();
        self.settings.font_size = 14;
        self.settings.line_height = 1.5;
        self
    }
    
    pub fn build(self) -> DocumentSettings {
        self.settings
    }
}

/// 完整功能的文档节点
/// 
/// 演示所有 Node 派生宏支持的功能：
/// - 基本属性和默认值
/// - 泛型类型支持（Option<T>, Vec<T>）
/// - 自定义类型表达式
/// - 复杂 JSON 默认值
/// - 非属性字段处理
#[derive(Node)]
#[node_type = "document"]
#[marks = "bold italic underline strikethrough"]
#[content = "block+"]
struct DocumentNode {
    // === 基本属性字段 ===
    
    /// 文档标题（必需属性）
    #[attr]
    title: String,
    
    /// 文档描述（带字符串默认值）
    #[attr(default="未命名文档")]
    description: String,
    
    /// 文档版本（带数值默认值）
    #[attr(default=1)]
    version: i32,
    
    /// 是否已发布（带布尔默认值）
    #[attr(default=true)]
    is_published: bool,
    
    /// 权重分数（带浮点数默认值）
    #[attr(default=5.0)]
    weight: f64,
    
    // === 可选类型字段 ===
    
    /// 可选的子标题
    #[attr]
    subtitle: Option<String>,
    
    /// 可选的优先级
    #[attr]
    priority: Option<i32>,
    
    /// 可选的标签列表（带 null 默认值）
    #[attr(default=null)]
    tags: Option<Vec<String>>,
    
    // === 复杂类型字段 ===
    
    /// 文档唯一标识符（UUID 类型）
    #[attr]
    document_id: Uuid,
    
    /// 二进制数据
    #[attr]
    binary_data: Vec<u8>,
    
    /// 字符串向量
    #[attr]
    categories: Vec<String>,
    
    // === 自定义类型表达式 ===
    
    /// 自定义配置（使用构造函数）
    #[attr(default="DocumentConfig::new()")]
    config: DocumentConfig,
    
    /// 元数据映射（使用带参数的构造函数）
    #[attr(default="HashMap::with_capacity(10)")]
    metadata: HashMap<String, String>,
    
    /// 构建器模式（链式调用）
    #[attr(default="SettingsBuilder::new().with_defaults().build()")]
    settings: DocumentSettings,
    
    // === JSON 默认值 ===
    
    /// 复杂 JSON 配置
    #[attr(default={"theme": "light", "auto_save": true, "max_history": 50})]
    ui_config: serde_json::Value,
    
    /// JSON 数组配置
    #[attr(default=["draft", "review", "published"])]
    workflow_states: serde_json::Value,
    
    // === 非属性字段（不会出现在 node_definition 中）===
    
    /// 运行时计算的字段
    computed_hash: String,
    
    /// 缓存数据
    cache: Option<Vec<u8>>,
    
    /// 内部状态标记
    _internal_state: std::marker::PhantomData<()>,
}
```

## 使用示例

### 1. 获取节点定义

```rust
// 获取节点定义（用于模式定义）
let node_definition = DocumentNode::node_definition();
println!("节点类型: {}", node_definition.name);
println!("支持的标记: {:?}", node_definition.spec.marks);
println!("内容表达式: {:?}", node_definition.spec.content);
```

### 2. 创建节点实例

```rust
// 创建实际的节点实例数据
let mut attrs = imbl::HashMap::new();
attrs.insert("title".to_string(), serde_json::json!("我的文档"));
attrs.insert("version".to_string(), serde_json::json!(2));
attrs.insert("is_published".to_string(), serde_json::json!(false));

let node_instance = mf_model::node::Node {
    id: "doc_001".into(),
    r#type: "document".to_string(),
    attrs: mf_model::attrs::Attrs { attrs },
    content: imbl::Vector::new(),
    marks: imbl::Vector::new(),
};

// 从 Node 转换为结构体（类型安全转换）
match DocumentNode::from(&node_instance) {
    Ok(doc_struct) => {
        println!("转换成功:");
        println!("  标题: {}", doc_struct.title);
        println!("  版本: {}", doc_struct.version);
        println!("  已发布: {}", doc_struct.is_published);
    },
    Err(e) => {
        println!("转换失败: {}", e);
    }
}
```

### 3. 双向转换

```rust
// 使用 .into() 方法进行转换（自动降级）
let doc_struct: DocumentNode = node_instance.into(); // 失败时自动使用 default_instance()

// 反向转换：从结构体到 Node 定义
let definition: mf_core::node::Node = doc_struct.into();
```

## 类型要求

### 自定义类型要求

使用自定义类型表达式时，类型必须满足以下 trait 要求：

```rust
// 自定义类型必须实现这两个 trait
#[derive(Default, Serialize)]
struct MyCustomType {
    // 字段定义
}

// 或者手动实现
impl Default for MyCustomType {
    fn default() -> Self {
        // 默认实现
    }
}

impl Serialize for MyCustomType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        // 序列化实现
    }
}
```

### 支持的表达式模式

宏系统能够识别以下自定义类型表达式模式：

#### 模块路径 + 构造函数
- `CustomStruct::new()`
- `CustomStruct::default()`
- `some_module::CustomStruct::new()`
- `std::collections::HashMap::with_capacity(10)`

#### 常见构造函数模式
- `::new()` - 最常用的构造函数
- `::default()` - 默认构造函数
- `::with_default()` - 带默认参数的构造函数
- `::with_capacity()` - 带容量的构造函数
- `::empty()` - 创建空实例
- `::create()` - 通用创建方法

#### 链式调用模式
- `Builder::new().build()`
- `Config::new().with_field("value").finalize()`
- `CustomStruct::builder().field(value).build()`

## 错误处理

### 类型验证错误

```rust
// 当节点类型不匹配时
let result = DocumentNode::from(&wrong_type_node);
match result {
    Ok(doc) => { /* 处理成功 */ },
    Err(e) => {
        // e 包含详细的错误信息，如：
        // "节点类型不匹配: 期望 'document', 实际 'paragraph'"
        println!("转换失败: {}", e);
    }
}
```

### 优雅降级

```rust
// 使用 .into() 方法时，失败会自动降级到默认实例
let doc_struct: DocumentNode = potentially_invalid_node.into();
// 即使转换失败，也会得到一个有效的默认实例
```

## 性能优化

### 内存效率
- 使用不可变数据结构 (`im-rs`) 进行结构共享
- `Arc` 引用计数减少内存分配
- 类型安全的零拷贝转换

### 编译时优化
- 宏在编译时生成代码，运行时零开销
- 类型检查在编译时完成
- 静态分派，无虚函数调用

### 运行时性能
- 高效的字段访问模式
- 最小化的 JSON 序列化/反序列化
- 优化的错误处理路径

## 集成示例

### 与状态系统集成

```rust
use mf_state::{Transaction, State};
use mf_transform::node_step::AddNodeStep;

async fn add_document_node(state: &State) -> Result<State, Box<dyn std::error::Error>> {
    // 创建文档节点定义
    let node_def = DocumentNode::node_definition();
    
    // 创建事务
    let mut transaction = Transaction::new();
    transaction.add_step(AddNodeStep::new(node_def, None));
    
    // 应用事务
    let new_state = state.apply_transaction(transaction).await?;
    Ok(new_state)
}
```

### 与插件系统集成

```rust
use mf_state::plugin::{Plugin, PluginTrait};
use async_trait::async_trait;

#[derive(Debug)]
struct DocumentValidationPlugin;

#[async_trait]
impl PluginTrait for DocumentValidationPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> Result<Option<Transaction>> {
        // 验证文档节点
        for transaction in transactions {
            for step in &transaction.steps {
                if let Some(add_step) = step.as_any().downcast_ref::<AddNodeStep>() {
                    if add_step.node.name == "document" {
                        // 执行文档特定的验证逻辑
                        validate_document_node(&add_step.node)?;
                    }
                }
            }
        }
        Ok(None)
    }
}

fn validate_document_node(node: &mf_core::node::Node) -> Result<(), ValidationError> {
    // 验证逻辑
    Ok(())
}
```

## 最佳实践

### 1. 设计原则
- **明确分离**: 区分节点定义（schema）和实例创建
- **类型安全**: 优先使用编译时类型检查
- **优雅降级**: 提供合理的默认值和错误处理

### 2. 命名规范
- 使用描述性的节点类型名称
- 属性名称采用 snake_case
- 自定义类型使用 PascalCase

### 3. 性能建议
- 对于大型结构体，考虑使用 `Arc<T>` 包装字段
- 复杂的默认值使用自定义表达式而不是 JSON
- 避免在热路径上进行频繁的类型转换

### 4. 调试技巧
- 使用 `cargo expand` 查看生成的宏代码
- 启用详细错误信息进行调试
- 使用单元测试验证宏生成的代码

## 故障排除

### 常见编译错误

#### 1. 自定义类型缺少 trait 实现
```
error: the trait `Default` is not implemented for `CustomType`
```
**解决方案**: 为自定义类型实现 `Default` 和 `Serialize` traits

#### 2. 无效的默认值表达式
```
error: expected expression, found `invalid_syntax`
```
**解决方案**: 检查默认值表达式的语法正确性

#### 3. 节点类型不匹配
```
节点类型不匹配: 期望 'document', 实际 'paragraph'
```
**解决方案**: 确保传入的 Node 实例类型与结构体的 `node_type` 一致

### 调试步骤

1. **检查宏属性**: 确保所有必需的属性都已正确设置
2. **验证类型要求**: 确保自定义类型实现了必需的 traits
3. **测试表达式**: 验证自定义类型表达式能够正确编译和执行
4. **单元测试**: 为生成的代码编写全面的单元测试

## 相关文档

- **[宏展开示例](./macro-expansion-example.md)** - 查看完整的宏展开结果和代码生成示例
- **[快速入门指南](./quick-start.md)** - 学习如何在项目中开始使用宏系统
- **[API 参考](./api-reference.md)** - 详细的宏API文档

这个宏系统为 ModuForge-RS 提供了强大而灵活的代码生成能力，通过遵循最佳实践和设计原则，可以构建出高性能、类型安全且易于维护的应用程序。