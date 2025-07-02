use std::collections::HashMap;

use mf_model::{mark_type::MarkSpec, schema::AttributeSpec};
use serde_json::Value;

#[derive(Clone, PartialEq, Debug, Eq, Default)]
pub struct Mark {
    pub name: String,
    pub r#type: MarkSpec,
}

impl Mark {
    pub fn new(
        name: &str,
        spec: MarkSpec,
    ) -> Mark {
        Mark { name: name.to_string(), r#type: spec }
    }
    pub fn set_name(
        &mut self,
        name: &str,
    ) -> &mut Self {
        self.name = name.to_string();
        self
    }
    pub fn get_name(&self) -> &str {
        &self.name
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
