use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mf_transform::transform::Transform;
use mf_transform::node_step::{AddNodeStep, RemoveNodeStep};
use mf_transform::attr_step::AttrStep;
use mf_transform::mark_step::{AddMarkStep, RemoveMarkStep};
use mf_model::{Node, Attrs, Mark, Schema, NodePool, node_type::NodeEnum};
use std::sync::Arc;

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

/// 基准测试：延迟计算 vs 立即计算
fn bench_lazy_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("lazy_evaluation");

    for step_count in [10, 50, 100].iter() {
        let doc = create_test_document(100);
        let schema = create_test_schema();

        // 延迟计算模式（默认）
        group.bench_with_input(
            BenchmarkId::new("lazy", step_count),
            step_count,
            |b, &count| {
                b.iter(|| {
                    let mut transform = Transform::new(doc.clone(), schema.clone());

                    // 添加多个步骤但不立即计算
                    for i in 0..count {
                        let mut attrs = imbl::HashMap::new();
                        attrs.insert("index".to_string(), serde_json::json!(i));

                        let step = Arc::new(AttrStep::new(
                            format!("node_{}", i % 100 + 1).into(),
                            attrs,
                        ));
                        let _ = transform.step(step);
                    }

                    // 只在最后获取文档（触发延迟计算）
                    let doc = transform.doc();
                    black_box(doc)
                });
            },
        );

        // 提交模式（强制立即计算）
        group.bench_with_input(
            BenchmarkId::new("commit_each", step_count),
            step_count,
            |b, &count| {
                b.iter(|| {
                    let mut transform = Transform::new(doc.clone(), schema.clone());

                    for i in 0..count {
                        let mut attrs = imbl::HashMap::new();
                        attrs.insert("index".to_string(), serde_json::json!(i));

                        let step = Arc::new(AttrStep::new(
                            format!("node_{}", i % 100 + 1).into(),
                            attrs,
                        ));
                        let _ = transform.step(step);
                        let _ = transform.commit(); // 每步都提交
                    }

                    black_box(transform)
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：批量操作 vs 单步操作
fn bench_batch_operations(c: &mut Criterion) {
    let doc = create_test_document(1000);
    let schema = create_test_schema();
    let mut group = c.benchmark_group("batch_operations");

    for batch_size in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));

        // 单步应用
        group.bench_with_input(
            BenchmarkId::new("sequential", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let mut transform = Transform::new(doc.clone(), schema.clone());

                    for i in 0..size {
                        let mut attrs = imbl::HashMap::new();
                        attrs.insert("batch_index".to_string(), serde_json::json!(i));

                        let step = Arc::new(AttrStep::new(
                            format!("node_{}", i % 1000 + 1).into(),
                            attrs,
                        ));
                        let _ = transform.step(step);
                    }

                    black_box(transform)
                });
            },
        );

        // 批量应用
        group.bench_with_input(
            BenchmarkId::new("batch", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let mut transform = Transform::new(doc.clone(), schema.clone());

                    let steps: Vec<_> = (0..size)
                        .map(|i| {
                            let mut attrs = imbl::HashMap::new();
                            attrs.insert("batch_index".to_string(), serde_json::json!(i));

                            Arc::new(AttrStep::new(
                                format!("node_{}", i % 1000 + 1).into(),
                                attrs,
                            )) as Arc<dyn mf_transform::step::Step>
                        })
                        .collect();

                    let _ = transform.apply_steps_batch(steps);
                    black_box(transform)
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：不同类型的步骤性能
fn bench_step_types(c: &mut Criterion) {
    let doc = create_test_document(100);
    let schema = create_test_schema();
    let mut group = c.benchmark_group("step_types");

    // 1. 属性更新步骤
    group.bench_function("attr_step", |b| {
        b.iter(|| {
            let mut transform = Transform::new(doc.clone(), schema.clone());
            let mut attrs = imbl::HashMap::new();
            attrs.insert("key".to_string(), serde_json::json!("value"));

            let step = Arc::new(AttrStep::new("node_1".into(), attrs));
            let _ = transform.step(step);
            black_box(transform)
        });
    });

    // 2. 添加节点步骤
    group.bench_function("add_node_step", |b| {
        b.iter(|| {
            let mut transform = Transform::new(doc.clone(), schema.clone());
            let new_node = Node::new(
                "new_node",
                "paragraph".to_string(),
                Attrs::default(),
                vec![],
                vec![],
            );

            let step = Arc::new(AddNodeStep::new(
                "root".into(),
                vec![NodeEnum(new_node, vec![])],
            ));
            let _ = transform.step(step);
            black_box(transform)
        });
    });

    // 3. 删除节点步骤
    group.bench_function("remove_node_step", |b| {
        b.iter(|| {
            let mut transform = Transform::new(doc.clone(), schema.clone());
            let step = Arc::new(RemoveNodeStep::new(
                "root".into(),
                vec!["node_1".into()],
            ));
            let _ = transform.step(step);
            black_box(transform)
        });
    });

    // 4. 添加标记步骤
    group.bench_function("add_mark_step", |b| {
        b.iter(|| {
            let mut transform = Transform::new(doc.clone(), schema.clone());
            let mark = Mark {
                r#type: "bold".to_string(),
                attrs: Attrs::default(),
            };

            let step = Arc::new(AddMarkStep::new("node_1".into(), vec![mark]));
            let _ = transform.step(step);
            black_box(transform)
        });
    });

    // 5. 删除标记步骤
    group.bench_function("remove_mark_step", |b| {
        b.iter(|| {
            let mut transform = Transform::new(doc.clone(), schema.clone());
            let step = Arc::new(RemoveMarkStep::new(
                "node_1".into(),
                vec!["bold".to_string()],
            ));
            let _ = transform.step(step);
            black_box(transform)
        });
    });

    group.finish();
}

/// 基准测试：回滚操作性能
fn bench_rollback_operations(c: &mut Criterion) {
    let doc = create_test_document(100);
    let schema = create_test_schema();
    let mut group = c.benchmark_group("rollback_operations");

    for step_count in [10, 50, 100].iter() {
        // 设置：应用多个步骤
        let mut transform = Transform::new(doc.clone(), schema.clone());
        for i in 0..*step_count {
            let mut attrs = imbl::HashMap::new();
            attrs.insert("index".to_string(), serde_json::json!(i));
            let step = Arc::new(AttrStep::new(
                format!("node_{}", i % 100 + 1).into(),
                attrs,
            ));
            let _ = transform.step(step);
        }

        // 回滚部分步骤
        group.bench_with_input(
            BenchmarkId::new("partial_rollback", step_count),
            step_count,
            |b, &count| {
                b.iter_batched(
                    || transform.clone(),
                    |mut tr| {
                        let _ = tr.rollback_steps(count / 2);
                        black_box(tr)
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        // 完全回滚
        group.bench_with_input(
            BenchmarkId::new("full_rollback", step_count),
            step_count,
            |b, _| {
                b.iter_batched(
                    || transform.clone(),
                    |mut tr| {
                        tr.rollback();
                        black_box(tr)
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// 基准测试：Copy-on-Write 效率
fn bench_copy_on_write(c: &mut Criterion) {
    let doc = create_test_document(1000);
    let schema = create_test_schema();
    let mut group = c.benchmark_group("copy_on_write");

    // 1. 首次修改（触发克隆）
    group.bench_function("first_modification", |b| {
        b.iter(|| {
            let mut transform = Transform::new(doc.clone(), schema.clone());
            let mut attrs = imbl::HashMap::new();
            attrs.insert("key".to_string(), serde_json::json!("value"));

            let step = Arc::new(AttrStep::new("node_1".into(), attrs));
            let _ = transform.step(step); // 触发 Copy-on-Write
            black_box(transform)
        });
    });

    // 2. 后续修改（使用已克隆的 draft）
    group.bench_function("subsequent_modifications", |b| {
        b.iter_batched(
            || {
                // 预先创建已修改的 transform
                let mut transform = Transform::new(doc.clone(), schema.clone());
                let mut attrs = imbl::HashMap::new();
                attrs.insert("init".to_string(), serde_json::json!("init"));
                let step = Arc::new(AttrStep::new("node_1".into(), attrs));
                let _ = transform.step(step);
                transform
            },
            |mut transform| {
                // 后续修改
                let mut attrs = imbl::HashMap::new();
                attrs.insert("key".to_string(), serde_json::json!("value"));
                let step = Arc::new(AttrStep::new("node_2".into(), attrs));
                let _ = transform.step(step);
                black_box(transform)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_lazy_evaluation,
    bench_batch_operations,
    bench_step_types,
    bench_rollback_operations,
    bench_copy_on_write,
);
criterion_main!(benches);
