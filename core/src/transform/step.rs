use std::sync::Arc;

use crate::model::{node_pool::NodePool, schema::Schema};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use super::{transform::TransformError, ConcreteStep};

pub trait Step: Send + Sync + Debug {
    fn apply(&self, doc: Arc<NodePool>, schema: Arc<Schema>) -> Result<StepResult, TransformError>;
    fn to_concrete(&self) -> ConcreteStep;
}

pub struct StepResult {
    pub doc: Option<Arc<NodePool>>,
    pub failed: Option<String>,
}

impl StepResult {
    pub fn ok(doc: Arc<NodePool>) -> Self {
        StepResult {
            doc: Some(doc),
            failed: None,
        }
    }

    pub fn fail(message: String) -> Self {
        StepResult {
            doc: None,
            failed: Some(message),
        }
    }
}

pub enum Steps {}
