#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use moduforge_model::{
        content::ContentMatch,
        node_type::NodeSpec,
        schema::{Schema, SchemaSpec},
    };
    #[allow(dead_code)]
    pub fn create_test_schema() -> Schema {
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
                content: Some("djgc*".to_string()),
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
    fn test_content_match() {
        let schema = create_test_schema();
        // Test DW djgc djgc content match
        let content_match =
            ContentMatch::parse("doc".to_string(), &schema.nodes);
        println!("Content match for DW djgc djgc: {}", content_match);
        dbg!(content_match);
    }
}
