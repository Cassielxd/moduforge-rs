use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use deno_core::{Extension, ModuleSpecifier, FastString};
use deno_runtime::worker::{MainWorker, WorkerOptions};
use deno_runtime::permissions::{PermissionsContainer, Permissions};
use deno_runtime::BootstrapOptions;
use mf_state::State;
use tokio::sync::{RwLock, Mutex};

use crate::error::{DenoError, DenoResult};
use crate::ops::create_moduforge_extension;
use crate::runtime::context::{ModuForgeContext, set_context_to_opstate};

/// MainWorker 配置
#[derive(Clone)]
pub struct MainWorkerConfig {
    pub extensions: Vec<Extension>,
    pub worker_options: WorkerOptions,
    pub bootstrap_options: BootstrapOptions,
    pub init_script: String,
}

impl Default for MainWorkerConfig {
    fn default() -> Self {
        let extensions = vec![create_moduforge_extension()];

        let worker_options = WorkerOptions::default();

        let bootstrap_options = BootstrapOptions::default();

        let init_script = r#"
            // ModuForge JavaScript API 初始化
            globalThis.ModuForge = {
                // 状态 API
                State: {
                    getVersion: () => Deno.core.ops.op_state_get_version(),
                    hasField: (name) => Deno.core.ops.op_state_has_field(name),
                    getField: (name) => Deno.core.ops.op_state_get_field(name),
                    getDoc: () => Deno.core.ops.op_state_get_doc(),
                    getSchema: () => Deno.core.ops.op_state_get_schema(),
                },

                // 事务 API
                Transaction: {
                    new: () => Deno.core.ops.op_transaction_new(),
                    setNodeAttribute: (trId, nodeId, attrs) =>
                        Deno.core.ops.op_transaction_set_node_attribute(trId, nodeId, JSON.stringify(attrs)),
                    addNode: (trId, parentId, nodes) =>
                        Deno.core.ops.op_transaction_add_node(trId, parentId, JSON.stringify(nodes)),
                    removeNode: (trId, parentId, nodeIds) =>
                        Deno.core.ops.op_transaction_remove_node(trId, parentId, JSON.stringify(nodeIds)),
                    setMeta: (trId, key, value) =>
                        Deno.core.ops.op_transaction_set_meta(trId, key, JSON.stringify(value)),
                    getMeta: (trId, key) => Deno.core.ops.op_transaction_get_meta(trId, key),
                },

                // 节点 API
                Node: {
                    getAttribute: (nodeId, attrName) =>
                        Deno.core.ops.op_node_get_attribute(nodeId, attrName),
                    getChildren: (nodeId) => Deno.core.ops.op_node_get_children(nodeId),
                    getParent: (nodeId) => Deno.core.ops.op_node_get_parent(nodeId),
                    findById: (nodeId) => Deno.core.ops.op_node_find_by_id(nodeId),
                    getInfo: (nodeId) => Deno.core.ops.op_node_get_info(nodeId),
                }
            };
        "#.to_string();

        Self {
            extensions: vec![create_moduforge_extension()],
            worker_options,
            bootstrap_options,
            init_script,
        }
    }
}

/// 运行时统计信息
#[derive(Debug, Clone, Default)]
pub struct RuntimeStats {
    pub workers_created: u64,
    pub plugin_executions: u64,
    pub total_execution_time: Duration,
    pub last_activity: Option<Instant>,
}

// 线程本地存储 MainWorker 实例
thread_local! {
    static MAIN_WORKER: std::cell::RefCell<Option<MainWorker>> = std::cell::RefCell::new(None);
}

/// 线程本地 MainWorker 管理器
/// 使用 MainWorker 提供完整的 Deno 功能
pub struct MainWorkerManager {
    /// MainWorker 配置
    config: MainWorkerConfig,

    /// 已加载的插件代码
    plugins: Arc<RwLock<HashMap<String, String>>>,

    /// 当前状态
    current_state: Arc<RwLock<Arc<State>>>,

    /// 统计信息
    stats: Arc<Mutex<RuntimeStats>>,
}

impl MainWorkerManager {
    /// 创建新的线程本地 MainWorker 管理器
    pub fn new(initial_state: Arc<State>) -> Self {
        Self {
            config: MainWorkerConfig::default(),
            plugins: Arc::new(RwLock::new(HashMap::new())),
            current_state: Arc::new(RwLock::new(initial_state)),
            stats: Arc::new(Mutex::new(RuntimeStats::default())),
        }
    }

