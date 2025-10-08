use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mf_core::event::{Event, EventBus, EventHandler};
use mf_core::runtime::{ForgeRuntime, RuntimeTrait};
use mf_core::config::ForgeConfig;
use mf_model::{Schema, NodePool, Node, Attrs, node_type::NodeEnum};
use std::sync::Arc;
use async_trait::async_trait;

/// 测试用的事件处理器
#[derive(Clone)]
struct TestEventHandler {
    counter: Arc<std::sync::atomic::AtomicUsize>,
}

#[async_trait]
impl EventHandler for TestEventHandler {
    fn handle(&self, _event: Event) -> mf_core::ForgeResult<()> {
        self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

/// 创建测试用的 Schema
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

/// 基准测试：EventBus 性能
fn bench_event_bus(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_bus");

    for handler_count in [1, 10, 50, 100].iter() {
        let event_bus = EventBus::new();

        // 注册多个处理器
        for i in 0..*handler_count {
            let handler = TestEventHandler {
                counter: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            };
            event_bus.subscribe(
                format!("test_event_{}", i),
                Arc::new(handler),
            );
        }

        group.throughput(Throughput::Elements(*handler_count as u64));

        // 单个事件分发
        group.bench_with_input(
            BenchmarkId::new("single_dispatch", handler_count),
            handler_count,
            |b, _| {
                b.iter(|| {
                    let event = Event::new("test_event_0".to_string());
                    let _ = event_bus.emit(event);
                    black_box(())
                });
            },
        );

        // 批量事件分发
        group.bench_with_input(
            BenchmarkId::new("batch_dispatch", handler_count),
            handler_count,
            |b, &count| {
                b.iter(|| {
                    for i in 0..count {
                        let event = Event::new(format!("test_event_{}", i));
                        let _ = event_bus.emit(event);
                    }
                    black_box(())
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：Runtime 初始化
fn bench_runtime_initialization(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("runtime_initialization");

    for doc_size in [10, 100, 1000].iter() {
        let doc = create_test_document(*doc_size);
        let schema = Arc::new(Schema::default());

        group.throughput(Throughput::Elements(*doc_size as u64));

        // ForgeRuntime 初始化
        group.bench_with_input(
            BenchmarkId::new("forge_runtime", doc_size),
            doc_size,
            |b, _| {
                b.iter(|| {
                    rt.block_on(async {
                        let config = ForgeConfig::builder()
                            .build()
                            .expect("构建配置失败");

                        let runtime = ForgeRuntime::new(
                            doc.clone(),
                            schema.clone(),
                            config,
                        )
                        .await
                        .expect("创建运行时失败");

                        black_box(runtime)
                    })
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：Runtime 事务处理
fn bench_runtime_transaction_processing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("runtime_transaction");

    for step_count in [1, 5, 10, 20].iter() {
        let doc = create_test_document(100);
        let schema = Arc::new(Schema::default());

        let runtime = rt.block_on(async {
            let config = ForgeConfig::builder()
                .build()
                .expect("构建配置失败");

            ForgeRuntime::new(doc.clone(), schema.clone(), config)
                .await
                .expect("创建运行时失败")
        });

        group.throughput(Throughput::Elements(*step_count as u64));
        group.bench_with_input(
            BenchmarkId::new("process_steps", step_count),
            step_count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut tr = runtime.state().tr();

                        for i in 0..count {
                            let mut attrs = imbl::HashMap::new();
                            attrs.insert("step".to_string(), serde_json::json!(i));

                            let _ = tr.set_node_attribute(
                                format!("node_{}", i % 100 + 1).into(),
                                attrs,
                            );
                        }

                        let result = runtime.apply_transaction(tr)
                            .await
                            .expect("应用事务失败");

                        black_box(result)
                    })
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：历史管理性能
fn bench_history_management(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("history_management");

    for history_size in [10, 50, 100].iter() {
        let doc = create_test_document(100);
        let schema = Arc::new(Schema::default());

        let config = ForgeConfig::builder()
            .with_history_limit(*history_size)
            .build()
            .expect("构建配置失败");

        let runtime = rt.block_on(async {
            ForgeRuntime::new(doc.clone(), schema.clone(), config)
                .await
                .expect("创建运行时失败")
        });

        // 填充历史记录
        rt.block_on(async {
            for i in 0..*history_size {
                let mut tr = runtime.state().tr();
                let mut attrs = imbl::HashMap::new();
                attrs.insert("history".to_string(), serde_json::json!(i));

                let _ = tr.set_node_attribute("node_1".into(), attrs);
                let _ = runtime.apply_transaction(tr).await;
            }
        });

        group.throughput(Throughput::Elements(*history_size as u64));

        // 撤销操作
        group.bench_with_input(
            BenchmarkId::new("undo", history_size),
            history_size,
            |b, _| {
                b.iter(|| {
                    rt.block_on(async {
                        let result = runtime.undo().await;
                        black_box(result)
                    })
                });
            },
        );

        // 重做操作
        group.bench_with_input(
            BenchmarkId::new("redo", history_size),
            history_size,
            |b, _| {
                b.iter(|| {
                    rt.block_on(async {
                        let result = runtime.redo().await;
                        black_box(result)
                    })
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：并发运行时访问
fn bench_concurrent_runtime_access(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_runtime_access");

    for concurrent_count in [2, 4, 8, 16].iter() {
        let doc = create_test_document(100);
        let schema = Arc::new(Schema::default());

        let runtime = rt.block_on(async {
            let config = ForgeConfig::builder()
                .build()
                .expect("构建配置失败");

            Arc::new(
                ForgeRuntime::new(doc.clone(), schema.clone(), config)
                    .await
                    .expect("创建运行时失败")
            )
        });

        group.throughput(Throughput::Elements(*concurrent_count as u64));
        group.bench_with_input(
            BenchmarkId::new("concurrent_reads", concurrent_count),
            concurrent_count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = vec![];

                        for _ in 0..count {
                            let runtime = Arc::clone(&runtime);
                            let handle = tokio::spawn(async move {
                                // 并发读取状态
                                let state = runtime.state();
                                let doc = state.doc();
                                black_box(doc)
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

        group.bench_with_input(
            BenchmarkId::new("concurrent_writes", concurrent_count),
            concurrent_count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = vec![];

                        for i in 0..count {
                            let runtime = Arc::clone(&runtime);
                            let handle = tokio::spawn(async move {
                                let mut tr = runtime.state().tr();
                                let mut attrs = imbl::HashMap::new();
                                attrs.insert("worker".to_string(), serde_json::json!(i));

                                let _ = tr.set_node_attribute(
                                    format!("node_{}", i % 100 + 1).into(),
                                    attrs,
                                );

                                runtime.apply_transaction(tr).await
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

/// 基准测试：扩展管理器性能
fn bench_extension_manager(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("extension_manager");

    for ext_count in [5, 10, 20].iter() {
        let doc = create_test_document(100);
        let schema = Arc::new(Schema::default());

        let config = ForgeConfig::builder()
            .build()
            .expect("构建配置失败");

        let runtime = rt.block_on(async {
            ForgeRuntime::new(doc.clone(), schema.clone(), config)
                .await
                .expect("创建运行时失败")
        });

        // 注册扩展
        rt.block_on(async {
            for i in 0..*ext_count {
                let ext_name = format!("extension_{}", i);
                // 这里需要实际的扩展实现，简化为概念性测试
                // runtime.register_extension(ext_name, extension).await;
            }
        });

        group.throughput(Throughput::Elements(*ext_count as u64));

        // 扩展查找
        group.bench_with_input(
            BenchmarkId::new("extension_lookup", ext_count),
            ext_count,
            |b, &count| {
                b.iter(|| {
                    for i in 0..count {
                        let ext_name = format!("extension_{}", i);
                        // let ext = runtime.get_extension(&ext_name);
                        black_box(ext_name)
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_event_bus,
    bench_runtime_initialization,
    bench_runtime_transaction_processing,
    bench_history_management,
    bench_concurrent_runtime_access,
    bench_extension_manager,
);
criterion_main!(benches);
