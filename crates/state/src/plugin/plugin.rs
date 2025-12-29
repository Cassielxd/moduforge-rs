use async_trait::async_trait;
use std::sync::Arc;

use crate::error::StateResult;
use crate::plugin::{PluginConfig, PluginMetadata};
use crate::resource::Resource;

use crate::state::{StateGeneric, StateConfigGeneric};
use crate::transaction::TransactionGeneric;
use mf_model::traits::{DataContainer, SchemaDefinition};
use mf_model::node_pool::NodePool;
use mf_model::schema::Schema;

/// 插件特征 (泛型版本)
/// 定义插件的核心行为，包括事务处理和过滤功能
#[async_trait]
pub trait PluginTraitGeneric<C, S>: Send + Sync + Debug
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 获取插件元数据（静态信息）- 提供默认实现
    fn metadata(&self) -> PluginMetadata;

    /// 获取插件配置（静态配置）- 提供默认实现
    fn config(&self) -> PluginConfig {
        PluginConfig {
            enabled: true,
            priority: 0,
            settings: std::collections::HashMap::new(),
        }
    }

    /// 追加事务处理
    /// 允许插件在事务执行前修改或扩展事务内容
    async fn append_transaction(
        &self,
        _: &[Arc<TransactionGeneric<C, S>>],
        _: &Arc<StateGeneric<C, S>>,
        _: &Arc<StateGeneric<C, S>>,
    ) -> StateResult<Option<TransactionGeneric<C, S>>> {
        Ok(None)
    }

    /// 事务过滤
    /// 决定是否允许事务执行
    async fn filter_transaction(
        &self,
        _: &TransactionGeneric<C, S>,
        _: &StateGeneric<C, S>,
    ) -> bool {
        true
    }
}

/// 向后兼容的类型别名
pub trait PluginTrait: PluginTraitGeneric<NodePool, Schema> {}

/// 状态字段特征 (泛型版本)
/// 使用关联类型保持类型信息，提供类型安全的插件状态管理
#[async_trait]
pub trait StateFieldGeneric<C, S>: Send + Sync + Debug
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 状态值类型，必须实现 Resource trait
    type Value: Resource;

    /// 初始化插件状态
    async fn init(
        &self,
        config: &StateConfigGeneric<C, S>,
        instance: &StateGeneric<C, S>,
    ) -> Arc<Self::Value>;

    /// 应用状态变更
    /// 根据事务内容更新插件状态
    async fn apply(
        &self,
        tr: &TransactionGeneric<C, S>,
        value: Arc<Self::Value>,
        old_state: &StateGeneric<C, S>,
        new_state: &StateGeneric<C, S>,
    ) -> Arc<Self::Value>;

    /// 序列化插件状态（可选）
    fn serialize(
        &self,
        _value: &Arc<Self::Value>,
    ) -> Option<Vec<u8>> {
        None
    }

    /// 反序列化插件状态（可选）
    fn deserialize(
        &self,
        _data: &[u8],
    ) -> Option<Arc<Self::Value>> {
        None
    }
}

/// 类型擦除的 StateField trait (泛型版本)
/// 用于在 PluginSpec 中存储不同类型的 StateField
#[async_trait]
pub trait ErasedStateFieldGeneric<C, S>: Send + Sync + Debug
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 初始化插件状态
    async fn init_erased(
        &self,
        config: &StateConfigGeneric<C, S>,
        instance: &StateGeneric<C, S>,
    ) -> Arc<dyn Resource>;

    /// 应用状态变更
    async fn apply_erased(
        &self,
        tr: &TransactionGeneric<C, S>,
        value: Arc<dyn Resource>,
        old_state: &StateGeneric<C, S>,
        new_state: &StateGeneric<C, S>,
    ) -> Arc<dyn Resource>;

    /// 序列化插件状态（可选）
    fn serialize_erased(
        &self,
        value: &Arc<dyn Resource>,
    ) -> Option<Vec<u8>>;

    /// 反序列化插件状态（可选）
    fn deserialize_erased(
        &self,
        data: &[u8],
    ) -> Option<Arc<dyn Resource>>;
}

/// Blanket implementation: 任何实现 StateFieldGeneric 的类型自动实现 ErasedStateFieldGeneric
#[async_trait]
impl<C, S, T> ErasedStateFieldGeneric<C, S> for T
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
    T: StateFieldGeneric<C, S> + Send + Sync + 'static,
{
    async fn init_erased(
        &self,
        config: &StateConfigGeneric<C, S>,
        instance: &StateGeneric<C, S>,
    ) -> Arc<dyn Resource> {
        self.init(config, instance).await
    }

    async fn apply_erased(
        &self,
        tr: &TransactionGeneric<C, S>,
        value: Arc<dyn Resource>,
        old_state: &StateGeneric<C, S>,
        new_state: &StateGeneric<C, S>,
    ) -> Arc<dyn Resource> {
        if let Some(typed_value) = value.downcast_arc::<T::Value>() {
            self.apply(tr, typed_value.clone(), old_state, new_state).await
        } else {
            value
        }
    }

    fn serialize_erased(
        &self,
        value: &Arc<dyn Resource>,
    ) -> Option<Vec<u8>> {
        value.downcast_arc::<T::Value>().and_then(|v| self.serialize(v))
    }

    fn deserialize_erased(
        &self,
        data: &[u8],
    ) -> Option<Arc<dyn Resource>> {
        self.deserialize(data).map(|v| v as Arc<dyn Resource>)
    }
}

