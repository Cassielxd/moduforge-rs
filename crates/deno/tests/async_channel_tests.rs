//! 异步通道通信测试
//! 验证 Deno 运行时与 ModuForge 插件系统的异步请求-响应机制

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
async fn test_async_channel_manager_creation() {
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
    assert_eq!(metrics["architecture"], "main_worker_channel");

    // 验证通道管理器可用
    assert!(manager.get_channel_manager().is_some());
}

#[tokio::test]
async fn test_async_plugin_execution() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new(state);

    let plugin_code = r#"
        // 简单的数学计算插件
        function calculateSum(args) {
            const numbers = args.numbers || [];
            const sum = numbers.reduce((a, b) => a + b, 0);
            return {
                sum: sum,
                count: numbers.length,
                runtime: "MainWorker-Channel",
                timestamp: Date.now()
            };
        }

        // 异步计算插件
        async function asyncCalculation(args) {
            // 模拟异步操作
            await new Promise(resolve => setTimeout(resolve, 10));

            const value = args.value || 0;
            return {
                result: value * 2,
                async: true,
                timestamp: Date.now()
            };
        }

        // 错误测试插件
        function throwError(args) {
            throw new Error("Test error from async plugin");
        }
    "#;

    // 加载插件
    manager.load_plugin("async-test".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 等待一小段时间确保插件和请求处理器启动
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 测试同步方法的异步执行
    let result = manager.execute_plugin_method_async(
        "async-test",
        "calculateSum",
        serde_json::json!({"numbers": [1, 2, 3, 4, 5]})
    ).await.expect("Failed to execute calculateSum");

    assert_eq!(result["sum"], 15);
    assert_eq!(result["count"], 5);
    assert_eq!(result["runtime"], "MainWorker-Channel");

    // 测试异步方法的异步执行
    let async_result = manager.execute_plugin_method_async(
        "async-test",
        "asyncCalculation",
        serde_json::json!({"value": 21})
    ).await.expect("Failed to execute asyncCalculation");

    assert_eq!(async_result["result"], 42);
    assert_eq!(async_result["async"], true);

    // 验证统计信息更新
    let stats = manager.get_stats().await;
    assert!(stats.plugin_executions >= 2);
    assert!(stats.total_execution_time > Duration::from_millis(0));
}

#[tokio::test]
async fn test_async_error_handling() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new(state);

    let plugin_code = r#"
        function throwError(args) {
            throw new Error("Test async error");
        }

        function validFunction(args) {
            return { success: true, message: "Valid execution" };
        }
    "#;

    // 加载插件
    manager.load_plugin("error-test".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 等待插件加载
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 测试错误处理
    let error_result = manager.execute_plugin_method_async(
        "error-test",
        "throwError",
        serde_json::json!({})
    ).await;

    assert!(error_result.is_err());
    if let Err(e) = error_result {
        assert!(e.to_string().contains("Test async error"));
    }

    // 测试调用不存在的方法
    let missing_method_result = manager.execute_plugin_method_async(
        "error-test",
        "nonExistentMethod",
        serde_json::json!({})
    ).await;

    assert!(missing_method_result.is_err());

    // 测试正常方法仍然工作
    let valid_result = manager.execute_plugin_method_async(
        "error-test",
        "validFunction",
        serde_json::json!({})
    ).await.expect("Valid function should work");

    assert_eq!(valid_result["success"], true);
    assert_eq!(valid_result["message"], "Valid execution");
}

#[tokio::test]
async fn test_async_timeout_functionality() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new(state);

    let plugin_code = r#"
        function quickTask(args) {
            return { completed: true, time: "quick" };
        }

        async function slowTask(args) {
            // 模拟慢任务（超过超时时间）
            await new Promise(resolve => setTimeout(resolve, 200));
            return { completed: true, time: "slow" };
        }
    "#;

    // 加载插件
    manager.load_plugin("timeout-test".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 等待插件加载
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 测试快速任务（正常执行）
    let quick_result = manager.execute_plugin_method_async_with_timeout(
        "timeout-test",
        "quickTask",
        serde_json::json!({}),
        1000, // 1秒超时
    ).await.expect("Quick task should complete");

    assert_eq!(quick_result["completed"], true);
    assert_eq!(quick_result["time"], "quick");

    // 测试慢任务（应该超时）
    let slow_result = manager.execute_plugin_method_async_with_timeout(
        "timeout-test",
        "slowTask",
        serde_json::json!({}),
        100, // 100ms超时
    ).await;

    assert!(slow_result.is_err());
    if let Err(e) = slow_result {
        assert!(e.to_string().contains("timeout") || e.to_string().contains("Request timeout"));
    }
}

