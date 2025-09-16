//! ModuForge Deno Integration Library
//!
//! 提供 Deno 运行时集成，允许使用 JavaScript/TypeScript 编写插件
//! 通过 Deno Op 系统实现零序列化的数据传递

pub mod error;
pub mod plugin;

// 条件编译：根据是否启用 deno_core 来选择实现
pub mod ops;
pub mod runtime;
pub mod integration;


pub use error::{DenoError, DenoResult};
pub use plugin::{DenoPlugin, DenoPluginBuilder};

pub use integration::{ModuForgeDeno, add_deno_plugins_to_state_config, create_sample_plugin_code};
pub use runtime::{DenoPluginManager, ModuForgeContext, RuntimePoolStats, MainWorkerManager};


// 重新导出，提供统一的接口
pub type PluginManager = DenoPluginManager;


/// 创建插件管理器的便捷函数
pub fn create_plugin_manager(initial_state: std::sync::Arc<mf_state::State>, pool_size: usize) -> PluginManager {
    DenoPluginManager::new(initial_state, pool_size)
}

/// 创建 MainWorker 管理器的便捷函数
pub fn create_main_worker_manager(initial_state: std::sync::Arc<mf_state::State>) -> MainWorkerManager {
    MainWorkerManager::new(initial_state)
}



/// 获取示例插件代码
pub fn get_sample_plugin_code() -> &'static str {
    r#"
// ModuForge 示例插件
function appendTransaction(args) {
    console.log('Plugin appendTransaction called:', args);
    return null;
}

function filterTransaction(args) {
    console.log('Plugin filterTransaction called:', args);
    return true;
}

function processData(args) {
    return {
        message: "Hello from plugin",
        input: args,
        timestamp: Date.now()
    };
}

console.log('Sample plugin loaded successfully');
"#
}