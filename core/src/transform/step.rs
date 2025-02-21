use std::sync::Arc;

use crate::model::{
    node_pool::{Draft, NodePool},
    patch::Patch,
    schema::Schema,
};
use std::fmt::Debug;

use super::{ConcreteStep, transform::TransformError};

pub trait Step: Send + Sync + Debug {
    fn apply(&self, doc: &mut Draft, schema: Arc<Schema>) -> Result<StepResult, TransformError>;
    fn to_concrete(&self) -> ConcreteStep;
}

pub struct StepResult {
    pub doc: Option<Arc<NodePool>>,
    pub failed: Option<String>,
    pub patches: Vec<Patch>,
}

impl StepResult {
    pub fn ok(doc: Arc<NodePool>, patches: Vec<Patch>) -> Self {
        StepResult {
            doc: Some(doc),
            failed: None,
            patches,
        }
    }

    pub fn fail(message: String) -> Self {
        StepResult {
            doc: None,
            failed: Some(message),
            patches: vec![],
        }
    }
}

pub enum Steps {}
