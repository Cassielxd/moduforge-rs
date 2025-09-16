use std::sync::Arc;
use mf_deno::*;

#[tokio::test]
async fn test_mock_plugin_manager() {
    let manager = create_plugin_manager(
        Arc::new(unsafe { std::mem::zeroed() }), // 模拟状态，在真实测试中需要创建真实状态
        2
    );

    // 测试初始化
    manager.initialize_pool().await.expect("Failed to initialize pool");

    let plugin_code = get_sample_plugin_code();

    // 测试加载插件
    manager.load_plugin("test-plugin".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 测试插件列表
    let plugins = manager.list_plugins().await;
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0], "test-plugin");

    // 测试方法执行
    let result = manager.execute_plugin_method(
        "test-plugin",
        "processData",
        serde_json::json!({"test": "data"})
    ).await.expect("Failed to execute method");

    println!("Method execution result: {}", serde_json::to_string_pretty(&result).unwrap());

    // 测试卸载插件
    manager.unload_plugin("test-plugin").await.expect("Failed to unload plugin");

    let plugins = manager.list_plugins().await;
    assert_eq!(plugins.len(), 0);

    manager.shutdown().await;
}

#[tokio::test]
async fn test_plugin_creation() {
    let plugin = DenoPlugin::new("test".to_string(), "console.log('test');".to_string());

    assert_eq!(plugin.id, "test");
    assert_eq!(plugin.code, "console.log('test');");

    let metadata = plugin.metadata();
    assert_eq!(metadata.name, "test");
    assert!(metadata.tags.contains(&"deno".to_string()));

    let config = plugin.config();
    assert!(config.enabled);
    assert_eq!(config.priority, 0);
}

#[tokio::test]
async fn test_plugin_builder() {
    let plugin = DenoPluginBuilder::new("builder-test")
        .code("function test() { return true; }")
        .priority(5)
        .enabled(false)
        .build()
        .expect("Failed to build plugin");

    assert_eq!(plugin.id, "builder-test");
    assert_eq!(plugin.config.priority, 5);
    assert!(!plugin.config.enabled);
}
