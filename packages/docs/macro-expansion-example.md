# ModuForge-RS Node 宏展开示例

本文档展示了 `#[derive(Node)]` 宏的完整展开结果，演示所有支持的功能特性。

## 原始代码（宏展开前）

```rust
use mf_derive::Node;
use uuid::Uuid;
use std::collections::HashMap;

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

// 支持类型定义
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

impl Serialize for DocumentSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        // 序列化实现...
    }
}
```

## 宏展开后的完整代码

```rust
// ==========================================
// 自动生成的实现代码
// 由 #[derive(Node)] 宏生成
// ==========================================

impl DocumentNode {
    /// 获取节点定义
    ///
    /// 此方法由 #[derive(Node)] 宏自动生成，根据结构体的字段
    /// 和宏属性配置创建节点定义（而非具体实例）。
    ///
    /// # 返回值
    /// 
    /// 返回配置好的 `mf_core::node::Node` 定义
    ///
    /// # 生成说明
    ///
    /// 这个方法是由 ModuForge-RS 宏系统自动生成的，
    /// 它遵循以下设计原则：
    /// - **单一职责**: 只负责 Node 定义的创建
    /// - **语义清晰**: 方法名明确表示返回的是定义而非实例
    /// - **里氏替换**: 生成的 Node 定义可以替换手动创建的定义
    /// - **属性精确性**: 只包含 #[attr] 标记的字段，符合节点定义语义
    pub fn node_definition() -> mf_core::node::Node {
        use mf_model::node_type::NodeSpec;
        use std::collections::HashMap;
        use serde_json::Value as JsonValue;
        
        // 只为有 #[attr] 标记的字段构建属性映射
        let mut attrs_map = std::collections::HashMap::new();
        
        // 基本类型字段
        attrs_map.insert("title".to_string(), mf_model::schema::AttributeSpec {
            default: Some(serde_json::json!(String::default()))
        });
        
        attrs_map.insert("description".to_string(), mf_model::schema::AttributeSpec {
            default: Some(serde_json::json!("未命名文档"))
        });
        
        attrs_map.insert("version".to_string(), mf_model::schema::AttributeSpec {
            default: Some(serde_json::json!(1))
        });
        
        attrs_map.insert("is_published".to_string(), mf_model::schema::AttributeSpec {
            default: Some(serde_json::json!(true))
        });
        
        attrs_map.insert("weight".to_string(), mf_model::schema::AttributeSpec {
            default: Some(serde_json::json!(5.0))
        });
        
        // 可选类型字段
        attrs_map.insert("subtitle".to_string(), mf_model::schema::AttributeSpec {
            default: Some(serde_json::json!(null))
        });
        
        attrs_map.insert("priority".to_string(), mf_model::schema::AttributeSpec {
            default: Some(serde_json::json!(null))
        });
        
        attrs_map.insert("tags".to_string(), mf_model::schema::AttributeSpec {
            default: Some(serde_json::json!(null))
        });
        
        // 复杂类型字段
        attrs_map.insert("document_id".to_string(), mf_model::schema::AttributeSpec {
            default: Some(serde_json::json!(uuid::Uuid::new_v4().to_string()))
        });
        
        attrs_map.insert("binary_data".to_string(), mf_model::schema::AttributeSpec {
            default: Some(serde_json::json!(Vec::<u8>::new()))
        });
        
        attrs_map.insert("categories".to_string(), mf_model::schema::AttributeSpec {
            default: Some(serde_json::json!(Vec::<String>::new()))
        });
        
        // 自定义类型表达式字段
        attrs_map.insert("config".to_string(), mf_model::schema::AttributeSpec {
            default: Some(
                serde_json::to_value(DocumentConfig::new())
                    .unwrap_or_else(|_| serde_json::json!(null))
            )
        });
        
        attrs_map.insert("metadata".to_string(), mf_model::schema::AttributeSpec {
            default: Some(
                serde_json::to_value(HashMap::with_capacity(10))
                    .unwrap_or_else(|_| serde_json::json!(null))
            )
        });
        
        attrs_map.insert("settings".to_string(), mf_model::schema::AttributeSpec {
            default: Some(
                serde_json::to_value(SettingsBuilder::new().with_defaults().build())
                    .unwrap_or_else(|_| serde_json::json!(null))
            )
        });
        
        // JSON 默认值字段
        attrs_map.insert("ui_config".to_string(), mf_model::schema::AttributeSpec {
            default: Some(
                serde_json::from_str(r#"{"theme": "light", "auto_save": true, "max_history": 50}"#)
                    .unwrap_or_else(|_| serde_json::json!(null))
            )
        });
        
        attrs_map.insert("workflow_states".to_string(), mf_model::schema::AttributeSpec {
            default: Some(
                serde_json::from_str(r#"["draft", "review", "published"]"#)
                    .unwrap_or_else(|_| serde_json::json!(null))
            )
        });
        
        let attrs = Some(attrs_map);
        
        let spec = mf_model::node_type::NodeSpec {
            content: Some("block+".to_string()),
            marks: Some("bold italic underline strikethrough".to_string()),
            attrs,
            group: None,
            desc: None,
        };
        
        // 创建并返回 Node 定义
        mf_core::node::Node::create("document", spec)
    }
    
    /// 从 mf_model::node::Node 创建结构体实例
    ///
    /// 此方法由 #[derive(Node)] 宏自动生成，根据 Node 的属性
    /// 创建相应的结构体实例。
    ///
    /// # 参数
    ///
    /// * `node` - 要转换的 Node 实例
    ///
    /// # 返回值
    /// 
    /// 成功时返回结构体实例，失败时返回错误信息
    ///
    /// # 错误
    ///
    /// 当节点类型不匹配时，返回包含错误信息的 Result
    ///
    /// # 生成说明
    ///
    /// 这个方法是由 ModuForge-RS 宏系统自动生成的，
    /// 它遵循以下设计原则：
    /// - **单一职责**: 只负责从 Node 创建结构体实例
    /// - **错误安全**: 使用 Result 类型处理类型不匹配错误
    /// - **字段分离**: #[attr] 字段从 attrs 提取，非 #[attr] 字段使用默认值
    /// - **类型安全**: 支持泛型类型和自定义类型的安全转换
    pub fn from(node: &mf_model::node::Node) -> Result<Self, String> {
        use serde_json::Value as JsonValue;
        
        // 验证节点类型匹配
        if node.r#type != "document" {
            return Err(format!("节点类型不匹配: 期望 'document', 实际 '{}'", node.r#type));
        }
        
        Ok(Self {
            // === #[attr] 字段：从 node.attrs 中提取 ===
            
            // 基本类型提取
            title: node.attrs.attrs.get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default(),
                
            description: node.attrs.attrs.get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "未命名文档".to_string()),
                
            version: node.attrs.attrs.get("version")
                .and_then(|v| v.as_i64())
                .map(|i| i as i32)
                .unwrap_or(1),
                
            is_published: node.attrs.attrs.get("is_published")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
                
            weight: node.attrs.attrs.get("weight")
                .and_then(|v| v.as_f64())
                .unwrap_or(5.0),
            
            // Option 类型处理
            subtitle: node.attrs.attrs.get("subtitle")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
                
            priority: node.attrs.attrs.get("priority")
                .and_then(|v| v.as_i64())
                .map(|i| i as i32),
                
            tags: node.attrs.attrs.get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()),
            
            // 复杂类型提取
            document_id: node.attrs.attrs.get("document_id")
                .and_then(|v| v.as_str())
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .unwrap_or_else(uuid::Uuid::new_v4),
                
            binary_data: node.attrs.attrs.get("binary_data")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_u64().map(|u| u as u8))
                    .collect())
                .unwrap_or_default(),
                
            categories: node.attrs.attrs.get("categories")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect())
                .unwrap_or_default(),
            
            // 自定义类型反序列化
            config: node.attrs.attrs.get("config")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_else(|| DocumentConfig::new()),
                
            metadata: node.attrs.attrs.get("metadata")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_else(|| HashMap::with_capacity(10)),
                
            settings: node.attrs.attrs.get("settings")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_else(|| SettingsBuilder::new().with_defaults().build()),
            
            // JSON 值字段
            ui_config: node.attrs.attrs.get("ui_config")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({
                    "theme": "light", 
                    "auto_save": true, 
                    "max_history": 50
                })),
                
            workflow_states: node.attrs.attrs.get("workflow_states")
                .cloned()
                .unwrap_or_else(|| serde_json::json!(["draft", "review", "published"])),
            
            // === 非 #[attr] 字段：使用类型默认值 ===
            
            computed_hash: <String as Default>::default(),
            cache: <Option<Vec<u8>> as Default>::default(),
            _internal_state: <std::marker::PhantomData<()> as Default>::default(),
        })
    }
    
    /// 创建默认实例
    ///
    /// 当从 Node 转换失败时使用此方法创建默认实例。
    /// 此方法由 #[derive(Node)] 宏自动生成。
    ///
    /// # 返回值
    ///
    /// 返回使用默认值初始化的结构体实例
    fn default_instance() -> Self {
        Self {
            // #[attr] 字段的默认值
            title: String::default(),
            description: "未命名文档".to_string(),
            version: 1,
            is_published: true,
            weight: 5.0,
            subtitle: None,
            priority: None,
            tags: None,
            document_id: uuid::Uuid::new_v4(),
            binary_data: Vec::new(),
            categories: Vec::new(),
            config: DocumentConfig::new(),
            metadata: HashMap::with_capacity(10),
            settings: SettingsBuilder::new().with_defaults().build(),
            ui_config: serde_json::json!({"theme": "light", "auto_save": true, "max_history": 50}),
            workflow_states: serde_json::json!(["draft", "review", "published"]),
            
            // 非 #[attr] 字段的默认值
            computed_hash: String::default(),
            cache: None,
            _internal_state: std::marker::PhantomData,
        }
    }
}

// ==========================================
// From trait 实现（双向转换）
// ==========================================

impl From<DocumentNode> for mf_core::node::Node {
    /// 将结构体实例转换为 mf_core::node::Node
    ///
    /// 实现标准的 From trait，支持使用 `.into()` 方法进行转换。
    /// 此实现由 #[derive(Node)] 宏自动生成。
    ///
    /// # 参数
    ///
    /// * `_value` - 结构体实例（当前实现中使用定义而非实例值）
    ///
    /// # 返回值
    ///
    /// 返回配置好的 `mf_core::node::Node` 定义
    ///
    /// # 使用示例
    ///
    /// ```rust
    /// let my_struct = DocumentNode { /* fields */ };
    /// let node: mf_core::node::Node = my_struct.into();
    /// // 或者
    /// let node = mf_core::node::Node::from(my_struct);
    /// ```
    fn from(_value: DocumentNode) -> Self {
        DocumentNode::node_definition()
    }
}

