use async_trait::async_trait;
use im::Vector;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::Join;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::model::node::Node;

use super::state::{State, StateConfig};
use super::transaction::Transaction;


static mut KEYS: Option<HashMap<String, i32>> = None;

/// Generates a unique plugin key
fn create_key(name: &str) -> String {
    unsafe {
        if KEYS.is_none() {
            KEYS = Some(HashMap::new());
        }

        if let Some(keys) = &mut KEYS {
            if keys.contains_key(name) {
                panic!("Plugin name ({}) is duplicate, not allowed", name);
            }
            keys.insert(name.to_string(), 0);
        }
    }
    format!("{}$", name)
}

#[async_trait]
pub trait Plugin:Send + Sync+Debug  {
    fn key(&self) -> &PluginKey;

    async fn init(&self, config: &StateConfig, instance: Option<&State>) -> PluginState{
        return PluginState::new(InnerPluginState::JSON(json!({})));
    }
    async fn apply(
        &self,
        tr: &Transaction
    ) -> PluginState{
        return PluginState::new(InnerPluginState::JSON(json!({})));
    }

    async fn filter_transaction(&self, _tr: &Transaction, _state: &State) -> bool{
        false
    }
     async fn append_transaction<'a>(
        &self,
        _trs: &'a mut Transaction,
        _old_state: &State,
        _new_state: &State,
    ) -> Option<&'a mut Transaction>{
        None
    }
} 
pub trait PluginStateTrait: Any + Serialize + for<'de> Deserialize<'de> {}

impl<T: Any + Serialize + for<'de> Deserialize<'de>> PluginStateTrait for T {}

pub type PluginState = Arc<InnerPluginState>;


use std::fmt::{Debug};


#[derive(Debug,Deserialize,Serialize,Clone,PartialEq)]
pub enum  InnerPluginState{
    MAP(im::HashMap<String, Node>),
    NODES(im::Vector<Node>),
    JSON(serde_json::Value)
} 


/// Plugin key instance
#[derive(Clone, Debug,Deserialize,Serialize)]
pub struct PluginKey {
    // Unique identifier
    pub key: String,
    // Plugin description
    pub desc: String,
}

impl PluginKey {
    pub fn new(name: Option<&str>, desc: Option<&str>) -> Self {
        let key = create_key(name.unwrap_or("key"));
        let desc = desc.unwrap_or("").to_string();
        PluginKey { key, desc }
    }
}
