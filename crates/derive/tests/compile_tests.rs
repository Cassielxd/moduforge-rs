//! 编译时测试
//!
//! 这个模块包含测试派生宏在编译时的行为，包括：
//! - 正确的宏展开验证
//! - 编译错误场景测试
//! - 宏属性验证测试

use mf_derive::{Node, Mark};

/// 测试正确的宏属性组合
#[derive(Node)]
#[node_type = "test_node"]
#[marks = "test_mark"]
#[content = "test_content"]
struct ValidNodeConfiguration {
    #[attr]
    field1: String,
    
    #[attr]
    field2: Option<i32>,
}

/// 测试最小配置的 Node
#[derive(Node)]
#[node_type = "minimal"]
struct MinimalNode {
    #[attr]
    name: String,
}

/// 测试正确的 Mark 配置
#[derive(Mark)]
#[mark_type = "test_mark"]
struct ValidMarkConfiguration {
    #[attr]
    style: String,
    
    #[attr]
    weight: Option<f64>,
}

/// 测试最小配置的 Mark
#[derive(Mark)]
#[mark_type = "minimal"]
struct MinimalMark {
    #[attr]
    level: i32,
}

/// 测试复杂字段类型支持
#[derive(Node)]
#[node_type = "complex_types"]
struct ComplexTypeTest {
    #[attr]
    string_field: String,
    
    #[attr]
    optional_string: Option<String>,
    
    #[attr]
    int_field: i32,
    
    #[attr]
    optional_int: Option<i32>,
    
    #[attr]
    float_field: f64,
    
    #[attr]
    optional_float: Option<f64>,
    
    #[attr]
    bool_field: bool,
    
    #[attr]
    optional_bool: Option<bool>,
    
    // 非属性字段 - 应该被忽略
    non_attr_field: Vec<String>,
}

/// 测试单字段结构体
#[derive(Node)]
#[node_type = "single_field"]
struct SingleFieldNode {
    #[attr]
    only_field: String,
}

#[derive(Mark)]
#[mark_type = "single_field"]
struct SingleFieldMark {
    #[attr]
    only_field: String,
}

/// 测试空结构体（无字段）
#[derive(Node)]
#[node_type = "empty"]
struct EmptyStructNode;

#[derive(Mark)]
#[mark_type = "empty"]  
struct EmptyStructMark;

/// 测试包含 PhantomData 的结构体（简化版本）
#[derive(Node)]
#[node_type = "with_phantom"]
struct NodeWithPhantom {
    #[attr]
    reference_data: String,
    
    // 非属性字段可以包含 PhantomData
    _phantom: std::marker::PhantomData<String>,
}

/// 测试包含复杂字段的结构体（暂时跳过泛型测试）
#[derive(Node)]
#[node_type = "complex_field"]
struct ComplexFieldNode {
    #[attr]
    data: String,
    
