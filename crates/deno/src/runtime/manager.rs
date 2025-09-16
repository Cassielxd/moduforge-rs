use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use deno_core::{JsRuntime, RuntimeOptions, Extension};
use mf_state::State;
use tokio::sync::{RwLock, Mutex};
use tokio::time::timeout;

use crate::error::{DenoError, DenoResult};
use crate::ops::create_moduforge_extension;
use crate::runtime::context::{ModuForgeContext, set_context_to_opstate};
use crate::plugin::DenoPlugin;
use crate::runtime::main_worker_manager::MainWorkerManager;

/// 运行时池统计信息
#[derive(Debug, Clone)]
pub struct RuntimePoolStats {
    pub total_capacity: usize,
    pub available_runtimes: usize,
    pub active_runtimes: usize,
    pub created_count: u64,
    pub reused_count: u64,
    pub last_activity: Option<Instant>,
}

/// Deno 插件管理器
/// 管理 Deno 运行时实例和插件生命周期
/// 使用线程本地存储解决 JsRuntime 多线程传递问题
pub struct DenoPluginManager {
    /// 已加载的插件
    plugins: Arc<RwLock<HashMap<String, Arc<DenoPlugin>>>>,

    /// 线程本地运行时管理器
    thread_runtime_manager: MainWorkerManager,

    /// 运行时池大小（用于统计和配置）
    pool_size: usize,

    /// 当前状态
    current_state: Arc<RwLock<Arc<State>>>,

    /// 池统计信息（兼容原有API）
    stats: Arc<Mutex<RuntimePoolStats>>,
}