    /// 获取或创建当前线程的 MainWorker 实例
    fn get_or_create_worker(&self) -> DenoResult<()> {
        MAIN_WORKER.with(|worker_cell| {
            let mut worker_opt = worker_cell.borrow_mut();

            if worker_opt.is_none() {
                tracing::debug!("Creating new MainWorker for thread {:?}", std::thread::current().id());

                // 创建权限容器
                let permissions = PermissionsContainer::new(Permissions::allow_all());

                // 创建主模块说明符
                let main_module = ModuleSpecifier::parse("file:///main.js")
                    .map_err(|e| DenoError::Runtime(anyhow::anyhow!("Invalid module specifier: {}", e)))?;

                // 创建 MainWorker
                let mut worker = MainWorker::bootstrap_from_options(
                    main_module,
                    self.config.worker_options.clone(),
                    self.config.bootstrap_options.clone(),
                );

                // 执行初始化脚本
                worker.execute_script("moduforge_init.js", FastString::from(self.config.init_script.clone()))
                    .map_err(|e| DenoError::JsExecution(format!("Failed to initialize MainWorker: {}", e)))?;

                *worker_opt = Some(worker);

                // 更新统计信息
                tokio::task::block_in_place(|| {
                    let handle = tokio::runtime::Handle::current();
                    handle.block_on(async {
                        let mut stats = self.stats.lock().await;
                        stats.workers_created += 1;
                        stats.last_activity = Some(Instant::now());
                    });
                });
            }

            Ok(())
        })
    }

    /// 在当前线程执行 JavaScript 代码
    fn execute_in_current_thread<F, R>(&self, f: F) -> DenoResult<R>
    where
        F: FnOnce(&mut MainWorker) -> DenoResult<R>,
    {
        self.get_or_create_worker()?;

        MAIN_WORKER.with(|worker_cell| {
            let mut worker_opt = worker_cell.borrow_mut();
            let worker = worker_opt.as_mut()
                .ok_or_else(|| DenoError::Runtime(anyhow::anyhow!("MainWorker not available")))?;

            f(worker)
        })
    }

    /// 加载插件
    pub async fn load_plugin(
        &self,
        plugin_id: String,
        plugin_code: String
    ) -> DenoResult<()> {
        // 存储插件代码
        let mut plugins = self.plugins.write().await;
        plugins.insert(plugin_id.clone(), plugin_code.clone());
        drop(plugins);

        // 在当前线程执行插件加载
        let current_state = self.current_state.read().await.clone();
        let plugin_id_clone = plugin_id.clone();

        tokio::task::spawn_blocking(move || {
            MAIN_WORKER.with(|worker_cell| {
                let mut worker_opt = worker_cell.borrow_mut();

                if worker_opt.is_none() {
                    // 创建 MainWorker
                    let permissions = PermissionsContainer::new(Permissions::allow_all());
                    let main_module = ModuleSpecifier::parse("file:///main.js").unwrap();

                    let worker_options = WorkerOptions {
                        extensions: vec![create_moduforge_extension()],
                        ..Default::default()
                    };

                    let bootstrap_options = BootstrapOptions::default();

                    let mut worker = MainWorker::bootstrap_from_options(
                        main_module,
                        worker_options,
                        bootstrap_options,
                    );

                    // 初始化脚本
                    let init_script = MainWorkerConfig::default().init_script;
                    worker.execute_script("moduforge_init.js", FastString::from(init_script))
                        .map_err(|e| DenoError::JsExecution(format!("Failed to initialize MainWorker: {}", e)))?;

                    *worker_opt = Some(worker);
                }

                let worker = worker_opt.as_mut().unwrap();

                // 设置插件上下文
                let context = ModuForgeContext::new(current_state, plugin_id_clone.clone());
                set_context_to_opstate(worker.js_runtime.op_state(), context);

                // 执行插件代码
                worker.execute_script(&plugin_id_clone, FastString::from(plugin_code))
                    .map_err(|e| DenoError::JsExecution(format!("Failed to execute plugin {}: {}", plugin_id_clone, e)))?;

                Ok::<(), DenoError>(())
            })
        }).await
        .map_err(|e| DenoError::Runtime(anyhow::anyhow!("Task join error: {}", e)))??;

        tracing::info!("Plugin {} loaded successfully", plugin_id);
        Ok(())
    }

