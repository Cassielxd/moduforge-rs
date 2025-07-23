//! 统一配置管理示例
//!
//! 这个示例展示了如何使用新的统一配置系统来配置 ModuForge 核心模块。

use std::time::Duration;
use mf_core::{
    config::{
        ForgeConfig, Environment, ProcessorConfig, PerformanceConfig,
        EventConfig, HistoryConfig, ExtensionConfig, CacheConfig,
    },
    ForgeRuntime, ForgeAsyncRuntime,
    types::{RuntimeOptions, EditorOptionsBuilder, Content},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ModuForge 统一配置管理示例 ===\n");

    // 1. 使用预设环境配置
    println!("1. 预设环境配置:");

    let dev_config = ForgeConfig::for_environment(Environment::Development);
    println!("开发环境配置:");
    println!("  - 队列大小: {}", dev_config.processor.max_queue_size);
    println!("  - 并发任务数: {}", dev_config.processor.max_concurrent_tasks);
    println!(
        "  - 中间件超时: {}ms",
        dev_config.performance.middleware_timeout_ms
    );
    println!(
        "  - 详细日志: {}",
        dev_config.performance.enable_detailed_logging
    );

    let prod_config = ForgeConfig::for_environment(Environment::Production);
    println!("\n生产环境配置:");
    println!("  - 队列大小: {}", prod_config.processor.max_queue_size);
    println!("  - 并发任务数: {}", prod_config.processor.max_concurrent_tasks);
    println!(
        "  - 中间件超时: {}ms",
        prod_config.performance.middleware_timeout_ms
    );
    println!(
        "  - 详细日志: {}",
        prod_config.performance.enable_detailed_logging
    );

    // 2. 使用配置构建器
    println!("\n2. 自定义配置构建:");

    let custom_config = ForgeConfig::builder()
        .environment(Environment::Custom)
        .processor_config(ProcessorConfig {
            max_queue_size: 2000,
            max_concurrent_tasks: 15,
            task_timeout: Duration::from_secs(45),
            max_retries: 5,
            retry_delay: Duration::from_millis(500),
            cleanup_timeout: Duration::from_secs(20),
        })
        .performance_config(PerformanceConfig {
            enable_monitoring: true,
            middleware_timeout_ms: 1500,
            log_threshold_ms: 75,
            task_receive_timeout_ms: 8000,
            enable_detailed_logging: false,
            metrics_sampling_rate: 0.1,
        })
        .event_config(EventConfig {
            max_queue_size: 20000,
            handler_timeout: Duration::from_secs(3),
            enable_persistence: true,
            batch_size: 200,
            max_concurrent_handlers: 8,
        })
        .history_config(HistoryConfig {
            max_entries: 500,
            enable_compression: true,
            persistence_interval: Duration::from_secs(120),
            enable_incremental_snapshots: true,
        })
        .extension_config(ExtensionConfig {
            load_timeout: Duration::from_secs(15),
            enable_hot_reload: true,
            max_memory_mb: 300,
            enable_sandbox: true,
            xml_schema_paths: vec![],
            enable_xml_auto_reload: false,
            xml_parse_timeout: Duration::from_secs(5),
        })
        .cache_config(CacheConfig {
            max_entries: 5000,
            entry_ttl: Duration::from_secs(900), // 15分钟
            enable_lru: true,
            cleanup_interval: Duration::from_secs(120),
        })
        .build()?;

    println!("自定义配置:");
    println!("  - 环境: {:?}", custom_config.environment);
    println!("  - 队列大小: {}", custom_config.processor.max_queue_size);
    println!("  - 事件持久化: {}", custom_config.event.enable_persistence);
    println!("  - 历史压缩: {}", custom_config.history.enable_compression);

    // 3. 配置验证
    println!("\n3. 配置验证:");

    match custom_config.validate() {
        Ok(()) => println!("✓ 配置验证通过"),
        Err(e) => println!("✗ 配置验证失败: {}", e),
    }

    // 4. 配置调优建议
    println!("\n4. 配置调优建议:");

    let suggestions = custom_config.get_tuning_suggestions();
    if suggestions.is_empty() {
        println!("  - 当前配置已优化");
    } else {
        for suggestion in suggestions {
            println!("  - {}", suggestion);
        }
    }

    // 5. 配置序列化
    println!("\n5. 配置序列化:");

    let json_config = custom_config.to_json()?;
    println!("配置已序列化为 JSON (前200字符):");
    println!("{}", &json_config[..json_config.len().min(200)]);
    if json_config.len() > 200 {
        println!("...");
    }

    // 6. 从环境变量覆盖配置
    println!("\n6. 环境变量覆盖:");

    // 设置一些示例环境变量
    unsafe {
        std::env::set_var("FORGE_ENVIRONMENT", "production");
        std::env::set_var("FORGE_PROCESSOR_MAX_QUEUE_SIZE", "15000");
        std::env::set_var("FORGE_PERFORMANCE_ENABLE_MONITORING", "true");
    }

    let env_config = ForgeConfig::default().from_env_override();
    println!("从环境变量覆盖后:");
    println!("  - 环境: {:?}", env_config.environment);
    println!("  - 队列大小: {}", env_config.processor.max_queue_size);
    println!("  - 监控启用: {}", env_config.performance.enable_monitoring);

    // 7. 配置合并
    println!("\n7. 配置合并:");

    let base_config = ForgeConfig::for_environment(Environment::Development);
    let override_config = ForgeConfig::for_environment(Environment::Production);
    let merged_config = base_config.merge_with(&override_config);

    println!("合并后的配置:");
    println!("  - 环境: {:?}", merged_config.environment);
    println!("  - 队列大小: {}", merged_config.processor.max_queue_size);

    // 8. 便捷配置方法
    println!("\n8. 便捷配置方法:");

    let quick_config = ForgeConfig::builder()
        .max_queue_size(3000)
        .max_concurrent_tasks(25)
        .task_timeout(Duration::from_secs(60))
        .middleware_timeout(2000)
        .enable_monitoring(true)
        .history_limit(300)
        .build()?;

    println!("快速配置:");
    println!("  - 队列大小: {}", quick_config.processor.max_queue_size);
    println!("  - 并发任务: {}", quick_config.processor.max_concurrent_tasks);
    println!("  - 历史限制: {}", quick_config.history.max_entries);

    println!("\n=== 示例完成 ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        // 测试有效配置
        let valid_config = ForgeConfig::default();
        assert!(valid_config.validate().is_ok());

        // 测试无效配置
        let invalid_config = ForgeConfig::builder()
            .max_queue_size(0) // 无效值
            .build_unchecked();
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_environment_configs() {
        let dev = ForgeConfig::for_environment(Environment::Development);
        let prod = ForgeConfig::for_environment(Environment::Production);

        // 开发环境应该有更长的超时时间
        assert!(
            dev.performance.middleware_timeout_ms
                > prod.performance.middleware_timeout_ms
        );

        // 生产环境应该有更大的队列
        assert!(prod.processor.max_queue_size > dev.processor.max_queue_size);
    }

    #[test]
    fn test_config_builder() {
        let config = ForgeConfig::builder()
            .max_queue_size(1000)
            .enable_monitoring(true)
            .build()
            .unwrap();

        assert_eq!(config.processor.max_queue_size, 1000);
        assert!(config.performance.enable_monitoring);
    }

    #[test]
    fn test_config_serialization() {
        let config = ForgeConfig::default();
        let json = config.to_json().unwrap();
        let deserialized = ForgeConfig::from_json(&json).unwrap();

        assert_eq!(
            config.processor.max_queue_size,
            deserialized.processor.max_queue_size
        );
    }
}
