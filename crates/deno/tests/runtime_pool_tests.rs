//! 运行时池优化测试
//! 验证修复的运行时池功能和性能优化

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
async fn test_runtime_pool_initialization() {
    let state = create_test_state().await;
    let manager = DenoPluginManager::new(state, 3);

    // 测试管理器初始化
    manager.initialize_pool().await.expect("Failed to initialize manager");

    // 验证统计信息（线程本地模式下的行为）
    let stats = manager.get_pool_stats().await;
    assert_eq!(stats.total_capacity, 3);
    assert_eq!(stats.available_runtimes, 3); // 线程本地模式下始终可用
    assert_eq!(stats.active_runtimes, 0);
    assert_eq!(stats.created_count, 0); // 按需创建

    // 健康检查
    let is_healthy = manager.health_check().await.expect("Health check failed");
    assert!(is_healthy);

    manager.shutdown().await;
}

#[tokio::test]
async fn test_plugin_execution_with_thread_local() {
    let state = create_test_state().await;
    let manager = DenoPluginManager::new(state, 2);
    manager.initialize_pool().await.expect("Failed to initialize manager");

    let plugin_code = r#"
        function testMethod(args) {
            return {
                message: "Thread local test",
                input: args,
                timestamp: Date.now()
            };
        }
    "#;

    // 加载插件
    manager.load_plugin("thread-test".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 执行插件方法
    let result = manager.execute_plugin_method(
        "thread-test",
        "testMethod",
        serde_json::json!({"test": "data"})
    ).await.expect("Failed to execute plugin method");

    assert_eq!(result["message"], "Thread local test");
    assert_eq!(result["input"]["test"], "data");

    // 验证统计信息
    let stats = manager.get_pool_stats().await;
    assert!(stats.reused_count > 0); // 应该有执行记录

    manager.shutdown().await;
}

#[tokio::test]
async fn test_runtime_pool_exhaustion() {
    let state = create_test_state().await;
    let manager = DenoPluginManager::new(state, 1);
    manager.initialize_pool().await.expect("Failed to initialize pool");

    // 获取唯一的运行时
    let runtime1 = manager.get_runtime().await.expect("Failed to get runtime");
    let stats = manager.get_pool_stats().await;
    assert_eq!(stats.available_runtimes, 0);
    assert_eq!(stats.active_runtimes, 1);

    // 尝试获取第二个运行时（应该创建新的临时实例）
    let runtime2 = manager.get_runtime().await.expect("Failed to get temporary runtime");
    let stats_after_temp = manager.get_pool_stats().await;
    assert_eq!(stats_after_temp.available_runtimes, 0);
    assert_eq!(stats_after_temp.active_runtimes, 2);
    assert_eq!(stats_after_temp.created_count, 2); // 池中1个 + 临时1个

    // 归还运行时
    manager.return_runtime(runtime1).await;
    manager.return_runtime(runtime2).await; // 临时实例应该被丢弃

    let final_stats = manager.get_pool_stats().await;
    assert_eq!(final_stats.available_runtimes, 1); // 只有原池中的运行时被保留
    assert_eq!(final_stats.active_runtimes, 0);

    manager.shutdown().await;
}

#[tokio::test]
async fn test_runtime_pool_metrics() {
    let state = create_test_state().await;
    let manager = DenoPluginManager::new(state, 2);
    manager.initialize_pool().await.expect("Failed to initialize pool");

    // 获取详细监控信息
    let metrics = manager.get_pool_metrics().await;

    assert_eq!(metrics["pool_capacity"], 2);
    assert_eq!(metrics["available_runtimes"], 2);
    assert_eq!(metrics["active_runtimes"], 0);
    assert_eq!(metrics["total_created"], 2);
    assert_eq!(metrics["total_reused"], 0);
    assert_eq!(metrics["utilization_rate"], 0.0);
    assert_eq!(metrics["health_status"], "healthy");

    // 使用运行时来更新指标
    let runtime = manager.get_runtime().await.expect("Failed to get runtime");
    let metrics_after_use = manager.get_pool_metrics().await;
    assert_eq!(metrics_after_use["active_runtimes"], 1);
    assert_eq!(metrics_after_use["total_reused"], 1);
    assert_eq!(metrics_after_use["utilization_rate"], 50.0); // 1 reused out of 2 created

    manager.return_runtime(runtime).await;
    manager.shutdown().await;
}

#[tokio::test]
async fn test_runtime_pool_rebuild() {
    let state = create_test_state().await;
    let manager = DenoPluginManager::new(state, 2);
    manager.initialize_pool().await.expect("Failed to initialize pool");

    // 获取初始统计
    let initial_stats = manager.get_pool_stats().await;
    assert_eq!(initial_stats.created_count, 2);

    // 重建池
    manager.rebuild_pool().await.expect("Failed to rebuild pool");

    // 验证池被重建
    let rebuilt_stats = manager.get_pool_stats().await;
    assert_eq!(rebuilt_stats.available_runtimes, 2);
    assert_eq!(rebuilt_stats.active_runtimes, 0);
    assert_eq!(rebuilt_stats.created_count, 4); // 原2个 + 重建2个

    manager.shutdown().await;
}

#[tokio::test]
async fn test_runtime_pool_concurrent_access() {
    let state = create_test_state().await;
    let manager = Arc::new(DenoPluginManager::new(state, 4));
    manager.initialize_pool().await.expect("Failed to initialize pool");

    // 并发获取和归还运行时
    let tasks = (0..8).map(|i| {
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            let runtime = manager_clone.get_runtime().await
                .expect(&format!("Failed to get runtime {}", i));

            // 模拟使用时间
            tokio::time::sleep(Duration::from_millis(10)).await;

            manager_clone.return_runtime(runtime).await;
            i
        })
    });

    let results = futures::future::join_all(tasks).await;

    // 验证所有任务完成
    assert_eq!(results.len(), 8);
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result.as_ref().unwrap(), &i);
    }

    // 验证最终状态
    let final_stats = manager.get_pool_stats().await;
    assert_eq!(final_stats.active_runtimes, 0);
    assert!(final_stats.available_runtimes <= 4); // 不应超过池大小

    // 健康检查
    let is_healthy = manager.health_check().await.expect("Health check failed");
    assert!(is_healthy);

    manager.shutdown().await;
}

