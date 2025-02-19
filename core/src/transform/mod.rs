use attr_step::AttrStep;
use bincode::{Decode, Encode};
use mark_step::AddMarkStep;
use node_step::{AddNodeStep, RemoveNodeStep};
use serde::{Deserialize, Serialize};
use step::{Step, StepResult};
use transform::TransformError;

use crate::model::{node_pool::Draft, patch::Patch};
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
        }
    }
    fn to_concrete(&self) -> ConcreteStep {
        self.clone()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Decode, Encode)]
pub struct PatchStep {
   pub  patches: Vec<Patch>,
}
impl Step for PatchStep {
    fn apply(
        &self,
        dart: &mut Draft,
        _: std::sync::Arc<crate::model::schema::Schema>,
    ) -> Result<step::StepResult, transform::TransformError> {
        match dart.apply_patches(&self.patches) {
            Ok(()) => {
                let (node_pool, _patches) = dart.commit();
                Ok(StepResult::ok(node_pool))
            }
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }

    fn to_concrete(&self) -> ConcreteStep {
        ConcreteStep::PatchStep(self.clone())
    }
}
