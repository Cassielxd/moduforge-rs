use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mf_model::*;
use serde_json::json;

/// 基础节点操作基准测试
fn bench_basic_node_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("基础节点操作");

    // 节点创建基准测试
    group.bench_function("节点创建", |b| {
        let mut counter = 0;
        b.iter(|| {
            let node = Node::new(
                &format!("node_{counter}"),
                "paragraph".to_string(),
                Attrs::default(),
                vec![],
                vec![],
            );
            counter += 1;
            criterion::black_box(node)
        })
    });

    // 节点克隆性能
    group.bench_function("节点克隆", |b| {
        let node = Node::new(
            "test_node",
            "paragraph".to_string(),
            Attrs::default(),
            vec![],
            vec![],
        );

        b.iter(|| criterion::black_box(node.clone()))
    });

    group.finish();
}

/// ID生成器基准测试
fn bench_id_generator(c: &mut Criterion) {
    let mut group = c.benchmark_group("ID生成");

    group.bench_function("ID生成", |b| {
        b.iter(|| criterion::black_box(IdGenerator::get_id()))
    });

    // 批量ID生成
    for id_count in [100, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("批量ID生成", id_count),
            id_count,
            |b, &count| {
                b.iter(|| {
                    let ids: Vec<Box<str>> =
                        (0..count).map(|_| IdGenerator::get_id()).collect();
                    criterion::black_box(ids)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_basic_node_operations, bench_id_generator,);
criterion_main!(benches);
