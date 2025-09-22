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
                &format!("node_{}", counter),
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

/// 属性系统基准测试
fn bench_attrs_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("属性操作");

    // 属性创建
    group.bench_function("属性创建", |b| {
        b.iter(|| {
            let mut map = imbl::HashMap::new();
            map.insert("class".to_string(), json!("paragraph"));
            map.insert("id".to_string(), json!("test-123"));
            map.insert("data-value".to_string(), json!(42));
            let attrs = Attrs::from(map);
            criterion::black_box(attrs)
        })
    });

    // 属性查找性能
    group.bench_function("属性查找", |b| {
        let mut map = imbl::HashMap::new();
        for i in 0..50 {
            map.insert(format!("attr_{}", i), json!(format!("value_{}", i)));
        }
        let attrs = Attrs::from(map);

        b.iter(|| criterion::black_box(attrs.get_safe("attr_25")))
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

/// 序列化性能基准测试
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("序列化性能");

    // 单节点序列化
    group.bench_function("节点JSON序列化", |b| {
        let mut map = imbl::HashMap::new();
        map.insert("class".to_string(), json!("highlight"));
        map.insert("id".to_string(), json!("test-123"));

        let node = Node::new(
            "test_node",
            "paragraph".to_string(),
            Attrs::from(map),
            vec!["child1".into(), "child2".into()],
            vec![],
        );

        b.iter(|| criterion::black_box(serde_json::to_string(&node).unwrap()))
    });

    // JSON反序列化
    group.bench_function("节点JSON反序列化", |b| {
        let mut map = imbl::HashMap::new();
        map.insert("class".to_string(), json!("highlight"));
        map.insert("id".to_string(), json!("test-123"));

        let node = Node::new(
            "test_node",
            "paragraph".to_string(),
            Attrs::from(map),
            vec!["child1".into(), "child2".into()],
            vec![],
        );
        let json_str = serde_json::to_string(&node).unwrap();

        b.iter(|| {
            criterion::black_box(
                serde_json::from_str::<Node>(&json_str).unwrap(),
            )
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_basic_node_operations,
    bench_attrs_operations,
    bench_id_generator,
    bench_serialization
);
criterion_main!(benches);
