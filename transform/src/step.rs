use std::{any::Any, sync::Arc};

use moduforge_model::{schema::Schema, tree::Tree};
use std::fmt::Debug;

use crate::TransformResult;

pub trait Step: Any + Send + Sync + Debug + 'static {
    fn name(&self) -> String;
    fn apply(
        &self,
        dart: &mut Tree,
        schema: Arc<Schema>,
    ) -> TransformResult<StepResult>;
    fn serialize(&self) -> Option<Vec<u8>>;
    fn invert(
        &self,
        dart: &Arc<Tree>,
    ) -> Option<Arc<dyn Step>>;
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
