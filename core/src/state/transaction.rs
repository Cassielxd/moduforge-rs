use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::state::State;
use crate::model::node_pool::NodePool;
use crate::model::schema::Schema;
use crate::transform::attr_step::AttrStep;
use crate::transform::step::Step;
use crate::transform::transform::{Transform, TransformError};
use crate::transform::ConcreteStep;
pub struct Transaction {
    pub meta: HashMap<String, Box<dyn std::any::Any>>,
    pub time: u64,
    pub steps: Vec<Box<dyn Step>>,
    pub docs: Vec<Arc<NodePool>>,
    pub doc: Arc<NodePool>,
    pub schema: Arc<Schema>,
}
unsafe impl Send for Transaction {}
unsafe impl Sync for Transaction {}
impl Transform for Transaction {
    fn before(&self) -> &NodePool {
        self.docs.get(0).unwrap_or(&self.doc)
    }

    fn step(&mut self, step: Box<dyn Step>) -> Result<(), TransformError> {
        let result = step.apply(self.doc.clone(), self.schema.clone())?;
        match result.failed {
            Some(message) => Err(TransformError::new(message)),
            None => {
                self.add_step(step, result.doc.unwrap());
                Ok(())
            }
        }
    }
    fn doc_changed(&self) -> bool {
        !self.steps.is_empty()
    }

    fn add_step(&mut self, step: Box<dyn Step>, doc: Arc<NodePool>) {
        self.steps.push(step);
        self.doc = doc;
    }
}
impl Transaction {
    pub fn new(state: &State) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let node = state.doc();
        Transaction {
            meta: HashMap::new(),
            time: now,
            steps: vec![],
            docs: vec![],
            doc: node,
            schema: state.schema(),
        }
    }
    pub fn doc(&self) -> Arc<NodePool> {
        self.doc.clone()
    }
    pub fn as_concrete(step: &Box<dyn Step>) -> ConcreteStep {
        step.to_concrete()
    }
    pub fn set_node_attribute(
        &mut self,
        id: String,
        values: im::HashMap<String, String>,
    ) {
        let _ = self.step(Box::new(AttrStep::new(id, values)));
    }
    pub fn set_time(&mut self, time: u64) -> &mut Self {
        self.time = time;
        self
    }

    pub fn set_meta<K>(&mut self, key: K, value: Box<dyn std::any::Any>) -> &mut Self
    where
        K: Into<String>,
    {
        let key_str = key.into();
        self.meta.insert(key_str, value);
        self
    }

    pub fn get_meta<T: 'static, K>(&self, key: K) -> Option<&T>
    where
        K: Into<String>,
    {
        let key_str = key.into();
        self.meta.get(&key_str)?.downcast_ref::<T>()
    }
}
