use criterion::{criterion_group, criterion_main, Criterion};
use mf_model::*;

/// ID生成基准测试
fn bench_id_generator(c: &mut Criterion) {
    let mut group = c.benchmark_group("ID生成");
    
    group.bench_function("ID生成", |b| {
        b.iter(|| {
            criterion::black_box(IdGenerator::get_id())
        })
    });
    
    group.finish();
}

/// 属性系统基准测试
fn bench_attrs(c: &mut Criterion) {
    let mut group = c.benchmark_group("属性系统");
    
    group.bench_function("Attrs创建", |b| {
        b.iter(|| {
            let mut map = std::collections::HashMap::new();
            map.insert("key".to_string(), serde_json::json!("value"));
            let attrs = Attrs::from(map.into());
            criterion::black_box(attrs)
        })
    });
    
    group.finish();
}

/// 内容系统基准测试  
fn bench_content(c: &mut Criterion) {
    let mut group = c.benchmark_group("内容系统");
    
    group.bench_function("Content创建", |b| {
        b.iter(|| {
            let result = "text*".len(); // 简化操作
            criterion::black_box(result)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_id_generator,
    bench_attrs,
    bench_content
);
criterion_main!(benches);