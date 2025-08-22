//! ModuForge-RS 派生宏集成测试
//!
//! 这个文件包含了完整的端到端集成测试，用于验证 #[derive(Node)] 和 #[derive(Mark)]
//! 派生宏的实际功能和用户体验。演示了所有支持的功能，包括自定义类型表达式。
//!
//! 集成测试遵循以下设计原则：
//! - **用户视角**: 从用户使用角度测试宏功能  
//! - **端到端**: 测试完整的宏处理流程
//! - **真实场景**: 使用真实的使用案例和数据结构
//! - **功能演示**: 展示泛型类型、自定义类型、默认值等高级功能
//!
//! # 支持的功能演示
//!
//! ## 基本功能
//! - `#[node_type = "type"]` - 节点类型定义
//! - `#[marks = "mark1 mark2"]` - 支持的标记
//! - `#[content = "expression"]` - 内容表达式
//! - `#[attr]` - 基本属性标记
//! - `#[attr(default="value")]` - 带默认值的属性
//!
//! ## 高级功能
//! - 泛型类型支持：`Option<String>`, `Vec<u8>`, `HashMap<String, i32>`
//! - 自定义类型表达式：`#[attr(default="CustomStruct::new()")]`
//! - 双向转换：`node_definition()` 和 `from()` 方法
//! - 错误处理：类型不匹配时的 Result 返回
//! - 降级策略：转换失败时使用 `default_instance()`
//!
//! ## 设计分离
//! - `node_definition()`: 只包含 `#[attr]` 字段，用于节点模式定义
//! - `from()`: 处理所有字段，用于实例创建
//! - 非 `#[attr]` 字段在实例化时使用类型默认值

use mf_derive::{Node, Mark};

/// 基本的 Node 派生宏测试结构体
/// 
/// 这个结构体用于测试 Node 派生宏的基本功能，包括：
/// - 基本的 node_type 属性设置
/// - 简单的属性字段处理
#[derive(Node)]
#[node_type = "paragraph"]
struct BasicNodeTest {
    #[attr]
    content: String,
    
    #[attr(default=1)]
    level: i32,
}

/// 完整功能的 Node 测试结构体
///
/// 测试所有 Node 派生宏支持的功能：
/// - node_type, marks, content 属性
/// - 多种类型的属性字段
/// - 可选类型字段
/// - 非属性字段（应被忽略）
#[derive(Node)]
#[node_type = "rich_content"]
#[marks = "bold italic underline"]
#[content = "text*"]
#[desc = "这是一个测试"]
struct FullNodeTest {
    #[attr]
    title: String,
    
    #[attr]
    subtitle: Option<String>,
    
    #[attr]
    priority: i32,
    
    #[attr]
    weight: Option<f64>,
    
    #[attr]
    is_published: bool,
    
    #[attr]
    tags: Option<String>,
    
    // 非属性字段，应该被忽略
    #[attr]
    internal_id: uuid::Uuid,
    #[attr]
    cache_data: Vec<u8>,
}

/// 基本的 Mark 派生宏测试结构体
///
/// 测试 Mark 派生宏的基本功能：
/// - mark_type 属性设置
/// - 简单的属性字段处理
#[derive(Mark)]
#[mark_type = "emphasis"]
struct BasicMarkTest {
    #[attr]
    level: String,
    
    #[attr]
    color: Option<String>,
}

/// 完整功能的 Mark 测试结构体
///
/// 测试所有 Mark 派生宏支持的功能：
/// - mark_type 属性
/// - 多种类型的属性字段
/// - 可选和非可选字段混合
#[derive(Mark)]
#[mark_type = "styling"]
struct FullMarkTest {
    #[attr]
    font_family: String,
    
    #[attr]
    font_size: Option<f32>,
    
    #[attr]
    is_bold: bool,
    
    #[attr]
    opacity: Option<f64>,
    
    #[attr]
    z_index: i32,
    
    // 非属性字段
    _phantom: std::marker::PhantomData<()>,
}

/// ID 字段测试结构体
///
/// 测试 #[id] 属性的功能：
/// - ID 字段映射到 Node 的 id 属性
/// - 与普通 attr 字段的组合使用
/// - 不同类型的 ID 字段支持
#[derive(Node, Debug)]
#[node_type = "document"]
#[marks = "important"]
struct NodeWithIdTest {
    #[id]
    document_id: String,
    
    #[attr]
    title: String,
    
    #[attr]
    content: Option<String>,
    
    #[attr]
    version: i32,
    
    // 普通字段（无标记）
    internal_data: Vec<u8>,
}

