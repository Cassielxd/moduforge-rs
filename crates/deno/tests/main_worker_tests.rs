//! MainWorker 集成测试
//! 验证 MainWorker 替代 JsRuntime 的功能

use std::sync::Arc;
use std::time::Duration;
use tokio_test;
use mf_deno::*;
use moduforge_state::{State, StateConfig};
use moduforge_model::schema::Schema;

/// 创建测试用的基础 Schema 和 State
async fn create_test_state() -> Arc<State> {
    let schema = Arc::new(Schema::default());

    let config = StateConfig {
        schema: Some(schema),
        doc: None,
        stored_marks: None,
        plugins: None,
        resource_manager: None,
    };

    let state = State::create(config).await.expect("Failed to create test state");
    Arc::new(state)
}

#[tokio::test]
async fn test_main_worker_manager_creation() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new(state);

    // 验证管理器创建成功
    let stats = manager.get_stats().await;
    assert_eq!(stats.workers_created, 0);
    assert_eq!(stats.plugin_executions, 0);

    // 获取监控信息
    let metrics = manager.get_metrics().await;
    assert_eq!(metrics["total_workers_created"], 0);
    assert_eq!(metrics["total_plugin_executions"], 0);
    assert_eq!(metrics["architecture"], "main_worker");
}

#[tokio::test]
async fn test_main_worker_plugin_loading() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new(state);

    let plugin_code = r#"
        function testMethod(args) {
            return {
                message: "MainWorker test",
                input: args,
                timestamp: Date.now(),
                deno_version: Deno.version.deno
            };
        }
    "#;

    // 加载插件
    manager.load_plugin("main-worker-test".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 验证插件已加载
    let plugins = manager.list_plugins().await;
    assert!(plugins.contains(&"main-worker-test".to_string()));

    // 验证统计信息更新
    let stats = manager.get_stats().await;
    assert!(stats.workers_created > 0);
}

#[tokio::test]
async fn test_main_worker_plugin_execution() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new(state);

    let plugin_code = r#"
        function calculateSum(args) {
            const numbers = args.numbers || [];
            const sum = numbers.reduce((a, b) => a + b, 0);
            return {
                sum: sum,
                count: numbers.length,
                runtime: "MainWorker",
                hasConsole: typeof console !== 'undefined',
                hasDeno: typeof Deno !== 'undefined'
            };
        }

        function getSystemInfo(args) {
            return {
                worker_type: "MainWorker",
                has_web_apis: typeof fetch !== 'undefined',
                has_timers: typeof setTimeout !== 'undefined',
                deno_version: Deno.version?.deno || "unknown"
            };
        }
    "#;

    // 加载插件
    manager.load_plugin("math-plugin".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 执行插件方法 - 数学计算
    let result = manager.execute_plugin_method(
        "math-plugin",
        "calculateSum",
        serde_json::json!({"numbers": [1, 2, 3, 4, 5]})
    ).await.expect("Failed to execute plugin method");

    assert_eq!(result["sum"], 15);
    assert_eq!(result["count"], 5);
    assert_eq!(result["runtime"], "MainWorker");
    assert_eq!(result["hasConsole"], true);
    assert_eq!(result["hasDeno"], true);

    // 执行插件方法 - 系统信息
    let system_result = manager.execute_plugin_method(
        "math-plugin",
        "getSystemInfo",
        serde_json::json!({})
    ).await.expect("Failed to execute getSystemInfo");

    assert_eq!(system_result["worker_type"], "MainWorker");
    assert_eq!(system_result["has_web_apis"], true); // MainWorker 包含 Web APIs
    assert_eq!(system_result["has_timers"], true);

    // 验证统计信息
    let stats = manager.get_stats().await;
    assert!(stats.plugin_executions >= 2);
    assert!(stats.total_execution_time > Duration::from_millis(0));
}

#[tokio::test]
async fn test_main_worker_deno_apis() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new(state);

    let plugin_code = r#"
        function testDenoAPIs(args) {
            const result = {
                has_deno: typeof Deno !== 'undefined',
                deno_version: null,
                has_console: typeof console !== 'undefined',
                has_global_this: typeof globalThis !== 'undefined',
                has_fetch: typeof fetch !== 'undefined',
                has_url: typeof URL !== 'undefined',
                has_text_encoder: typeof TextEncoder !== 'undefined',
                has_crypto: typeof crypto !== 'undefined'
            };

            if (typeof Deno !== 'undefined') {
                result.deno_version = Deno.version?.deno;
                result.deno_ops_available = typeof Deno.core?.ops === 'object';
            }

            return result;
        }

        function testModuForgeAPIs(args) {
            return {
                has_moduforge: typeof globalThis.ModuForge !== 'undefined',
                has_state_api: typeof globalThis.ModuForge?.State !== 'undefined',
                has_transaction_api: typeof globalThis.ModuForge?.Transaction !== 'undefined',
                has_node_api: typeof globalThis.ModuForge?.Node !== 'undefined'
            };
        }
    "#;

    // 加载插件
    manager.load_plugin("api-test".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 测试 Deno APIs
    let deno_result = manager.execute_plugin_method(
        "api-test",
        "testDenoAPIs",
        serde_json::json!({})
    ).await.expect("Failed to execute testDenoAPIs");

    assert_eq!(deno_result["has_deno"], true);
    assert_eq!(deno_result["has_console"], true);
    assert_eq!(deno_result["has_global_this"], true);
    assert_eq!(deno_result["has_fetch"], true); // MainWorker 包含 fetch API
    assert_eq!(deno_result["has_url"], true);
    assert_eq!(deno_result["has_text_encoder"], true);
    assert_eq!(deno_result["has_crypto"], true);
    assert!(deno_result["deno_version"].as_str().is_some());

    // 测试 ModuForge APIs
    let moduforge_result = manager.execute_plugin_method(
        "api-test",
        "testModuForgeAPIs",
        serde_json::json!({})
    ).await.expect("Failed to execute testModuForgeAPIs");

    assert_eq!(moduforge_result["has_moduforge"], true);
    assert_eq!(moduforge_result["has_state_api"], true);
    assert_eq!(moduforge_result["has_transaction_api"], true);
    assert_eq!(moduforge_result["has_node_api"], true);
}

