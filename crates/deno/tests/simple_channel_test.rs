//! 简单的通道测试 - 验证基本的异步通信功能

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
async fn test_channel_manager_creation() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new(state);

    // 验证通道管理器可用
    assert!(manager.get_channel_manager().is_some());

    // 获取监控信息，确认是通道版本
    let metrics = manager.get_metrics().await;
    assert_eq!(metrics["architecture"], "main_worker_channel");
}

#[tokio::test]
async fn test_simple_async_execution() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new(state);

    let plugin_code = r#"
        function hello(args) {
            return {
                message: "Hello from async channel",
                input: args,
                timestamp: Date.now()
            };
        }
    "#;

    // 加载插件
    manager.load_plugin("test".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 等待插件和请求处理器初始化
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 异步执行插件方法
    let result = manager.execute_plugin_method_async(
        "test",
        "hello",
        serde_json::json!({"name": "channel_test"})
    ).await.expect("Failed to execute async method");

    assert_eq!(result["message"], "Hello from async channel");
    assert_eq!(result["input"]["name"], "channel_test");
}

#[tokio::test]
async fn test_without_channel_compatibility() {
    let state = create_test_state().await;
    let manager = MainWorkerManager::new_without_channel(state);

    // 验证通道管理器不可用
    assert!(manager.get_channel_manager().is_none());

    // 获取监控信息，确认是普通版本
    let metrics = manager.get_metrics().await;
    assert_eq!(metrics["architecture"], "main_worker");

    // 异步方法应该返回错误
    let result = manager.execute_plugin_method_async(
        "test",
        "hello",
        serde_json::json!({})
    ).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Channel manager not available"));
}