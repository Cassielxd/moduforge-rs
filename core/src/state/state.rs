use std::{any::Any, collections::HashMap, sync::Arc};

use crate::model::{mark::Mark, node_pool::NodePool, schema::Schema};

use super::{
    plugin::{Plugin, PluginState, StateField},
    transaction::Transaction,
};
use async_trait::async_trait;


#[derive(Clone, Debug)]
pub struct State {
    pub config: Configuration,
    pub fields_instances: HashMap<String, PluginState>,
}

impl State {
    pub async fn create(state_config: StateConfig) -> Result<State, Box<dyn std::error::Error>> {
        let schema = match &state_config.schema {
            Some(schema) => schema.clone(),
            None => state_config.schema.clone().ok_or("Schema is required")?,
        };
        let config = Configuration::new(schema, state_config.plugins.clone());
        let mut instance = State::new(config);
        let mut field_values = Vec::new();
        for field in &instance.config.fields {
            if let Some(value) = field.init(&state_config, Some(&instance)).await {
                field_values.push((field.name.clone(), value));
            }
        }
        for (name, value) in field_values {
            instance.set_field(&name, value)?;
        }
        Ok(instance)
    }

    pub fn new(config: Configuration) -> Self {
        State {
            fields_instances: HashMap::new(),
            config,
        }
    }
    pub fn doc(&self) -> Arc<NodePool> {
        match self.get_field("doc") {
            Some(doc) => {
                if let Ok(node) = Arc::downcast::<NodePool>(doc) {
                    node
                } else {
                    panic!("doc field is not of type NodePool")
                }
            }
            None => panic!("doc field not found"),
        }
    }
    pub fn schema(&self) -> Arc<Schema> {
        self.config.schema.clone()
    }

    pub fn plugins(&self) -> &Vec<Plugin> {
        &self.config.plugins
    }

    pub async fn apply(&self, tr: &mut Transaction) -> Result<State, Box<dyn std::error::Error>> {
        Ok(self.apply_transaction(tr).await?.state)
    }

