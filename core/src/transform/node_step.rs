use std::sync::Arc;

use crate::model::{
    node::Node,
    node_pool::{Draft, NodePool},
    schema::Schema,
    types::NodeId,
};

use super::{
    step::{Step, StepResult},
    transform::TransformError,
    ConcreteStep,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone, Decode, Encode)]
pub struct AddNodeStep {
    parent_id: NodeId,
    node: Node,
}
impl AddNodeStep {
    pub fn new(parent_id: NodeId, node: Node) -> Self {
        AddNodeStep { parent_id, node }
    }
}
impl Step for AddNodeStep {
    fn apply(&self, dart: &mut Draft, schema: Arc<Schema>) -> Result<StepResult, TransformError> {
        let _ = schema;

        match dart.add_node(&self.parent_id, self.node.clone()) {
            Ok(()) => {
                let (node_pool, _patches) = dart.commit();
                Ok(StepResult::ok(node_pool, _patches))
            }
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }

    fn to_concrete(&self) -> super::ConcreteStep {
        ConcreteStep::AddNodeStep(self.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Decode, Encode)]
pub struct RemoveNodeStep {
    parent_id: NodeId,
    node_ids: Vec<NodeId>,
}
impl RemoveNodeStep {
    pub fn new(parent_id: NodeId, node_ids: Vec<NodeId>) -> Self {
        RemoveNodeStep {
            parent_id,
            node_ids,
        }
    }
}
impl Step for RemoveNodeStep {
    fn apply(&self, dart: &mut Draft, schema: Arc<Schema>) -> Result<StepResult, TransformError> {
        let _ = schema;

        match dart.remove_node(&self.parent_id, self.node_ids.clone()) {
            Ok(()) => {
                let (node_pool, _patches) = dart.commit();
                Ok(StepResult::ok(node_pool, _patches))
            }
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }

    fn to_concrete(&self) -> super::ConcreteStep {
        ConcreteStep::RemoveNodeStep(self.clone())
    }
}
