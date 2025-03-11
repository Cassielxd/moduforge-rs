use std::sync::Arc;

use crate::model::{mark::Mark, node_pool::Draft, schema::Schema, types::NodeId};

use super::{
    ConcreteStep,
    step::{Step, StepResult},
    transform::TransformError,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone, Decode, Encode)]
pub struct AddMarkStep {
    id: NodeId,
    mark: Mark,
}
impl AddMarkStep {
    pub fn new(
        id: NodeId,
        mark: Mark,
    ) -> Self {
        AddMarkStep { id, mark }
    }
}
impl Step for AddMarkStep {
    fn apply(
        &self,
        dart: &mut Draft,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError> {
        let _ = schema;

        match dart.add_mark(&self.id, self.mark.clone()) {
            Ok(_) => {
                Ok(dart.commit())
            },
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }

    fn to_concrete(&self) -> super::ConcreteStep {
        ConcreteStep::AddMarkStep(self.clone())
    }
}
