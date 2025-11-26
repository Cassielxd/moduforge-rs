use serde::{Deserialize, Serialize};

pub mod dependency;
pub mod manager;
#[allow(clippy::module_inception)]
pub mod plugin;

pub use plugin::*;
pub use dependency::DependencyManager;
pub use manager::{PluginManager, PluginManagerBuilder, PluginManagerGeneric, PluginManagerBuilderGeneric};
/// 插件元数据
/// 插件的元数据，用于描述插件的名称、版本、描述、作者、依赖、冲突、状态字段、标签等信息
/// dependencies 主要是 事务处理的依赖 B插件依赖于A插件 产生的事务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,              //插件名称
    pub version: String,           //插件版本
    pub description: String,       //插件描述
    pub author: String,            //插件作者
    pub dependencies: Vec<String>, //插件依赖
    pub conflicts: Vec<String>,    //插件冲突
    pub state_fields: Vec<String>, //插件状态字段
    pub tags: Vec<String>,         //插件标签
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool, //插件是否启用
    pub priority: i32, //插件优先级
    pub settings: std::collections::HashMap<String, serde_json::Value>, //插件配置
}
