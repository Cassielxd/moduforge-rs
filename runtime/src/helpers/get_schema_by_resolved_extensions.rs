use std::collections::HashMap;

use moduforge_core::model::schema::{AttributeSpec, Schema, SchemaSpec};

use crate::types::{Extensions, GlobalAttributeItem};

pub fn get_schema_by_resolved_extensions(
    extensions: &Vec<Extensions>,
) -> Result<Schema, Box<dyn std::error::Error>> {
    let mut extension_attributes = vec![];
    for extension in extensions {
        match extension {
            Extensions::E(extension) => {
                for item in extension.get_global_attributes().iter() {
                    extension_attributes.push(item);
                }
            }
            _ => {}
        }
    }
    let mut nodes = HashMap::new();
    let mut marks = HashMap::new();
    let mut top_name = "doc".to_string();
    for extension in extensions {
        match extension {
            Extensions::N(node) => {
                let name = node.name.clone();
                if node.is_top_node() {
                    top_name = node.name.clone();
                }
                let mut attrs = get_attr_dfn(name, &extension_attributes);

                let attrs_def = match &node.r#type.attrs {
                    Some(m) => {
                        m.iter().for_each(|e| {
                            attrs.insert(e.0.clone(), e.1.clone());
                        });
                        attrs
                    }
                    None => attrs,
                };
                let mut t = node.r#type.clone();
                t.attrs = Some(attrs_def);
                nodes.insert(node.name.clone(), t);
            }
            Extensions::M(mark) => {
                marks.insert(mark.name.clone(), mark.r#type.clone());
            }
            _ => {}
        }
    }
    let instance_spec = SchemaSpec {
        nodes,
        marks,
        top_node: Some(top_name),
    };
    let schema = Schema::compile(instance_spec)?;
    Ok(schema)
}

fn get_attr_dfn(
    name: String,
    extension_attributes: &Vec<&GlobalAttributeItem>,
) -> HashMap<String, AttributeSpec> {
    let mut attributes: HashMap<String, AttributeSpec> = HashMap::new();
    for attr in extension_attributes.iter() {
        if attr.types.contains(&name) {
            attr.attributes.iter().for_each(|e| {
                attributes.insert(e.0.clone(), e.1.clone());
            });
        }
    }
    attributes
}
