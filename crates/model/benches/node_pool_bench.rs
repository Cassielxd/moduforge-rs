use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
    Throughput,
};
use mf_model::node_pool::{
    NodePool, QueryCacheConfig, LazyQueryConfig, QueryCondition,
};
use mf_model::{Node, Attrs, node_type::NodeTree};
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

/// 基准测试：节点池查询性能对比
fn bench_query_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_pool_query");

    for size in [100, 1000, 10000].iter() {
        let pool = create_large_node_pool(*size);

        // 1. 全扫描查询
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("full_scan", size),
            size,
            |b, _| {
                b.iter(|| {
                    let results = pool.filter_nodes(|node| {
                        black_box(node.r#type == "paragraph")
                    });
                    black_box(results)
                });
            },
        );

        // 2. 并行查询
        group.bench_with_input(
            BenchmarkId::new("parallel_query", size),
            size,
            |b, _| {
                b.iter(|| {
                    let results = pool.parallel_query(|node| {
                        black_box(node.r#type == "paragraph")
                    });
                    black_box(results)
                });
            },
        );

        // 3. 优化查询引擎（带索引）
        let config = QueryCacheConfig { capacity: 1000, enabled: true };
        let opt_engine =
            pool.optimized_query(config).expect("创建优化查询引擎失败");

        group.bench_with_input(
            BenchmarkId::new("optimized_indexed", size),
            size,
            |b, _| {
                b.iter(|| {
                    let results = opt_engine.by_type("paragraph");
                    black_box(results)
                });
            },
        );

        // 4. 懒加载查询引擎
        let lazy_config = LazyQueryConfig {
            cache_capacity: 1000,
            index_cache_capacity: 100,
            cache_enabled: true,
            index_build_threshold: 1, // 立即构建索引以测试性能
        };
        let mut lazy_engine = pool.lazy_query(lazy_config);

        // 预热：触发索引构建
        let _ = lazy_engine.by_type_lazy("paragraph");

        group.bench_with_input(
            BenchmarkId::new("lazy_cached", size),
            size,
            |b, _| {
                b.iter(|| {
                    let results = lazy_engine.by_type_lazy("paragraph");
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：复杂查询组合
fn bench_complex_queries(c: &mut Criterion) {
    let pool = create_large_node_pool(10000);
    let mut group = c.benchmark_group("complex_queries");

    // 1. 多条件过滤
    group.bench_function("multi_condition_filter", |b| {
        b.iter(|| {
            let results = pool.filter_nodes(|node| {
                black_box(
                    node.r#type == "paragraph"
                        && node.content.len() > 0
                        && !node.marks.is_empty(),
                )
            });
            black_box(results)
        });
    });

    // 2. 深度查询
    group.bench_function("depth_query", |b| {
        b.iter(|| {
            let results = pool.filter_nodes(|node| {
                let depth = pool.get_node_depth(&node.id);
                black_box(depth == Some(2))
            });
            black_box(results)
        });
    });

    // 3. 组合查询（使用懒加载引擎）
    let lazy_config = LazyQueryConfig::default();
    let mut lazy_engine = pool.lazy_query(lazy_config);

    group.bench_function("combined_query", |b| {
        b.iter(|| {
            let conditions = vec![
                QueryCondition::ByType("paragraph".to_string()),
                QueryCondition::ByDepth(2),
                QueryCondition::HasChildren,
            ];
            let results = lazy_engine.combined_query(&conditions);
            black_box(results)
        });
    });

    group.finish();
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

    // 3. 属性更新（需要克隆）
    group.bench_function("update_attr", |b| {
        b.iter(|| {
            let mut attrs = imbl::HashMap::new();
            attrs.insert("key".to_string(), serde_json::json!("value"));
            let updated = node.update_attr(attrs);
            black_box(updated)
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

/// 基准测试：查询引擎缓存效果
fn bench_cache_effectiveness(c: &mut Criterion) {
    let pool = create_large_node_pool(10000);
    let mut group = c.benchmark_group("cache_effectiveness");

    let lazy_config = LazyQueryConfig {
        cache_capacity: 1000,
        index_cache_capacity: 100,
        cache_enabled: true,
        index_build_threshold: 1,
    };
    let mut lazy_engine = pool.lazy_query(lazy_config);

    // 冷查询（首次）
    group.bench_function("cold_query", |b| {
        b.iter(|| {
            // 每次迭代使用不同的查询名称模拟冷查询
            let results = lazy_engine.smart_query("unique_query", || {
                pool.parallel_query(|node| node.r#type == "paragraph")
            });
            black_box(results)
        });
    });

    // 预热缓存
    for _ in 0..10 {
        let _ = lazy_engine.by_type_lazy("paragraph");
    }

    // 热查询（缓存命中）
    group.bench_function("hot_query", |b| {
        b.iter(|| {
            let results = lazy_engine.by_type_lazy("paragraph");
            black_box(results)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_query_strategies,
    bench_complex_queries,
    bench_hierarchy_operations,
    bench_node_operations,
    bench_cache_effectiveness,
);
criterion_main!(benches);
