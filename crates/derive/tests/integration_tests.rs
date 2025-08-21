//! ModuForge-RS 派生宏集成测试
//!
//! 这个文件包含了完整的端到端集成测试，用于验证 #[derive(Node)] 和 #[derive(Mark)]
//! 派生宏的实际功能和用户体验。
//!
//! 集成测试遵循以下设计原则：
//! - **用户视角**: 从用户使用角度测试宏功能  
//! - **端到端**: 测试完整的宏处理流程
//! - **真实场景**: 使用真实的使用案例和数据结构

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

}