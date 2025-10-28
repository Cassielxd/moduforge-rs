use mf_model::{
    id_generator::IdGenerator, mark::Mark, node_pool::NodePool, schema::Schema,
};
use imbl::HashMap as ImHashMap;
use std::fmt::{self, Debug};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Instant,
};

use crate::plugin::PluginManager;
use crate::{ops::GlobalResourceManager, resource::Resource};

use super::{
    error::{error, StateResult},
    plugin::{Plugin},
    transaction::Transaction,
};

static VERSION: AtomicU64 = AtomicU64::new(1);
pub fn get_state_version() -> u64 {
    //生成 全局自增的版本号，用于兼容性
    VERSION.fetch_add(1, Ordering::SeqCst)
}
/// State 结构体代表编辑器的整体状态
/// - 配置信息: 存储编辑器的配置信息
/// - 字段实例: 存储插件的状态数据
/// - 节点池: 文档的节点池
/// - 版本号: 状态版本号，用于追踪变更
#[derive(Clone)]
pub struct State {
    pub config: Arc<Configuration>,
    pub fields_instances: Arc<ImHashMap<String, Arc<dyn Resource>>>,
    pub node_pool: Arc<NodePool>,
    pub version: u64,
}
impl Debug for State {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "State {{ 字段数量: {} }}", self.fields_instances.len())
    }
}

