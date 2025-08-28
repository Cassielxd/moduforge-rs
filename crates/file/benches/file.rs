use criterion::{criterion_group, criterion_main, Criterion};
use mf_file::*;

/// 基础文件操作基准测试
fn bench_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("文件操作");

    // 基础文件读写
    group.bench_function("基础读写操作", |b| {
        b.iter(|| {
            let data = b"test data";
            let result = data.len();
            criterion::black_box(result)
        })
    });

    // TypeWrapper操作（使用正确的构造函数）
    group.bench_function("TypeWrapper操作", |b| {
        b.iter(|| {
            let wrapper = TypeWrapper { 
                type_id: "test".to_string(),
                data: b"test data".to_vec()
            };
            criterion::black_box(wrapper)
        })
    });

    // 历史编码（使用正确的参数）
    group.bench_function("历史编码", |b| {
        let test_data = vec![TypeWrapper { 
            type_id: "test".to_string(),
            data: b"test data".to_vec()
        }];
        b.iter(|| {
            let encoded = encode_history_frames(&test_data, false);
            criterion::black_box(encoded)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_file_operations
);
criterion_main!(benches);