use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
    Throughput,
};
use mf_model::node_pool::{
    NodePool, QueryCacheConfig, LazyQueryConfig, QueryCondition,
};
use mf_model::{Node, Attrs, node_definition::NodeTree};
use std::sync::Arc;

/// 创建大规模节点池用于测试
fn create_large_node_pool(node_count: usize) -> Arc<NodePool> {
    // 创建子节点
    let mut child_nodes = Vec::new();
    for i in 1..node_count {
        let node_type = match i % 5 {
            0 => "paragraph",
            1 => "heading",
            2 => "text",
            3 => "list_item",
            _ => "block",
        };

        let node = Node::new(
            &format!("node_{}", i),
            node_type.to_string(),
            Attrs::default(),
            vec![],
            vec![],
        );
        child_nodes.push(NodeTree(node, vec![]));
    }

    // 创建根节点
    let root = Node::new(
        "root",
        "document".to_string(),
        Attrs::default(),
        (1..node_count).map(|i| format!("node_{}", i).into()).collect(),
        vec![],
    );

    NodePool::from(NodeTree(root, child_nodes))
}

/// 基准测试：节点池层级操作
fn bench_hierarchy_operations(c: &mut Criterion) {
    let pool = create_large_node_pool(10000);
    let mut group = c.benchmark_group("hierarchy_ops");

    // 选择中间节点进行测试
    let test_node_id = "node_5000".into();

    group.bench_function("get_ancestors", |b| {
        b.iter(|| {
            let ancestors = pool.ancestors(&test_node_id);
            black_box(ancestors)
        });
    });

    group.bench_function("get_descendants", |b| {
        b.iter(|| {
            let descendants = pool.descendants(&test_node_id);
            black_box(descendants)
        });
    });

    group.bench_function("get_siblings", |b| {
        b.iter(|| {
            let siblings = pool.get_all_siblings(&test_node_id);
            black_box(siblings)
        });
    });

    group.bench_function("get_path", |b| {
        b.iter(|| {
            let path = pool.get_node_path(&test_node_id);
            black_box(path)
        });
    });

    group.bench_function("validate_hierarchy", |b| {
        b.iter(|| {
            let result = pool.validate_hierarchy();
            black_box(result)
        });
    });

    group.finish();
}

/// 基准测试：节点克隆 vs 结构共享
fn bench_node_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_operations");

    let node = Arc::new(Node::new(
        "test_node",
        "paragraph".to_string(),
        Attrs::default(),
        vec!["child1".into(), "child2".into(), "child3".into()],
        vec![],
    ));

    // 1. 节点克隆
    group.bench_function("node_clone", |b| {
        b.iter(|| {
            let cloned = (*node).clone();
            black_box(cloned)
        });
    });

    // 2. Arc 克隆（结构共享）
    group.bench_function("arc_clone", |b| {
        b.iter(|| {
            let cloned = Arc::clone(&node);
            black_box(cloned)
        });
    });

    // 4. 添加子节点（需要克隆）
    group.bench_function("insert_content", |b| {
        b.iter(|| {
            let updated = node.insert_content("new_child");
            black_box(updated)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_hierarchy_operations, bench_node_operations,);
criterion_main!(benches);