#[tokio::test]
async fn test_main_worker_error_handling() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new(state);

    let plugin_code = r#"
        function throwError(args) {
            throw new Error("Test error from MainWorker");
        }

        function validFunction(args) {
            return { success: true };
        }
    "#;

    // 加载插件
    manager.load_plugin("error-test".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 测试错误处理
    let error_result = manager.execute_plugin_method(
        "error-test",
        "throwError",
        serde_json::json!({})
    ).await;

    assert!(error_result.is_err());

    // 测试调用不存在的方法
    let missing_method_result = manager.execute_plugin_method(
        "error-test",
        "nonExistentMethod",
        serde_json::json!({})
    ).await;

    assert!(missing_method_result.is_err());

    // 测试正常方法仍然工作
    let valid_result = manager.execute_plugin_method(
        "error-test",
        "validFunction",
        serde_json::json!({})
    ).await.expect("Valid function should work");

    assert_eq!(valid_result["success"], true);
}

#[tokio::test]
async fn test_main_worker_multiple_plugins() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new(state);

    // 插件 1: 数学运算
    let math_plugin = r#"
        function add(args) {
            return args.a + args.b;
        }
    "#;

    // 插件 2: 字符串处理
    let string_plugin = r#"
        function concatenate(args) {
            return args.strings.join(args.separator || "");
        }
    "#;

    // 加载多个插件
    manager.load_plugin("math".to_string(), math_plugin.to_string())
        .await
        .expect("Failed to load math plugin");

    manager.load_plugin("string".to_string(), string_plugin.to_string())
        .await
        .expect("Failed to load string plugin");

    // 验证插件列表
    let plugins = manager.list_plugins().await;
    assert_eq!(plugins.len(), 2);
    assert!(plugins.contains(&"math".to_string()));
    assert!(plugins.contains(&"string".to_string()));

    // 执行数学插件
    let math_result = manager.execute_plugin_method(
        "math",
        "add",
        serde_json::json!({"a": 10, "b": 20})
    ).await.expect("Failed to execute math plugin");

    assert_eq!(math_result, 30);

    // 执行字符串插件
    let string_result = manager.execute_plugin_method(
        "string",
        "concatenate",
        serde_json::json!({"strings": ["Hello", " ", "MainWorker"], "separator": ""})
    ).await.expect("Failed to execute string plugin");

    assert_eq!(string_result, "Hello MainWorker");

    // 卸载一个插件
    manager.unload_plugin("math").await.expect("Failed to unload math plugin");

    let plugins_after_unload = manager.list_plugins().await;
    assert_eq!(plugins_after_unload.len(), 1);
    assert!(plugins_after_unload.contains(&"string".to_string()));
    assert!(!plugins_after_unload.contains(&"math".to_string()));
}

#[tokio::test]
async fn test_main_worker_performance_metrics() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new(state);

    let plugin_code = r#"
        function quickTask(args) {
            return { completed: true, timestamp: Date.now() };
        }

        function slowTask(args) {
            // 模拟一些计算
            let sum = 0;
            for (let i = 0; i < 1000; i++) {
                sum += i;
            }
            return { sum: sum, completed: true };
        }
    "#;

    manager.load_plugin("perf-test".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load performance test plugin");

    // 执行多次任务
    for _ in 0..5 {
        manager.execute_plugin_method(
            "perf-test",
            "quickTask",
            serde_json::json!({})
        ).await.expect("Failed to execute quick task");

        manager.execute_plugin_method(
            "perf-test",
            "slowTask",
            serde_json::json!({})
        ).await.expect("Failed to execute slow task");
    }

    // 检查性能指标
    let metrics = manager.get_metrics().await;
    assert_eq!(metrics["total_plugin_executions"], 10);
    assert!(metrics["average_execution_time_ms"].as_f64().unwrap() > 0.0);

    let stats = manager.get_stats().await;
    assert_eq!(stats.plugin_executions, 10);
    assert!(stats.total_execution_time > Duration::from_millis(0));
    assert!(stats.last_activity.is_some());
}