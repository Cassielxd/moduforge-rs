use std::{collections::HashMap, sync::Arc};

use crate::model::{node_pool::NodePool, schema::Schema};

use super::step::{Step, StepResult};



pub struct AttrStep {
    id: String,
    values: HashMap<String, serde_json::Value>,
}
impl AttrStep {
    pub fn new(id: String, values: HashMap<String, serde_json::Value>) -> Self {
        AttrStep { id, values }
    }
}
impl Step for AttrStep {
    fn apply(&self, node_pool: Arc<NodePool>, schema: Arc<Schema>) -> StepResult {
        todo!()
    }

    fn to_json(&self) -> serde_json::Value {
        todo!()
    }
    
    fn invert(&self) -> Box<(dyn Step + 'static)> {
        todo!()
    }
}
