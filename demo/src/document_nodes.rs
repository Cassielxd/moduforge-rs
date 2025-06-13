use moduforge_core::node::Node;
use moduforge_model::node_type::NodeSpec;
use moduforge_model::schema::AttributeSpec;
use std::collections::HashMap;
use serde_json::Value;

/// 创建文档根节点
/// 作为整个文档的容器，可以包含其他所有类型的节点
pub fn create_document_node() -> Node {
    let mut attrs = HashMap::new();
    attrs.insert(
        "title".to_string(),
        AttributeSpec { default: Some(Value::String("新文档".to_string())) },
    );
    attrs.insert(
        "description".to_string(),
        AttributeSpec { default: Some(Value::String("".to_string())) },
    );
    attrs.insert(
        "created_at".to_string(),
        AttributeSpec {
            default: Some(Value::String(chrono::Utc::now().to_rfc3339())),
        },
    );
    attrs.insert(
        "author".to_string(),
        AttributeSpec { default: Some(Value::String("".to_string())) },
    );

    let spec = NodeSpec {
        content: Some("block".to_string()),
        marks: None,
        attrs: Some(attrs),
        desc: Some("文档根节点，包含所有文档内容".to_string()),
        ..Default::default()
    };

    let mut node = Node::create("document", spec);
    node.set_top_node(); // 设置为顶级节点
    node
}

/// 创建段落节点
/// 用于表示文档中的段落内容
pub fn create_paragraph_node() -> Node {
    let mut attrs = HashMap::new();
    attrs.insert(
        "align".to_string(),
        AttributeSpec { default: Some(Value::String("left".to_string())) },
    );
    attrs.insert(
        "indent".to_string(),
        AttributeSpec {
            default: Some(Value::Number(serde_json::Number::from(0))),
        },
    );
    attrs.insert(
        "line_height".to_string(),
        AttributeSpec {
            default: Some(Value::Number(
                serde_json::Number::from_f64(1.5).unwrap(),
            )),
        },
    );

    let spec = NodeSpec {
        content: Some("inline".to_string()),
        marks: None,
        attrs: Some(attrs),
        desc: Some("段落节点，包含文本内容".to_string()),
        ..Default::default()
    };

    Node::create("paragraph", spec)
}

/// 创建标题节点
/// 用于表示不同级别的标题
pub fn create_heading_node() -> Node {
    let mut attrs = HashMap::new();
    attrs.insert(
        "level".to_string(),
        AttributeSpec {
            default: Some(Value::Number(serde_json::Number::from(1))),
        },
    );
    attrs.insert(
        "id".to_string(),
        AttributeSpec { default: Some(Value::String("".to_string())) },
    );
    attrs.insert(
        "anchor".to_string(),
        AttributeSpec { default: Some(Value::Bool(true)) },
    );

    let spec = NodeSpec {
        content: Some("inline".to_string()),
        marks: None,
        attrs: Some(attrs),
        desc: Some("标题节点，支持1-6级标题".to_string()),
        ..Default::default()
    };

    Node::create("heading", spec)
}

/// 创建列表节点
/// 用于表示有序或无序列表
pub fn create_list_node() -> Node {
    let mut attrs = HashMap::new();
    attrs.insert(
        "list_type".to_string(),
        AttributeSpec {
            default: Some(Value::String("bullet".to_string())), // bullet, ordered, todo
        },
    );
    attrs.insert(
        "tight".to_string(),
        AttributeSpec { default: Some(Value::Bool(false)) },
    );
    attrs.insert(
        "start".to_string(),
        AttributeSpec {
            default: Some(Value::Number(serde_json::Number::from(1))),
        },
    );

    let spec = NodeSpec {
        content: Some("listitem+".to_string()),
        marks: None,
        attrs: Some(attrs),
        desc: Some("列表节点，包含列表项".to_string()),
        ..Default::default()
    };

    Node::create("list", spec)
}

