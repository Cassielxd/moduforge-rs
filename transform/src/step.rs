use std::sync::Arc;

use moduforge_model::{node_pool::NodePool, schema::Schema};
use std::fmt::Debug;

use super::{draft::Draft, transform::TransformError};

pub trait Step: Send + Sync + Debug {
    fn name(&self) -> String;
    fn apply(
        &self,
        dart: &mut Draft,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError>;
    fn serialize(&self) -> Option<Vec<u8>>;
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub doc: Option<Arc<NodePool>>,
    pub failed: Option<String>
}

impl StepResult {
    pub fn ok(
        doc: Arc<NodePool>
    ) -> Self {
        StepResult { doc: Some(doc), failed: None }
    }

    pub fn fail(message: String) -> Self {
        StepResult { doc: None, failed: Some(message) }
    }
}

