use async_trait::async_trait;
use std::sync::Arc;

use crate::error::StateResult;
use crate::plugin::{PluginConfig, PluginMetadata};
use crate::resource::Resource;

use crate::state::{State, StateConfig};
use crate::transaction::Transaction;

/// 插件特征
/// 定义插件的核心行为，包括事务处理和过滤功能
#[async_trait]
pub trait PluginTrait: Send + Sync + Debug {
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
        _: &[Arc<Transaction>],
        _: &Arc<State>,
        _: &Arc<State>,
    ) -> StateResult<Option<Transaction>> {
        Ok(None)
    }
    /// 事务过滤
    /// 决定是否允许事务执行
    async fn filter_transaction(
        &self,
        _: &Transaction,
        _: &State,
    ) -> bool {
        true
    }
}
///PluginTrait实现一个 default 实现
/// 状态字段特征
/// 使用关联类型保持类型信息，提供类型安全的插件状态管理
///
/// # 优势
/// - ✅ 编译期类型检查，无需运行时 downcast
/// - ✅ 更好的性能和代码可读性
/// - ✅ IDE 支持更好的自动补全
///
/// # 示例
///
/// ```ignore
/// #[derive(Debug)]
/// struct MyValue {
///     count: u32,
/// }
/// impl Resource for MyValue {}
///
/// #[derive(Debug)]
/// struct MyStateField;
///
/// #[async_trait]
/// impl StateField for MyStateField {
///     type Value = MyValue;
///
///     async fn init(&self, _config: &StateConfig, _state: &State) -> Arc<MyValue> {
///         Arc::new(MyValue { count: 0 })
///     }
///
///     async fn apply(&self, _tr: &Transaction, value: Arc<MyValue>,
///                   _old: &State, _new: &State) -> Arc<MyValue> {
///         // ✅ 类型安全，无需 downcast
///         Arc::new(MyValue { count: value.count + 1 })
///     }
/// }
/// ```
#[async_trait]
pub trait StateField: Send + Sync + Debug {
    /// 状态值类型，必须实现 Resource trait
    type Value: Resource;

    /// 初始化插件状态
    async fn init(
        &self,
        config: &StateConfig,
        instance: &State,
    ) -> Arc<Self::Value>;

    /// 应用状态变更
    /// 根据事务内容更新插件状态
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<Self::Value>,
        old_state: &State,
        new_state: &State,
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

/// 类型擦除的 StateField trait
/// 用于在 PluginSpec 中存储不同类型的 StateField
#[async_trait]
pub trait ErasedStateField: Send + Sync + Debug {
    /// 初始化插件状态
    async fn init_erased(
        &self,
        config: &StateConfig,
        instance: &State,
    ) -> Arc<dyn Resource>;

    /// 应用状态变更
    async fn apply_erased(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        old_state: &State,
        new_state: &State,
    ) -> Arc<dyn Resource>;

    /// 序列化插件状态
    fn serialize_erased(
        &self,
        value: Arc<dyn Resource>,
    ) -> Option<Vec<u8>>;

