use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use super::dependency::DependencyManager;
use super::plugin::Plugin;

/// 插件管理器
#[derive(Debug, Clone)]
pub struct PluginManager {
    pub plugins: Arc<RwLock<HashMap<String, Arc<Plugin>>>>,
    pub dependency_manager: Arc<RwLock<DependencyManager>>,
    pub plugin_order: Arc<RwLock<Vec<String>>>,
    // 新增：标记是否已完成初始化
    pub initialized: Arc<RwLock<bool>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            dependency_manager: Arc::new(RwLock::new(DependencyManager::new())),
            plugin_order: Arc::new(RwLock::new(Vec::new())),
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    /// 注册插件
    pub async fn register_plugin(
        &self,
        plugin: Arc<Plugin>,
    ) -> Result<()> {
        let plugin_name = plugin.spec.tr.metadata().name.clone();

        // 检查插件是否已存在
        {
            let plugins = self.plugins.read().await;
            if plugins.contains_key(&plugin_name) {
                return Err(anyhow::anyhow!("插件 '{}' 已存在", plugin_name));
            }
        }
        // 注册插件
        {
            let mut plugins = self.plugins.write().await;
            plugins.insert(plugin_name.clone(), plugin.clone());
        }

        // 更新依赖图
        self.update_dependency_graph(&plugin).await?;

        tracing::info!("插件 '{}' 注册成功", plugin_name);
        Ok(())
    }
    /// 验证插件依赖
    pub async fn finalize_registration(&self) -> Result<()> {
        // 检查循环依赖
        self.check_circular_dependencies().await?;

        // 检查缺失的依赖
        self.check_missing_dependencies().await?;

        // 检查插件冲突
        self.check_plugin_conflicts().await?;

        // 更新插件执行顺序
        self.update_plugin_order().await?;

        // 标记为已初始化
        {
            let mut initialized = self.initialized.write().await;
            *initialized = true;
        }

        tracing::info!("插件注册完成，共注册 {} 个插件", {
            let plugins = self.plugins.read().await;
            plugins.len()
        });

        Ok(())
    }
    /// 检查循环依赖
    async fn check_circular_dependencies(&self) -> Result<()> {
        let dependency_manager = self.dependency_manager.read().await;
        if dependency_manager.has_circular_dependencies() {
            let report = dependency_manager.get_circular_dependency_report();
            return Err(anyhow::anyhow!(
                "检测到循环依赖: {:?}",
                report.to_string()
            ));
        }
        Ok(())
    }
    /// 检查缺失的依赖
    async fn check_missing_dependencies(&self) -> Result<()> {
        let dependency_manager = self.dependency_manager.read().await;
        let report = dependency_manager.check_missing_dependencies();
        if report.has_missing_dependencies {
            return Err(anyhow::anyhow!(
                "检测到缺失依赖: {:?}",
                report.to_string()
            ));
        }
        Ok(())
    }
    /// 检查插件冲突
    async fn check_plugin_conflicts(&self) -> Result<()> {
        let plugins = self.plugins.read().await;
        let available_plugins: HashSet<String> =
            plugins.keys().cloned().collect();

        for (name, plugin) in plugins.iter() {
            let metadata = plugin.spec.tr.metadata();

            for conflict in &metadata.conflicts {
                if available_plugins.contains(conflict) {
                    return Err(anyhow::anyhow!(
                        "插件 '{}' 与插件 '{}' 冲突",
                        name,
                        conflict
                    ));
                }
            }
        }

        Ok(())
    }

    /// 更新依赖图
    async fn update_dependency_graph(
        &self,
        plugin: &Arc<Plugin>,
    ) -> Result<()> {
        let mut dependency_manager = self.dependency_manager.write().await;

        let metadata = plugin.spec.tr.metadata();

        // 添加插件节点
        dependency_manager.add_plugin(&metadata.name);

        // 添加依赖关系
        for dep in &metadata.dependencies {
            dependency_manager.add_dependency(&metadata.name, dep)?;
        }

        Ok(())
    }
    /// 更新插件执行顺序
    async fn update_plugin_order(&self) -> Result<()> {
        let dependency_manager = self.dependency_manager.read().await;
        let order = dependency_manager.get_topological_order()?;

        let mut plugin_order = self.plugin_order.write().await;
        *plugin_order = order;

        Ok(())
    }
    /// 获取排序后的插件
    /// 按照依赖关系排序
    pub async fn get_sorted_plugins(&self) -> Vec<Arc<Plugin>> {
        let plugin_order = self.plugin_order.read().await;
        let plugins = self.plugins.read().await;
        let mut sorted_plugins = Vec::new();
        for plugin_name in plugin_order.iter() {
            if let Some(plugin) = plugins.get(plugin_name) {
                sorted_plugins.push(plugin.clone());
            }
        }
        sorted_plugins
    }

    /// 检查初始化状态
    pub async fn is_initialized(&self) -> bool {
        let initialized = self.initialized.read().await;
        *initialized
    }
}
