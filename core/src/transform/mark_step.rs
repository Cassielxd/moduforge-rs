use std::sync::Arc;

use crate::model::{mark::Mark, node_pool::NodePool, schema::Schema, types::NodeId};

use super::{
    step::{Step, StepResult},
    transform::TransformError,
    ConcreteStep,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone, Decode, Encode)]
pub struct AddMarkStep {
    id: NodeId,
    mark: Mark,
}
impl AddMarkStep {
    pub fn new(id: NodeId, mark: Mark) -> Self {
        AddMarkStep { id, mark }
    }
}
impl Step for AddMarkStep {
    fn apply(
        &self,
        node_pool: Arc<NodePool>,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError> {
        let _ = schema;

        match node_pool.add_mark(&self.id, self.mark.clone()) {
            Ok(node_pool) => Ok(StepResult::ok(Arc::new(node_pool))),
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }

    fn to_concrete(&self) -> super::ConcreteStep {
        ConcreteStep::AddMarkStep(self.clone())
    }
}
