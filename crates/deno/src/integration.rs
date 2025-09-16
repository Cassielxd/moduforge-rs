use std::sync::Arc;
use mf_state::{State, StateConfig, plugin::Plugin};
use crate::{DenoPluginManager, DenoPlugin, DenoPluginBuilder, DenoResult};

/// ModuForge Deno 集成入口
/// 提供将 Deno 插件集成到 ModuForge 插件系统的便捷方法
pub struct ModuForgeDeno {
    manager: Arc<DenoPluginManager>,
}

impl ModuForgeDeno {
    /// 创建新的 ModuForge Deno 集成实例
    pub fn new(initial_state: Arc<State>, pool_size: Option<usize>) -> Self {
        let pool_size = pool_size.unwrap_or(4);
        let manager = Arc::new(DenoPluginManager::new(initial_state, pool_size));

        Self { manager }
    }

    /// 初始化 Deno 运行时池
    pub async fn initialize(&self) -> DenoResult<()> {
        self.manager.initialize_pool().await
    }

    /// 从文件加载 JavaScript/TypeScript 插件
    pub async fn load_plugin_from_file(
        &self,
        plugin_id: impl Into<String>,
        file_path: impl AsRef<std::path::Path>,
    ) -> DenoResult<Arc<Plugin>> {
        let plugin_id = plugin_id.into();
        let code = tokio::fs::read_to_string(file_path).await?;

        let deno_plugin = DenoPlugin::new(plugin_id.clone(), code.clone())
            .with_manager(self.manager.clone());

        // 加载到管理器中
        self.manager.load_plugin(plugin_id.clone(), code).await?;

        // 转换为 ModuForge 插件
        let plugin = Plugin::new(mf_state::plugin::PluginSpec {
            state_field: None,
            tr: Arc::new(deno_plugin),
        });

        Ok(Arc::new(plugin))
    }

    /// 从代码字符串创建插件
    pub async fn create_plugin_from_code(
        &self,
        plugin_id: impl Into<String>,
        code: impl Into<String>,
    ) -> DenoResult<Arc<Plugin>> {
        let plugin_id = plugin_id.into();
        let code = code.into();

        let deno_plugin = DenoPlugin::new(plugin_id.clone(), code.clone())
            .with_manager(self.manager.clone());

        // 加载到管理器中
        self.manager.load_plugin(plugin_id, code).await?;

        // 转换为 ModuForge 插件
        let plugin = Plugin::new(mf_state::plugin::PluginSpec {
            state_field: None,
            tr: Arc::new(deno_plugin),
        });

        Ok(Arc::new(plugin))
    }

    /// 使用构建器创建插件
    pub async fn build_plugin(
        &self,
        builder: DenoPluginBuilder,
    ) -> DenoResult<Arc<Plugin>> {
        let deno_plugin = builder.build()?
            .with_manager(self.manager.clone());

        // 加载到管理器中
        self.manager.load_plugin(
            deno_plugin.id.clone(),
            deno_plugin.code.clone(),
        ).await?;

        // 转换为 ModuForge 插件
        let plugin = Plugin::new(mf_state::plugin::PluginSpec {
            state_field: None,
            tr: Arc::new(deno_plugin),
        });

        Ok(Arc::new(plugin))
    }

    /// 卸载插件
    pub async fn unload_plugin(&self, plugin_id: &str) -> DenoResult<()> {
        self.manager.unload_plugin(plugin_id).await
    }

    /// 更新状态
    pub async fn update_state(&self, new_state: Arc<State>) {
        self.manager.update_state(new_state).await;
    }

    /// 获取已加载的插件列表
    pub async fn list_plugins(&self) -> Vec<String> {
        self.manager.list_plugins().await
    }

    /// 关闭集成，清理资源
    pub async fn shutdown(self) {
        self.manager.shutdown().await;
    }

    /// 获取管理器引用（用于高级操作）
    pub fn manager(&self) -> Arc<DenoPluginManager> {
        self.manager.clone()
    }
}

/// 便捷函数：为 StateConfig 添加 Deno 插件支持
pub async fn add_deno_plugins_to_state_config(
    mut config: StateConfig,
    deno: &ModuForgeDeno,
    plugin_specs: Vec<(&str, &str)>, // (plugin_id, file_path) pairs
) -> DenoResult<StateConfig> {
    let mut plugins = config.plugins.unwrap_or_default();

    for (plugin_id, file_path) in plugin_specs {
        let plugin = deno.load_plugin_from_file(plugin_id, file_path).await?;
        plugins.push(plugin);
    }

    config.plugins = Some(plugins);
    Ok(config)
}

/// 创建简单的插件示例代码
pub fn create_sample_plugin_code() -> &'static str {
    r#"
// ModuForge Deno 插件示例
// 这个插件会在每次事务时添加一个时间戳元数据

function appendTransaction(args) {
    console.log('Plugin appendTransaction called:', args);

    // 创建新事务
    const transactionId = ModuForge.Transaction.new();

    // 添加时间戳元数据
    const timestamp = Date.now();
    ModuForge.Transaction.setMeta(transactionId, 'timestamp', timestamp);
    ModuForge.Transaction.setMeta(transactionId, 'plugin', 'deno-sample-plugin');

    return { transactionId };
}

function filterTransaction(args) {
    console.log('Plugin filterTransaction called:', args);

    // 默认允许所有事务
    return true;
}

// 插件初始化
console.log('Sample Deno plugin loaded successfully');
"#
}