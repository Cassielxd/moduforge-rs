use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::model::{node_pool::NodePool, schema::Schema};

use super::{transform::TransformError, ConcreteStep};

pub trait Step: Send + Sync {
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