impl State {
    /// 创建新的编辑器状态
    /// - 初始化基础配置
    /// - 初始化所有插件的状态
    /// - 返回完整的编辑器状态实例
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(state_config), fields(
        crate_name = "state",
        has_schema = state_config.schema.is_some(),
        has_doc = state_config.doc.is_some()
    )))]
    pub async fn create(state_config: StateConfig) -> StateResult<State> {
        tracing::info!("正在创建新的state");
        let schema: Arc<Schema> = match &state_config.schema {
            Some(schema) => schema.clone(),
            None => state_config.schema.clone().ok_or_else(|| {
                error::schema_error("必须提供结构定义".to_string())
            })?,
        };
        let config = Configuration::new(
            schema,
            state_config.plugins.clone(),
            state_config.doc.clone(),
            state_config.resource_manager.clone(),
        )
        .await?;
        let mut instance = State::new(Arc::new(config))?;
        let mut field_values = Vec::new();
        let mut fields_instances = ImHashMap::new();
        for plugin in instance.config.plugin_manager.get_sorted_plugins().await
        {
            if let Some(field) = &plugin.spec.state_field {
                tracing::debug!("正在初始化插件状态: {}", plugin.key);
                let value = field.init_erased(&state_config, &instance).await;
                field_values.push((plugin.key.clone(), value));
            }
        }
        for (name, value) in field_values {
            fields_instances.insert(name, value);
        }
        instance.fields_instances = Arc::new(fields_instances);
        tracing::info!("state创建成功");
        Ok(instance)
    }
    /// 根据配置创建新的状态实例
    /// - 如果没有提供文档，则创建一个空的顶层节点
    /// - 初始化基本状态信息
    pub fn new(config: Arc<Configuration>) -> StateResult<Self> {
        let doc: Arc<NodePool> = match &config.doc {
            Some(doc) => doc.clone(),
            None => {
                let id = IdGenerator::get_id();
                let factory = config.schema.factory();
                let nodes = factory.create_top_node(
                    Some(id.clone()),
                    None,
                    vec![],
                    None,
                )?;
                NodePool::from(nodes)
            },
        };

        Ok(State {
            fields_instances: Arc::new(ImHashMap::new()),
            config,
            node_pool: doc,
            version: get_state_version(),
        })
    }
    pub fn doc(&self) -> Arc<NodePool> {
        Arc::clone(&self.node_pool)
    }
    /// 获取资源管理器
    pub fn resource_manager(&self) -> Arc<GlobalResourceManager> {
        Arc::clone(&self.config.resource_manager)
    }
    /// 获取结构定义
    pub fn schema(&self) -> Arc<Schema> {
        Arc::clone(&self.config.schema)
    }
    /// 获取插件列表
    pub async fn plugins(&self) -> Vec<Arc<Plugin>> {
        self.config.plugin_manager.get_sorted_plugins().await
    }

    /// 获取已排序的插件列表
    /// 按照优先级排序，优先级低的先执行
    pub async fn sorted_plugins(&self) -> Vec<Arc<Plugin>> {
        // 由于在 Configuration::new 中已经排序，这里直接返回即可
        self.config.plugin_manager.get_sorted_plugins().await
    }

    /// 异步应用事务到当前状态
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        crate_name = "state",
        tr_id = %transaction.id,
        step_count = transaction.steps.len(),
        version = self.version
    )))]
    pub async fn apply(
        self: &Arc<Self>,
        transaction: Transaction,
    ) -> StateResult<TransactionResult> {
        let start_time = Instant::now();
        let initial_step_count = transaction.steps.len();
        tracing::info!("开始应用事务，初始步骤数: {}", initial_step_count);
        // 应用事务并获取结果
        let result = self.apply_transaction(Arc::new(transaction)).await?;
        // 检查是否需要重新应用事务
        let duration = start_time.elapsed();
        tracing::debug!("事务应用成功，步骤数保持不变，耗时: {:?}", duration);
        Ok(result)
    }

    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, tr), fields(
        crate_name = "state",
        tr_id = %tr.id,
        ignore_plugin = ?ignore
    )))]
    pub async fn filter_transaction(
        self: &Arc<Self>,
        tr: &Transaction,
        ignore: Option<usize>,
    ) -> StateResult<bool> {
        // 获取已排序的插件列表
        let sorted_plugins = self.sorted_plugins().await;

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
    /// 返回新的状态实例和应用事务的步骤
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, root_tr), fields(
        crate_name = "state",
        tr_id = %root_tr.id,
        step_count = root_tr.steps.len()
    )))]
    pub async fn apply_transaction(
        self: &Arc<Self>,
        root_tr: Arc<Transaction>,
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
        let mut new_state: Arc<State> = self.apply_inner(&root_tr).await?;
        trs.push(root_tr.clone());
        let mut seen: Option<Vec<SeenState>> = None;

        // 获取排序后的插件列表
        let sorted_plugins = self.sorted_plugins().await;

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
                        .await?
                    {
                        if new_state.filter_transaction(&tr, Some(i)).await? {
                            tr.set_meta("rootTr", root_tr.clone());
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
                                plugin.spec.tr.metadata().name.clone()
                            );
                            new_state = new_state.apply_inner(&tr).await?;
                            trs.push(Arc::new(tr));
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
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, tr), fields(
        crate_name = "state",
        tr_id = %tr.id,
        step_count = tr.steps.len(),
        current_version = self.version
    )))]
    pub async fn apply_inner(
        self: &Arc<Self>,
        tr: &Transaction,
    ) -> StateResult<Arc<State>> {
        let mut config = self.config.as_ref().clone();
        config.doc = Some(tr.doc());
        let mut new_instance = State::new(Arc::new(config))?;
        let mut fields_instances = ImHashMap::new();
        // 获取已排序的插件列表
        let sorted_plugins = self.sorted_plugins().await;

        for plugin in sorted_plugins.iter() {
            if let Some(field) = &plugin.spec.state_field {
                if let Some(old_plugin_state) = self.get_field(&plugin.key) {
                    let value = field
                        .apply_erased(tr, old_plugin_state, self, &new_instance)
                        .await;
                    fields_instances.insert(plugin.key.clone(), value);
                }
            }
        }
        new_instance.fields_instances = Arc::new(fields_instances);
        Ok(Arc::new(new_instance))
    }

    #[must_use]
    pub fn tr(&self) -> Transaction {
        Transaction::new(self)
    }

    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, state_config), fields(
        crate_name = "state",
        current_version = self.version,
        has_plugins = state_config.plugins.is_some()
    )))]
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
        )
        .await?;
        let mut instance = State::new(Arc::new(config))?;
        let mut field_values = Vec::new();
        let mut fields_instances = ImHashMap::new();
        for plugin in &instance.config.plugin_manager.get_sorted_plugins().await
        {
            if let Some(field) = &plugin.spec.state_field {
                let key = plugin.key.clone();
                tracing::debug!("正在重新配置插件: {}", key);
                let value = if self.has_field(&key) {
                    if let Some(old_plugin_state) = self.get_field(&key) {
                        old_plugin_state
                    } else {
                        field.init_erased(&state_config, &instance).await
                    }
                } else {
                    field.init_erased(&state_config, &instance).await
                };
                field_values.push((key, value));
            }
        }
        for (name, value) in field_values {
            fields_instances.insert(name, value);
        }
        instance.fields_instances = Arc::new(fields_instances);
        tracing::info!("状态重新配置完成");
        Ok(instance)
    }

    pub fn get_field(
        &self,
        name: &str,
    ) -> Option<Arc<dyn Resource>> {
        self.fields_instances.get(name).cloned()
    }
    pub fn get<T: Resource>(
        &self,
        name: &str,
    ) -> Option<Arc<T>> {
        self.fields_instances
            .get(name)
            .cloned()
            .and_then(|state| state.downcast_arc::<T>().cloned())
    }

    pub fn has_field(
        &self,
        name: &str,
    ) -> bool {
        self.fields_instances.contains_key(name)
    }
    /// 序列化状态
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self), fields(
        crate_name = "state",
        version = self.version,
        doc_size = self.node_pool.size()
    )))]
    pub async fn serialize(&self) -> StateResult<StateSerialize> {
        let mut state_fields: HashMap<String, Vec<u8>> = HashMap::new();
        for plugin in self.plugins().await {
            if let Some(state_field) = &plugin.spec.state_field {
                if let Some(value) = self.get_field(&plugin.key) {
                    if let Some(json) = state_field.serialize_erased(value) {
                        state_fields.insert(plugin.key.clone(), json);
                    }
                };
            }
        }
        let node_pool_str =
            serde_json::to_string(&self.doc()).map_err(|e| {
                error::serialize_error(format!("node pool 序列化失败: {e}"))
            })?;
        let state_fields_str =
            serde_json::to_string(&state_fields).map_err(|e| {
                error::serialize_error(format!("fields 序列化失败: {e}"))
            })?;
        Ok(StateSerialize {
            state_fields: state_fields_str.as_bytes().to_vec(),
            node_pool: node_pool_str.as_bytes().to_vec(),
        })
    }
    /// 反序列化状态
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(s, configuration), fields(
        crate_name = "state",
        state_fields_size = s.state_fields.len(),
        node_pool_size = s.node_pool.len()
    )))]
    pub async fn deserialize(
        s: &StateSerialize,
        configuration: &Configuration,
    ) -> StateResult<State> {
        let state_fields: HashMap<String, Vec<u8>> =
            serde_json::from_slice(&s.state_fields).map_err(|e| {
                error::deserialize_error(format!(
                    "state fields 反序列化失败{e}"
                ))
            })?;
        let node_pool: Arc<NodePool> = serde_json::from_slice(&s.node_pool)
            .map_err(|e| {
                error::deserialize_error(format!("node pool 反序列化失败: {e}"))
            })?;
        let mut config = configuration.clone();
        config.doc = Some(node_pool);
        let mut state = State::new(Arc::new(config))?;

        let mut map_instances = ImHashMap::new();
        for plugin in &configuration.plugin_manager.get_sorted_plugins().await {
            if let Some(state_field) = &plugin.spec.state_field {
                if let Some(value) = state_fields.get(&plugin.key) {
                    if let Some(p_state) = state_field.deserialize_erased(value)
                    {
                        let key = plugin.key.clone();
                        map_instances.insert(key, p_state);
                    }
                }
            }
        }
        state.fields_instances = Arc::new(map_instances);
        Ok(state)
    }
}