impl From<mf_model::node::Node> for DocumentNode {
    /// 从 mf_model::node::Node 转换为结构体实例
    ///
    /// 实现标准的 From trait，支持使用 `.into()` 方法进行反向转换。
    /// 此实现由 #[derive(Node)] 宏自动生成。
    ///
    /// # 参数
    ///
    /// * `node` - mf_model::node::Node 实例
    ///
    /// # 返回值
    ///
    /// 返回结构体实例，如果转换失败则使用默认值
    ///
    /// # 使用示例
    ///
    /// ```rust
    /// let node: mf_model::node::Node = /* ... */;
    /// let my_struct: DocumentNode = node.into();
    /// // 或者
    /// let my_struct = DocumentNode::from(node);
    /// ```
    fn from(node: mf_model::node::Node) -> Self {
        DocumentNode::from(&node).unwrap_or_else(|_| {
            // 如果转换失败，使用默认值创建实例
            Self::default_instance()
        })
    }
}
```

## 使用示例

```rust
// ==========================================
// 生成的代码使用示例
// ==========================================

fn example_usage() {
    // 1. 获取节点定义（用于模式定义）
    let node_definition = DocumentNode::node_definition();
    println!("节点类型: {}", node_definition.name);
    println!("支持的标记: {:?}", node_definition.spec.marks);
    println!("内容表达式: {:?}", node_definition.spec.content);
    println!("属性数量: {}", node_definition.spec.attrs.as_ref().map(|a| a.len()).unwrap_or(0));
    
    // 2. 创建实际的节点实例数据
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
    
    // 3. 从 Node 转换为结构体（类型安全转换）
    match DocumentNode::from(&node_instance) {
        Ok(doc_struct) => {
            println!("转换成功:");
            println!("  标题: {}", doc_struct.title);
            println!("  版本: {}", doc_struct.version);
            println!("  已发布: {}", doc_struct.is_published);
            println!("  文档ID: {}", doc_struct.document_id);
        },
        Err(e) => {
            println!("转换失败: {}", e);
        }
    }
    
    // 4. 使用 .into() 方法进行转换（自动降级）
    let doc_struct: DocumentNode = node_instance.into(); // 失败时自动使用 default_instance()
    
    // 5. 反向转换：从结构体到 Node 定义
    let definition: mf_core::node::Node = doc_struct.into();
}
```

## 关键特性总结

### 1. **设计分离**
- `node_definition()`: 只包含 `#[attr]` 字段，用于节点模式定义
- `from()`: 处理所有字段，用于实例创建
- 非 `#[attr]` 字段在实例化时使用类型默认值

### 2. **类型支持**
- **基本类型**: `String`, `i32`, `f64`, `bool`
- **泛型类型**: `Option<T>`, `Vec<T>`, `HashMap<K,V>`
- **自定义类型**: 要求实现 `Default + Serialize` traits
- **JSON 值**: 支持复杂 JSON 默认值

### 3. **自定义表达式**
- 构造函数调用: `CustomStruct::new()`
- 带参数构造: `HashMap::with_capacity(10)`
- 链式调用: `Builder::new().with_defaults().build()`

### 4. **错误处理**
- 类型验证与错误报告
- 优雅降级策略
- 默认值回退机制

### 5. **双向转换**
- `From<DocumentNode>` for `mf_core::node::Node`
- `From<mf_model::node::Node>` for `DocumentNode`
- 支持 `.into()` 语法糖

这个展开示例展示了 ModuForge-RS Node 宏的完整功能，包括所有支持的类型、表达式和转换机制。生成的代码遵循 Rust 的最佳实践，提供类型安全、错误处理和性能优化。