use std::sync::Arc;

use crate::model::{node_pool::Draft, schema::Schema, types::NodeId};

use super::{
    ConcreteStep,
    step::{Step, StepResult},
    transform::TransformError,
};
use im::HashMap;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttrStep {
    id: NodeId,
    values: HashMap<String, String>,
}
impl AttrStep {
    pub fn new(
        id: String,
        values: HashMap<String, String>,
    ) -> Self {
        AttrStep { id, values }
    }
}
impl Step for AttrStep {
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

    fn to_concrete(&self) -> super::ConcreteStep {
        ConcreteStep::UpdateAttrs(self.clone())
    }
}
