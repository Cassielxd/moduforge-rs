use std::{sync::Arc};

use moduforge_model::{mark::Mark, schema::Schema, tree::Tree, types::NodeId};

use crate::{transform_error, TransformResult};

use super::{
    step::{Step, StepResult},
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
    ) -> TransformResult<StepResult> {
        let _ = schema;
        let result = dart.mark(&self.id) + self.marks.clone();
        match result {
            Ok(_) => Ok(StepResult::ok()),
            Err(e) => Ok(StepResult::fail(e.to_string())),
        }
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }

    fn invert(
        &self,
        dart: &Arc<Tree>,
    ) -> Option<Arc<dyn Step>> {
        match dart.get_node(&self.id) {
            Some(_) => Some(Arc::new(RemoveMarkStep::new(
                self.id.clone(),
                self.marks.clone(),
            ))),
            None => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoveMarkStep {
    id: NodeId,
    marks: Vec<Mark>,
}
impl RemoveMarkStep {
    pub fn new(
        id: NodeId,
        marks: Vec<Mark>,
    ) -> Self {
        RemoveMarkStep { id, marks }
    }
}
impl Step for RemoveMarkStep {
    fn name(&self) -> String {
        "remove_mark_step".to_string()
    }
    fn apply(
        &self,
        dart: &mut Tree,
        schema: Arc<Schema>,
    ) -> TransformResult<StepResult> {
        let _ = schema;
        let result = dart.mark(&self.id) - self.marks.clone();
        match result {
            Ok(_) => Ok(StepResult::ok()),
            Err(e) => Err(transform_error(e.to_string())),
        }
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }

    fn invert(
        &self,
        dart: &Arc<Tree>,
    ) -> Option<Arc<dyn Step>> {
        match dart.get_node(&self.id) {
            Some(_) => Some(Arc::new(AddMarkStep::new(
                self.id.clone(),
                self.marks.clone(),
            ))),
            None => None,
        }
    }
}
