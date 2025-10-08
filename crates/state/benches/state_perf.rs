use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mf_state::{State, StateConfig, Transaction};
use mf_state::plugin::{Plugin, PluginSpec, PluginManagerBuilder};
use mf_model::{Schema, NodePool, Node, Attrs, node_type::NodeEnum};
use std::sync::Arc;

/// 创建测试 Schema
fn create_test_schema() -> Arc<Schema> {
    use mf_model::schema::SchemaSpec;
    use std::collections::HashMap;

    let spec = SchemaSpec {
        nodes: HashMap::new(),
        marks: HashMap::new(),
    };

    Arc::new(Schema::new(spec))
}

/// 创建测试文档
fn create_test_document(node_count: usize) -> Arc<NodePool> {
    let mut child_nodes = Vec::new();

    for i in 1..=node_count {
        let node = Node::new(
            &format!("node_{}", i),
            "paragraph".to_string(),
            Attrs::default(),
            vec![],
            vec![],
        );
        child_nodes.push(NodeEnum(node, vec![]));
    }

    let root = Node::new(
        "root",
        "document".to_string(),
        Attrs::default(),
        (1..=node_count).map(|i| format!("node_{}", i).into()).collect(),
        vec![],
    );

    NodePool::from(NodeEnum(root, child_nodes))
}

/// 创建测试插件
fn create_test_plugin(name: &str, priority: i32) -> Arc<Plugin> {
    use mf_state::plugin::TransactionMetadata;

    let metadata = TransactionMetadata {
        name: name.to_string(),
        dependencies: vec![],
        conflicts: vec![],
    };

    let spec = PluginSpec {
        tr: Arc::new(metadata),
        state_field: None,
    };

    Arc::new(Plugin {
        key: name.to_string(),
        spec,
        priority,
    })
}

