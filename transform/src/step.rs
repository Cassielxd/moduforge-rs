use std::sync::Arc;

use moduforge_model::{schema::Schema, tree::Tree};
use std::fmt::Debug;

use super::transform::TransformError;

pub trait Step: Send + Sync + Debug {
    fn name(&self) -> String;
    fn apply(
        &self,
        dart: &mut Tree,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError>;
    fn serialize(&self) -> Option<Vec<u8>>;
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub failed: Option<String>,
}

impl StepResult {
    pub fn ok() -> Self {
        StepResult { failed: None }
    }

    pub fn fail(message: String) -> Self {
        StepResult { failed: Some(message) }
    }
}