/// 可选 ID 字段测试结构体
///
/// 测试 Option<String> 类型的 ID 字段
#[derive(Node, Debug)]
#[node_type = "section"]
#[desc = "测试"]
struct NodeWithOptionalIdTest {
    #[id]
    section_id: Option<String>,
    
    #[attr]
    name: String,
    
    #[attr]
    order: i32,
}

/// 最小化 ID 测试结构体
///
/// 只有 ID 字段，没有其他属性字段
#[derive(Node, Debug)]
#[node_type = "minimal"]
struct MinimalNodeWithIdTest {
    #[id]
    node_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;


    /// 测试完整 Node 派生宏功能
    #[test]  
    fn test_full_node_derivation() {
        // 调用生成的 node_definition() 静态方法，返回使用默认值的 Node 定义
        let mf_node = FullNodeTest::node_definition();
        println!("{mf_node:?}");
        
        // 验证节点类型
        assert_eq!(mf_node.name, "rich_content");
        
    }


    /// 测试反向 From trait 的实现（从 mf_model::node::Node 转换为结构体）
    #[test]
    fn test_reverse_from_trait_implementation() {
        // 首先创建一个 Node 实例
        let mut attrs_map = imbl::HashMap::new();
        attrs_map.insert("title".to_string(), serde_json::json!("测试标题"));
        attrs_map.insert("priority".to_string(), serde_json::json!(1));
        attrs_map.insert("is_published".to_string(), serde_json::json!(true));
        
        let node = mf_model::node::Node {
            id: "test_node".into(),
            r#type: "rich_content".to_string(),
            attrs: mf_model::attrs::Attrs {
                attrs: attrs_map,
            },
            content: imbl::Vector::new(),
            marks: imbl::Vector::new(),
        };

        // 使用 .into() 方法转换
        let struct_via_into: FullNodeTest = node.clone().into();
        assert_eq!(struct_via_into.title, "测试标题");
        assert_eq!(struct_via_into.priority, 1);
        assert_eq!(struct_via_into.is_published, true);

        // 使用 From::from() 方法转换
        let struct_via_from = FullNodeTest::from(&node);
        let struct_via_from = struct_via_from.unwrap();
        assert_eq!(struct_via_from.title, "测试标题");
        assert_eq!(struct_via_from.priority, 1);
        assert_eq!(struct_via_from.is_published, true);

        println!("反向 From trait 实现测试通过");
    }

    /// 测试包含所有字段的反向 From trait 实现
    #[test]
    fn test_reverse_from_with_all_fields() {
        // 创建一个 Node 实例，包含 attr 字段的数据
        let mut attrs_map = imbl::HashMap::new();
        attrs_map.insert("title".to_string(), serde_json::json!("测试标题"));
        attrs_map.insert("priority".to_string(), serde_json::json!(5));
        attrs_map.insert("is_published".to_string(), serde_json::json!(false));
        // 注意：internal_id 和 cache_data 也有 #[attr] 标记，但通常不会从外部设置
        
        let node = mf_model::node::Node {
            id: "test_all_fields".into(),
            r#type: "rich_content".to_string(),
            attrs: mf_model::attrs::Attrs {
                attrs: attrs_map,
            },
            content: imbl::Vector::new(),
            marks: imbl::Vector::new(),
        };

        // 使用 .into() 方法转换
        let struct_instance: FullNodeTest = node.into();
        
        // 验证从 attrs 中提取的字段
        assert_eq!(struct_instance.title, "测试标题");
        assert_eq!(struct_instance.priority, 5);
        assert_eq!(struct_instance.is_published, false);
        
        // 验证没有在 attrs 中设置的字段使用了默认值
        assert_eq!(struct_instance.subtitle, None);
        assert_eq!(struct_instance.weight, None);
        assert_eq!(struct_instance.tags, None);
        
        // 验证自定义类型字段
        // internal_id 应该是一个新生成的 UUID（因为 attrs 中没有设置）
        assert_ne!(struct_instance.internal_id.to_string(), "");
        
        // cache_data 应该是空 Vec（因为 attrs 中没有设置）
        assert!(struct_instance.cache_data.is_empty());

        println!("包含所有字段的反向 From trait 实现测试通过");
    }

