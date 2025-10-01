//! 统一配置使用示例
//!
//! 展示如何在 ForgeRuntime 和 ForgeAsyncRuntime 中使用统一配置

use mf_core::{
    config::{ForgeConfig, Environment, PerformanceConfig},
    types::{EditorOptionsBuilder, Content},
    ForgeResult,
};

#[tokio::main]
async fn main() -> ForgeResult<()> {
    println!("=== 统一配置使用示例 ===\n");

    // 1. 创建自定义配置
    let custom_config = ForgeConfig::builder()
        .environment(Environment::Development)
        .max_queue_size(2000)
        .max_concurrent_tasks(15)
        .middleware_timeout(2000) // 2秒
        .enable_monitoring(true)
        .history_limit(200)
        .build()?;

    println!("自定义配置:");
    println!("  - 环境: {:?}", custom_config.environment);
    println!("  - 队列大小: {}", custom_config.processor.max_queue_size);
    println!("  - 并发任务: {}", custom_config.processor.max_concurrent_tasks);
    println!(
        "  - 中间件超时: {}ms",
        custom_config.performance.middleware_timeout_ms
    );
    println!("  - 监控启用: {}", custom_config.performance.enable_monitoring);
    println!("  - 历史限制: {}", custom_config.history.max_entries);

    // 2. 创建运行时选项
    let options = EditorOptionsBuilder::new()
        .content(Content::None)
        .extensions(vec![])
        .build();

    // 3. 使用配置创建 ForgeRuntime
    println!("\n=== 创建 ForgeRuntime ===");
    let runtime = mf_core::ForgeRuntime::create_with_config(
        options.clone(),
        custom_config.clone(),
    )
    .await?;

    println!("ForgeRuntime 配置:");
    let runtime_config = runtime.get_config();
    println!(
        "  - 中间件超时: {}ms",
        runtime_config.performance.middleware_timeout_ms
    );
    println!("  - 事件队列大小: {}", runtime_config.event.max_queue_size);
    println!("  - 历史记录限制: {}", runtime_config.history.max_entries);

    // 4. 使用配置创建 ForgeAsyncRuntime
    println!("\n=== 创建 ForgeAsyncRuntime ===");
    let async_runtime =
        mf_core::ForgeAsyncRuntime::create_with_config(options, custom_config)
            .await?;

    println!("ForgeAsyncRuntime 配置:");
    let async_config = async_runtime.get_config();
    println!(
        "  - 任务接收超时: {}ms",
        async_config.performance.task_receive_timeout_ms
    );
    println!("  - 处理器队列大小: {}", async_config.processor.max_queue_size);
    println!("  - 缓存条目数: {}", async_config.cache.max_entries);

    // 5. 展示配置继承 - ForgeAsyncRuntime 通过 ForgeRuntime 访问配置
    println!("\n=== 配置访问验证 ===");
    println!("ForgeAsyncRuntime 通过基础 ForgeRuntime 访问配置:");
    println!("  - 基础运行时配置地址: {:p}", runtime.get_config());
    println!("  - 异步运行时配置地址: {:p}", async_runtime.get_config());
    println!(
        "  - 配置值一致性验证: {}",
        runtime.get_config().performance.middleware_timeout_ms
            == async_runtime.get_config().performance.middleware_timeout_ms
    );

    // 6. 动态配置更新示例
    println!("\n=== 动态配置更新 ===");
    let mut mutable_async_runtime = async_runtime;

    // 更新性能配置
    let new_perf_config = PerformanceConfig {
        enable_monitoring: true,
        middleware_timeout_ms: 3000, // 增加到3秒
        log_threshold_ms: 100,
        task_receive_timeout_ms: 8000,
        enable_detailed_logging: true,
        metrics_sampling_rate: 0.5,
    };

    mutable_async_runtime.set_performance_config(new_perf_config);

    println!("更新后的配置:");
    let updated_config = mutable_async_runtime.get_config();
    println!(
        "  - 新的中间件超时: {}ms",
        updated_config.performance.middleware_timeout_ms
    );
    println!(
        "  - 详细日志启用: {}",
        updated_config.performance.enable_detailed_logging
    );
    println!(
        "  - 指标采样率: {}",
        updated_config.performance.metrics_sampling_rate
    );

    // 7. 环境特定配置建议
    println!("\n=== 配置调优建议 ===");
    let suggestions = updated_config.get_tuning_suggestions();
    if suggestions.is_empty() {
        println!("  - 当前配置已优化");
    } else {
        for suggestion in suggestions {
            println!("  - {suggestion}");
        }
    }

    println!("\n=== 示例完成 ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mf_core::config::Environment;

    #[tokio::test]
    async fn test_config_inheritance() -> ForgeResult<()> {
        // 测试配置继承 - ForgeAsyncRuntime 不应该重复持有配置
        let config = ForgeConfig::for_environment(Environment::Testing);
        let options = EditorOptionsBuilder::new()
            .content(Content::None)
            .extensions(vec![])
            .build();

        let async_runtime = mf_core::ForgeAsyncRuntime::create_with_config(
            options,
            config.clone(),
        )
        .await?;

        // 验证配置值一致性
        assert_eq!(
            async_runtime.get_config().processor.max_queue_size,
            config.processor.max_queue_size
        );

        assert_eq!(
            async_runtime.get_config().performance.middleware_timeout_ms,
            config.performance.middleware_timeout_ms
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_config_update() -> ForgeResult<()> {
        // 测试配置更新
        let initial_config =
            ForgeConfig::for_environment(Environment::Development);
        let options = EditorOptionsBuilder::new()
            .content(Content::None)
            .extensions(vec![])
            .build();

        let mut async_runtime = mf_core::ForgeAsyncRuntime::create_with_config(
            options,
            initial_config,
        )
        .await?;

        // 更新配置
        let new_config = ForgeConfig::for_environment(Environment::Production);
        async_runtime.update_config(new_config.clone());

        // 验证配置已更新
        assert_eq!(
            async_runtime.get_config().processor.max_queue_size,
            new_config.processor.max_queue_size
        );

        Ok(())
    }

    #[test]
    fn test_config_validation() {
        // 测试配置验证
        let valid_config = ForgeConfig::default();
        assert!(valid_config.validate().is_ok());

        // 测试无效配置
        let invalid_config = ForgeConfig::builder()
            .max_queue_size(0) // 无效值
            .build_unchecked();
        assert!(invalid_config.validate().is_err());
    }
}
