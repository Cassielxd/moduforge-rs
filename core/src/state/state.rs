use crate::model::{id_generator::IdGenerator, mark::Mark, node_pool::NodePool, schema::Schema};
use crate::error::{StateError, StateResult};
use im::HashMap as ImHashMap;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use super::{
    plugin::{Plugin, PluginState},
    transaction::Transaction,
};

static VERSION: AtomicU64 = AtomicU64::new(0);
pub fn get_state_version() -> u64 {
    //生成 全局自增的版本号，用于兼容性
    VERSION.fetch_add(1, Ordering::SeqCst)
}
/// State 结构体代表编辑器的整体状态
/// - config: 存储编辑器的配置信息
/// - fields_instances: 存储插件的状态数据
/// - node_pool: 文档的节点池
/// - version: 状态版本号，用于追踪变更
#[derive(Clone, Debug)]
pub struct State {
    pub config: Arc<Configuration>,
    pub fields_instances: ImHashMap<String, PluginState>,
    pub node_pool: Arc<NodePool>,
    pub version: u64,
}

impl State {
    /// 创建新的编辑器状态
    /// - 初始化基础配置
    /// - 初始化所有插件的状态
    /// - 返回完整的编辑器状态实例
    pub async fn create(state_config: StateConfig) -> StateResult<State> {
        let schema = match &state_config.schema {
            Some(schema) => schema.clone(),
            None => state_config.schema.clone().ok_or_else(|| StateError::SchemaError("Schema is required".to_string()))?,
        };
        let config = Configuration::new(schema, state_config.plugins.clone(), state_config.doc.clone());
        let mut instance = State::new(Arc::new(config));
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
    /// 根据配置创建新的状态实例
    /// - 如果没有提供文档，则创建一个空的顶层节点
    /// - 初始化基本状态信息
    pub fn new(config: Arc<Configuration>) -> Self {
        let doc: Arc<NodePool> = match &config.doc {
            Some(doc) => doc.clone(),
            None => {
                let id = IdGenerator::get_id();
                let nodes = config.schema.top_node_type.clone().unwrap().create_and_fill(
                    Some(id.clone()),
                    None,
                    vec![],
                    None,
                    &config.schema,
                );
                NodePool::from(nodes, id).into()
            },
        };

        State {
            fields_instances: ImHashMap::new(),
            config,
            node_pool: doc,
            version: get_state_version(), //版本好全局自增
        }
    }
    pub fn doc(&self) -> Arc<NodePool> {
        Arc::clone(&self.node_pool)
    }

    pub fn schema(&self) -> Arc<Schema> {
        Arc::clone(&self.config.schema)
    }

    pub fn plugins(&self) -> &Vec<Arc<Plugin>> {
        &self.config.plugins
    }

    /// 异步应用事务到当前状态
    pub async fn apply(
        &self,
        tr: &mut Transaction,
    ) -> StateResult<State> {
        self.before_apply_transaction(tr).await?;
        let result = self.apply_transaction(tr).await?;
        let befor = tr.steps.len();
        self.after_apply_transaction(&result.state, tr).await?;
        let after = tr.steps.len();
        match befor.cmp(&after) {
            std::cmp::Ordering::Equal => Ok(result.state),
            _ => {
                let new_state = result.state.apply_inner(tr).await?;
                Ok(new_state)
            },
        }
    }

    /// 事务应用前的处理器
    pub async fn before_apply_transaction(
        &self,
        tr: &mut Transaction,
    ) -> StateResult<()> {
        for plugin in &self.config.plugins {
            if let Err(e) = plugin.before_apply_transaction(tr, self).await {
                return Err(StateError::TransactionError(format!("Plugin {} before_apply_transaction failed: {}", plugin.key, e)));
            }
        }
        Ok(())
    }
    /// 事务应用后的处理器
    async fn after_apply_transaction(
        &self,
        new_state: &State,
        tr: &Transaction,
    ) -> StateResult<()> {
        for plugin in &self.config.plugins {
            if let Err(e) = plugin.after_apply_transaction(new_state, tr, self).await {
                return Err(StateError::TransactionError(format!("Plugin {} after_apply_transaction failed: {}", plugin.key, e)));
            }
        }
        Ok(())
    }

    pub async fn filter_transaction(
        &self,
        tr: &Transaction,
        ignore: Option<usize>,
    ) -> StateResult<bool> {
        for (i, plugin) in self.config.plugins.iter().enumerate() {
            if Some(i) != ignore && !plugin.apply_filter_transaction(tr, self).await {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// 异步应用事务到当前状态
    pub async fn apply_transaction(
        &self,
        root_tr: &mut Transaction,
    ) -> StateResult<TransactionResult> {
        if !self.filter_transaction(root_tr, None).await? {
            return Ok(TransactionResult { state: self.clone() });
        }

        let mut trs = Vec::new();
        trs.push(1);
        let mut new_state: State = self.apply_inner(root_tr).await?;
        let mut seen: Option<Vec<SeenState>> = None;

        loop {
            let mut have_new = false;
            for (i, plugin) in self.config.plugins.iter().enumerate() {
                let n: usize = seen.as_ref().map(|s| s[i].n).unwrap_or(0);
                let old_state = seen.as_ref().map(|s| &s[i].state).unwrap_or(self);
                if n < trs.len() {
                    if let Some(tr) = plugin.apply_append_transaction(root_tr, old_state, &new_state).await {
                        if new_state.filter_transaction(tr, Some(i)).await? {
                            if seen.is_none() {
                                let mut s = Vec::new();
                                for j in 0..self.config.plugins.len() {
                                    s.push(if j < i {
                                        SeenState { state: new_state.clone(), n: trs.len() }
                                    } else {
                                        SeenState { state: self.clone(), n: 0 }
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
                if let Some(seen) = &mut seen {
                    seen[i] = SeenState { state: new_state.clone(), n: trs.len() };
                }
            }

            if !have_new {
                return Ok(TransactionResult { state: new_state });
            }
        }
    }

    /// 异步应用内部事务
    pub async fn apply_inner(
        &self,
        tr: &Transaction,
    ) -> StateResult<State> {
        let mut new_instance = State::new(self.config.clone());
        new_instance.node_pool = tr.doc.clone();
        for plugin in &self.config.plugins {
            if let Some(field) = &plugin.spec.state {
                if let Some(old_plugin_state) = self.get_field(&plugin.key) {
                    let value = field.apply(tr, old_plugin_state, self, &new_instance).await;
                    new_instance.set_field(&plugin.key, value)?;
                }
            }
        }
        Ok(new_instance)
    }

    #[must_use]
    pub fn tr(&self) -> Transaction {
        Transaction::new(self)
    }

    pub async fn reconfigure(
        &self,
        state_config: StateConfig,
    ) -> StateResult<State> {
        let config = Configuration::new(self.schema(), state_config.plugins.clone(), state_config.doc.clone());
        let mut instance = State::new(Arc::new(config));
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

    pub fn get_field(
        &self,
        name: &str,
    ) -> Option<PluginState> {
        self.fields_instances.get(name).cloned()
    }

    pub fn set_field(
        &mut self,
        name: &str,
        value: PluginState,
    ) -> StateResult<()> {
        self.fields_instances.insert(name.to_owned(), value);
        Ok(())
    }

    pub fn has_field(
        &self,
        name: &str,
    ) -> bool {
        self.fields_instances.contains_key(name)
    }
}
/// 状态配置结构体，用于初始化编辑器状态
/// - schema: 文档结构定义
/// - doc: 初始文档内容
/// - stored_marks: 存储的标记
/// - plugins: 插件列表
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
}
/// 配置结构体，存储编辑器的核心配置信息
/// - plugins: 已加载的插件列表
/// /// - plugins_by_key: 插件索引，用于快速查找
/// - doc: 文档实例
/// - schema: 文档结构定义
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
        let mut config = Configuration { doc, plugins: Vec::new(), plugins_by_key: HashMap::new(), schema };

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