    // 非属性字段可以是复杂类型
    metadata: std::collections::HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证有效配置能够正确编译和运行
    #[test]
    fn test_valid_configurations_compile() {
        let node = ValidNodeConfiguration {
            field1: "test".to_string(),
            field2: Some(42),
        };
        
        let mf_node = node.to_node();
        assert_eq!(mf_node.name, "test_node");

        let mark = ValidMarkConfiguration {
            style: "bold".to_string(),
            weight: Some(1.5),
        };
        
        let mf_mark = mark.to_mark();
        assert_eq!(mf_mark.r#type, "test_mark");
        
        println!("有效配置编译测试通过");
    }

    /// 测试最小配置
    #[test]
    fn test_minimal_configurations() {
        let node = MinimalNode {
            name: "minimal_test".to_string(),
        };
        
        let mf_node = node.to_node();
        assert_eq!(mf_node.name, "minimal");

        let mark = MinimalMark {
            level: 1,
        };
        
        let mf_mark = mark.to_mark();
        assert_eq!(mf_mark.r#type, "minimal");
        
        println!("最小配置编译测试通过");
    }

    /// 测试复杂类型支持
    #[test]
    fn test_complex_type_support() {
        let node = ComplexTypeTest {
            string_field: "test".to_string(),
            optional_string: Some("optional".to_string()),
            int_field: 42,
            optional_int: None,
            float_field: 3.14,
            optional_float: Some(2.71),
            bool_field: true,
            optional_bool: Some(false),
            non_attr_field: vec!["ignored".to_string()],
        };
        
        let mf_node = node.to_node();
        assert_eq!(mf_node.name, "complex_types");
        
        println!("复杂类型支持测试通过");
    }

    /// 测试单字段结构体
    #[test]
    fn test_single_field_structs() {
        let node = SingleFieldNode {
            only_field: "single".to_string(),
        };
        
        let mf_node = node.to_node();
        assert_eq!(mf_node.name, "single_field");

        let mark = SingleFieldMark {
            only_field: "single".to_string(),
        };
        
        let mf_mark = mark.to_mark();
        assert_eq!(mf_mark.r#type, "single_field");
        
        println!("单字段结构体测试通过");
    }

    /// 测试空结构体
    #[test]
    fn test_empty_structs() {
        let node = EmptyStructNode;
        let mf_node = node.to_node();
        assert_eq!(mf_node.name, "empty");

        let mark = EmptyStructMark;
        let mf_mark = mark.to_mark();
        assert_eq!(mf_mark.r#type, "empty");
        
        println!("空结构体测试通过");
    }

    /// 测试带 PhantomData 的结构体
    #[test]
    fn test_phantom_data_structs() {
        let node = NodeWithPhantom {
            reference_data: "phantom_test".to_string(),
            _phantom: std::marker::PhantomData,
        };
        
        let mf_node = node.to_node();
        assert_eq!(mf_node.name, "with_phantom");
        
        println!("PhantomData 结构体测试通过");
    }

    /// 测试复杂字段结构体
    #[test]
    fn test_complex_field_structs() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("author".to_string(), "test".to_string());
        
        let node = ComplexFieldNode {
            data: "complex_test".to_string(),
            metadata,
        };
        
        let mf_node = node.to_node();
        assert_eq!(mf_node.name, "complex_field");
        
        println!("复杂字段结构体测试通过");
    }

    /// 测试宏生成方法的存在性
    #[test]
    fn test_generated_methods_exist() {
        // 验证生成的方法确实存在且可调用
        let node = MinimalNode {
            name: "method_test".to_string(),
        };
        
        // 这里调用 to_node() 方法，如果不存在会编译失败
        let _result = node.to_node();
        
        let mark = MinimalMark {
            level: 1,
        };
        
        // 这里调用 to_mark() 方法，如果不存在会编译失败
        let _result = mark.to_mark();
        
        println!("生成方法存在性测试通过");
    }

    /// 测试方法返回类型正确性
    #[test]
    fn test_return_types() {
        let node = MinimalNode {
            name: "return_type_test".to_string(),
        };
        
        let result = node.to_node();
        
        // 验证返回类型是 mf_core::node::Node
        let _type_check: mf_core::node::Node = result;

        let mark = MinimalMark {
            level: 1,
        };
        
        let result = mark.to_mark();
        
        // 验证返回类型是 mf_core::mark::Mark
        let _type_check: mf_model::mark::Mark = result;
        
        println!("返回类型正确性测试通过");
    }

    /// 测试宏在不同模块中的可见性
    #[test]
    fn test_macro_visibility() {
        mod inner {
            use mf_derive::{Node, Mark};

            #[derive(Node)]
            #[node_type = "inner"]
            pub struct InnerNode {
                #[attr]
                pub data: String,
            }

            #[derive(Mark)]
            #[mark_type = "inner"]
            pub struct InnerMark {
                #[attr]
                pub style: String,
            }
        }

        let node = inner::InnerNode {
            data: "inner_test".to_string(),
        };
        
        let mf_node = node.to_node();
        assert_eq!(mf_node.name, "inner");

        let mark = inner::InnerMark {
            style: "inner_style".to_string(),
        };
        
        let mf_mark = mark.to_mark();
        assert_eq!(mf_mark.r#type, "inner");
        
        println!("宏可见性测试通过");
    }
}