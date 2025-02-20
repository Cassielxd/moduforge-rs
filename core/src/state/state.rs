use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use crate::model::{id_generator::IdGenerator, mark::Mark, node_pool::NodePool, schema::Schema};

use super::{
    plugin::{Plugin, PluginState},
    transaction::Transaction,
};

static VERSION: AtomicU64 = AtomicU64::new(0);
pub fn get_state_version() -> u64 {
    //生成 全局自增的版本号，用于兼容性
    VERSION.fetch_add(1, Ordering::SeqCst)
}

#[derive(Clone, Debug)]
pub struct State {
    pub config: Configuration,
    pub fields_instances: HashMap<String, PluginState>,
    pub node_pool: Arc<NodePool>,
    pub version: u64,
}

impl State {
    pub async fn create(state_config: StateConfig) -> Result<State, Box<dyn std::error::Error>> {
        let schema = match &state_config.schema {
            Some(schema) => schema.clone(),
            None => state_config.schema.clone().ok_or("Schema is required")?,
        };
        let config = Configuration::new(
            schema,
            state_config.plugins.clone(),
            state_config.doc.clone(),
        );
        let mut instance = State::new(config);
        let mut field_values = Vec::new();
        for plugin in &instance.config.plugins {
            if let Some(field) = &plugin.spec.state {
                let value = field.init(&state_config, Some(&instance)).await;
                field_values.push((plugin.key.clone(), value));
            }
        }
        for (name, value) in field_values {
            instance.set_field(&name, value)?;
        }
        Ok(instance)
    }

    pub fn new(config: Configuration) -> Self {
        let doc: Arc<NodePool> = match &config.doc {
            Some(doc) => doc.clone(),
            None => {
                let id = IdGenerator::get_id();
                let nodes = config
                    .schema
                    .top_node_type
                    .clone()
                    .unwrap()
                    .create_and_fill(Some(id.clone()), None, vec![], None, &config.schema);
                NodePool::from(nodes, id).into()
            }
        };

        State {
            fields_instances: HashMap::new(),
            config,
            node_pool: doc,
            version: get_state_version(), //版本好全局自增
        }
    }
    pub fn doc(&self) -> Arc<NodePool> {
        self.node_pool.clone()
    }

    pub fn schema(&self) -> Arc<Schema> {
        self.config.schema.clone()
    }

    pub fn plugins(&self) -> &Vec<Arc<Plugin>> {
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
        if !self.filter_transaction(root_tr, None).await? {
            return Ok(TransactionResult {
                state: self.clone(),
                transactions: vec![],
            });
        }

        let mut trs = Vec::new();
        trs.push(1);
        let mut new_state = self.apply_inner(root_tr).await?;
        let mut seen: Option<Vec<SeenState>> = None;

        loop {
            let mut have_new = false;

            for (i, plugin) in self.config.plugins.iter().enumerate() {
                let n: usize = seen.as_ref().map(|s| s[i].n).unwrap_or(0);
                let old_state = seen.as_ref().map(|s| &s[i].state).unwrap_or(self);
                if n < trs.len() {
                    if let Some(trappend) = &plugin.spec.append_transaction {
                        if let Some(tr) = trappend
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
                                new_state = new_state.apply_inner(tr).await?;
                                trs.push(1);
                                have_new = true;
                            }
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
        new_instance.node_pool = tr.doc.clone();
        for plugin in &self.config.plugins {
            if let Some(field) = &plugin.spec.state {
                //如果有插件的情况下是一定存在的
                let old_plugin_state = self.get_field(&plugin.key).expect("不存在");
                let value = field.apply(tr, old_plugin_state, self, &new_instance).await;
                new_instance.set_field(&plugin.key, value)?;
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
        let config = Configuration::new(
            self.schema(),
            state_config.plugins.clone(),
            state_config.doc.clone(),
        );
        let mut instance = State::new(config);
        let mut field_values = Vec::new();
        for plugin in &instance.config.plugins {
            if let Some(field) = &plugin.spec.state {
                let key = plugin.key.clone();
                let value = if self.has_field(&key) {
                    self.get_field(&key).unwrap()
                } else {
                    field.init(&state_config, Some(&instance)).await
                };
                field_values.push((key.clone(), value));
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
    pub plugins: Option<Vec<Arc<Plugin>>>,
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
    plugins: Vec<Arc<Plugin>>,
    plugins_by_key: HashMap<String, Arc<Plugin>>,
    pub doc: Option<Arc<NodePool>>,
    schema: Arc<Schema>,
}

impl Configuration {
    pub fn new(
        schema: Arc<Schema>,
        plugins: Option<Vec<Arc<Plugin>>>,
        doc: Option<Arc<NodePool>>,
    ) -> Self {
        let mut config = Configuration {
            doc,
            plugins: Vec::new(),
            plugins_by_key: HashMap::new(),
            schema,
        };

        if let Some(plugin_list) = plugins {
            for plugin in plugin_list {
                let key = plugin.key.clone();
                if config.plugins_by_key.contains_key(&key) {
                    panic!("插件请不要重复添加 ({})", key);
                }
                config.plugins.push(plugin.clone());
                config.plugins_by_key.insert(key, plugin);
            }
        }
        config
    }
}
