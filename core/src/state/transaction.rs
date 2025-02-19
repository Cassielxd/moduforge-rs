use std::collections::HashMap;

use std::future::Future;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::state::State;
use crate::model::node::Node;
use crate::model::node_pool::{Draft, NodePool};
use crate::model::schema::Schema;
use crate::transform::attr_step::AttrStep;
use crate::transform::node_step::AddNodeStep;
use crate::transform::step::{Step, StepResult};
use crate::transform::transform::{Transform, TransformError};
use crate::transform::{ConcreteStep, PatchStep};

#[derive(Debug)]
pub struct Transaction {
    pub meta: HashMap<String, Box<dyn std::any::Any>>,
    pub time: u64,
    pub steps: Vec<Box<dyn Step>>,
    pub doc: Arc<NodePool>,
    pub draft: Draft,
    pub schema: Arc<Schema>,
}
unsafe impl Send for Transaction {}
unsafe impl Sync for Transaction {}
impl Transform for Transaction {
    fn step(&mut self, step: Box<dyn Step>) -> Result<(), TransformError> {
        let result = step.apply(&mut self.draft, self.schema.clone())?;
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
    pub async fn transaction<F, O>(&mut self, call_back: F)
    where
        F: Fn(&mut Transaction) -> O + Sync + Send,
        O: Future<Output = Result<(), TransformError>> + Send,
    {
        let result = call_back(self).await;
        match result {
            Ok(_) => {
                let (node_pool, patches) = self.draft.commit();
                self.add_step(Box::new(PatchStep { patches }), node_pool);
            }
            Err(_) => {}
        }
    }

    pub fn new(state: &State) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let node = state.doc();
        Transaction {
            meta: HashMap::new(),
            time: now,
            steps: vec![],
            doc: node,
            schema: state.schema(),
            draft: Draft::new(state.doc()),
        }
    }
    pub fn doc(&self) -> Arc<NodePool> {
        self.doc.clone()
    }
    pub fn as_concrete(step: &Box<dyn Step>) -> ConcreteStep {
        step.to_concrete()
    }
    pub fn set_node_attribute(&mut self, id: String, values: im::HashMap<String, String>) {
        let _ = self.step(Box::new(AttrStep::new(id, values)));
    }
    pub fn add_node(&mut self, parent_id: String, node: Node) {
        let _ = self.step(Box::new(AddNodeStep::new(parent_id, node)));
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
