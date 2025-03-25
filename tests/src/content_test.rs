
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
        top_node: Some("doc".to_string())
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
fn test_content_match_parse_doc() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("DW+".to_string(), &schema.nodes);
    assert!(content.valid_end);
    assert_eq!(content.next.len(), 1);
    assert_eq!(content.next[0].node_type.name, "DW");
}

#[test]
fn test_content_match_parse_dw() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("djgc+".to_string(), &schema.nodes);
    assert!(content.valid_end);
    assert_eq!(content.next.len(), 1);
    assert_eq!(content.next[0].node_type.name, "djgc");
}

#[test]
fn test_content_match_parse_djgc() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("djgcNode+".to_string(), &schema.nodes);
    assert!(content.valid_end);
    assert_eq!(content.next.len(), 1);
    assert_eq!(content.next[0].node_type.name, "djgcNode");
}

#[test]
fn test_content_match_parse_djgc_node() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("qfCode standard".to_string(), &schema.nodes);
    assert!(!content.valid_end);
    assert_eq!(content.next.len(), 1);
    assert_eq!(content.next[0].node_type.name, "qfCode");
}

#[test]
fn test_content_match_parse_repetition() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("DW+".to_string(), &schema.nodes);
    assert!(content.valid_end);
    assert_eq!(content.next.len(), 2); // One for the direct path, one for the loop
}

#[test]
fn test_content_match_parse_sequence() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("qfCode standard".to_string(), &schema.nodes);
    assert!(!content.valid_end);
    assert_eq!(content.next.len(), 1);
    assert_eq!(content.next[0].node_type.name, "qfCode");
}

#[test]
fn test_content_match_compatible() {
    let schema = create_test_schema();
    let match1 = ContentMatch::parse("DW".to_string(), &schema.nodes);
    let match2 = ContentMatch::parse("DW+".to_string(), &schema.nodes);
    assert!(match1.compatible(&match2));
}

#[test]
fn test_content_match_edge_count() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("DW+".to_string(), &schema.nodes);
    assert_eq!(content.edge_count(), 2);
}

#[test]
fn test_content_match_edge() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("DW+".to_string(), &schema.nodes);
    assert!(content.edge(0).is_ok());
    assert!(content.edge(2).is_err());
}

#[test]
fn test_token_stream() {
    let mut stream = TokenStream::new(
        "DW+".to_string(),
        create_test_schema().nodes,
    );
    assert_eq!(stream.next(), Some("DW"));
    stream.eat("DW");
    assert_eq!(stream.next(), Some("+"));
}

#[test]
fn test_content_match_fill() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("djgcNode+".to_string(), &schema.nodes);
    let after = vec![];
    let filled = content.fill(&after, true, &schema);
    assert!(filled.is_some());
    let nodes = filled.unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].r#type, "djgcNode");
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
    println!("Generated nodes: {:?}", nodes);
    
}

#[test]
fn test_content_match_fill_repetition() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("DW+".to_string(), &schema.nodes);
    let after = vec![];
    let filled = content.fill(&after, true, &schema);
    assert!(filled.is_some());
    let nodes = filled.unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].r#type, "DW");
}

#[test]
fn test_content_match_fragment_empty() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("DW+".to_string(), &schema.nodes);
    let fragment = vec![];
    let result = content.match_fragment(&fragment, &schema);
    assert!(result.is_some());
    assert!(result.unwrap().valid_end);
}

#[test]
fn test_content_match_fragment_single() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("DW+".to_string(), &schema.nodes);
    let fragment = vec![Node::new("1", "DW".to_string(), Default::default(), vec![], vec![])];
    let result = content.match_fragment(&fragment, &schema);
    assert!(result.is_some());
    assert!(result.unwrap().valid_end);
}

#[test]
fn test_content_match_fragment_sequence() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("qfCode standard".to_string(), &schema.nodes);
    let fragment = vec![
        Node::new("1", "qfCode".to_string(), Default::default(), vec![], vec![]),
        Node::new("2", "standard".to_string(), Default::default(), vec![], vec![]),
    ];
    let result = content.match_fragment(&fragment, &schema);
    assert!(result.is_some());
    assert!(result.unwrap().valid_end);
}

#[test]
fn test_content_match_fragment_invalid_sequence() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("qfCode standard".to_string(), &schema.nodes);
    let fragment = vec![
        Node::new("1", "standard".to_string(), Default::default(), vec![], vec![]),
        Node::new("2", "qfCode".to_string(), Default::default(), vec![], vec![]),
    ];
    let result = content.match_fragment(&fragment, &schema);
    assert!(result.is_some());
    assert!(!result.unwrap().valid_end);
}

#[test]
fn test_content_match_fragment_repetition() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("DW+".to_string(), &schema.nodes);
    let fragment = vec![
        Node::new("1", "DW".to_string(), Default::default(), vec![], vec![]),
        Node::new("2", "DW".to_string(), Default::default(), vec![], vec![]),
        Node::new("3", "DW".to_string(), Default::default(), vec![], vec![]),
    ];
    let result = content.match_fragment(&fragment, &schema);
    assert!(result.is_some());
    assert!(result.unwrap().valid_end);
}

#[test]
fn test_content_match_fragment_nested() {
    let schema = create_test_schema();
    let content = ContentMatch::parse("DW+".to_string(), &schema.nodes);
    let fragment = vec![
        Node::new("1", "DW".to_string(), Default::default(), vec![], vec![]),
        Node::new("2", "DW".to_string(), Default::default(), vec![], vec![]),
    ];
    let result = content.match_fragment(&fragment, &schema);
    assert!(result.is_some());
    assert!(result.unwrap().valid_end);
} 