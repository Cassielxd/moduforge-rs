use attr_step::AttrStep;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use step::Step;
pub mod attr_step;
pub mod step;
pub mod transform;

#[derive(Debug, Clone, Serialize, Deserialize, Decode, Encode)]
pub enum ConcreteStep {
    UpdateAttrs(AttrStep),
}
impl Step for ConcreteStep {
    fn apply(
        &self,
        doc: std::sync::Arc<crate::model::node_pool::NodePool>,
        schema: std::sync::Arc<crate::model::schema::Schema>,
    ) -> Result<step::StepResult, transform::TransformError> {
        match self {
            ConcreteStep::UpdateAttrs(attr_step) => attr_step.apply(doc, schema),
        }
    }
    fn to_concrete(&self) -> ConcreteStep {
        self.clone()
    }
}
