use std::sync::Arc;

use moduforge_model::{mark::Mark, schema::Schema, types::NodeId};
use crate::draft::Draft;

use super::{
    step::{Step, StepResult},
    transform::TransformError,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddMarkStep {
    id: NodeId,
    marks: Vec<Mark>,
}
impl AddMarkStep {
    pub fn new(
        id: NodeId,
        marks: Vec<Mark>,
    ) -> Self {
        AddMarkStep { id, marks }
    }
}
impl Step for AddMarkStep {
    fn name(&self) -> String {
        "add_mark_step".to_string()
    }
    fn apply(
        &self,
        dart: &mut Draft,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError> {
        let _ = schema;

        match dart.add_mark(&self.id, &self.marks) {
            Ok(_) => Ok(dart.commit()),
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }

}