    /// 反序列化插件状态
    fn deserialize_erased(
        &self,
        data: &[u8],
    ) -> Option<Arc<dyn Resource>>;
}

/// StateField 到 ErasedStateField 的自动实现
#[async_trait]
impl<T: StateField + 'static> ErasedStateField for T {
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, config, instance), fields(
        crate_name = "state",
        state_field_type = std::any::type_name::<T>(),
        value_type = std::any::type_name::<T::Value>()
    )))]
    async fn init_erased(
        &self,
        config: &StateConfig,
        instance: &State,
    ) -> Arc<dyn Resource> {
        let value = self.init(config, instance).await;
        value as Arc<dyn Resource>
    }

    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, tr, value, old_state, new_state), fields(
        crate_name = "state",
        state_field_type = std::any::type_name::<T>(),
        value_type = std::any::type_name::<T::Value>(),
        tr_id = %tr.id
    )))]
    async fn apply_erased(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        old_state: &State,
        new_state: &State,
    ) -> Arc<dyn Resource> {
        // 尝试向下转型到具体类型
        if let Some(typed_value) = value.downcast_arc::<T::Value>() {
            let new_value =
                self.apply(tr, typed_value.clone(), old_state, new_state).await;
            new_value as Arc<dyn Resource>
        } else {
            // 类型不匹配，记录警告并返回原值
            tracing::warn!(
                "StateField 类型不匹配，期望 {}，跳过应用",
                std::any::type_name::<T::Value>()
            );
            value
        }
    }

    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, value), fields(
        crate_name = "state",
        state_field_type = std::any::type_name::<T>(),
        value_type = std::any::type_name::<T::Value>()
    )))]
    fn serialize_erased(
        &self,
        value: Arc<dyn Resource>,
    ) -> Option<Vec<u8>> {
        if let Some(typed_value) = value.downcast_arc::<T::Value>() {
            let result = self.serialize(typed_value);
            #[cfg(feature = "dev-tracing")]
            if let Some(ref data) = result {
                tracing::debug!(serialized_size = data.len(), "序列化成功");
            }
            result
        } else {
            None
        }
    }

    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, data), fields(
        crate_name = "state",
        state_field_type = std::any::type_name::<T>(),
        value_type = std::any::type_name::<T::Value>(),
        data_size = data.len()
    )))]
    fn deserialize_erased(
        &self,
        data: &[u8],
    ) -> Option<Arc<dyn Resource>> {
        let result = self.deserialize(data).map(|v| v as Arc<dyn Resource>);
        #[cfg(feature = "dev-tracing")]
        if result.is_some() {
            tracing::debug!("反序列化成功");
        }
        result
    }
}

/// 插件规范结构体
/// 定义插件的配置和行为
#[derive(Clone, Debug)]
pub struct PluginSpec {
    pub state_field: Option<Arc<dyn ErasedStateField>>,
    pub tr: Arc<dyn PluginTrait>,
}

// PluginSpec 所有字段满足 Send+Sync 约束（Arc 指针），无需不安全实现

impl PluginSpec {
    /// 插件状态管理器
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, tr, state), fields(
        crate_name = "state",
        plugin_name = %self.tr.metadata().name,
        tr_id = %tr.id
    )))]
    async fn filter_transaction(
        &self,
        tr: &Transaction,
        state: &State,
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
    async fn append_transaction(
        &self,
        trs: &[Arc<Transaction>],
        old_state: &Arc<State>,
        new_state: &Arc<State>,
    ) -> StateResult<Option<Transaction>> {
        let tr = self.tr.append_transaction(trs, old_state, new_state).await?;
        if let Some(mut tr) = tr {
            let _ = tr.commit(); // 在插件系统中，commit 错误可以被忽略
            #[cfg(feature = "dev-tracing")]
            tracing::debug!(step_count = tr.steps.len(), "追加事务成功");
            Ok(Some(tr))
        } else {
            #[cfg(feature = "dev-tracing")]
            tracing::debug!("无需追加事务");
            Ok(None)
        }
    }
}
/// 插件实例结构体
/// 表示一个具体的插件实例
#[derive(Clone, Debug)]
pub struct Plugin {
    pub spec: PluginSpec,
    pub key: String,
}

// Plugin 包含的字段满足 Auto Traits

impl Plugin {
    /// 创建新的插件实例
    pub fn new(spec: PluginSpec) -> Self {
        let key = spec.tr.metadata().name.clone();

        Plugin { spec, key }
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
        state: &State,
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
        tr: &Transaction,
        state: &State,
    ) -> bool {
        self.spec.filter_transaction(tr, state).await
    }

    /// 应用事务追加逻辑
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, trs, old_state, new_state), fields(
        crate_name = "state",
        plugin_key = %self.key,
        tr_count = trs.len()
    )))]
    pub async fn apply_append_transaction(
        &self,
        trs: &[Arc<Transaction>],
        old_state: &Arc<State>,
        new_state: &Arc<State>,
    ) -> StateResult<Option<Transaction>> {
        self.spec.append_transaction(trs, old_state, new_state).await
    }
}

/// 插件状态类型
/// 使用 Arc 包装的任意类型作为插件状态
//pub type PluginState = Arc<dyn std::any::Any + Send + Sync>;
use std::fmt::Debug;
