use std::sync::Arc;
use async_trait::async_trait;
use mf_state::{State, transaction::Transaction, plugin::{PluginTrait, PluginMetadata, PluginConfig}};
use crate::error::{DenoError, DenoResult};

use crate::runtime::manager::DenoPluginManager;


/// Deno 插件实现
/// 将 JavaScript/TypeScript 插件与 ModuForge 插件系统集成
#[derive(Clone)]
pub struct DenoPlugin {
    pub id: String,
    pub code: String,
    pub metadata: PluginMetadata,
    pub config: PluginConfig,
    manager: Option<Arc<DenoPluginManager>>,
}

impl DenoPlugin {
    /// 创建新的 Deno 插件
    pub fn new(id: String, code: String) -> Self {
        let metadata = PluginMetadata {
            name: id.clone(),
            version: "1.0.0".to_string(),
            description: "Deno-based plugin".to_string(),
            author: "Unknown".to_string(),
            dependencies: vec![],
            conflicts: vec![],
            state_fields: vec![],
            tags: vec!["deno".to_string(), "javascript".to_string()],
        };

        let config = PluginConfig {
            enabled: true,
            priority: 0,
            settings: std::collections::HashMap::new(),
        };

        Self {
            id,
            code,
            metadata,
            config,
            manager: None,
        }
    }

    /// 设置插件管理器引用
    pub fn with_manager(mut self, manager: Arc<DenoPluginManager>) -> Self {
        self.manager = Some(manager);
        self
    }

    /// 从元数据创建插件
    pub fn from_metadata(
        id: String,
        code: String,
        metadata: PluginMetadata,
        config: Option<PluginConfig>,
    ) -> Self {
        let config = config.unwrap_or_else(|| PluginConfig {
            enabled: true,
            priority: 0,
            settings: std::collections::HashMap::new(),
        });

        Self {
            id,
            code,
            metadata,
            config,
            manager: None,
        }
    }

    /// 执行 JavaScript 函数
    async fn execute_js_function(
        &self,
        function_name: &str,
        args: serde_json::Value,
    ) -> DenoResult<serde_json::Value> {
        if let Some(manager) = &self.manager {
            manager.execute_plugin_method(&self.id, function_name, args).await
        } else {
            Err(DenoError::Runtime(anyhow::anyhow!("Plugin manager not set")))
        }
    }
}

impl std::fmt::Debug for DenoPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DenoPlugin {{ id: {}, enabled: {} }}", self.id, self.config.enabled)
    }
}

#[async_trait]
impl PluginTrait for DenoPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn config(&self) -> PluginConfig {
        self.config.clone()
    }

    /// 追加事务处理
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> mf_state::error::StateResult<Option<Transaction>> {
        if !self.config.enabled {
            return Ok(None);
        }

        // 构造传递给 JavaScript 的参数
        let args = serde_json::json!({
            "transactionCount": transactions.len(),
            "oldStateVersion": old_state.version,
            "newStateVersion": new_state.version,
        });

        // 调用 JavaScript 的 appendTransaction 函数
        match self.execute_js_function("appendTransaction", args).await {
            Ok(result) => {
                // 解析返回结果
                if result.is_null() {
                    Ok(None)
                } else {
                    // 这里需要根据返回的 JSON 创建 Transaction
                    // 简化实现：创建一个空的事务
                    let tr = Transaction::new(new_state);
                    Ok(Some(tr))
                }
            }
            Err(e) => {
                tracing::error!("Failed to execute appendTransaction for plugin {}: {}", self.id, e);
                Ok(None)
            }
        }
    }

    /// 事务过滤
    async fn filter_transaction(
        &self,
        transaction: &Transaction,
        state: &State,
    ) -> bool {
        if !self.config.enabled {
            return true;
        }

        // 构造传递给 JavaScript 的参数
        let args = serde_json::json!({
            "transactionId": transaction.id,
            "stateVersion": state.version,
        });

        // 调用 JavaScript 的 filterTransaction 函数
        match self.execute_js_function("filterTransaction", args).await {
            Ok(result) => {
                result.as_bool().unwrap_or(true)
            }
            Err(e) => {
                tracing::error!("Failed to execute filterTransaction for plugin {}: {}", self.id, e);
                // 出错时默认允许事务
                true
            }
        }
    }
}

/// Deno 插件构建器
/// 提供更方便的插件创建方式
pub struct DenoPluginBuilder {
    id: String,
    code: Option<String>,
    metadata: PluginMetadata,
    config: PluginConfig,
}

impl DenoPluginBuilder {
    /// 创建新的构建器
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        let metadata = PluginMetadata {
            name: id.clone(),
            version: "1.0.0".to_string(),
            description: "Deno-based plugin".to_string(),
            author: "Unknown".to_string(),
            dependencies: vec![],
            conflicts: vec![],
            state_fields: vec![],
            tags: vec!["deno".to_string()],
        };

        let config = PluginConfig {
            enabled: true,
            priority: 0,
            settings: std::collections::HashMap::new(),
        };

        Self {
            id,
            code: None,
            metadata,
            config,
        }
    }

    /// 设置插件代码
    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// 从文件加载插件代码
    pub async fn code_from_file(mut self, file_path: impl AsRef<std::path::Path>) -> DenoResult<Self> {
        let code = tokio::fs::read_to_string(file_path).await?;
        self.code = Some(code);
        Ok(self)
    }

    /// 设置插件元数据
    pub fn metadata(mut self, metadata: PluginMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// 设置插件配置
    pub fn config(mut self, config: PluginConfig) -> Self {
        self.config = config;
        self
    }

    /// 设置插件优先级
    pub fn priority(mut self, priority: i32) -> Self {
        self.config.priority = priority;
        self
    }

    /// 设置插件是否启用
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// 构建插件
    pub fn build(self) -> DenoResult<DenoPlugin> {
        let code = self.code.ok_or_else(|| {
            DenoError::Runtime(anyhow::anyhow!("Plugin code not set"))
        })?;

        Ok(DenoPlugin::from_metadata(self.id, code, self.metadata, Some(self.config)))
    }
}