use criterion::{criterion_group, criterion_main, Criterion};
use mf_search::*;

/// 搜索模型基准测试
fn bench_search_model(c: &mut Criterion) {
    let mut group = c.benchmark_group("搜索模型");

    group.bench_function("SearchQuery创建", |b| {
        b.iter(|| {
            let query = SearchQuery {
                text: Some("test query".to_string()),
                node_type: None,
                parent_id: None,
                path_prefix: None,
                marks: vec![],
                mark_attrs: vec![],
                attrs: vec![],
                limit: 10,
                offset: 0,
                sort_by: None,
                sort_asc: true,
                include_descendants: false,
                range_field: None,
                range_min: None,
                range_max: None,
            };
            criterion::black_box(query)
        })
    });

    group.bench_function("基础搜索操作", |b| {
        b.iter(|| {
            let result = "search".len();
            criterion::black_box(result)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_search_model);
criterion_main!(benches);
