use std::collections::HashMap;

use moduforge_model::{node_type::NodeSpec, schema::AttributeSpec};
use serde_json::Value;
#[derive(Clone, PartialEq, Debug, Eq, Default)]
pub struct Node {
    pub name: String,
    pub r#type: NodeSpec,
    pub top_node: bool,
}

impl Node {
    pub fn create(
        name: &str,
        spec: NodeSpec,
    ) -> Node {
        Node { name: name.to_string(), r#type: spec, top_node: false }
    }
    pub fn set_name(
        &mut self,
        name: &str,
    ) -> &mut Self {
        self.name = name.to_string();
        self
    }
    pub fn set_top_node(&mut self) -> &mut Self {
        self.top_node = true;
        self
    }
    pub fn is_top_node(&self) -> bool {
        self.top_node
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn set_content(
        &mut self,
        content: &str,
    ) -> &mut Self {
        self.r#type.content = Some(content.to_string());
        self
    }

    pub fn set_marks(
        &mut self,
        marks: String,
    ) -> &mut Self {
        self.r#type.marks = Some(marks);
        self
    }

    pub fn set_attrs(
        &mut self,
        attrs: HashMap<String, AttributeSpec>,
    ) -> &mut Self {
        self.r#type.attrs = Some(attrs);
        self
    }
    pub fn set_attr(
        &mut self,
        name: &str,
        default: Option<Value>,
    ) -> &mut Self {
        match &mut self.r#type.attrs {
            Some(map) => {
                map.insert(name.to_string(), AttributeSpec { default });
            },
            None => {
                let mut new_map = HashMap::new();
                new_map.insert(name.to_string(), AttributeSpec { default });
                self.r#type.attrs = Some(new_map);
            },
        }
        self
    }
    pub fn set_desc(
        &mut self,
        desc: &str,
    ) -> &mut Self {
        self.r#type.desc = Some(desc.to_string());
        self
    }
}
