use criterion::{criterion_group, criterion_main, Criterion};

/// 基础持久化基准测试
fn bench_basic_persistence(c: &mut Criterion) {
    let mut group = c.benchmark_group("基础持久化");

    // 简单的基准测试，避免复杂API
    group.bench_function("持久化基础操作", |b| {
        b.iter(|| {
            // 简单的计算操作作为占位符
            let result = (0..100).sum::<i32>();
            criterion::black_box(result)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_basic_persistence);
criterion_main!(benches);