/// 基准测试：State 创建性能
fn bench_state_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_creation");

    for plugin_count in [0, 5, 10, 20].iter() {
        let schema = create_test_schema();
        let doc = create_test_document(100);

        // 创建插件列表
        let plugins: Vec<_> = (0..*plugin_count)
            .map(|i| create_test_plugin(&format!("plugin_{}", i), i))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("with_plugins", plugin_count),
            plugin_count,
            |b, _| {
                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let config = StateConfig {
                            schema: Some(schema.clone()),
                            doc: Some(doc.clone()),
                            stored_marks: None,
                            plugins: Some(plugins.clone()),
                            resource_manager: None,
                        };

                        let state = State::create(config).await.expect("创建 State 失败");
                        black_box(state)
                    })
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：Transaction 应用性能
fn bench_transaction_apply(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("transaction_apply");

    for step_count in [1, 5, 10, 20].iter() {
        let schema = create_test_schema();
        let doc = create_test_document(100);

        let state = rt.block_on(async {
            let config = StateConfig {
                schema: Some(schema.clone()),
                doc: Some(doc.clone()),
                stored_marks: None,
                plugins: None,
                resource_manager: None,
            };
            Arc::new(State::create(config).await.expect("创建 State 失败"))
        });

        group.throughput(Throughput::Elements(*step_count as u64));
        group.bench_with_input(
            BenchmarkId::new("steps", step_count),
            step_count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut tr = state.tr();

                        // 添加多个属性更新步骤
                        for i in 0..count {
                            let mut attrs = imbl::HashMap::new();
                            attrs.insert("step".to_string(), serde_json::json!(i));

                            let _ = tr.set_node_attribute(
                                format!("node_{}", i % 100 + 1).into(),
                                attrs,
                            );
                        }

                        let result = state.apply(tr).await.expect("应用事务失败");
                        black_box(result)
                    })
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：插件管理器构建性能
fn bench_plugin_manager_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_manager_build");

    for plugin_count in [10, 50, 100, 200].iter() {
        group.throughput(Throughput::Elements(*plugin_count as u64));

        // 无依赖场景
        group.bench_with_input(
            BenchmarkId::new("no_deps", plugin_count),
            plugin_count,
            |b, &count| {
                b.iter(|| {
                    let mut builder = PluginManagerBuilder::new();

                    for i in 0..count {
                        let plugin = create_test_plugin(&format!("plugin_{}", i), i);
                        builder.register_plugin(plugin).expect("注册插件失败");
                    }

                    let manager = builder.build().expect("构建管理器失败");
                    black_box(manager)
                });
            },
        );

        // 线性依赖链场景（最坏情况）
        group.bench_with_input(
            BenchmarkId::new("linear_deps", plugin_count),
            plugin_count,
            |b, &count| {
                b.iter(|| {
                    use mf_state::plugin::TransactionMetadata;
                    let mut builder = PluginManagerBuilder::new();

                    for i in 0..count {
                        let dependencies = if i > 0 {
                            vec![format!("plugin_{}", i - 1)]
                        } else {
                            vec![]
                        };

                        let metadata = TransactionMetadata {
                            name: format!("plugin_{}", i),
                            dependencies,
                            conflicts: vec![],
                        };

                        let spec = PluginSpec {
                            tr: Arc::new(metadata),
                            state_field: None,
                        };

                        let plugin = Arc::new(Plugin {
                            key: format!("plugin_{}", i),
                            spec,
                            priority: i,
                        });

                        builder.register_plugin(plugin).expect("注册插件失败");
                    }

                    let manager = builder.build().expect("构建管理器失败");
                    black_box(manager)
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：State 序列化/反序列化
fn bench_state_serialization(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("state_serialization");

    for doc_size in [100, 500, 1000].iter() {
        let schema = create_test_schema();
        let doc = create_test_document(*doc_size);

        let state = rt.block_on(async {
            let config = StateConfig {
                schema: Some(schema.clone()),
                doc: Some(doc.clone()),
                stored_marks: None,
                plugins: None,
                resource_manager: None,
            };
            State::create(config).await.expect("创建 State 失败")
        });

        // 序列化
        group.throughput(Throughput::Elements(*doc_size as u64));
        group.bench_with_input(
            BenchmarkId::new("serialize", doc_size),
            doc_size,
            |b, _| {
                b.iter(|| {
                    rt.block_on(async {
                        let serialized = state.serialize().await.expect("序列化失败");
                        black_box(serialized)
                    })
                });
            },
        );

        // 反序列化
        let serialized = rt.block_on(async {
            state.serialize().await.expect("序列化失败")
        });

        group.bench_with_input(
            BenchmarkId::new("deserialize", doc_size),
            doc_size,
            |b, _| {
                b.iter(|| {
                    rt.block_on(async {
                        let config = mf_state::state::Configuration::new(
                            schema.clone(),
                            None,
                            None,
                            None,
                        )
                        .await
                        .expect("创建配置失败");

                        let deserialized = State::deserialize(&serialized, &config)
                            .await
                            .expect("反序列化失败");
                        black_box(deserialized)
                    })
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：并发事务应用
fn bench_concurrent_transactions(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_transactions");

    for concurrent_count in [2, 4, 8].iter() {
        let schema = create_test_schema();
        let doc = create_test_document(100);

        let state = rt.block_on(async {
            let config = StateConfig {
                schema: Some(schema.clone()),
                doc: Some(doc.clone()),
                stored_marks: None,
                plugins: None,
                resource_manager: None,
            };
            Arc::new(State::create(config).await.expect("创建 State 失败"))
        });

        group.throughput(Throughput::Elements(*concurrent_count as u64));
        group.bench_with_input(
            BenchmarkId::new("concurrent", concurrent_count),
            concurrent_count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = vec![];

                        for i in 0..count {
                            let state = Arc::clone(&state);
                            let handle = tokio::spawn(async move {
                                let mut tr = state.tr();
                                let mut attrs = imbl::HashMap::new();
                                attrs.insert("worker".to_string(), serde_json::json!(i));

                                let _ = tr.set_node_attribute(
                                    format!("node_{}", i % 100 + 1).into(),
                                    attrs,
                                );

                                state.apply(tr).await.expect("应用事务失败")
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            let _ = handle.await;
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_state_creation,
    bench_transaction_apply,
    bench_plugin_manager_build,
    bench_state_serialization,
    bench_concurrent_transactions,
);
criterion_main!(benches);
