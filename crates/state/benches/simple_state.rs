use criterion::{criterion_group, criterion_main, Criterion};
use mf_state::*;
use mf_model::*;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// 状态创建和初始化基准测试
fn bench_state_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("状态创建");

    // 基本状态创建
    group.bench_function("基本状态创建", |b| {
        b.to_async(&rt).iter(|| async {
            let schema = Arc::new(Schema::default());
            let state_config = StateConfig::new()
                .with_schema(schema);
            
            let state = State::create(state_config).await.unwrap();
            criterion::black_box(state)
        })
    });

    // 状态版本号生成
    group.bench_function("状态版本生成", |b| {
        b.iter(|| {
            criterion::black_box(get_state_version())
        })
    });

    group.finish();
}

/// 事务操作基准测试
fn bench_transaction_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("事务操作");

    // 事务创建
    group.bench_function("事务创建", |b| {
        b.to_async(&rt).iter_batched(
            || {
                rt.block_on(async {
                    let schema = Arc::new(Schema::default());
                    let state_config = StateConfig::new()
                        .with_schema(schema);
                    State::create(state_config).await.unwrap()
                })
            },
            |state| async move {
                let transaction = Transaction::new(&state);
                criterion::black_box(transaction)
            },
            criterion::BatchSize::SmallInput
        )
    });

    // 事务ID生成性能
    group.bench_function("事务ID生成", |b| {
        b.iter(|| {
            criterion::black_box(get_transaction_id())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_state_creation,
    bench_transaction_operations
);
criterion_main!(benches);