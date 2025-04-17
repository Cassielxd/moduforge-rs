use moduforge_model::{
    id_generator::IdGenerator, mark::Mark, node_pool::NodePool, schema::Schema,
};

use im::HashMap as ImHashMap;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
    time::Instant,
};

use crate::ops::GlobalResourceManager;

use super::{
    error::{StateError, StateResult},
    plugin::{Plugin, PluginState},
    transaction::Transaction,
};

static VERSION: AtomicU64 = AtomicU64::new(1);
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
        tracing::info!("正在创建新的state");
        let schema = match &state_config.schema {
            Some(schema) => schema.clone(),
            None => state_config.schema.clone().ok_or_else(|| {
                StateError::SchemaError("Schema is required".to_string())
            })?,
        };
        let config = Configuration::new(
            schema,
            state_config.plugins.clone(),
            state_config.doc.clone(),
            state_config.resource_manager.clone(),
        );
        let mut instance = State::new(Arc::new(config));
        let mut field_values = Vec::new();
        for plugin in &instance.config.plugins {
            if let Some(field) = &plugin.spec.state {
                tracing::debug!("正在初始化插件状态: {}", plugin.key);
                let value = field.init(&state_config, Some(&instance)).await;
                field_values.push((plugin.key.clone(), value));
            }
        }
        for (name, value) in field_values {
            instance.set_field(&name, value)?;
        }
        tracing::info!("state创建成功");
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
                let nodes = config
                    .schema
                    .top_node_type
                    .clone()
                    .unwrap()
                    .create_and_fill(
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
            version: get_state_version(),
        }
    }
    pub fn doc(&self) -> Arc<NodePool> {
        Arc::clone(&self.node_pool)
    }
    pub fn resource_manager(&self) -> Arc<RwLock<GlobalResourceManager>> {
        Arc::clone(&self.config.resource_manager)
    }
    pub fn schema(&self) -> Arc<Schema> {
        Arc::clone(&self.config.schema)
    }

    pub fn plugins(&self) -> &Vec<Arc<Plugin>> {
        &self.config.plugins
    }

    /// 获取已排序的插件列表
    /// 按照优先级排序，优先级低的先执行
    pub fn sorted_plugins(&self) -> &Vec<Arc<Plugin>> {
        // 由于在 Configuration::new 中已经排序，这里直接返回即可
        &self.config.plugins
    }

    /// 异步应用事务到当前状态
    pub async fn apply(
        &self,
        transaction: Transaction,
    ) -> StateResult<TransactionResult> {
        let start_time = Instant::now();
        let initial_step_count = transaction.steps.len();
        tracing::info!("开始应用事务，初始步骤数: {}", initial_step_count);
        // 应用事务并获取结果
        let result = self.apply_transaction(transaction).await?;
        // 检查是否需要重新应用事务
        let duration = start_time.elapsed();
        tracing::debug!("事务应用成功，步骤数保持不变，耗时: {:?}", duration);
        Ok(result)
    }

    pub async fn filter_transaction(
        &self,
        tr: &Transaction,
        ignore: Option<usize>,
    ) -> StateResult<bool> {
        // 获取已排序的插件列表
        let sorted_plugins = self.sorted_plugins();

        for (i, plugin) in sorted_plugins.iter().enumerate() {
            if Some(i) != ignore
                && !plugin.apply_filter_transaction(tr, self).await
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// 异步应用事务到当前状态
    pub async fn apply_transaction(
        &self,
        root_tr: Transaction,
    ) -> StateResult<TransactionResult> {
        tracing::info!("开始应用事务");
        if !self.filter_transaction(&root_tr, None).await? {
            tracing::debug!("事务被过滤，返回原始状态");
            return Ok(TransactionResult {
                state: self.clone(),
                transactions: vec![root_tr],
            });
        }

        let mut trs = Vec::new();
        let mut new_state: State = self.apply_inner(&root_tr).await?;
        trs.push(root_tr.clone());
        let mut seen: Option<Vec<SeenState>> = None;

        // 获取排序后的插件列表
        let sorted_plugins = self.sorted_plugins();

        loop {
            let mut have_new = false;
            for (i, plugin) in sorted_plugins.iter().enumerate() {
                let n: usize = seen.as_ref().map(|s| s[i].n).unwrap_or(0);
                let old_state =
                    seen.as_ref().map(|s| &s[i].state).unwrap_or(self);
                if n < trs.len() {
                    if let Some(mut tr) = plugin
                        .apply_append_transaction(
                            &trs[n..],
                            old_state,
                            &new_state,
                        )
                        .await
                    {
                        if new_state.filter_transaction(&tr, Some(i)).await? {
                            tr.set_meta("appendedTransaction", root_tr.clone());
                            if seen.is_none() {
                                let mut s: Vec<SeenState> = Vec::new();
                                for j in 0..sorted_plugins.len() {
                                    s.push(if j < i {
                                        SeenState {
                                            state: new_state.clone(),
                                            n: trs.len(),
                                        }
                                    } else {
                                        SeenState { state: self.clone(), n: 0 }
                                    });
                                }
                                seen = Some(s);
                            }
                            tracing::debug!(
                                "插件 {} 添加了新事务",
                                plugin.spec.key.1
                            );
                            new_state = new_state.apply_inner(&tr).await?;
                            trs.push(tr);
                            have_new = true;
                        }
                    }
                }
                if let Some(seen) = &mut seen {
                    seen[i] =
                        SeenState { state: new_state.clone(), n: trs.len() };
                }
            }

            if !have_new {
                tracing::info!("事务应用完成，共 {} 个步骤", trs.len());
                return Ok(TransactionResult {
                    state: new_state,
                    transactions: trs,
                });
            }
        }
    }

    /// 异步应用内部事务
    pub async fn apply_inner(
        &self,
        tr: &Transaction,
    ) -> StateResult<State> {
        let mut config = self.config.as_ref().clone();
        config.doc = Some(tr.doc.clone());
        let mut new_instance = State::new(Arc::new(config));

        // 获取已排序的插件列表
        let sorted_plugins = self.sorted_plugins();

        for plugin in sorted_plugins.iter() {
            if let Some(field) = &plugin.spec.state {
                if let Some(old_plugin_state) = self.get_field(&plugin.key) {
                    let value = field
                        .apply(tr, old_plugin_state, self, &new_instance)
                        .await;
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
        tracing::info!("正在重新配置状态");
        let config = Configuration::new(
            self.schema(),
            state_config.plugins.clone(),
            state_config.doc.clone(),
            state_config.resource_manager.clone(),
        );
        let mut instance = State::new(Arc::new(config));
        let mut field_values = Vec::new();
        for plugin in &instance.config.plugins {
            if let Some(field) = &plugin.spec.state {
                let key = plugin.key.clone();
                tracing::debug!("正在重新配置插件: {}", key);
                let value = if self.has_field(&key) {
                    if let Some(old_plugin_state) = self.get_field(&key) {
                        old_plugin_state
                    } else {
                        field.init(&state_config, Some(&instance)).await
                    }
                } else {
                    field.init(&state_config, Some(&instance)).await
                };
                field_values.push((key, value));
            }
        }
        for (name, value) in field_values {
            instance.set_field(&name, value)?;
        }
        tracing::info!("状态重新配置完成");
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
    pub resource_manager: Option<Arc<RwLock<GlobalResourceManager>>>,
}

pub struct SeenState {
    state: State,
    n: usize,
}
#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub state: State,
    pub transactions: Vec<Transaction>,
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
    pub resource_manager: Arc<RwLock<GlobalResourceManager>>,
}

impl Configuration {
    pub fn new(
        schema: Arc<Schema>,
        plugins: Option<Vec<Arc<Plugin>>>,
        doc: Option<Arc<NodePool>>,
        resource_manager: Option<Arc<RwLock<GlobalResourceManager>>>,
    ) -> Self {
        let mut config = Configuration {
            doc,
            plugins: Vec::new(),
            plugins_by_key: HashMap::new(),
            schema,
            resource_manager: resource_manager.unwrap_or_else(|| {
                Arc::new(RwLock::new(GlobalResourceManager::default()))
            }),
        };

        if let Some(plugin_list) = plugins {
            // 按照优先级排序插件
            let mut sorted_plugins = plugin_list;
            sorted_plugins
                .sort_by(|a, b| a.spec.priority.cmp(&b.spec.priority));

            for plugin in sorted_plugins {
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
