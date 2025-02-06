use attr_step::AttrStep;
use serde::{Deserialize, Serialize};

pub mod attr_step;
pub mod step;
pub mod transform;

#[derive(Debug, Clone, Serialize,Deserialize)]
pub enum ConcreteStep {
    UpdateAttrs(AttrStep),
}
