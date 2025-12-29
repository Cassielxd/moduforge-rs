use mf_model::{
    id_generator::IdGenerator,
    mark::Mark,
    node_pool::NodePool,
    schema::Schema,
    traits::{DataContainer, SchemaDefinition},
};
use std::fmt::{self, Debug};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Instant,
};
use mf_model::rpds::HashTrieMapSync;
use crate::plugin::PluginManagerGeneric;
use crate::{ops::GlobalResourceManager, resource::Resource};

use super::{
    error::{error, StateResult},
    plugin::{Plugin, PluginGeneric},
    transaction::{Transaction, TransactionGeneric},
};

static VERSION: AtomicU64 = AtomicU64::new(1);
pub fn get_state_version() -> u64 {
    //生成 全局自增的版本号，用于兼容性
    VERSION.fetch_add(1, Ordering::SeqCst)
}
/// State 结构体代表编辑器的整体状态 (泛型版本)
/// - 配置信息: 存储编辑器的配置信息
/// - 字段实例: 存储插件的状态数据
/// - 节点池: 文档的节点池
/// - 版本号: 状态版本号，用于追踪变更
#[derive(Clone)]
pub struct StateGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub config: Arc<ConfigurationGeneric<C, S>>,
    pub fields_instances: Arc<HashTrieMapSync<String, Arc<dyn Resource>>>,
    pub node_pool: Arc<C>,
    pub version: u64,
}

impl<C, S> Debug for StateGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(
            f,
            "State {{ 字段数量: {} }}",
            self.fields_instances.keys().len()
        )
    }
}