#[tokio::test]
async fn test_concurrent_async_executions() {
    let state = create_test_state().await;
    let manager = Arc::new(MainWorkerManager::new(state));

    let plugin_code = r#"
        async function concurrentTask(args) {
            const taskId = args.taskId || 0;
            // 模拟一些异步工作
            await new Promise(resolve => setTimeout(resolve, Math.random() * 50));

            return {
                taskId: taskId,
                completed: true,
                timestamp: Date.now(),
                threadInfo: "main_worker_channel"
            };
        }
    "#;

    // 加载插件
    manager.load_plugin("concurrent-test".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 等待插件加载
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 并发执行多个任务
    let mut tasks = Vec::new();
    for i in 0..10 {
        let manager_clone = manager.clone();
        let task = tokio::spawn(async move {
            manager_clone.execute_plugin_method_async(
                "concurrent-test",
                "concurrentTask",
                serde_json::json!({"taskId": i})
            ).await
        });
        tasks.push(task);
    }

    // 等待所有任务完成
    let mut results = Vec::new();
    for task in tasks {
        let result = task.await.expect("Task should complete")
            .expect("Execution should succeed");
        results.push(result);
    }

    // 验证所有任务都成功完成
    assert_eq!(results.len(), 10);
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result["taskId"], i);
        assert_eq!(result["completed"], true);
        assert_eq!(result["threadInfo"], "main_worker_channel");
    }

    // 验证统计信息
    let stats = manager.get_stats().await;
    assert!(stats.plugin_executions >= 10);
}

#[tokio::test]
async fn test_channel_manager_direct_usage() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new(state);

    let plugin_code = r#"
        function directChannelTest(args) {
            return {
                message: "Direct channel communication works",
                input: args,
                channelType: "request-response"
            };
        }
    "#;

    // 加载插件
    manager.load_plugin("direct-test".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 等待插件加载
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 直接使用通道管理器
    let channel_manager = manager.get_channel_manager()
        .expect("Channel manager should be available");

    let response = channel_manager.send_request(
        "direct-test".to_string(),
        "directChannelTest".to_string(),
        serde_json::json!({"test": "direct_channel"})
    ).await.expect("Direct channel request should succeed");

    assert!(response.success);
    assert!(response.result.is_some());

    let result = response.result.unwrap();
    assert_eq!(result["message"], "Direct channel communication works");
    assert_eq!(result["channelType"], "request-response");
    assert_eq!(result["input"]["test"], "direct_channel");
}

#[tokio::test]
async fn test_multiple_plugins_async() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new(state);

    // 插件 1: 数学运算
    let math_plugin = r#"
        function multiply(args) {
            return { result: args.a * args.b, plugin: "math" };
        }
    "#;

    // 插件 2: 字符串处理
    let string_plugin = r#"
        function reverse(args) {
            return {
                result: args.text.split("").reverse().join(""),
                plugin: "string"
            };
        }
    "#;

    // 加载两个插件
    manager.load_plugin("math".to_string(), math_plugin.to_string())
        .await
        .expect("Failed to load math plugin");

    manager.load_plugin("string".to_string(), string_plugin.to_string())
        .await
        .expect("Failed to load string plugin");

    // 等待插件加载
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 验证插件列表
    let plugins = manager.list_plugins().await;
    assert_eq!(plugins.len(), 2);
    assert!(plugins.contains(&"math".to_string()));
    assert!(plugins.contains(&"string".to_string()));

    // 并发执行不同插件的方法
    let math_task = manager.execute_plugin_method_async(
        "math",
        "multiply",
        serde_json::json!({"a": 6, "b": 7})
    );

    let string_task = manager.execute_plugin_method_async(
        "string",
        "reverse",
        serde_json::json!({"text": "hello"})
    );

    let (math_result, string_result) = tokio::join!(math_task, string_task);

    let math_result = math_result.expect("Math plugin should work");
    let string_result = string_result.expect("String plugin should work");

    assert_eq!(math_result["result"], 42);
    assert_eq!(math_result["plugin"], "math");

    assert_eq!(string_result["result"], "olleh");
    assert_eq!(string_result["plugin"], "string");
}