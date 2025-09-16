//! 执行上下文 - 解决循环引用问题
//!
//! 通过引入执行上下文，将插件和管理器解耦

use async_trait::async_trait;
use serde_json::Value;
use crate::error::DenoResult;

/// 插件执行上下文特质
/// 提供插件执行所需的基础能力，解耦插件和管理器
#[async_trait]
pub trait PluginExecutionContext: Send + Sync {
    /// 执行插件方法
    async fn execute_plugin_method(
        &self,
        plugin_id: &str,
        method_name: &str,
        args: Value,
    ) -> DenoResult<Value>;

    /// 异步执行插件方法
    async fn execute_plugin_method_async(
        &self,
        plugin_id: &str,
        method_name: &str,
        args: Value,
    ) -> DenoResult<Value>;

    /// 获取插件是否已加载
    async fn is_plugin_loaded(&self, plugin_id: &str) -> bool;

    /// 获取执行统计信息
    async fn get_execution_stats(&self) -> ExecutionStats;
}

/// 执行统计信息
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time_ms: f64,
}

/// 空的执行上下文（用于测试或无管理器情况）
pub struct NullExecutionContext;

#[async_trait]
impl PluginExecutionContext for NullExecutionContext {
    async fn execute_plugin_method(
        &self,
        _plugin_id: &str,
        _method_name: &str,
        _args: Value,
    ) -> DenoResult<Value> {
        Err(crate::error::DenoError::Runtime(anyhow::anyhow!(
            "No execution context available"
        )))
    }

    async fn execute_plugin_method_async(
        &self,
        _plugin_id: &str,
        _method_name: &str,
        _args: Value,
    ) -> DenoResult<Value> {
        Err(crate::error::DenoError::Runtime(anyhow::anyhow!(
            "No execution context available"
        )))
    }

    async fn is_plugin_loaded(&self, _plugin_id: &str) -> bool {
        false
    }

    async fn get_execution_stats(&self) -> ExecutionStats {
        ExecutionStats {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_execution_time_ms: 0.0,
        }
    }
}