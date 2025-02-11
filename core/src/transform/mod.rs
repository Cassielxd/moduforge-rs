use attr_step::AttrStep;
use bincode::{Decode, Encode};
use mark_step::AddMarkStep;
use node_step::AddNodeStep;
use serde::{Deserialize, Serialize};
use step::Step;
pub mod attr_step;
pub mod mark_step;
pub mod node_step;
pub mod step;
pub mod transform;

#[derive(Debug, Clone, Serialize, Deserialize, Decode, Encode)]
pub enum ConcreteStep {
    UpdateAttrs(AttrStep),
    AddNodeStep(AddNodeStep),
    AddMarkStep(AddMarkStep)
}
impl Step for ConcreteStep {
    fn apply(
        &self,
        doc: std::sync::Arc<crate::model::node_pool::NodePool>,
        schema: std::sync::Arc<crate::model::schema::Schema>,
    ) -> Result<step::StepResult, transform::TransformError> {
        match self {
            ConcreteStep::UpdateAttrs(attr_step) => attr_step.apply(doc, schema),
            ConcreteStep::AddNodeStep(add_node_step) => add_node_step.apply(doc, schema),
            ConcreteStep::AddMarkStep(add_mark_step) => add_mark_step.apply(doc, schema),
        }
    }
    fn to_concrete(&self) -> ConcreteStep {
        self.clone()
    }
}