    /// 测试自定义类型字段的处理
    #[test] 
    fn test_custom_type_field_handling() {
        // 测试包含 Uuid 和 Vec<u8> 自定义类型的结构体
        let mut attrs_map = imbl::HashMap::new();
        attrs_map.insert("title".to_string(), serde_json::json!("测试"));
        
        let node = mf_model::node::Node {
            id: "test_custom".into(),
            r#type: "rich_content".to_string(),
            attrs: mf_model::attrs::Attrs {
                attrs: attrs_map,
            },
            content: imbl::Vector::new(),
            marks: imbl::Vector::new(),
        };

        // 转换应该成功
        let struct_instance: FullNodeTest = node.into();
        
        // 验证基本字段
        assert_eq!(struct_instance.title, "测试");
        
        // 验证自定义类型字段使用了默认值
        // internal_id 应该是一个新生成的 UUID
        assert_ne!(struct_instance.internal_id.to_string(), "");
        
        // cache_data 应该是空 Vec
        assert!(struct_instance.cache_data.is_empty());

        println!("自定义类型字段处理测试通过");
    }

    /// 测试带有 ID 字段的 Node 的 to_node() 方法
    #[test]
    fn test_node_with_id_to_node_conversion() {
        // 创建一个带有 ID 的结构体实例
        let node_instance = NodeWithIdTest {
            document_id: "doc_123".to_string(),
            title: "测试文档".to_string(),
            content: Some("这是内容".to_string()),
            version: 1,
            internal_data: vec![1, 2, 3],
        };
        
        // 转换为 Node
        let node = node_instance.to_node();
        
        // 验证 ID 字段被正确映射
        assert_eq!(node.id.as_ref(), "doc_123");
        
        // 验证节点类型
        assert_eq!(node.r#type, "document");
        
        // 验证属性字段被正确映射
        assert_eq!(
            node.attrs.attrs.get("title").and_then(|v| v.as_str()),
            Some("测试文档")
        );
        assert_eq!(
            node.attrs.attrs.get("content").and_then(|v| v.as_str()),
            Some("这是内容")
        );
        assert_eq!(
            node.attrs.attrs.get("version").and_then(|v| v.as_i64()),
            Some(1)
        );
        
        // 验证非 attr 字段不会出现在节点属性中
        assert!(node.attrs.attrs.get("internal_data").is_none());
        
        println!("带有 ID 字段的 to_node() 转换测试通过");
    }
    
    /// 测试从 Node 转换为带有 ID 字段的结构体
    #[test]
    fn test_node_with_id_from_node_conversion() {
        // 创建一个 Node 实例，设置 ID 和属性
        let mut attrs_map = imbl::HashMap::new();
        attrs_map.insert("title".to_string(), serde_json::json!("测试文档"));
        attrs_map.insert("content".to_string(), serde_json::json!("这是内容"));
        attrs_map.insert("version".to_string(), serde_json::json!(2));
        
        let node = mf_model::node::Node {
            id: "doc_456".into(),
            r#type: "document".to_string(),
            attrs: mf_model::attrs::Attrs {
                attrs: attrs_map,
            },
            content: imbl::Vector::new(),
            marks: imbl::Vector::new(),
        };
        
        // 使用 from() 方法转换
        let struct_result = NodeWithIdTest::from(&node);
        assert!(struct_result.is_ok());
        
        let struct_instance = struct_result.unwrap();
        
        // 验证 ID 字段被正确提取
        assert_eq!(struct_instance.document_id, "doc_456");
        
        // 验证属性字段被正确提取
        assert_eq!(struct_instance.title, "测试文档");
        assert_eq!(struct_instance.content, Some("这是内容".to_string()));
        assert_eq!(struct_instance.version, 2);
        
        // 验证非 attr 字段使用了默认值
        assert!(struct_instance.internal_data.is_empty());
        
        println!("从 Node 转换为带有 ID 字段的结构体测试通过");
    }
    
    /// 测试通过 From trait 的双向转换
    #[test]
    fn test_node_with_id_bidirectional_conversion() {
        // 创建原始结构体实例
        let original = NodeWithIdTest {
            document_id: "bidirectional_test".to_string(),
            title: "双向转换测试".to_string(),
            content: None,
            version: 5,
            internal_data: vec![4, 5, 6],
        };
        
        // 转换为 Node
        let node: mf_model::node::Node = original.into();
        
        // 验证 Node 的内容
        assert_eq!(node.id.as_ref(), "bidirectional_test");
        assert_eq!(node.r#type, "document");
        
        // 从 Node 转换回结构体
        let recovered: NodeWithIdTest = node.into();
        
        // 验证恢复的结构体
        assert_eq!(recovered.document_id, "bidirectional_test");
        assert_eq!(recovered.title, "双向转换测试");
        assert_eq!(recovered.content, None);
        assert_eq!(recovered.version, 5);
        // 注意：internal_data 不是 attr 字段，所以会被重置为默认值
        assert!(recovered.internal_data.is_empty());
        
        println!("双向转换测试通过");
    }
    
    /// 测试可选 ID 字段的处理
    #[test]
    fn test_optional_id_field() {
        // 测试有值的可选 ID
        let node_with_id = NodeWithOptionalIdTest {
            section_id: Some("section_123".to_string()),
            name: "测试章节".to_string(),
            order: 1,
        };
        
        let node = node_with_id.to_node();
        assert_eq!(node.id.as_ref(), "section_123");
        assert_eq!(node.r#type, "section");
        
        // 测试没有值的可选 ID
        let node_without_id = NodeWithOptionalIdTest {
            section_id: None,
            name: "无 ID 章节".to_string(),
            order: 2,
        };
        
        let node2 = node_without_id.to_node();
        assert_eq!(node2.id.as_ref(), "default_id"); // 应该使用默认 ID
        assert_eq!(node2.r#type, "section");
        
        // 测试反向转换
        let mut attrs_map = imbl::HashMap::new();
        attrs_map.insert("name".to_string(), serde_json::json!("恢复章节"));
        attrs_map.insert("order".to_string(), serde_json::json!(3));
        
        let node3 = mf_model::node::Node {
            id: "recovered_section".into(),
            r#type: "section".to_string(),
            attrs: mf_model::attrs::Attrs {
                attrs: attrs_map,
            },
            content: imbl::Vector::new(),
            marks: imbl::Vector::new(),
        };
        
        let recovered = NodeWithOptionalIdTest::from(&node3).unwrap();
        assert_eq!(recovered.section_id, Some("recovered_section".to_string()));
        assert_eq!(recovered.name, "恢复章节");
        assert_eq!(recovered.order, 3);
        
        println!("可选 ID 字段测试通过");
    }
    
    /// 测试最小化结构体（只有 ID 字段）
    #[test]
    fn test_minimal_node_with_id() {
        let minimal = MinimalNodeWithIdTest {
            node_id: "minimal_123".to_string(),
        };
        
        // 转换为 Node
        let node = minimal.to_node();
        assert_eq!(node.id.as_ref(), "minimal_123");
        assert_eq!(node.r#type, "minimal");
        
        // 验证没有额外的属性
        assert!(node.attrs.attrs.is_empty());
        
        // 测试反向转换
        let node2 = mf_model::node::Node {
            id: "minimal_456".into(),
            r#type: "minimal".to_string(),
            attrs: mf_model::attrs::Attrs::default(),
            content: imbl::Vector::new(),
            marks: imbl::Vector::new(),
        };
        
        let recovered = MinimalNodeWithIdTest::from(&node2).unwrap();
        assert_eq!(recovered.node_id, "minimal_456");
        
        println!("最小化结构体测试通过");
    }
    
    /// 测试类型不匹配时的错误处理
    #[test]
    fn test_id_field_type_mismatch_error() {
        // 创建一个类型不匹配的 Node
        let node = mf_model::node::Node {
            id: "test_id".into(),
            r#type: "wrong_type".to_string(), // 错误的类型
            attrs: mf_model::attrs::Attrs::default(),
            content: imbl::Vector::new(),
            marks: imbl::Vector::new(),
        };
        
        // 尝试转换，应该返回错误
        let result = NodeWithIdTest::from(&node);
        assert!(result.is_err());
        
        let error_message = result.unwrap_err();
        assert!(error_message.contains("节点类型不匹配"));
        assert!(error_message.contains("期望 'document'"));
        assert!(error_message.contains("实际 'wrong_type'"));
        
        println!("类型不匹配错误处理测试通过");
    }
    
    /// 测试 From trait 转换失败时的降级处理
    #[test]
    fn test_id_field_fallback_to_default_instance() {
        // 创建一个类型不匹配的 Node
        let node = mf_model::node::Node {
            id: "fallback_test".into(),
            r#type: "invalid_type".to_string(),
            attrs: mf_model::attrs::Attrs::default(),
            content: imbl::Vector::new(),
            marks: imbl::Vector::new(),
        };
        
        // 使用 Into trait，这应该会降级到 default_instance()
        let struct_instance: NodeWithIdTest = node.into();
        
        // 验证使用了默认值（由 default_instance 方法生成）
        // 注意：这里的具体值取决于 default_instance 的实现
        assert!(!struct_instance.document_id.is_empty()); // ID 字段应该有某个有意义的默认的 ID
        assert_eq!(struct_instance.document_id, "default_node_id"); // 具体的默认 ID 值
        assert_eq!(struct_instance.title, ""); // 普通 String 字段使用空字符串默认值
        assert_eq!(struct_instance.content, None); // Option 字段使用 None 默认值
        assert_eq!(struct_instance.version, 0); // i32 的默认值
        assert!(struct_instance.internal_data.is_empty()); // Vec 的默认值
        
        println!("降级处理测试通过");
    }

}