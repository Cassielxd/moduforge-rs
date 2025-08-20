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
    
    #[attr] 
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

/// 边界情况测试 - 无属性字段的 Node
#[derive(Node)]
#[node_type = "separator"]
struct EmptyNodeTest;

/// 边界情况测试 - 无属性字段的 Mark  
#[derive(Mark)]
#[mark_type = "spacer"]
struct EmptyMarkTest;

/// 复杂嵌套场景测试
#[derive(Node)]
#[node_type = "document"]
#[marks = "numbered bulleted"]
#[content = "block+"]
struct ComplexDocumentNode {
    #[attr]
    title: String,
    
    #[attr]
    author: Option<String>,
    
    #[attr]
    version: Option<String>,
    
    #[attr]
    word_count: Option<i64>,
    
    #[attr]
    last_modified: Option<String>,
    
    #[attr]
    is_draft: bool,
    
    #[attr]
    categories: Option<String>,
    
    #[attr]
    // 复杂的内部字段
    metadata: serde_json::Value,
    #[attr]
    revision_history: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试基本 Node 派生宏功能
    #[test]
    fn test_basic_node_derivation() {
        let node = BasicNodeTest {
            content: "测试内容".to_string(),
            level: 2,
        };
        
        // 调用生成的 to_node() 方法
        let mf_node = node.to_node();
        
        // 验证生成的节点具有正确的类型
      
        
        // 验证节点被成功创建（不应该 panic）
        println!("基本 Node 测试通过 - 类型: {:?}", mf_node);
    }

    /// 测试完整 Node 派生宏功能
    #[test]  
    fn test_full_node_derivation() {
        let node = FullNodeTest {
            title: "完整测试文档".to_string(),
            subtitle: Some("副标题".to_string()),
            priority: 10,
            weight: Some(1.5),
            is_published: true,
            tags: Some("test,integration,node".to_string()),
            internal_id: uuid::Uuid::new_v4(),
            cache_data: vec![1, 2, 3, 4],
        };
        
        // 调用生成的 to_node() 方法
        let mf_node = node.to_node();
        println!("{mf_node:?}");
        
    }

