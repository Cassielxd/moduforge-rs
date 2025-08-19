use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tokio::sync::RwLock;

use crate::plugin::PluginConfig;
/// 插件配置管理器
#[derive(Debug)]
pub struct ConfigManager {
    configs: Arc<RwLock<HashMap<String, PluginConfig>>>,
    config_file_path: Option<String>,
}
impl ConfigManager {
    pub fn new() -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            config_file_path: None,
        }
    }

    /// 设置配置文件路径
    pub fn set_config_file(
        &mut self,
        path: String,
    ) {
        self.config_file_path = Some(path);
    }

    /// 加载配置
    pub async fn load_configs(&self) -> Result<()> {
        if let Some(path) = &self.config_file_path {
            if std::path::Path::new(path).exists() {
                let content = tokio::fs::read_to_string(path).await?;
                let configs: HashMap<String, PluginConfig> =
                    serde_json::from_str(&content)?;
                let mut configs_guard = self.configs.write().await;
                *configs_guard = configs;
            }
        }
        Ok(())
    }

    /// 保存配置
    pub async fn save_configs(&self) -> Result<()> {
        if let Some(path) = &self.config_file_path {
            let configs = self.configs.read().await;
            let content = serde_json::to_string_pretty(&*configs)?;
            tokio::fs::write(path, content).await?;
        }
        Ok(())
    }

    /// 获取插件配置
    pub async fn get_config(
        &self,
        plugin_name: &str,
    ) -> Option<PluginConfig> {
        let configs = self.configs.read().await;
        configs.get(plugin_name).cloned()
    }

    /// 设置插件配置
    pub async fn set_config(
        &self,
        plugin_name: &str,
        config: PluginConfig,
    ) {
        let mut configs = self.configs.write().await;
        configs.insert(plugin_name.to_string(), config);
    }

    /// 更新插件设置
    pub async fn update_settings(
        &self,
        plugin_name: &str,
        settings: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let mut configs = self.configs.write().await;
        if let Some(config) = configs.get_mut(plugin_name) {
            config.settings = settings;
        }
        Ok(())
    }
}