    pub async fn filter_transaction(
        &self,
        tr: &Transaction,
        ignore: Option<usize>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        for (i, plugin) in self.config.plugins.iter().enumerate() {
            if Some(i) != ignore {
                if let Some(filter) = &plugin.spec.filter_transaction {
                    if !filter.filter_transaction(tr, self).await {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }

    pub async fn apply_transaction(
        &self,
        root_tr: &mut Transaction,
    ) -> Result<TransactionResult, Box<dyn std::error::Error>> {
        if !self.filter_transaction(&root_tr, None).await? {
            return Ok(TransactionResult {
                state: self.clone(),
                transactions: vec![],
            });
        }

        let mut trs = Vec::new();
        trs.push(1);
        let mut new_state = self.apply_inner(&root_tr).await?;
        let mut seen: Option<Vec<SeenState>> = None;

        loop {
            let mut have_new = false;

            for (i, plugin) in self.config.plugins.iter().enumerate() {
                if let Some(append) = &plugin.spec.append_transaction {
                    let n: usize = seen.as_ref().map(|s| s[i].n).unwrap_or(0);
                    let old_state = seen.as_ref().map(|s| &s[i].state).unwrap_or(self);
                    if n < trs.len() {
                        if let Some(tr) = append
                            .append_transaction(root_tr, old_state, &new_state)
                            .await
                        {
                            if new_state.filter_transaction(tr, Some(i)).await? {
                                if seen.is_none() {
                                    let mut s = Vec::new();
                                    for j in 0..self.config.plugins.len() {
                                        s.push(if j < i {
                                            SeenState {
                                                state: new_state.clone(),
                                                n: trs.len(),
                                            }
                                        } else {
                                            SeenState {
                                                state: self.clone(),
                                                n: 0,
                                            }
                                        });
                                    }
                                    seen = Some(s);
                                }
                                new_state = new_state.apply_inner(&tr).await?;
                                trs.push(1);
                                have_new = true;
                            }
                        }
                    }
                    if let Some(seen) = &mut seen {
                        seen[i] = SeenState {
                            state: new_state.clone(),
                            n: trs.len(),
                        };
                    }
                }
            }

            if !have_new {
                return Ok(TransactionResult {
                    state: new_state,
                    transactions: Vec::new(),
                });
            }
        }
    }

    pub async fn apply_inner(&self, tr: &Transaction) -> Result<State, Box<dyn std::error::Error>> {
        let mut new_instance = State::new(self.config.clone());

        for field in &self.config.fields {
            if let Some(value) = field.apply(tr).await {
                new_instance.set_field(&field.name, value)?;
            }
        }

        Ok(new_instance)
    }

    pub fn tr(&self) -> Transaction {
        Transaction::new(self)
    }

    pub async fn reconfigure(
        &self,
        state_config: StateConfig,
    ) -> Result<State, Box<dyn std::error::Error>> {
        let config = Configuration::new(self.schema(), state_config.plugins.clone());
        let mut instance = State::new(config);
        let mut field_values = Vec::new();
        for field in &instance.config.fields {
            let value = if self.has_field(&field.name) {
                self.get_field(&field.name)
            } else {
                field.init(&state_config, Some(&instance)).await
            };
            if let Some(value) = value {
                field_values.push((field.name.clone(), value));
            }
        }
        for (name, value) in field_values {
            instance.set_field(&name, value)?;
        }
        Ok(instance)
    }

    pub fn get_field(&self, name: &str) -> Option<PluginState> {
        self.fields_instances.get(name).cloned()
    }

    pub fn set_field(
        &mut self,
        name: &str,
        value: PluginState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.fields_instances.insert(name.to_owned(), value);
        Ok(())
    }

    pub fn has_field(&self, name: &str) -> bool {
        self.fields_instances.contains_key(name)
    }
}
pub struct StateConfig {
    pub schema: Option<Arc<Schema>>,
    pub doc: Option<Arc<NodePool>>,
    pub stored_marks: Option<Vec<Mark>>,
    pub plugins: Option<Vec<Plugin>>,
}

pub struct SeenState {
    state: State,
    n: usize,
}

pub struct TransactionResult {
    pub state: State,
    pub transactions: Vec<Transaction>,
}

#[derive(Clone, Debug)]
pub struct Configuration {
    fields: Vec<FieldDesc>,
    plugins: Vec<Plugin>,
    plugins_by_key: HashMap<String, Plugin>,
    schema: Arc<Schema>,
}

impl Configuration {
    pub fn new(schema: Arc<Schema>, plugins: Option<Vec<Plugin>>) -> Self {
        let mut config = Configuration {
            fields: base_fields(),
            plugins: Vec::new(),
            plugins_by_key: HashMap::new(),
            schema,
        };

        if let Some(plugin_list) = plugins {
            for plugin in plugin_list {
                let key = plugin.key.clone();
                if config.plugins_by_key.contains_key(&plugin.key) {
                    panic!("请不要重复添加 ({})", plugin.key);
                }
                config.plugins.push(plugin.clone());
                if let Some(state) = &plugin.spec.state {
                    config
                        .fields
                        .push(FieldDesc::new(state.clone(), key.clone()));
                }
                config.plugins_by_key.insert(key, plugin);
            }
        }

        config
    }
}
#[derive(Clone, Debug)]
pub struct FieldDesc {
    field: Arc<dyn StateField>,
    name: String,
}

impl FieldDesc {
    fn new(field: Arc<dyn StateField>, name: String) -> Self {
        FieldDesc { field, name }
    }

    async fn init(&self, config: &StateConfig, instance: Option<&State>) -> Option<PluginState> {
        Some(self.field.init(config, instance).await)
    }

    async fn apply(&self, tr: &Transaction) -> Option<PluginState> {
        Some(self.field.apply(tr, None, None, None).await)
    }
}
#[derive(Debug, Clone)]
struct DocField;

#[async_trait]
impl StateField for DocField {
    async fn init(&self, config: &StateConfig, _: Option<&State>) -> PluginState {
        match &config.doc {
            Some(doc) => doc.clone(),
            None => {
                let schema = config.schema.clone().unwrap();
                let top_node_type = schema
                    .top_node_type
                    .as_ref()
                    .expect("Top node type is required");

                let nodes = top_node_type.create_and_fill(None, None, vec![], None, &schema);
                return Arc::new(NodePool::from(nodes));
            } // 如果 Node 实现了 Default trait
        }
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Option<&PluginState>,
        old_state: Option<&State>,
        new_state: Option<&State>,
    ) -> PluginState {
        tr.doc()
    }

    fn to_json(&self, value: &PluginState) -> Option<serde_json::Value> {
        None
    }

    fn from_json(
        &self,
        config: &StateConfig,
        value: &serde_json::Value,
        state: &State,
    ) -> Option<PluginState> {
        None
    }
}

fn base_fields() -> Vec<FieldDesc> {
    vec![FieldDesc::new(Arc::new(DocField), "doc".to_string())]
}
