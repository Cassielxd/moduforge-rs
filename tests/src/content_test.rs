use std::collections::HashMap;

use moduforge_core::model::content::{ContentMatch, TokenStream};
use moduforge_core::model::id_generator::IdGenerator;
use moduforge_core::model::node::Node;
use moduforge_core::model::node_type::NodeSpec;
use moduforge_core::model::schema::{Schema, SchemaSpec};
#[warn(dead_code)]
fn create_test_schema() -> Schema {
    let mut nodes = HashMap::new();
    nodes.insert(
        "doc".to_string(),
        NodeSpec {
            content: Some("DW+".to_string()),
            marks: None,
            group: None,
            desc: Some("工程项目".to_string()),
            attrs: None,
        },
    );

    nodes.insert(
        "DW".to_string(),
        NodeSpec {
            content: Some("djgc+".to_string()),
            marks: None,
            group: None,
            desc: Some("单项工程".to_string()),
            attrs: None,
        },
    );
    nodes.insert(
        "djgc".to_string(),
        NodeSpec {
            content: Some("djgcNode+".to_string()),
            marks: None,
            group: None,
            desc: Some("单价构成".to_string()),
            attrs: None,
        },
    );
    nodes.insert(
        "djgcNode".to_string(),
        NodeSpec {
            content: Some("qfCode standard code".to_string()),
            marks: None,
            group: None,
            desc: Some("单价构行节点".to_string()),
            attrs: None,
        },
    );

    nodes.insert(
        "qfCode".to_string(),
        NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("取费代码".to_string()),
            attrs: None,
        },
    );
    nodes.insert(
        "standard".to_string(),
        NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("标准".to_string()),
            attrs: None,
        },
    );
    nodes.insert(
        "code".to_string(),
        NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("代码".to_string()),
            attrs: None,
        },
    );
    let instance_spec = SchemaSpec {
        nodes,
        marks: HashMap::new(),
        top_node: Some("doc".to_string()),
    };
    Schema::compile(instance_spec).unwrap()
}

#[test]
fn test_content_match_empty() {
    let empty = ContentMatch::empty();
    assert!(empty.valid_end);
    assert!(empty.next.is_empty());
    assert!(empty.wrap_cache.is_empty());
}

#[test]
fn test_content_match_fill_sequence() {
    let schema = create_test_schema();
    let id = IdGenerator::get_id();
    let nodes = schema.top_node_type.clone().unwrap().create_and_fill(
        Some(id.clone()),
        None,
        vec![],
        None,
        &schema,
    );
    dbg!(nodes);
}

#[test]
fn test_content_match_group() {
    let mut nodes = HashMap::new();

    // 创建一个测试用的schema
    nodes.insert(
        "doc".to_string(),
        NodeSpec {
            content: Some("(A B)+".to_string()), // 使用分组来匹配一个或多个 A B 序列
            marks: None,
            group: None,
            desc: Some("测试文档".to_string()),
            attrs: None,
        },
    );

    nodes.insert(
        "A".to_string(),
        NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("A节点".to_string()),
            attrs: None,
        },
    );

    nodes.insert(
        "B".to_string(),
        NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("B节点".to_string()),
            attrs: None,
        },
    );

    let instance_spec = SchemaSpec {
        nodes,
        marks: HashMap::new(),
        top_node: Some("doc".to_string()),
    };
    let schema = Schema::compile(instance_spec).unwrap();

    // 测试有效的分组序列
    let id = IdGenerator::get_id();
    let nodes = schema.top_node_type.clone().unwrap().create_and_fill(
        Some(id.clone()),
        None,
        vec![],
        None,
        &schema,
    );

    dbg!(nodes);

    // 测试分组中的选择表达式
    let mut nodes = HashMap::new();
    nodes.insert(
        "doc2".to_string(),
        NodeSpec {
            content: Some("(A | B)+".to_string()), // 使用分组和选择表达式
            marks: None,
            group: None,
            desc: Some("测试文档2".to_string()),
            attrs: None,
        },
    );

    nodes.insert(
        "A".to_string(),
        NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("A节点".to_string()),
            attrs: None,
        },
    );

    nodes.insert(
        "B".to_string(),
        NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("B节点".to_string()),
            attrs: None,
        },
    );

    let instance_spec = SchemaSpec {
        nodes,
        marks: HashMap::new(),
        top_node: Some("doc2".to_string()),
    };
    let schema = Schema::compile(instance_spec).unwrap();

    let id = IdGenerator::get_id();
    let nodes = schema.top_node_type.clone().unwrap().create_and_fill(
        Some(id.clone()),
        None,
        vec![],
        None,
        &schema,
    );
    dbg!(nodes);

    // 测试嵌套分组
    let mut nodes = HashMap::new();
    nodes.insert(
        "doc3".to_string(),
        NodeSpec {
            content: Some("((A B) | (B A))+".to_string()), // 使用嵌套分组
            marks: None,
            group: None,
            desc: Some("测试文档3".to_string()),
            attrs: None,
        },
    );

    nodes.insert(
        "A".to_string(),
        NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("A节点".to_string()),
            attrs: None,
        },
    );

    nodes.insert(
        "B".to_string(),
        NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("B节点".to_string()),
            attrs: None,
        },
    );

    let instance_spec = SchemaSpec {
        nodes,
        marks: HashMap::new(),
        top_node: Some("doc3".to_string()),
    };
    let schema = Schema::compile(instance_spec).unwrap();

    let id = IdGenerator::get_id();
    let nodes = schema.top_node_type.clone().unwrap().create_and_fill(
        Some(id.clone()),
        None,
        vec![],
        None,
        &schema,
    );
    dbg!(nodes);
}
