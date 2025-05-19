use std::{any::Any, sync::Arc};

use moduforge_model::{schema::Schema, tree::Tree};
use std::fmt::Debug;

use super::transform::TransformError;

pub trait Step: Any + Send + Sync + Debug + 'static {
    fn name(&self) -> String;
    fn apply(
        &self,
        dart: &mut Tree,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError>;
    fn serialize(&self) -> Option<Vec<u8>>;
}
impl dyn Step {
    pub fn downcast_ref<E: Step>(&self) -> Option<&E> {
        <dyn Any>::downcast_ref::<E>(self)
    }
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
