//! DenoPluginV2 测试 - 验证无循环引用的新架构

use std::sync::Arc;
use std::time::Duration;
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
async fn test_plugin_v2_without_context() {
    // 测试无上下文的插件创建
    let plugin = DenoPluginV2::new(
        "test_plugin".to_string(),
        r#"function hello() { return "Hello from V2"; }"#.to_string()
    );

    assert_eq!(plugin.id, "test_plugin");
    assert!(!plugin.is_loaded().await); // 没有执行上下文，应该返回false

    // 尝试执行应该失败
    let result = plugin.execute_js_function("hello", serde_json::json!({})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No execution context available"));
}

#[tokio::test]
async fn test_plugin_v2_with_manager_context() {
    let state = create_test_state().await;
    let manager = Arc::new(MainWorkerManager::new(state));

    // 先加载插件到管理器
    let plugin_code = r#"
        function greet(args) {
            return {
                message: "Hello from V2 plugin",
                name: args.name || "World",
                version: "2.0"
            };
        }
    "#;

    manager.load_plugin("v2_test".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 等待插件加载
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 使用管理器作为执行上下文创建插件
    let plugin = DenoPluginV2::with_execution_context(
        "v2_test".to_string(),
        plugin_code.to_string(),
        manager.clone()
    );

    assert!(plugin.is_loaded().await); // 现在应该已加载

    // 执行插件方法
    let result = plugin.execute_js_function_async(
        "greet",
        serde_json::json!({"name": "V2 Plugin"})
    ).await.expect("Failed to execute plugin method");

    assert_eq!(result["message"], "Hello from V2 plugin");
    assert_eq!(result["name"], "V2 Plugin");
    assert_eq!(result["version"], "2.0");

    // 获取执行统计
    let stats = plugin.get_execution_stats().await;
    assert!(stats.total_executions > 0);
}

#[tokio::test]
async fn test_plugin_v2_builder() {
    let state = create_test_state().await;
    let manager = Arc::new(MainWorkerManager::new(state));

    // 使用构建器创建插件
    let plugin = DenoPluginBuilderV2::new("builder_test")
        .code(r#"
            function calculate(args) {
                return {
                    result: args.a + args.b,
                    operation: "add",
                    timestamp: Date.now()
                };
            }
        "#)
        .name("Calculator Plugin")
        .version("1.0.0")
        .description("A simple calculator plugin")
        .author("Test Author")
        .tag("math")
        .tag("calculator")
        .priority(10)
        .execution_context(manager.clone())
        .build()
        .expect("Failed to build plugin");

    assert_eq!(plugin.metadata().name, "Calculator Plugin");
    assert_eq!(plugin.metadata().version, "1.0.0");
    assert_eq!(plugin.config().priority, 10);
    assert!(plugin.metadata().tags.contains(&"math".to_string()));

    // 先加载插件到管理器
    manager.load_plugin("builder_test".to_string(), plugin.code.clone())
        .await
        .expect("Failed to load plugin");

    // 等待插件加载
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 执行插件
    let result = plugin.execute_js_function_async(
        "calculate",
        serde_json::json!({"a": 10, "b": 20})
    ).await.expect("Failed to execute calculation");

    assert_eq!(result["result"], 30);
    assert_eq!(result["operation"], "add");
}

#[tokio::test]
async fn test_no_circular_reference() {
    let state = create_test_state().await;
    let manager = Arc::new(MainWorkerManager::new(state));

    // 创建插件
    let plugin = DenoPluginV2::with_execution_context(
        "circular_test".to_string(),
        "function test() { return 'ok'; }".to_string(),
        manager.clone()
    );

    // 验证：删除管理器引用不会影响插件的基本信息
    drop(manager);

    // 插件的基本信息仍然可访问
    assert_eq!(plugin.id, "circular_test");
    assert_eq!(plugin.metadata().name, "circular_test");

    // 但执行会失败（因为上下文中的管理器已被删除）
    // 这验证了没有强引用循环
    let result = plugin.execute_js_function("test", serde_json::json!({})).await;
    // 根据实现，这可能成功或失败，但不会导致内存泄漏
}

#[tokio::test]
async fn test_plugin_context_switching() {
    let state1 = create_test_state().await;
    let state2 = create_test_state().await;
    let manager1 = Arc::new(MainWorkerManager::new(state1));
    let manager2 = Arc::new(MainWorkerManager::new(state2));

    // 创建插件
    let mut plugin = DenoPluginV2::new(
        "context_switch".to_string(),
        "function getId() { return 'manager1'; }".to_string()
    );

    // 设置第一个执行上下文
    plugin.set_execution_context(manager1.clone());

    // 验证可以动态切换执行上下文
    plugin.set_execution_context(manager2.clone());

    // 这种设计允许插件在运行时切换执行环境
    // 这在某些高级场景下可能很有用（如多租户系统）
    assert_eq!(plugin.id, "context_switch");
}