/// 向后兼容的类型别名
pub trait ErasedStateField: ErasedStateFieldGeneric<NodePool, Schema> {}

/// 插件规范结构体 (泛型版本)
/// 定义插件的配置和行为
#[derive(Clone, Debug)]
pub struct PluginSpecGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub state_field: Option<Arc<dyn ErasedStateFieldGeneric<C, S>>>,
    pub tr: Arc<dyn PluginTraitGeneric<C, S>>,
}

impl<C, S> PluginSpecGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 插件状态管理器
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, tr, state), fields(
        crate_name = "state",
        plugin_name = %self.tr.metadata().name,
        tr_id = %tr.id
    )))]
    pub async fn filter_transaction(
        &self,
        tr: &TransactionGeneric<C, S>,
        state: &StateGeneric<C, S>,
    ) -> bool {
        let filter = &self.tr;
        let result = filter.filter_transaction(tr, state).await;
        #[cfg(feature = "dev-tracing")]
        tracing::debug!(allowed = result, "过滤结果");
        result
    }

    /// 执行事务追加
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, trs, old_state, new_state), fields(
        crate_name = "state",
        plugin_name = %self.tr.metadata().name,
        tr_count = trs.len()
    )))]
    pub async fn append_transaction(
        &self,
        trs: &[Arc<TransactionGeneric<C, S>>],
        old_state: &Arc<StateGeneric<C, S>>,
        new_state: &Arc<StateGeneric<C, S>>,
    ) -> StateResult<Option<TransactionGeneric<C, S>>> {
        let tr = self.tr.append_transaction(trs, old_state, new_state).await?;
        #[cfg(feature = "dev-tracing")]
        if let Some(ref tr) = tr {
            tracing::debug!(step_count = tr.steps.len(), "追加事务成功");
        } else {
            tracing::debug!("无需追加事务");
        }
        Ok(tr)
    }
}

/// 向后兼容的类型别名
pub type PluginSpec = PluginSpecGeneric<NodePool, Schema>;

/// 插件实例结构体 (泛型版本)
/// 表示一个具体的插件实例
#[derive(Clone, Debug)]
pub struct PluginGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub spec: PluginSpecGeneric<C, S>,
    pub key: String,
}

impl<C, S> PluginGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 创建新的插件实例
    pub fn new(spec: PluginSpecGeneric<C, S>) -> Self {
        let key = spec.tr.metadata().name.clone();
        PluginGeneric { spec, key }
    }

    /// 获取插件名称
    pub fn get_name(&self) -> &str {
        &self.key
    }

    /// 获取插件元数据
    pub fn get_metadata(&self) -> PluginMetadata {
        self.spec.tr.metadata()
    }

    /// 获取插件配置
    pub fn get_config(&self) -> PluginConfig {
        self.spec.tr.config()
    }

    /// 从全局状态中获取插件状态
    pub fn get_state(
        &self,
        state: &StateGeneric<C, S>,
    ) -> Option<Arc<dyn Resource>> {
        state.get_field(&self.key)
    }

    /// 应用事务过滤逻辑
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, tr, state), fields(
        crate_name = "state",
        plugin_key = %self.key,
        tr_id = %tr.id
    )))]
    pub async fn apply_filter_transaction(
        &self,
        tr: &TransactionGeneric<C, S>,
        state: &StateGeneric<C, S>,
    ) -> bool {
        self.spec.filter_transaction(tr, state).await
    }

    /// 追加事务（使用旧版签名）
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, old_state, new_state, trs), fields(
        crate_name = "state",
        plugin_key = %self.key,
        tr_count = trs.len(),
        start_index = n
    )))]
    pub async fn append_transaction(
        &self,
        old_state: &Arc<StateGeneric<C, S>>,
        new_state: &Arc<StateGeneric<C, S>>,
        trs: &[Arc<TransactionGeneric<C, S>>],
        n: usize,
    ) -> Option<Arc<TransactionGeneric<C, S>>> {
        if n >= trs.len() {
            return None;
        }
        match self
            .spec
            .append_transaction(&trs[n..], old_state, new_state)
            .await
        {
            Ok(Some(tr)) => Some(Arc::new(tr)),
            Ok(None) => None,
            Err(e) => {
                tracing::error!("插件 {} 追加事务失败: {}", self.key, e);
                None
            },
        }
    }

    /// 应用事务追加逻辑
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, trs, old_state, new_state), fields(
        crate_name = "state",
        plugin_key = %self.key,
        tr_count = trs.len()
    )))]
    pub async fn apply_append_transaction(
        &self,
        trs: &[Arc<TransactionGeneric<C, S>>],
        old_state: &Arc<StateGeneric<C, S>>,
        new_state: &Arc<StateGeneric<C, S>>,
    ) -> StateResult<Option<TransactionGeneric<C, S>>> {
        self.spec.append_transaction(trs, old_state, new_state).await
    }
}

/// 向后兼容的类型别名
pub type Plugin = PluginGeneric<NodePool, Schema>;

/// 插件状态类型
/// 使用 Arc 包装的任意类型作为插件状态
//pub type PluginState = Arc<dyn std::any::Any + Send + Sync>;
use std::fmt::Debug;
