use std::sync::Arc;

use crate::model::{node_pool::NodePool, schema::Schema};



pub trait Step {
    fn apply(&self, doc: Arc<NodePool>, schema: Arc<Schema>) -> StepResult;
    fn to_json(&self) -> serde_json::Value;
    //翻转
    fn invert(&self) -> Box<dyn Step>;
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
