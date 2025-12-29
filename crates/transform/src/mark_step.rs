use std::{sync::Arc};

use mf_model::{mark::Mark, schema::Schema, tree::Tree, types::NodeId, node_pool::NodePool};

use crate::{transform_error, TransformResult};

use super::{
    step::{StepGeneric, StepResult},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddMarkStep {
    pub id: NodeId,
    pub marks: Vec<Mark>,
}
impl AddMarkStep {
    pub fn new(
        id: NodeId,
        marks: Vec<Mark>,
    ) -> Self {
        AddMarkStep { id, marks }
    }
}
impl StepGeneric<NodePool, Schema> for AddMarkStep {
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
            Err(e) => Err(transform_error(e.to_string())),
        }
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }

    fn invert(
        &self,
        dart: &Arc<Tree>,
    ) -> Option<Arc<dyn StepGeneric<NodePool, Schema>>> {
        match dart.get_node(&self.id) {
            Some(_) => Some(Arc::new(RemoveMarkStep::new(
                self.id.clone(),
                self.marks.clone().iter().map(|m| m.r#type.clone()).collect(),
            ))),
            None => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoveMarkStep {
    pub id: NodeId,
    pub mark_types: Vec<String>,
}
impl RemoveMarkStep {
    pub fn new(
        id: NodeId,
        mark_types: Vec<String>,
    ) -> Self {
        RemoveMarkStep { id, mark_types }
    }
}
impl StepGeneric<NodePool, Schema> for RemoveMarkStep {
    fn name(&self) -> String {
        "remove_mark_step".to_string()
    }
    fn apply(
        &self,
        dart: &mut Tree,
        schema: Arc<Schema>,
    ) -> TransformResult<StepResult> {
        let _ = schema;
        let result = dart.mark(&self.id) - self.mark_types.clone();
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
    ) -> Option<Arc<dyn StepGeneric<NodePool, Schema>>> {
        match dart.get_node(&self.id) {
            Some(node) => {
                // 仅恢复被移除的 mark 类型，避免把未移除的也加回
                let removed_types = &self.mark_types;
                let to_restore: Vec<Mark> = node
                    .marks
                    .iter()
                    .filter(|m| removed_types.contains(&m.r#type))
                    .cloned()
                    .collect();
                if to_restore.is_empty() {
                    None
                } else {
                    Some(Arc::new(AddMarkStep::new(
                        self.id.clone(),
                        to_restore,
                    )))
                }
            },
            None => None,
        }
    }
}