impl<C, S> StateGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 创建新的编辑器状态 (泛型版本，需要提供doc)
    /// - 初始化基础配置
    /// - 返回完整的编辑器状态实例
    pub fn new_generic(
        config: Arc<ConfigurationGeneric<C, S>>,
        doc: Arc<C>,
    ) -> StateResult<Self> {
        Ok(StateGeneric {
            fields_instances: Arc::new(HashTrieMapSync::new_sync()),
            config,
            node_pool: doc,
            version: get_state_version(),
        })
    }

    pub fn doc(&self) -> Arc<C> {
        Arc::clone(&self.node_pool)
    }

    /// 获取资源管理器
    pub fn resource_manager(&self) -> Arc<GlobalResourceManager> {
        Arc::clone(&self.config.resource_manager)
    }

    /// 获取结构定义
    pub fn schema(&self) -> Arc<S> {
        Arc::clone(&self.config.schema)
    }

    /// 获取插件列表
    pub async fn plugins(&self) -> Vec<Arc<PluginGeneric<C, S>>> {
        self.config.plugin_manager.get_sorted_plugins().await
    }

    /// 获取已排序的插件列表
    /// 按照优先级排序，优先级低的先执行
    pub async fn sorted_plugins(&self) -> Vec<Arc<PluginGeneric<C, S>>> {
        // 由于在 Configuration::new 中已经排序，这里直接返回即可
        self.config.plugin_manager.get_sorted_plugins().await
    }

    /// 获取字段值
    pub fn get_field(
        &self,
        key: &str,
    ) -> Option<Arc<dyn Resource>> {
        self.fields_instances.get(key).cloned()
    }

    /// 获取强类型字段值
    pub fn get<T: Resource>(
        &self,
        name: &str,
    ) -> Option<Arc<T>> {
        self.fields_instances
            .get(name)
            .cloned()
            .and_then(|state| state.downcast_arc::<T>().cloned())
    }

    /// 检查字段是否存在
    pub fn has_field(
        &self,
        name: &str,
    ) -> bool {
        self.fields_instances.contains_key(name)
    }

    /// 创建新的事务 (泛型版本)
    #[must_use]
    pub fn tr_generic(&self) -> TransactionGeneric<C, S> {
        TransactionGeneric::new_generic(self.node_pool.clone(), self.schema())
    }

    /// 重新配置状态 (泛型版本)
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, state_config), fields(
        crate_name = "state",
        current_version = self.version,
        has_plugins = state_config.plugins.is_some()
    )))]
    pub async fn reconfigure_generic(
        &self,
        state_config: StateConfigGeneric<C, S>,
    ) -> StateResult<Arc<StateGeneric<C, S>>> {
        tracing::info!("正在重新配置状态");
        let config = ConfigurationGeneric::new(
            self.schema(),
            state_config.plugins.clone(),
            state_config.doc.clone(),
            state_config.resource_manager.clone(),
        )
        .await?;
        let mut instance = Self::new_generic(Arc::new(config), self.node_pool.clone())?;
        let mut field_values = Vec::new();
        let mut fields_instances = HashTrieMapSync::new_sync();
        for plugin in instance.config.plugin_manager.get_sorted_plugins().await {
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
            fields_instances.insert_mut(name, value);
        }
        instance.fields_instances = Arc::new(fields_instances);
        tracing::info!("状态重新配置完成");
        Ok(Arc::new(instance))
    }

    /// 异步应用事务到当前状态 (泛型版本)
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        crate_name = "state",
        tr_id = %transaction.id,
        step_count = transaction.steps.len(),
        version = self.version
    )))]
    pub async fn apply_generic(
        self: &Arc<Self>,
        transaction: TransactionGeneric<C, S>,
    ) -> StateResult<TransactionResultGeneric<C, S>> {
        let start_time = Instant::now();
        let initial_step_count = transaction.steps.len();
        tracing::info!("开始应用事务，初始步骤数: {}", initial_step_count);
        // 应用事务并获取结果
        let result = self.apply_transaction_generic(Arc::new(transaction)).await?;
        // 检查是否需要重新应用事务
        let duration = start_time.elapsed();
        tracing::debug!("事务应用成功，步骤数保持不变，耗时: {:?}", duration);
        Ok(result)
    }

    /// 过滤事务 (泛型版本)
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, tr), fields(
        crate_name = "state",
        tr_id = %tr.id,
        ignore_plugin = ?ignore
    )))]
    pub async fn filter_transaction_generic(
        self: &Arc<Self>,
        tr: &TransactionGeneric<C, S>,
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

    /// 异步应用事务到当前状态 (泛型版本)
    /// 返回新的状态实例和应用事务的步骤
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, root_tr), fields(
        crate_name = "state",
        tr_id = %root_tr.id,
        step_count = root_tr.steps.len()
    )))]
    pub async fn apply_transaction_generic(
        self: &Arc<Self>,
        root_tr: Arc<TransactionGeneric<C, S>>,
    ) -> StateResult<TransactionResultGeneric<C, S>> {
        tracing::info!("开始应用事务");
        if !self.filter_transaction_generic(&root_tr, None).await? {
            tracing::debug!("事务被过滤，返回原始状态");
            return Ok(TransactionResultGeneric {
                state: self.clone(),
                transactions: vec![root_tr],
            });
        }

        let mut trs = Vec::new();
        let mut new_state: Arc<StateGeneric<C, S>> =
            self.apply_inner_generic(&root_tr).await?;
        trs.push(root_tr.clone());
        let mut seen: Option<Vec<SeenStateGeneric<C, S>>> = None;

        // 获取排序后的插件列表
        let sorted_plugins = self.sorted_plugins().await;

        loop {
            let mut have_new = false;
            for (i, plugin) in sorted_plugins.iter().enumerate() {
                let n: usize = seen.as_ref().map(|s| s[i].n).unwrap_or(0);
                if let Some(appended) = plugin
                    .append_transaction(self, &new_state, &trs[n..], n)
                    .await
                {
                    have_new = true;
                    if let Some(ref mut s) = seen {
                        s[i].n = trs.len();
                        s[i].state = new_state.clone();
                    } else {
                        let mut new_seen = Vec::new();
                        for _ in 0..sorted_plugins.len() {
                            new_seen.push(SeenStateGeneric {
                                state: self.clone(),
                                n: 0,
                            });
                        }
                        new_seen[i] = SeenStateGeneric {
                            state: new_state.clone(),
                            n: trs.len(),
                        };
                        seen = Some(new_seen);
                    }

                    if !self
                        .filter_transaction_generic(&appended, Some(i))
                        .await?
                    {
                        return Ok(TransactionResultGeneric {
                            state: self.clone(),
                            transactions: trs,
                        });
                    }

                    new_state = self.apply_inner_generic(&appended).await?;
                    trs.push(appended);
                }
            }

            if !have_new {
                tracing::info!("事务应用完成，共 {} 个步骤", trs.len());
                return Ok(TransactionResultGeneric {
                    state: new_state,
                    transactions: trs,
                });
            }
        }
    }

    /// 异步应用内部事务 (泛型版本)
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, tr), fields(
        crate_name = "state",
        tr_id = %tr.id,
        step_count = tr.steps.len(),
        current_version = self.version
    )))]
    pub async fn apply_inner_generic(
        self: &Arc<Self>,
        tr: &TransactionGeneric<C, S>,
    ) -> StateResult<Arc<StateGeneric<C, S>>> {
        let mut config = self.config.as_ref().clone();
        let new_doc = tr.doc();
        config.doc = Some(new_doc.clone());
        let mut new_instance = Self::new_generic(Arc::new(config), new_doc)?;
        let mut fields_instances = HashTrieMapSync::new_sync();
        // 获取已排序的插件列表
        let sorted_plugins = self.sorted_plugins().await;

        for plugin in sorted_plugins.iter() {
            if let Some(field) = &plugin.spec.state_field {
                if let Some(old_plugin_state) = self.get_field(&plugin.key) {
                    let value = field
                        .apply_erased(tr, old_plugin_state, self, &new_instance)
                        .await;
                    fields_instances.insert_mut(plugin.key.clone(), value);
                }
            }
        }
        new_instance.fields_instances = Arc::new(fields_instances);
        Ok(Arc::new(new_instance))
    }

    /// 序列化状态 (泛型版本)
    /// 需要容器类型 C 实现 Serialize
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self), fields(
        crate_name = "state",
        version = self.version,
        doc_size = self.node_pool.size()
    )))]
    pub async fn serialize_generic(&self) -> StateResult<StateSerializeGeneric<C>>
    where
        C: serde::Serialize,
    {
        let mut state_fields: HashMap<String, Vec<u8>> = HashMap::new();
        for plugin in self.plugins().await {
            if let Some(state_field) = &plugin.spec.state_field {
                if let Some(value) = self.get_field(&plugin.key) {
                    if let Some(json) = state_field.serialize_erased(&value) {
                        state_fields.insert(plugin.key.clone(), json);
                    }
                };
            }
        }
        let container_str =
            serde_json::to_string(&self.doc()).map_err(|e| {
                error::serialize_error(format!("容器序列化失败: {e}"))
            })?;
        Ok(StateSerializeGeneric {
            state_fields,
            container: container_str.as_bytes().to_vec(),
            _phantom: std::marker::PhantomData,
        })
    }

    /// 反序列化状态 (泛型版本)
    /// 需要容器类型 C 实现 DeserializeOwned
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(s, configuration), fields(
        crate_name = "state",
        state_fields_size = s.state_fields.len(),
        container_size = s.container.len()
    )))]
    pub async fn deserialize_generic(
        s: &StateSerializeGeneric<C>,
        configuration: &ConfigurationGeneric<C, S>,
    ) -> StateResult<StateGeneric<C, S>>
    where
        C: serde::de::DeserializeOwned,
    {
        let container: Arc<C> = serde_json::from_slice(&s.container)
            .map_err(|e| {
                error::deserialize_error(format!("容器反序列化失败: {e}"))
            })?;
        let mut config = configuration.clone();
        config.doc = Some(container.clone());
        let mut state = Self::new_generic(Arc::new(config), container)?;

        let mut map_instances = HashTrieMapSync::new_sync();
        for plugin in configuration.plugin_manager.get_sorted_plugins().await {
            if let Some(state_field) = &plugin.spec.state_field {
                if let Some(value) = s.state_fields.get(&plugin.key) {
                    if let Some(p_state) = state_field.deserialize_erased(value)
                    {
                        let key = plugin.key.clone();
                        map_instances.insert_mut(key, p_state);
                    }
                }
            }
        }
        state.fields_instances = Arc::new(map_instances);
        Ok(state)
    }
}