    /// 执行插件方法
    pub async fn execute_plugin_method(
        &self,
        plugin_id: &str,
        method_name: &str,
        args: serde_json::Value,
    ) -> DenoResult<serde_json::Value> {
        let start_time = Instant::now();

        // 获取插件代码
        let plugins = self.plugins.read().await;
        let plugin_code = plugins.get(plugin_id)
            .ok_or_else(|| DenoError::PluginNotFound(plugin_id.to_string()))?
            .clone();
        drop(plugins);

        let current_state = self.current_state.read().await.clone();
        let plugin_id = plugin_id.to_string();
        let method_name = method_name.to_string();

        // 在阻塞任务中执行 JavaScript
        let result = tokio::task::spawn_blocking(move || {
            MAIN_WORKER.with(|worker_cell| {
                let mut worker_opt = worker_cell.borrow_mut();

                if worker_opt.is_none() {
                    // 创建 MainWorker
                    let permissions = PermissionsContainer::new(Permissions::allow_all());
                    let main_module = ModuleSpecifier::parse("file:///main.js").unwrap();

                    let worker_options = WorkerOptions {
                        extensions: vec![create_moduforge_extension()],
                        ..Default::default()
                    };

                    let bootstrap_options = BootstrapOptions::default();

                    let mut worker = MainWorker::bootstrap_from_options(
                        main_module,
                        worker_options,
                        bootstrap_options,
                    );

                    // 初始化脚本
                    let init_script = MainWorkerConfig::default().init_script;
                    worker.execute_script("moduforge_init.js", FastString::from(init_script))
                        .map_err(|e| DenoError::JsExecution(format!("Failed to initialize MainWorker: {}", e)))?;

                    *worker_opt = Some(worker);
                }

                let worker = worker_opt.as_mut().unwrap();

                // 设置插件上下文
                let context = ModuForgeContext::new(current_state, plugin_id.clone());
                set_context_to_opstate(worker.js_runtime.op_state(), context);

                // 重新加载插件代码
                worker.execute_script(&plugin_id, FastString::from(plugin_code))
                    .map_err(|e| DenoError::JsExecution(format!("Failed to reload plugin {}: {}", plugin_id, e)))?;

                // 构造调用脚本
                let call_script = format!(
                    r#"
                    (() => {{
                        if (typeof {} === 'function') {{
                            return {}({});
                        }} else {{
                            throw new Error('Method {} not found');
                        }}
                    }})()
                    "#,
                    method_name, method_name, args, method_name
                );

                // 执行方法调用
                let result = worker.execute_script("plugin_call", FastString::from(call_script))
                    .map_err(|e| DenoError::JsExecution(format!("Failed to call method {}: {}", method_name, e)))?;

                // 转换结果
                let result_json = serde_json::from_str(&result.to_string())
                    .unwrap_or(serde_json::Value::Null);

                Ok::<serde_json::Value, DenoError>(result_json)
            })
        }).await
        .map_err(|e| DenoError::Runtime(anyhow::anyhow!("Task join error: {}", e)))??;

        // 更新统计信息
        let execution_time = start_time.elapsed();
        let mut stats = self.stats.lock().await;
        stats.plugin_executions += 1;
        stats.total_execution_time += execution_time;
        stats.last_activity = Some(Instant::now());

        Ok(result)
    }

    /// 卸载插件
    pub async fn unload_plugin(&self, plugin_id: &str) -> DenoResult<()> {
        let mut plugins = self.plugins.write().await;

        if plugins.remove(plugin_id).is_some() {
            tracing::info!("Plugin {} unloaded successfully", plugin_id);
            Ok(())
        } else {
            Err(DenoError::PluginNotFound(plugin_id.to_string()))
        }
    }

    /// 更新状态
    pub async fn update_state(&self, new_state: Arc<State>) {
        let mut current_state = self.current_state.write().await;
        *current_state = new_state;

        tracing::debug!("MainWorker manager state updated");
    }

    /// 获取已加载的插件列表
    pub async fn list_plugins(&self) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins.keys().cloned().collect()
    }

    /// 获取运行时统计信息
    pub async fn get_stats(&self) -> RuntimeStats {
        let stats = self.stats.lock().await;
        stats.clone()
    }

    /// 清理线程本地 MainWorker（当线程结束时调用）
    pub fn cleanup_thread_worker() {
        MAIN_WORKER.with(|worker_cell| {
            let mut worker_opt = worker_cell.borrow_mut();
            if worker_opt.is_some() {
                tracing::debug!("Cleaning up MainWorker for thread {:?}", std::thread::current().id());
                *worker_opt = None;
            }
        });
    }

    /// 获取详细的监控信息
    pub async fn get_metrics(&self) -> serde_json::Value {
        let stats = self.stats.lock().await;
        let plugins = self.plugins.read().await;

        serde_json::json!({
            "total_workers_created": stats.workers_created,
            "total_plugin_executions": stats.plugin_executions,
            "average_execution_time_ms": if stats.plugin_executions > 0 {
                stats.total_execution_time.as_millis() as f64 / stats.plugin_executions as f64
            } else {
                0.0
            },
            "loaded_plugins_count": plugins.len(),
            "loaded_plugins": plugins.keys().cloned().collect::<Vec<_>>(),
            "last_activity": stats.last_activity.map(|t| t.elapsed().as_secs()),
            "architecture": "main_worker"
        })
    }
}

impl Drop for MainWorkerManager {
    fn drop(&mut self) {
        tracing::info!("MainWorkerManager dropped");
    }
}

// 提供清理函数，在线程结束时调用
pub fn cleanup_current_thread_worker() {
    MainWorkerManager::cleanup_thread_worker();
}