/// 创建列表项节点
pub fn create_list_item_node() -> Node {
    let mut attrs = HashMap::new();
    attrs.insert(
        "checked".to_string(),
        AttributeSpec {
            default: Some(Value::Null), // null表示不是todo项，true/false表示已选中/未选中
        },
    );

    let spec = NodeSpec {
        content: Some("paragraph".to_string()),
        marks: None,
        attrs: Some(attrs),
        desc: Some("列表项节点".to_string()),
        ..Default::default()
    };

    Node::create("listitem", spec)
}

/// 创建表格节点
/// 用于表示结构化的表格数据
pub fn create_table_node() -> Node {
    let mut attrs = HashMap::new();
    attrs.insert(
        "rows".to_string(),
        AttributeSpec {
            default: Some(Value::Number(serde_json::Number::from(1))),
        },
    );
    attrs.insert(
        "cols".to_string(),
        AttributeSpec {
            default: Some(Value::Number(serde_json::Number::from(1))),
        },
    );
    attrs.insert(
        "has_header".to_string(),
        AttributeSpec { default: Some(Value::Bool(true)) },
    );
    attrs.insert(
        "border".to_string(),
        AttributeSpec { default: Some(Value::Bool(true)) },
    );

    let spec = NodeSpec {
        content: Some("tablerow+".to_string()),
        marks: None,
        attrs: Some(attrs),
        desc: Some("表格节点，包含表格行".to_string()),
        ..Default::default()
    };

    Node::create("table", spec)
}

/// 创建表格行节点
pub fn create_table_row_node() -> Node {
    let spec = NodeSpec {
        content: Some("tablecell+".to_string()),
        marks: None,
        attrs: None,
        desc: Some("表格行节点，包含表格单元格".to_string()),
        ..Default::default()
    };

    Node::create("tablerow", spec)
}

/// 创建表格单元格节点
pub fn create_table_cell_node() -> Node {
    let mut attrs = HashMap::new();
    attrs.insert(
        "colspan".to_string(),
        AttributeSpec {
            default: Some(Value::Number(serde_json::Number::from(1))),
        },
    );
    attrs.insert(
        "rowspan".to_string(),
        AttributeSpec {
            default: Some(Value::Number(serde_json::Number::from(1))),
        },
    );
    attrs.insert(
        "align".to_string(),
        AttributeSpec { default: Some(Value::String("left".to_string())) },
    );
    attrs.insert(
        "is_header".to_string(),
        AttributeSpec { default: Some(Value::Bool(false)) },
    );

    let spec = NodeSpec {
        content: Some("paragraph".to_string()),
        marks: None,
        attrs: Some(attrs),
        desc: Some("表格单元格节点".to_string()),
        ..Default::default()
    };

    Node::create("tablecell", spec)
}

/// 创建代码块节点
/// 用于显示代码
pub fn create_code_block_node() -> Node {
    let mut attrs = HashMap::new();
    attrs.insert(
        "language".to_string(),
        AttributeSpec { default: Some(Value::String("".to_string())) },
    );
    attrs.insert(
        "line_numbers".to_string(),
        AttributeSpec { default: Some(Value::Bool(false)) },
    );
    attrs.insert(
        "wrap".to_string(),
        AttributeSpec { default: Some(Value::Bool(false)) },
    );

    let spec = NodeSpec {
        content: Some("text".to_string()),
        marks: None,
        attrs: Some(attrs),
        desc: Some("代码块节点，用于显示代码".to_string()),
        ..Default::default()
    };

    Node::create("codeblock", spec)
}

/// 创建引用块节点
/// 用于表示引用内容
pub fn create_blockquote_node() -> Node {
    let spec = NodeSpec {
        content: Some("paragraph+".to_string()),
        marks: None,
        attrs: None,
        desc: Some("引用块节点，用于引用内容".to_string()),
        ..Default::default()
    };

    Node::create("blockquote", spec)
}

/// 创建水平分割线节点
/// 用于分割内容
pub fn create_horizontal_rule_node() -> Node {
    let spec = NodeSpec {
        content: None,
        marks: None,
        attrs: None,
        desc: Some("水平分割线节点".to_string()),
        ..Default::default()
    };

    Node::create("horizontalrule", spec)
}