#[tokio::test]
async fn test_runtime_pool_graceful_shutdown() {
    let state = create_test_state().await;
    let manager = Arc::new(DenoPluginManager::new(state, 2));
    manager.initialize_pool().await.expect("Failed to initialize pool");

    // 获取运行时但不立即归还
    let runtime = manager.get_runtime().await.expect("Failed to get runtime");

    let manager_for_shutdown = manager.clone();

    // 在后台启动关闭过程
    let shutdown_task = tokio::spawn(async move {
        manager_for_shutdown.shutdown().await;
    });

    // 短暂延迟后归还运行时
    tokio::time::sleep(Duration::from_millis(100)).await;
    manager.return_runtime(runtime).await;

    // 等待关闭完成
    shutdown_task.await.expect("Shutdown task failed");

    // 验证最终状态
    let final_stats = manager.get_pool_stats().await;
    assert_eq!(final_stats.available_runtimes, 0);
    assert_eq!(final_stats.active_runtimes, 0);
}

#[tokio::test]
async fn test_runtime_pool_with_plugin_execution() {
    let state = create_test_state().await;
    let manager = DenoPluginManager::new(state, 3);
    manager.initialize_pool().await.expect("Failed to initialize pool");

    let plugin_code = r#"
        function testMethod(args) {
            return {
                message: "Runtime pool test",
                input: args,
                timestamp: Date.now()
            };
        }
    "#;

    // 加载插件
    manager.load_plugin("pool-test".to_string(), plugin_code.to_string())
        .await
        .expect("Failed to load plugin");

    // 执行插件方法多次以测试池的使用
    for i in 0..5 {
        let result = manager.execute_plugin_method(
            "pool-test",
            "testMethod",
            serde_json::json!({"iteration": i})
        ).await.expect("Failed to execute plugin method");

        assert_eq!(result["message"], "Runtime pool test");
        assert_eq!(result["input"]["iteration"], i);
    }

    // 验证池统计
    let stats = manager.get_pool_stats().await;
    assert!(stats.reused_count > 0); // 应该有运行时复用
    assert_eq!(stats.active_runtimes, 0); // 所有运行时应该已归还

    manager.shutdown().await;
}