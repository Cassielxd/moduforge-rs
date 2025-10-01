use criterion::{criterion_group, criterion_main, Criterion};
use mf_core::*;
use std::time::Duration;

/// 配置系统基准测试
fn bench_configuration_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("配置系统");

    // 简单配置创建
    group.bench_function("简单配置创建", |b| {
        b.iter(|| {
            let config = ForgeConfig::for_environment(Environment::Development);
            criterion::black_box(config)
        })
    });

    // 复杂配置创建
    group.bench_function("配置Builder使用", |b| {
        b.iter(|| {
            let config = ForgeConfig::builder()
                .processor_config(ProcessorConfig {
                    max_queue_size: 5000,
                    max_concurrent_tasks: 20,
                    task_timeout: Duration::from_secs(30),
                    cleanup_timeout: Duration::from_secs(10),
                    max_retries: 3,
                    retry_delay: Duration::from_millis(500),
                })
                .performance_config(PerformanceConfig {
                    enable_monitoring: true,
                    middleware_timeout_ms: 1000,
                    log_threshold_ms: 100,
                    task_receive_timeout_ms: 5000,
                    enable_detailed_logging: true,
                    metrics_sampling_rate: 0.8,
                })
                .build();
            criterion::black_box(config)
        })
    });

    // 配置验证
    group.bench_function("配置验证", |b| {
        b.iter_batched(
            || {
                ForgeConfig::builder()
                    .processor_config(ProcessorConfig::default())
                    .build()
                    .expect("配置构建失败")
            },
            |config| {
                let is_valid = config.validate();
                criterion::black_box((config, is_valid))
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

/// 历史管理器基准测试
fn bench_history_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("历史管理器");

    // HistoryManager创建
    group.bench_function("HistoryManager创建", |b| {
        b.iter(|| {
            let history_manager = HistoryManager::<String>::new(
                "initial".to_string(),
                Some(1000),
            );
            criterion::black_box(history_manager)
        })
    });

    // 带配置的HistoryManager创建
    group.bench_function("带配置HistoryManager创建", |b| {
        b.iter(|| {
            let config = HistoryConfig {
                max_entries: 1000,
                enable_compression: true,
                persistence_interval: Duration::from_secs(60),
            };
            let history_manager = HistoryManager::<String>::with_config(
                "initial".to_string(),
                config,
            );
            criterion::black_box(history_manager)
        })
    });

    group.finish();
}

/// XML Schema解析基准测试
fn bench_xml_schema_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("XML解析");

    let simple_xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <schema top_node="doc">
          <nodes>
            <node name="doc" desc="文档根节点" content="paragraph+"/>
            <node name="paragraph" desc="段落节点" content="text*"/>
            <node name="text" desc="文本节点"/>
          </nodes>
        </schema>
    "#;

    // XML解析
    group.bench_function("简单XML解析", |b| {
        b.iter(|| {
            let result = XmlSchemaParser::parse_from_str(simple_xml);
            criterion::black_box(result)
        })
    });

    // 扩展转换
    group.bench_function("XML转Extensions", |b| {
        b.iter(|| {
            let result = XmlSchemaParser::parse_to_extensions(simple_xml);
            criterion::black_box(result)
        })
    });

    group.finish();
}

/// 错误处理基准测试
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("错误处理");

    // ForgeError创建
    group.bench_function("ForgeError创建", |b| {
        b.iter(|| {
            let error = ForgeError::State {
                message: "运行时错误".to_string(),
                source: None,
            };
            criterion::black_box(error)
        })
    });

    // 错误转换（使用anyhow Error）
    group.bench_function("错误转换", |b| {
        b.iter(|| {
            let anyhow_error = anyhow::anyhow!("测试错误");
            let forge_error = ForgeError::from(anyhow_error);
            criterion::black_box(forge_error)
        })
    });

    group.finish();
}

/// 基础类型基准测试
fn bench_basic_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("基础类型");

    // 环境枚举操作
    group.bench_function("Environment枚举操作", |b| {
        b.iter(|| {
            let env = Environment::Production;
            let debug_str = format!("{:?}", env);
            criterion::black_box((env, debug_str))
        })
    });

    // ProcessorConfig默认创建
    group.bench_function("ProcessorConfig默认", |b| {
        b.iter(|| {
            let config = ProcessorConfig::default();
            criterion::black_box(config)
        })
    });

    // PerformanceConfig默认创建
    group.bench_function("PerformanceConfig默认", |b| {
        b.iter(|| {
            let config = PerformanceConfig::default();
            criterion::black_box(config)
        })
    });

    group.finish();
}

/// 集成功能基准测试
fn bench_integration_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("集成功能");

    // 配置和历史管理器组合
    group.bench_function("配置历史管理器组合", |b| {
        b.iter(|| {
            let config = ForgeConfig::for_environment(Environment::Development);
            let history_config = config.history.clone();
            let history_manager = HistoryManager::<String>::with_config(
                "initial".to_string(),
                history_config,
            );
            criterion::black_box((config, history_manager))
        })
    });

    // 完整配置链
    group.bench_function("完整配置链", |b| {
        b.iter(|| {
            let config = ForgeConfig::builder()
                .processor_config(ProcessorConfig::default())
                .performance_config(PerformanceConfig::default())
                .event_config(EventConfig::default())
                .history_config(HistoryConfig::default())
                .extension_config(ExtensionConfig::default())
                .cache_config(CacheConfig::default())
                .build()
                .expect("配置构建失败");

            let is_valid = config.validate();
            criterion::black_box((config, is_valid))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_configuration_system,
    bench_history_manager,
    bench_xml_schema_parser,
    bench_error_handling,
    bench_basic_types,
    bench_integration_features
);
criterion_main!(benches);
