use std::sync::Arc;


use super::{
    step::{Step, StepResult},
    transform::TransformError,
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
    ) -> Result<StepResult, TransformError> {
        let _ = schema;
       let _ = dart.attrs(&self.id)+self.values.clone();
        Ok(StepResult::ok())
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }
    
}
