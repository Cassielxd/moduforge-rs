use std::collections::HashMap;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;

use super::state::State;
use crate::model::node::Node;
use crate::model::node_pool::{Draft, NodePool};
use crate::model::patch::Patch;
use crate::model::schema::Schema;
use crate::transform::attr_step::AttrStep;
use crate::transform::node_step::AddNodeStep;
use crate::transform::step::{Step, StepResult};
use crate::transform::transform::{Transform, TransformError};
use crate::transform::{ConcreteStep, PatchStep};
use std::fmt::Debug;
#[async_trait]
pub trait Command: Send + Sync + Debug {
    async fn execute(&self, tr: &mut Transaction) -> Result<(), TransformError>;
    fn name(&self) -> String;
}
#[derive(Debug)]
pub struct Transaction {
    pub meta: HashMap<String, Arc<dyn std::any::Any>>,
    pub time: u64,
    pub steps: Vec<Arc<dyn Step>>,
    pub patches: Vec<Vec<Patch>>,
    pub doc: Arc<NodePool>,
    pub draft: Draft,
    pub schema: Arc<Schema>,
}
unsafe impl Send for Transaction {}
unsafe impl Sync for Transaction {}

impl Transform for Transaction {
    fn step(&mut self, step: Arc<dyn Step>) -> Result<(), TransformError> {
        let result = step.apply(&mut self.draft, self.schema.clone())?;
        match result.failed {
            Some(message) => Err(TransformError::new(message)),
            None => {
                self.add_step(step, result);
                Ok(())
            }
        }
    }
    fn doc_changed(&self) -> bool {
        !self.steps.is_empty()
    }
    /// 添加一个步骤 steps 和patches 一 一对应
    fn add_step(&mut self, step: Arc<dyn Step>, result: StepResult) {
        self.steps.push(step);
        self.patches.push(result.patches);
        self.doc = result.doc.unwrap();
    }
}
impl Transaction {
    pub async fn transaction(&mut self, call_back: Arc<dyn Command>) {
        let result = call_back.execute(self).await;
        if result.is_ok() {
            let (node_pool, patches) = self.draft.commit();
            self.add_step(
                Arc::new(PatchStep {
                    patches: patches.clone(),
                }),
                StepResult::ok(node_pool, patches),
            );
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
            patches: vec![],
        }
    }
    pub fn doc(&self) -> Arc<NodePool> {
        self.doc.clone()
    }
    pub fn as_concrete(step: &Arc<dyn Step>) -> ConcreteStep {
        step.to_concrete()
    }
    pub fn set_node_attribute(&mut self, id: String, values: im::HashMap<String, String>) {
        let _ = self.step(Arc::new(AttrStep::new(id, values)));
    }
    pub fn add_node(&mut self, parent_id: String, node: Node) {
        let _ = self.step(Arc::new(AddNodeStep::new(parent_id, node)));
    }
    pub fn set_time(&mut self, time: u64) -> &mut Self {
        self.time = time;
        self
    }

    pub fn set_meta<K>(&mut self, key: K, value: Arc<dyn std::any::Any>) -> &mut Self
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
