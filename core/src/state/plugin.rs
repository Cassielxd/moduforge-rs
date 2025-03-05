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
pub trait PluginTrait: Send + Sync + Debug {
    async fn append_transaction<'a>(
        &self,
        tr: &'a mut Transaction,
        old_state: &State,
        new_state: &State,
    ) -> Option<&'a mut Transaction>;
    async fn filter_transaction(&self, tr: &Transaction, state: &State) -> bool;
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

    fn serialize(&self, _value: PluginState) -> Option<Vec<u8>> {
        None
    }

    fn deserialize(&self, _value: &Vec<u8>) -> Option<PluginState> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct PluginSpec {
    pub state: Option<Arc<dyn StateField>>,
    pub key: PluginKey,
    pub transaction: Option<Arc<dyn PluginTrait>>,
}
impl PluginSpec {
    async fn filter_transaction(&self, tr: &Transaction, state: &State) -> bool {
        if let Some(filter) = self.transaction.clone() {
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
        if let Some(transaction) = self.transaction.clone() {
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
        let key = spec.key.0.clone();

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

pub type PluginState = Arc<dyn std::any::Any + Send + Sync>;

use std::fmt::Debug;

/// Plugin key instance
pub type PluginKey = (String, String);
