use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::node_pool::NodePool;
use crate::transform::transform::Transform;

use super::state::State;



pub struct Transaction {
    pub meta: HashMap<String, Box<dyn std::any::Any>>,
    pub time: u64,
    pub transform: Transform,
}
unsafe impl Send for Transaction {}
unsafe impl Sync for Transaction {}
impl Transaction {
    pub fn new(state: &State) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let node = state.doc();
        Transaction {
            transform: Transform::new(node, state.schema()),
            meta: HashMap::new(),
            time: now,
        }
    }
    pub fn doc(&self) -> Arc<NodePool> {
        self.transform.doc.clone()
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
