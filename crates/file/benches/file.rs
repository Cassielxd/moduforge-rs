use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use mf_file::*;
use std::io::Cursor;

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

/// ZIP memmap2 性能基准测试
fn bench_zip_mmap_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("ZIP_memmap2_性能");
    
    // 测试不同大小的文件
    for size_mb in [1, 5, 10, 20].iter() {
        let size_bytes = size_mb * 1024 * 1024;
        
        // 创建测试数据
        let test_data = vec![42u8; size_bytes];
        
        // 创建 ZIP 文件
        let mut zip_data = Vec::new();
        {
            let cursor = Cursor::new(&mut zip_data);
            let mut writer = ZipDocumentWriter::new(cursor).unwrap();
            writer.add_stored("test.bin", &test_data).unwrap();
            writer.finalize().unwrap();
        }
        
        // 标准读取基准
        group.bench_with_input(
            BenchmarkId::new("标准读取", format!("{}MB", size_mb)),
            &zip_data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(data.clone());
                    let mut reader = ZipDocumentReader::with_mmap_config(
                        cursor, 
                        MmapConfig { 
                            threshold: u64::MAX, // 禁用 mmap
                            huge_file_threshold: 20 * 1024 * 1024,
                            stream_chunk_size: 8 * 1024 * 1024,
                            enable_streaming: false,
                            ..Default::default() 
                        }
                    ).unwrap();
                    let result = reader.read_standard("test.bin").unwrap();
                    criterion::black_box(result)
                })
            },
        );
        
        // memmap2 读取基准
        group.bench_with_input(
            BenchmarkId::new("memmap2读取", format!("{}MB", size_mb)),
            &zip_data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(data.clone());
                    let mut reader = ZipDocumentReader::with_mmap_config(
                        cursor,
                        MmapConfig {
                            threshold: 0, // 强制使用 mmap
                            huge_file_threshold: 20 * 1024 * 1024,
                            stream_chunk_size: 8 * 1024 * 1024,
                            enable_streaming: false,
                            ..Default::default()
                        }
                    ).unwrap();
                    let result = reader.read_all("test.bin").unwrap();
                    criterion::black_box(result)
                })
            },
        );
        
        // 自动选择读取基准
        group.bench_with_input(
            BenchmarkId::new("自动选择读取", format!("{}MB", size_mb)),
            &zip_data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(data.clone());
                    let mut reader = ZipDocumentReader::new(cursor).unwrap();
                    let result = reader.read_all("test.bin").unwrap();
                    criterion::black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

/// ZIP 缓存性能基准测试
fn bench_zip_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("ZIP_缓存性能");
    
    // 创建包含多个大文件的 ZIP
    let mut zip_data = Vec::new();
    {
        let cursor = Cursor::new(&mut zip_data);
        let mut writer = ZipDocumentWriter::new(cursor).unwrap();
        
        for i in 1..=5 {
            let data = vec![(i * 10) as u8; 2 * 1024 * 1024]; // 2MB each
            writer.add_stored(&format!("file{}.bin", i), &data).unwrap();
        }
        
        writer.finalize().unwrap();
    }
    
    // 无缓存重复读取
    group.bench_function("无缓存重复读取", |b| {
        b.iter(|| {
            let cursor = Cursor::new(zip_data.clone());
            let mut reader = ZipDocumentReader::with_mmap_config(
                cursor,
                MmapConfig {
                    threshold: u64::MAX, // 禁用 mmap
                    huge_file_threshold: 20 * 1024 * 1024,
                    stream_chunk_size: 8 * 1024 * 1024,
                    enable_streaming: false,
                    ..Default::default()
                }
            ).unwrap();
            
            for i in 1..=5 {
                let result = reader.read_standard(&format!("file{}.bin", i)).unwrap();
                criterion::black_box(result);
            }
        })
    });
    
    // 有缓存重复读取
    group.bench_function("有缓存重复读取", |b| {
        b.iter(|| {
            let cursor = Cursor::new(zip_data.clone());
            let mut reader = ZipDocumentReader::with_mmap_config(
                cursor,
                MmapConfig {
                    threshold: 0, // 强制使用 mmap
                    max_maps: 10,
                    huge_file_threshold: 20 * 1024 * 1024,
                    stream_chunk_size: 8 * 1024 * 1024,
                    enable_streaming: false,
                    ..Default::default()
                }
            ).unwrap();
            
            // 第一次读取建立缓存
            for i in 1..=5 {
                let _result = reader.read_all(&format!("file{}.bin", i)).unwrap();
            }
            
            // 第二次读取命中缓存
            for i in 1..=5 {
                let result = reader.read_all(&format!("file{}.bin", i)).unwrap();
                criterion::black_box(result);
            }
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_file_operations,
    bench_zip_mmap_performance,
    bench_zip_cache_performance
);
criterion_main!(benches);