pub struct StateSerialize {
    pub state_fields: Vec<u8>,
    pub node_pool: Vec<u8>,
}

/// 状态配置结构体，用于初始化编辑器状态
/// - 结构定义: 文档结构定义
/// - 文档内容: 初始文档内容
/// - 存储标记: 存储的标记
/// - 插件列表: 插件列表
#[derive(Debug)]
pub struct StateConfig {
    pub schema: Option<Arc<Schema>>,
    pub doc: Option<Arc<NodePool>>,
    pub stored_marks: Option<Vec<Mark>>,
    pub plugins: Option<Vec<Arc<Plugin>>>,
    pub resource_manager: Option<Arc<GlobalResourceManager>>,
}

pub struct SeenState {
    state: Arc<State>,
    n: usize,
}
#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub state: Arc<State>,
    pub transactions: Vec<Arc<Transaction>>,
}
/// 配置结构体，存储编辑器的核心配置信息
/// - 插件列表: 已加载的插件列表
/// - 插件索引: 插件索引，用于快速查找
/// - 文档实例: 文档实例
/// - 结构定义: 文档结构定义
#[derive(Clone, Debug)]
pub struct Configuration {
    pub plugin_manager: PluginManager,
    pub doc: Option<Arc<NodePool>>,
    schema: Arc<Schema>,
    pub resource_manager: Arc<GlobalResourceManager>,
}

impl Configuration {
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(schema, plugins, doc, resource_manager), fields(
        crate_name = "state",
        plugin_count = plugins.as_ref().map(|p| p.len()).unwrap_or(0),
        has_doc = doc.is_some()
    )))]
    pub async fn new(
        schema: Arc<Schema>,
        plugins: Option<Vec<Arc<Plugin>>>,
        doc: Option<Arc<NodePool>>,
        resource_manager: Option<Arc<GlobalResourceManager>>,
    ) -> StateResult<Self> {
        // 使用 Builder 模式构建插件管理器
        let plugin_manager = if let Some(plugin_list) = plugins {
            use crate::plugin::PluginManagerBuilder;

            let mut builder = PluginManagerBuilder::new();
            for plugin in plugin_list {
                builder.register_plugin(plugin)?;
            }
            builder.build()?
        } else {
            PluginManager::new()
        };

        Ok(Configuration {
            doc,
            plugin_manager,
            schema,
            resource_manager: resource_manager
                .unwrap_or_else(|| Arc::new(GlobalResourceManager::default())),
        })
    }
}
