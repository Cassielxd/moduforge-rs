use criterion::{criterion_group, criterion_main, Criterion};

/// 基础变换基准测试
fn bench_basic_transform(c: &mut Criterion) {
    let mut group = c.benchmark_group("基础变换");

    group.bench_function("变换基础操作", |b| {
        b.iter(|| {
            // 简单的变换操作基准测试
            let result = (0..75).map(|x| x * 2).sum::<i32>();
            criterion::black_box(result)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_basic_transform);
criterion_main!(benches);
