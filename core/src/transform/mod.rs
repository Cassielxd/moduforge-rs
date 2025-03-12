use std::sync::Arc;

use attr_step::AttrStep;
use bincode::{Decode, Encode};
use mark_step::AddMarkStep;
use node_step::{AddNodeStep, MoveNodeStep, RemoveNodeStep, ReplaceNodeStep};
use serde::{Deserialize, Serialize};
use step::{Step, StepResult};
use transform::TransformError;

use crate::model::{node_pool::Draft, patch::Patch, schema::Schema};
pub mod attr_step;
pub mod mark_step;
pub mod node_step;
pub mod step;
pub mod transform;

#[derive(Debug, Clone, Serialize, Deserialize, Decode, Encode)]
pub enum ConcreteStep {
    UpdateAttrs(AttrStep),
    AddNodeStep(AddNodeStep),
    AddMarkStep(AddMarkStep),
    RemoveNodeStep(RemoveNodeStep),
    PatchStep(PatchStep),
    MoveNodeStep(MoveNodeStep),
    ReplaceNodeStep(ReplaceNodeStep),
    BatchStep(BatchStep),
}
impl Step for ConcreteStep {
    fn apply(
        &self,
        dart: &mut Draft,
        schema: std::sync::Arc<crate::model::schema::Schema>,
    ) -> Result<step::StepResult, transform::TransformError> {
        match self {
            ConcreteStep::UpdateAttrs(attr_step) => attr_step.apply(dart, schema),
            ConcreteStep::AddNodeStep(add_node_step) => add_node_step.apply(dart, schema),
            ConcreteStep::AddMarkStep(add_mark_step) => add_mark_step.apply(dart, schema),
            ConcreteStep::RemoveNodeStep(remove_node_step) => remove_node_step.apply(dart, schema),
            ConcreteStep::PatchStep(patch_step) => patch_step.apply(dart, schema),
            ConcreteStep::MoveNodeStep(move_node_step) => move_node_step.apply(dart, schema),
            ConcreteStep::BatchStep(batch_step) => batch_step.apply(dart, schema),
            ConcreteStep::ReplaceNodeStep(replace_node_step) => replace_node_step.apply(dart, schema),
        }
    }
    fn to_concrete(&self) -> ConcreteStep {
        self.clone()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Decode, Encode)]
pub struct PatchStep {
    pub patches: Vec<Patch>,
}
impl Step for PatchStep {
    fn apply(
        &self,
        dart: &mut Draft,
        _: std::sync::Arc<crate::model::schema::Schema>,
    ) -> Result<step::StepResult, transform::TransformError> {
        match dart.apply_patches(&self.patches) {
            Ok(()) => {
                Ok(dart.commit())
            },
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }

    fn to_concrete(&self) -> ConcreteStep {
        ConcreteStep::PatchStep(self.clone())
    }
}
/// 批量操作步骤
#[derive(Debug, Serialize, Deserialize, Clone, Decode, Encode)]
pub struct BatchStep {
    steps: Vec<ConcreteStep>,
}

impl BatchStep {
    pub fn new(steps: Vec<ConcreteStep>) -> Self {
        BatchStep { steps }
    }
}
impl Step for BatchStep {
    fn apply(
        &self,
        dart: &mut Draft,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError> {
        dart.begin=true;
        for step in &self.steps {
            let schema = schema.clone();
            let result = match step {
                ConcreteStep::UpdateAttrs(attr_step) => attr_step.apply(dart, schema),
                ConcreteStep::AddNodeStep(add_node_step) => add_node_step.apply(dart, schema),
                ConcreteStep::AddMarkStep(add_mark_step) => add_mark_step.apply(dart, schema),
                ConcreteStep::RemoveNodeStep(remove_node_step) => remove_node_step.apply(dart, schema),
                ConcreteStep::PatchStep(patch_step) => patch_step.apply(dart, schema),
                ConcreteStep::MoveNodeStep(move_node_step) => move_node_step.apply(dart, schema),
                ConcreteStep::ReplaceNodeStep(replace_node_step) => replace_node_step.apply(dart, schema),
                ConcreteStep::BatchStep(batch_setp) =>batch_setp.apply(dart, schema),
            };
            match result {
                Ok(result) => {
                    if let Some(message) = result.failed {
                        return Ok(StepResult::fail(message));
                    }
                    // 继续执行下一个步骤
                },
                Err(err) => return Err(err),
            }
        }
        dart.begin=false;
        // 所有步骤执行成功，提交更改
        Ok(dart.commit())
    }

    fn to_concrete(&self) -> ConcreteStep {
        ConcreteStep::BatchStep(self.clone())
    }
}
