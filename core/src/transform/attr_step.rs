use std::sync::Arc;

use crate::model::{node_pool::NodePool, schema::Schema, types::NodeId};

use super::{
    step::{Step, StepResult},
    transform::TransformError,
    ConcreteStep,
};
use im::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttrStep {
    id: NodeId,
    values: HashMap<String, serde_json::Value>,
}
impl AttrStep {
    pub fn new(id: String, values: HashMap<String, serde_json::Value>) -> Self {
        AttrStep {
            id: id.into(),
            values,
        }
    }
}
impl Step for AttrStep {
    fn apply(
        &self,
        node_pool: Arc<NodePool>,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError> {
        let _ = schema;
        match node_pool.update_attr(&self.id, &self.values) {
            Ok(node_pool) => Ok(StepResult::ok(Arc::new(node_pool))),
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }

    fn to_concrete(&self) -> super::ConcreteStep {
        ConcreteStep::UpdateAttrs(self.clone())
    }
}