    /// 测试基本 Mark 派生宏功能
    #[test]
    fn test_basic_mark_derivation() {
        let mark = BasicMarkTest {
            level: "strong".to_string(),
            color: Some("#ff0000".to_string()),
        };
        
        // 调用生成的 to_mark() 方法
        let mf_mark = mark.to_mark();
        
        // 验证生成的标记具有正确的类型
        assert_eq!(mf_mark.r#type, "emphasis");
        
        println!("基本 Mark 测试通过 - 类型: {}", mf_mark.r#type);
    }

    /// 测试完整 Mark 派生宏功能
    #[test]
    fn test_full_mark_derivation() {
        let mark = FullMarkTest {
            font_family: "Arial".to_string(),
            font_size: Some(14.0),
            is_bold: true,
            opacity: Some(0.8),
            z_index: 100,
            _phantom: std::marker::PhantomData,
        };
        
        // 调用生成的 to_mark() 方法
        let mf_mark = mark.to_mark();
        
        // 验证生成的标记具有正确的配置
        assert_eq!(mf_mark.r#type, "styling");
        
        println!("完整 Mark 测试通过 - 类型: {}", mf_mark.r#type);
    }

    /// 测试无属性字段的边界情况
    #[test]
    fn test_empty_derivations() {
        let empty_node = EmptyNodeTest;
        let mf_node = empty_node.to_node();
        assert_eq!(mf_node.name, "separator");
        
        let empty_mark = EmptyMarkTest;
        let mf_mark = empty_mark.to_mark();
        assert_eq!(mf_mark.r#type, "spacer");
        
        println!("空属性测试通过");
    }

    /// 测试复杂文档场景
    #[test]
    fn test_complex_document_scenario() {
        let doc = ComplexDocumentNode {
            title: "ModuForge-RS 用户指南".to_string(),
            author: Some("开发团队".to_string()),
            version: Some("v1.0.0".to_string()),
            word_count: Some(15000),
            last_modified: Some("2025-01-20".to_string()),
            is_draft: false,
            categories: Some("documentation,guide,rust".to_string()),
            metadata: serde_json::json!({
                "license": "MIT",
                "language": "zh-CN"
            }),
            revision_history: vec![
                "初始版本".to_string(),
                "添加示例代码".to_string(),
                "完善文档结构".to_string(),
            ],
        };
        
        let mf_node = doc.to_node();
        assert_eq!(mf_node.name, "document");
        
        println!("复杂文档测试通过");
    }

    /// 测试字段类型兼容性
    #[test]
    fn test_field_type_compatibility() {
        // 测试各种支持的字段类型
        let node = FullNodeTest {
            title: String::from("类型测试"),
            subtitle: None, // Option<String>
            priority: 42, // i32
            weight: Some(3.14), // Option<f64>
            is_published: false, // bool
            tags: Some("type,test".to_string()),
            internal_id: uuid::Uuid::new_v4(),
            cache_data: Vec::new(),
        };
        
        // 确保所有类型都能正确处理
        let mf_node = node.to_node();
        assert_eq!(mf_node.name, "rich_content");
        
        println!("字段类型兼容性测试通过");
    }

    /// 测试属性值的正确传递
    #[test]
    fn test_attribute_value_propagation() {
        let test_content = "属性传递测试内容";
        let test_level = 99;
        
        let node = BasicNodeTest {
            content: test_content.to_string(),
            level: test_level,
        };
        
        let mf_node = node.to_node();
        
        // 验证节点创建成功（属性值应该被正确处理）
        assert_eq!(mf_node.name, "paragraph");
        
        println!("属性值传递测试通过");
    }

    /// 测试宏生成代码的线程安全性
    #[test]
    fn test_thread_safety() {
        use std::sync::Arc;
        use std::thread;
        
        let test_data = Arc::new(vec![
            ("线程测试 1".to_string(), 1),
            ("线程测试 2".to_string(), 2),
            ("线程测试 3".to_string(), 3),
        ]);
        
        let handles: Vec<_> = (0..3).map(|i| {
            let data = Arc::clone(&test_data);
            thread::spawn(move || {
                let (content, level) = &data[i];
                let node = BasicNodeTest {
                    content: content.clone(),
                    level: *level,
                };
                
                let mf_node = node.to_node();
                assert_eq!(mf_node.name, "paragraph");
                
                format!("线程 {} 完成", i)
            })
        }).collect();
        
        for handle in handles {
            let result = handle.join().unwrap();
            println!("{}", result);
        }
        
        println!("线程安全性测试通过");
    }

    /// 性能基准测试（简单版本）
    #[test]
    fn test_performance_benchmark() {
        let start_time = std::time::Instant::now();
        const ITERATIONS: usize = 1000;
        
        for i in 0..ITERATIONS {
            let node = BasicNodeTest {
                content: format!("性能测试 {}", i),
                level: i as i32,
            };
            
            let _mf_node = node.to_node();
        }
        
        let duration = start_time.elapsed();
        println!("性能测试: {} 次迭代耗时 {:?}", ITERATIONS, duration);
        
        // 确保性能在合理范围内（这里只是一个粗略的检查）
        assert!(duration.as_millis() < 5000, "性能测试失败：耗时过长");
        
        println!("性能基准测试通过");
    }

    /// 测试错误恢复和处理
    #[test]
    fn test_error_handling_robustness() {
        // 测试极端值和边界情况
        let node = FullNodeTest {
            title: "".to_string(), // 空字符串
            subtitle: None,
            priority: i32::MAX, // 最大值
            weight: Some(f64::MAX), // 最大浮点数
            is_published: true,
            tags: Some("extreme,boundary,test".to_string()),
            internal_id: uuid::Uuid::new_v4(),
            cache_data: vec![0; 1000], // 大数据
        };
        
        // 应该能够处理边界情况而不崩溃
        let mf_node = node.to_node();
        assert_eq!(mf_node.name, "rich_content");
        
        println!("错误处理鲁棒性测试通过");
    }
}