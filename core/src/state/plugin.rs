use crate::model::node::Node;
use async_trait::async_trait;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;

use super::state::{State, StateConfig};
use super::transaction::Transaction;

/// Generates a unique plugin key
fn create_key(name: &str) -> String {
    format!("{}$", name)
}
pub trait Reset: Send + Sync + Debug {
    fn reset(&self) -> Self;
}

#[async_trait]
pub trait PluginTrFilterTrait: Send + Sync + Debug {
    async fn filter_transaction(&self, tr: &Transaction, state: &State) -> bool;
}
#[async_trait]
pub trait PluginTrTrait: Send + Sync + Debug {
    async fn append_transaction<'a>(
        &self,
        tr: &'a mut Transaction,
        old_state: &State,
        new_state: &State,
    ) -> Option<&'a mut Transaction>;
}

#[async_trait]
pub trait StateField: Send + Sync + Debug {
    async fn init(&self, config: &StateConfig, instance: Option<&State>) -> PluginState;

    async fn apply(
        &self,
        tr: &Transaction,
        value: PluginState,
        old_state: &State,
        new_state: &State,
    ) -> PluginState;

    fn to_json(&self, _value: &PluginState) -> Option<serde_json::Value> {
        None
    }

    fn from_json(
        &self,
        _config: &StateConfig,
        _value: &serde_json::Value,
        _state: &State,
    ) -> Option<PluginState> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct PluginSpec {
    pub state: Option<Arc<dyn StateField>>,
    pub key: Option<PluginKey>,
    pub filter_transaction: Option<Arc<dyn PluginTrFilterTrait>>,
    pub append_transaction: Option<Arc<dyn PluginTrTrait>>,
}
impl PluginSpec {
    async fn filter_transaction(&self, tr: &Transaction, state: &State) -> bool {
        if let Some(filter) = self.filter_transaction.clone() {
            return filter.filter_transaction(tr, state).await;
        }
        false
    }
    async fn append_transaction<'a>(
        &self,
        trs: &'a mut Transaction,
        old_state: &State,
        new_state: &State,
    ) -> Option<&'a mut Transaction> {
        if let Some(transaction) = self.append_transaction.clone() {
            return transaction
                .append_transaction(trs, old_state, new_state)
                .await;
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct Plugin {
    pub spec: PluginSpec,
    pub key: String,
}

impl Plugin {
    pub fn new(spec: PluginSpec) -> Self {
        let key = match &spec.key {
            Some(plugin_key) => plugin_key.key.clone(),
            None => create_key("plugin"),
        };

        Plugin { spec, key }
    }

    /// Gets the plugin's state from the global state
    pub fn get_state(&self, state: &State) -> Option<PluginState> {
        state.get_field(&self.key)
    }
    pub async fn apply_filter_transaction(&self, tr: &Transaction, state: &State) -> bool {
        self.spec.filter_transaction(tr, state).await
    }

    /// Apply append transaction logic if available
    pub async fn apply_append_transaction<'a>(
        &self,
        trs: &'a mut Transaction,
        old_state: &State,
        new_state: &State,
    ) -> Option<&'a mut Transaction> {
        self.spec
            .append_transaction(trs, old_state, new_state)
            .await
    }
}
pub trait PluginStateTrait: Any + Serialize + for<'de> Deserialize<'de> {}

impl<T: Any + Serialize + for<'de> Deserialize<'de>> PluginStateTrait for T {}

pub type PluginState = Arc<InnerPluginState>;

use std::fmt::Debug;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Decode, Encode)]
pub enum InnerPluginState {
    MAP(#[bincode(with_serde)] im::HashMap<String, Node>),
    NODES(#[bincode(with_serde)] im::Vector<Node>),
    String(String),
}

/// Plugin key instance
#[derive(Clone, Debug, Deserialize, Serialize)]
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
