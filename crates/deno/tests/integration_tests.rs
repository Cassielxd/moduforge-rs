use std::sync::Arc;
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
async fn test_deno_plugin_manager_creation() {
    let state = create_test_state().await;
    let manager = DenoPluginManager::new(state, 2);

    // 测试初始化
    manager.initialize_pool().await.expect("Failed to initialize pool");

    // 测试获取运行时
    let runtime = manager.get_runtime().await.expect("Failed to get runtime");

    // 归还运行时
    manager.return_runtime(runtime).await;

    // 关闭管理器
    manager.shutdown().await;
}

#[tokio::test]
async fn test_load_simple_plugin() {
    let state = create_test_state().await;
    let manager = DenoPluginManager::new(state, 1);
    manager.initialize_pool().await.expect("Failed to initialize pool");

    let plugin_code = r#"
        function appendTransaction(args) {
            console.log("Plugin called with:", args);
            return null;
        }

        function filterTransaction(args) {
            return true;
        }
    "#;

    // 测试加载插件
    manager.load_plugin("test-plugin".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 测试插件列表
    let plugins = manager.list_plugins().await;
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0], "test-plugin");

    // 测试卸载插件
    manager.unload_plugin("test-plugin").await.expect("Failed to unload plugin");

    let plugins = manager.list_plugins().await;
    assert_eq!(plugins.len(), 0);

    manager.shutdown().await;
}

#[tokio::test]
async fn test_plugin_method_execution() {
    let state = create_test_state().await;
    let manager = DenoPluginManager::new(state, 1);
    manager.initialize_pool().await.expect("Failed to initialize pool");

    let plugin_code = r#"
        function testMethod(args) {
            return {
                message: "Hello from plugin",
                args: args,
                timestamp: Date.now()
            };
        }
    "#;

    manager.load_plugin("test-plugin".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 测试方法执行
    let args = serde_json::json!({"test": "data"});
    let result = manager.execute_plugin_method("test-plugin", "testMethod", args)
        .await
        .expect("Failed to execute plugin method");

    // 验证结果
    assert!(result.is_object());
    assert_eq!(result["message"], "Hello from plugin");
    assert_eq!(result["args"]["test"], "data");

    manager.shutdown().await;
}

#[tokio::test]
async fn test_deno_plugin_trait() {
    let state = create_test_state().await;
    let manager = Arc::new(DenoPluginManager::new(state.clone(), 1));
    manager.initialize_pool().await.expect("Failed to initialize pool");

    let plugin_code = r#"
        function appendTransaction(args) {
            console.log("appendTransaction called:", args);
            return null;
        }

        function filterTransaction(args) {
            console.log("filterTransaction called:", args);
            return args.transactionId % 2 === 0; // 只允许偶数 ID 的事务
        }
    "#;

    // 创建 Deno 插件
    let deno_plugin = DenoPlugin::new("test-plugin".to_string(), plugin_code.to_string())
        .with_manager(manager.clone());

    // 加载到管理器
    manager.load_plugin("test-plugin".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 测试插件元数据
    let metadata = deno_plugin.metadata();
    assert_eq!(metadata.name, "test-plugin");
    assert!(metadata.tags.contains(&"deno".to_string()));

    // 测试插件配置
    let config = deno_plugin.config();
    assert!(config.enabled);
    assert_eq!(config.priority, 0);

    // 测试事务过滤（这里简化测试，实际需要真实的 Transaction）
    // 在实际使用中，需要创建真实的 Transaction 对象进行测试

    manager.shutdown().await;
}

#[tokio::test]
async fn test_moduforge_deno_integration() {
    let state = create_test_state().await;
    let deno = ModuForgeDeno::new(state, Some(2));

    deno.initialize().await.expect("Failed to initialize ModuForgeDeno");

    let plugin_code = mf_deno::create_sample_plugin_code();

    // 测试从代码创建插件
    let plugin = deno.create_plugin_from_code("sample-plugin", plugin_code)
        .await
        .expect("Failed to create plugin");

    assert_eq!(plugin.get_name(), "sample-plugin");

    // 测试插件列表
    let plugins = deno.list_plugins().await;
    assert_eq!(plugins.len(), 1);

    // 测试卸载
    deno.unload_plugin("sample-plugin").await.expect("Failed to unload plugin");

    let plugins = deno.list_plugins().await;
    assert_eq!(plugins.len(), 0);

    deno.shutdown().await;
}

#[tokio::test]
async fn test_plugin_builder() {
    let state = create_test_state().await;
    let deno = ModuForgeDeno::new(state, Some(1));
    deno.initialize().await.expect("Failed to initialize");

    let plugin_code = r#"
        function appendTransaction(args) { return null; }
        function filterTransaction(args) { return true; }
    "#;

    // 测试插件构建器
    let builder = DenoPluginBuilder::new("builder-test")
        .code(plugin_code)
        .priority(10)
        .enabled(true);

    let plugin = deno.build_plugin(builder).await.expect("Failed to build plugin");

    let config = plugin.spec.tr.config();
    assert_eq!(config.priority, 10);
    assert!(config.enabled);

    deno.shutdown().await;
}

/// 性能测试：运行时池效率
#[tokio::test]
async fn test_runtime_pool_performance() {
    let state = create_test_state().await;
    let manager = DenoPluginManager::new(state, 4);
    manager.initialize_pool().await.expect("Failed to initialize pool");

    let plugin_code = r#"
        function compute(args) {
            return args.a + args.b;
        }
    "#;

    manager.load_plugin("perf-test".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    let start = std::time::Instant::now();

    // 并发执行多个插件方法调用
    let tasks = (0..20).map(|i| {
        let manager_clone = &manager;
        async move {
            let args = serde_json::json!({"a": i, "b": i * 2});
            manager_clone.execute_plugin_method("perf-test", "compute", args).await
        }
    });

    let results = futures::future::try_join_all(tasks).await.expect("Failed to execute tasks");

    let duration = start.elapsed();
    println!("Executed 20 concurrent plugin calls in: {:?}", duration);

    // 验证结果
    assert_eq!(results.len(), 20);
    for (i, result) in results.iter().enumerate() {
        let expected = i + i * 2;
        assert_eq!(result.as_i64().unwrap(), expected as i64);
    }

    manager.shutdown().await;
}