// ========================================
// NodePool 特化实现
// ========================================

/// 默认的 State 实现（NodePool + Schema）
pub type State = StateGeneric<NodePool, Schema>;

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
        let mut fields_instances = HashTrieMapSync::new_sync();
        for plugin in instance.config.plugin_manager.get_sorted_plugins().await
        {
            if let Some(field) = &plugin.spec.state_field {
                tracing::debug!("正在初始化插件状态: {}", plugin.key);
                let value = field.init_erased(&state_config, &instance).await;
                field_values.push((plugin.key.clone(), value));
            }
        }
        for (name, value) in field_values {
            fields_instances.insert_mut(name, value);
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
            fields_instances: Arc::new(HashTrieMapSync::new_sync()),
            config,
            node_pool: doc,
            version: get_state_version(),
        })
    }

    #[must_use]
    pub fn tr(&self) -> Transaction {
        self.tr_generic()
    }

    /// 异步应用事务到当前状态（便捷方法）
    /// 委托给 apply_generic 实现
    pub async fn apply(
        self: &Arc<Self>,
        transaction: Transaction,
    ) -> StateResult<TransactionResult> {
        self.apply_generic(transaction).await
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
        self.reconfigure_generic(state_config).await.map(|arc| (*arc).clone())
    }

    /// 序列化状态
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self), fields(
        crate_name = "state",
        version = self.version,
        doc_size = self.node_pool.size()
    )))]
    pub async fn serialize(&self) -> StateResult<StateSerialize> {
        let generic_result = self.serialize_generic().await?;
        Ok(StateSerialize {
            state_fields: generic_result.state_fields,
            node_pool: generic_result.container,
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
        let generic_s = StateSerializeGeneric {
            state_fields: s.state_fields.clone(),
            container: s.node_pool.clone(),
            _phantom: std::marker::PhantomData,
        };
        Self::deserialize_generic(&generic_s, configuration).await
    }
}

/// 泛型的序列化结构
pub struct StateSerializeGeneric<C>
where
    C: DataContainer + 'static,
{
    pub state_fields: HashMap<String, Vec<u8>>,
    pub container: Vec<u8>,
    _phantom: std::marker::PhantomData<C>,
}

pub struct StateSerialize {
    pub state_fields: HashMap<String, Vec<u8>>,
    pub node_pool: Vec<u8>,
}

/// 状态配置结构体，用于初始化编辑器状态 (泛型版本)
/// - 结构定义: 文档结构定义
/// - 文档内容: 初始文档内容
/// - 存储标记: 存储的标记
/// - 插件列表: 插件列表
#[derive(Debug)]
pub struct StateConfigGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub schema: Option<Arc<S>>,
    pub doc: Option<Arc<C>>,
    pub stored_marks: Option<Vec<Mark>>,
    pub plugins: Option<Vec<Arc<PluginGeneric<C, S>>>>,
    pub resource_manager: Option<Arc<GlobalResourceManager>>,
}

