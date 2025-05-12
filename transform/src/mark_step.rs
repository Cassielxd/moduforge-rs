use std::sync::Arc;

use moduforge_model::{mark::Mark, schema::Schema, tree::Tree, types::NodeId};

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
        dart: &mut Tree,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError> {
        let _ = schema;
        dart.mark(&self.id)+self.marks.clone();
        Ok(StepResult::ok())
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }

}
