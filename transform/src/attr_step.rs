use std::sync::Arc;

use crate::{transform_error, TransformResult};

use super::{
    step::{Step, StepResult},
};
use im::HashMap;
use moduforge_model::{schema::Schema, tree::Tree, types::NodeId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttrStep {
    id: NodeId,
    values: HashMap<String, Value>,
}
impl AttrStep {
    pub fn new(
        id: String,
        values: HashMap<String, Value>,
    ) -> Self {
        AttrStep { id, values }
    }
}
impl Step for AttrStep {
    fn name(&self) -> String {
        "attr_step".to_string()
    }
    fn apply(
        &self,
        dart: &mut Tree,
        schema: Arc<Schema>,
    ) -> TransformResult<StepResult> {
        let _ = schema;
        match dart.get_node(&self.id) {
            Some(node) => {
                let attr = &schema.nodes.get(&node.r#type).unwrap().attrs;
                // 删除 self.values 中 attr中没有定义的属性
                let mut new_values = self.values.clone();
                for (key, _) in self.values.iter() {
                    if !attr.contains_key(key) {
                        new_values.remove(key);
                    }
                }
                let result = dart.attrs(&self.id) + new_values;
                match result {
                    Ok(_) => Ok(StepResult::ok()),
                    Err(e) => Err(transform_error(e.to_string())),
                }
            },
            None => {
                return Err(transform_error("节点不存在".to_string()));
            },
        }
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }
}