pub struct SeenStateGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    state: Arc<StateGeneric<C, S>>,
    n: usize,
}

pub struct SeenState {
    state: Arc<State>,
    n: usize,
}

#[derive(Debug, Clone)]
pub struct TransactionResultGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub state: Arc<StateGeneric<C, S>>,
    pub transactions: Vec<Arc<TransactionGeneric<C, S>>>,
}

// ========================================
// 向后兼容的类型别名
// ========================================

/// 默认的 StateConfig 实现（NodePool + Schema）
pub type StateConfig = StateConfigGeneric<NodePool, Schema>;

/// 默认的 TransactionResult 实现（NodePool + Schema）
pub type TransactionResult = TransactionResultGeneric<NodePool, Schema>;
/// 配置结构体，存储编辑器的核心配置信息 (泛型版本)
/// - 插件列表: 已加载的插件列表
/// - 插件索引: 插件索引，用于快速查找
/// - 文档实例: 文档实例
/// - 结构定义: 文档结构定义
#[derive(Clone, Debug)]
pub struct ConfigurationGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub plugin_manager: PluginManagerGeneric<C, S>,
    pub doc: Option<Arc<C>>,
    pub schema: Arc<S>,
    pub resource_manager: Arc<GlobalResourceManager>,
}

impl<C, S> ConfigurationGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(schema, plugins, doc, resource_manager), fields(
        crate_name = "state",
        plugin_count = plugins.as_ref().map(|p| p.len()).unwrap_or(0),
        has_doc = doc.is_some()
    )))]
    pub async fn new(
        schema: Arc<S>,
        plugins: Option<Vec<Arc<PluginGeneric<C, S>>>>,
        doc: Option<Arc<C>>,
        resource_manager: Option<Arc<GlobalResourceManager>>,
    ) -> StateResult<Self> {
        // 使用 Builder 模式构建插件管理器
        let plugin_manager = if let Some(plugin_list) = plugins {
            use crate::plugin::PluginManagerBuilderGeneric;

            let mut builder = PluginManagerBuilderGeneric::new();
            for plugin in plugin_list {
                builder.register_plugin(plugin)?;
            }
            builder.build()?
        } else {
            PluginManagerGeneric::new()
        };

        Ok(ConfigurationGeneric {
            doc,
            plugin_manager,
            schema,
            resource_manager: resource_manager
                .unwrap_or_else(|| Arc::new(GlobalResourceManager::default())),
        })
    }
}

// ========================================
// 向后兼容的类型别名
// ========================================

/// 默认的 Configuration 实现（NodePool + Schema）
pub type Configuration = ConfigurationGeneric<NodePool, Schema>;
