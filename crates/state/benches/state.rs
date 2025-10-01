use criterion::{criterion_group, criterion_main, Criterion};
// Basic math benchmark, no imports needed

/// 基础状态基准测试
fn bench_basic_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("基础状态");

    // Transaction创建基准测试
    group.bench_function("基础操作", |b| {
        b.iter(|| {
            // 简单的状态操作基准测试
            let result = (0..50).sum::<i32>();
            criterion::black_box(result)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_basic_state);
criterion_main!(benches);
