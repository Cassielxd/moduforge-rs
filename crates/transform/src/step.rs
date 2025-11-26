use std::{
    any::{type_name, Any},
    sync::Arc,
};

use mf_model::traits::{DataContainer, SchemaDefinition};
use std::fmt::Debug;

use crate::TransformResult;

/// Generic Step trait for any container and schema type
pub trait StepGeneric<C, S>: Any + Send + Sync + Debug + 'static
where
    C: DataContainer,
    S: SchemaDefinition<Container = C>,
{
    fn name(&self) -> String {
        type_name::<Self>().to_string()
    }

    fn apply(
        &self,
        inner: &mut C::InnerState,
        schema: Arc<S>,
    ) -> TransformResult<StepResult>;

    fn serialize(&self) -> Option<Vec<u8>>;

    fn invert(
        &self,
        inner: &Arc<C::InnerState>,
    ) -> Option<Arc<dyn StepGeneric<C, S>>>;
}

// Generic downcast implementation
impl<C, S> dyn StepGeneric<C, S>
where
    C: DataContainer,
    S: SchemaDefinition<Container = C>,
{
    pub fn downcast_ref<E: StepGeneric<C, S>>(&self) -> Option<&E> {
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
