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
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "default_plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "默认插件".to_string(),
            author: "系统".to_string(),
            dependencies: vec![],
            conflicts: vec![],
            state_fields: vec![],
            tags: vec![],
        }
    }

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
        _: &[Transaction],
        _: &State,
        _: &State,
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
/// 定义插件状态的管理方式，包括初始化、应用更改和序列化
#[async_trait]
pub trait StateField: Send + Sync + Debug {
    /// 初始化插件状态
    async fn init(
        &self,
        config: &StateConfig,
        instance: &State,
    ) -> Arc<dyn Resource>;
    /// 应用状态变更
    /// 根据事务内容更新插件状态
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        old_state: &State,
        new_state: &State,
    ) -> Arc<dyn Resource>;
    /// 序列化插件状态
    fn serialize(
        &self,
        _value: Arc<dyn Resource>,
    ) -> Option<Vec<u8>> {
        None
    }
    /// 反序列化插件状态
    fn deserialize(
        &self,
        _value: &Vec<u8>,
    ) -> Option<Arc<dyn Resource>> {
        None
    }
}
/// 插件规范结构体
/// 定义插件的配置和行为
#[derive(Clone, Debug)]
pub struct PluginSpec {
    pub state_field: Option<Arc<dyn StateField>>,
    pub tr: Arc<dyn PluginTrait>,
}

// PluginSpec 所有字段满足 Send+Sync 约束（Arc 指针），无需不安全实现

impl PluginSpec {
    /// 插件状态管理器
    async fn filter_transaction(
        &self,
        tr: &Transaction,
        state: &State,
    ) -> bool {
        match &self.tr {
            filter => filter.filter_transaction(tr, state).await,
        }
    }
    /// 执行事务追加
    async fn append_transaction<'a>(
        &self,
        trs: &'a [Transaction],
        old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        let tr = self.tr.append_transaction(trs, old_state, new_state).await?;
        if let Some(mut tr) = tr {
            tr.commit();
            Ok(Some(tr))
        } else {
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

    /// 从全局状态中获取插件状态
    pub fn get_state(
        &self,
        state: &State,
    ) -> Option<Arc<dyn Resource>> {
        state.get_field(&self.key)
    }
    /// 应用事务过滤逻辑
    pub async fn apply_filter_transaction(
        &self,
        tr: &Transaction,
        state: &State,
    ) -> bool {
        self.spec.filter_transaction(tr, state).await
    }

    /// 应用事务追加逻辑
    pub async fn apply_append_transaction(
        &self,
        trs: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        self.spec.append_transaction(trs, old_state, new_state).await
    }
}

/// 插件状态类型
/// 使用 Arc 包装的任意类型作为插件状态
//pub type PluginState = Arc<dyn std::any::Any + Send + Sync>;
use std::fmt::Debug;
