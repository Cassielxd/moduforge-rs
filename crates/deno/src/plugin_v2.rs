//! 重构的 DenoPlugin - 移除循环引用
//!
//! 使用执行上下文模式，避免插件与管理器的直接循环引用

use std::sync::Arc;
use async_trait::async_trait;
use mf_state::{State, transaction::Transaction, plugin::{PluginTrait, PluginMetadata, PluginConfig}};
use crate::error::{DenoError, DenoResult};
use crate::execution_context::{PluginExecutionContext, NullExecutionContext};

/// 重构的 Deno 插件实现
/// 移除了对 DenoPluginManager 的直接引用，使用执行上下文
#[derive(Clone)]
pub struct DenoPluginV2 {
    pub id: String,
    pub code: String,
    pub metadata: PluginMetadata,
    pub config: PluginConfig,
    /// 执行上下文 - 解耦插件和管理器
    execution_context: Arc<dyn PluginExecutionContext>,
}

impl DenoPluginV2 {
    /// 创建新的 Deno 插件（使用空上下文）
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
            execution_context: Arc::new(NullExecutionContext),
        }
    }

    /// 使用执行上下文创建插件
    pub fn with_execution_context(
        id: String,
        code: String,
        execution_context: Arc<dyn PluginExecutionContext>
    ) -> Self {
        let mut plugin = Self::new(id, code);
        plugin.execution_context = execution_context;
        plugin
    }

    /// 从元数据创建插件
    pub fn from_metadata(
        id: String,
        code: String,
        metadata: PluginMetadata,
        config: Option<PluginConfig>,
        execution_context: Option<Arc<dyn PluginExecutionContext>>,
    ) -> Self {
        let config = config.unwrap_or_else(|| PluginConfig {
            enabled: true,
            priority: 0,
            settings: std::collections::HashMap::new(),
        });

        let execution_context = execution_context
            .unwrap_or_else(|| Arc::new(NullExecutionContext));

        Self {
            id,
            code,
            metadata,
            config,
            execution_context,
        }
    }

    /// 设置执行上下文
    pub fn set_execution_context(&mut self, context: Arc<dyn PluginExecutionContext>) {
        self.execution_context = context;
    }

    /// 执行 JavaScript 函数（同步方式）
    async fn execute_js_function(
        &self,
        function_name: &str,
        args: serde_json::Value,
    ) -> DenoResult<serde_json::Value> {
        self.execution_context
            .execute_plugin_method(&self.id, function_name, args)
            .await
    }

    /// 执行 JavaScript 函数（异步方式）
    async fn execute_js_function_async(
        &self,
        function_name: &str,
        args: serde_json::Value,
    ) -> DenoResult<serde_json::Value> {
        self.execution_context
            .execute_plugin_method_async(&self.id, function_name, args)
            .await
    }

    /// 检查插件是否已加载到执行环境
    pub async fn is_loaded(&self) -> bool {
        self.execution_context.is_plugin_loaded(&self.id).await
    }

    /// 获取执行统计信息
    pub async fn get_execution_stats(&self) -> crate::execution_context::ExecutionStats {
        self.execution_context.get_execution_stats().await
    }
}

impl std::fmt::Debug for DenoPluginV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DenoPluginV2 {{ id: {}, enabled: {} }}", self.id, self.config.enabled)
    }
}

#[async_trait]
impl PluginTrait for DenoPluginV2 {
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
        match self.execute_js_function_async("appendTransaction", args).await {
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
        match self.execute_js_function_async("filterTransaction", args).await {
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

/// Deno 插件构建器 V2
/// 提供更方便的插件创建方式，支持执行上下文注入
pub struct DenoPluginBuilderV2 {
    id: String,
    code: Option<String>,
    metadata: PluginMetadata,
    config: PluginConfig,
    execution_context: Option<Arc<dyn PluginExecutionContext>>,
}

impl DenoPluginBuilderV2 {
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
            execution_context: None,
        }
    }

    /// 设置插件代码
    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// 从文件加载插件代码
    pub async fn code_from_file(mut self, file_path: impl AsRef<std::path::Path>) -> DenoResult<Self> {
        let code = tokio::fs::read_to_string(file_path).await
            .map_err(|e| DenoError::Runtime(anyhow::anyhow!("Failed to read file: {}", e)))?;
        self.code = Some(code);
        Ok(self)
    }

    /// 设置执行上下文
    pub fn execution_context(mut self, context: Arc<dyn PluginExecutionContext>) -> Self {
        self.execution_context = Some(context);
        self
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

    /// 设置插件名称
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.metadata.name = name.into();
        self
    }

    /// 设置插件版本
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.metadata.version = version.into();
        self
    }

    /// 设置插件描述
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.metadata.description = description.into();
        self
    }

    /// 设置插件作者
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.metadata.author = author.into();
        self
    }

    /// 添加标签
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.metadata.tags.push(tag.into());
        self
    }

    /// 构建插件
    pub fn build(self) -> DenoResult<DenoPluginV2> {
        let code = self.code.ok_or_else(|| {
            DenoError::Runtime(anyhow::anyhow!("Plugin code not set"))
        })?;

        Ok(DenoPluginV2::from_metadata(
            self.id,
            code,
            self.metadata,
            Some(self.config),
            self.execution_context,
        ))
    }
}