use std::sync::Arc;

use crate::draft::Draft;

use super::{
    step::{Step, StepResult},
    transform::TransformError,
};
use im::HashMap;
use moduforge_model::{schema::Schema, types::NodeId};
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
        dart: &mut Draft,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError> {
        let _ = schema;
        match dart.update_attr(&self.id, self.values.clone()) {
            Ok(_) => Ok(dart.commit()),
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }
    
}