impl DenoPluginManager {
    /// 创建新的插件管理器
    pub fn new(initial_state: Arc<State>, pool_size: usize) -> Self {
        let pool_size = pool_size.max(1); // 确保至少有一个运行时实例

        let thread_runtime_manager = MainWorkerManager::new(initial_state.clone());

        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            thread_runtime_manager,
            pool_size,
            current_state: Arc::new(RwLock::new(initial_state)),
            stats: Arc::new(Mutex::new(RuntimePoolStats {
                total_capacity: pool_size,
                available_runtimes: pool_size, // 线程本地模式下始终可用
                active_runtimes: 0,
                created_count: 0,
                reused_count: 0,
                last_activity: None,
            })),
        }
    }

    /// 初始化运行时管理器（兼容原有API）
    pub async fn initialize_pool(&self) -> DenoResult<()> {
        // 线程本地模式下不需要预创建运行时，只需更新统计信息
        let mut stats = self.stats.lock().await;
        stats.available_runtimes = self.pool_size; // 线程本地模式下始终可用
        stats.created_count = 0; // 按需创建
        stats.last_activity = Some(Instant::now());
        drop(stats);

        tracing::info!("Deno thread-local runtime manager initialized with capacity {}", self.pool_size);
        Ok(())
    }

    // 移除 create_runtime 方法，由 ThreadLocalRuntimeManager 处理运行时创建

    // 移除 get_runtime 方法，由于线程本地模式不需要显式获取运行时

    // 移除 return_runtime 方法，线程本地模式下运行时自动管理

    /// 加载插件
    pub async fn load_plugin(
        &self,
        plugin_id: String,
        plugin_code: String
    ) -> DenoResult<()> {
        // 使用线程本地运行时管理器加载插件
        self.thread_runtime_manager.load_plugin(plugin_id.clone(), plugin_code.clone()).await?;

        // 创建插件对象
        let plugin = Arc::new(DenoPlugin::new(plugin_id.clone(), plugin_code));

        // 存储插件
        let mut plugins = self.plugins.write().await;
        plugins.insert(plugin_id.clone(), plugin);

        // 更新统计信息
        {
            let mut stats = self.stats.lock().await;
            stats.created_count += 1; // 可能创建了新的线程本地运行时
            stats.last_activity = Some(Instant::now());
        }

        tracing::info!("Plugin {} loaded successfully", plugin_id);
        Ok(())
    }

    /// 卸载插件
    pub async fn unload_plugin(&self, plugin_id: &str) -> DenoResult<()> {
        let mut plugins = self.plugins.write().await;

        if plugins.remove(plugin_id).is_some() {
            drop(plugins);

            // 从线程本地运行时管理器中卸载插件
            self.thread_runtime_manager.unload_plugin(plugin_id).await?;

            tracing::info!("Plugin {} unloaded successfully", plugin_id);
            Ok(())
        } else {
            Err(DenoError::PluginNotFound(plugin_id.to_string()))
        }
    }

    /// 执行插件方法
    pub async fn execute_plugin_method(
        &self,
        plugin_id: &str,
        method_name: &str,
        args: serde_json::Value,
    ) -> DenoResult<serde_json::Value> {
        // 检查插件是否存在
        let plugins = self.plugins.read().await;
        if !plugins.contains_key(plugin_id) {
            return Err(DenoError::PluginNotFound(plugin_id.to_string()));
        }
        drop(plugins);

        // 使用线程本地运行时管理器执行插件方法
        let result = self.thread_runtime_manager.execute_plugin_method(
            plugin_id,
            method_name,
            args
        ).await?;

        // 更新统计信息
        {
            let mut stats = self.stats.lock().await;
            stats.reused_count += 1; // 计为复用
            stats.last_activity = Some(Instant::now());
        }

        Ok(result)
    }

    /// 更新状态
    pub async fn update_state(&self, new_state: Arc<State>) {
        let mut current_state = self.current_state.write().await;
        *current_state = new_state.clone();
        drop(current_state);

        // 同步更新线程本地运行时管理器的状态
        self.thread_runtime_manager.update_state(new_state).await;

        tracing::debug!("Plugin manager state updated");
    }

    /// 获取已加载的插件列表
    pub async fn list_plugins(&self) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins.keys().cloned().collect()
    }

    /// 获取运行时池统计信息
    pub async fn get_pool_stats(&self) -> RuntimePoolStats {
        let stats = self.stats.lock().await;
        stats.clone()
    }

    /// 检查运行时管理器健康状态
    pub async fn health_check(&self) -> DenoResult<bool> {
        let stats = self.stats.lock().await;

        // 线程本地模式下的健康检查
        let is_healthy = stats.active_runtimes >= 0
            && stats.available_runtimes > 0; // 线程本地模式下始终可用

        if !is_healthy {
            tracing::warn!(
                "Runtime manager health check failed: active={}, available={}",
                stats.active_runtimes, stats.available_runtimes
            );
        }

        Ok(is_healthy)
    }

    /// 获取详细的运行时监控信息
    pub async fn get_pool_metrics(&self) -> serde_json::Value {
        let stats = self.stats.lock().await;
        let thread_stats = self.thread_runtime_manager.get_stats().await;

        serde_json::json!({
            "pool_capacity": self.pool_size,
            "available_runtimes": stats.available_runtimes,
            "active_runtimes": stats.active_runtimes,
            "total_created": thread_stats.runtimes_created, // 使用线程本地统计
            "total_reused": stats.reused_count,
            "total_executions": thread_stats.plugin_executions,
            "average_execution_time_ms": if thread_stats.plugin_executions > 0 {
                thread_stats.total_execution_time.as_millis() as f64 / thread_stats.plugin_executions as f64
            } else {
                0.0
            },
            "utilization_rate": if thread_stats.runtimes_created > 0 {
                (stats.reused_count as f64 / thread_stats.runtimes_created as f64) * 100.0
            } else {
                0.0
            },
            "last_activity": stats.last_activity.map(|t| t.elapsed().as_secs()),
            "health_status": "healthy", // 线程本地模式下始终健康
            "architecture": "thread_local"
        })
    }

    /// 重建运行时管理器（兼容原有API）
    pub async fn rebuild_pool(&self) -> DenoResult<()> {
        tracing::info!("Rebuilding thread-local runtime manager...");

        // 线程本地模式下重建主要是重置统计信息
        {
            let mut stats = self.stats.lock().await;
            let old_created = stats.created_count;
            stats.created_count = 0;
            stats.reused_count = 0;
            stats.active_runtimes = 0;
            stats.available_runtimes = self.pool_size;
            stats.last_activity = Some(Instant::now());
            tracing::debug!("Reset statistics, previous created count: {}", old_created);
        }

        // 重新初始化
        self.initialize_pool().await?;

        tracing::info!("Thread-local runtime manager rebuilt successfully");
        Ok(())
    }

    /// 关闭管理器，清理资源
    pub async fn shutdown(&self) {
        tracing::info!("Shutting down Deno plugin manager...");

        // 等待活跃的操作完成（最多等待10秒）
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(10) {
            let stats = self.stats.lock().await;
            if stats.active_runtimes == 0 {
                break;
            }
            drop(stats);

            tracing::debug!("Waiting for {} active operations to complete...",
                           self.stats.lock().await.active_runtimes);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // 清理插件
        {
            let mut plugins = self.plugins.write().await;
            let plugin_count = plugins.len();
            plugins.clear();
            tracing::debug!("Cleared {} loaded plugins", plugin_count);
        }

        // 重置统计信息
        {
            let mut stats = self.stats.lock().await;
            *stats = RuntimePoolStats {
                total_capacity: self.pool_size,
                available_runtimes: 0,
                active_runtimes: 0,
                created_count: 0,
                reused_count: 0,
                last_activity: None,
            };
        }

        tracing::info!("Deno plugin manager shut down successfully");
    }
}