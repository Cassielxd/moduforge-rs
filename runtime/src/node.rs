use std::collections::HashMap;

use moduforge_core::model::{node_type::NodeSpec, schema::AttributeSpec};

#[derive(Clone, PartialEq, Debug, Eq, Default)]
pub struct Node {
    pub name: String,
    pub r#type: NodeSpec,
    pub top_node: bool,
}

impl Node {
    pub fn new(name: &str, spec: NodeSpec) -> Node {
        Node {
            name: name.to_string(),
            r#type: spec,
            top_node: false,
        }
    }
    pub fn set_top_node(&mut self, top_node: bool) -> &mut Self {
        self.top_node = top_node;
        self
    }
    pub fn is_top_node(&self) -> bool {
        self.top_node
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn set_content(&mut self, content: &str) -> &mut Self {
        self.r#type.content = Some(content.to_string());
        self
    }

    pub fn set_marks(&mut self, marks: String) -> &mut Self {
        self.r#type.marks = Some(marks);
        self
    }

    pub fn set_attrs(&mut self, attrs: HashMap<String, AttributeSpec>) -> &mut Self {
        self.r#type.attrs = Some(attrs);
        self
    }
}
