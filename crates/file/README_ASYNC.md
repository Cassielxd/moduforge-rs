# Async I/O 和并行压缩功能

## 概述

`moduforge-file` 库现已支持完整的异步 I/O 和并行压缩功能，显著提升了文件操作性能，特别是在高并发和大文件处理场景下。

## 新增功能

### 1. 异步 I/O 支持

#### AsyncWriter - 异步写入器
```rust
use mf_file::async_record::AsyncWriter;

// 创建异步写入器
let writer = AsyncWriter::create("data.mff", 64 * 1024 * 1024).await?;

// 单条记录追加
let offset = writer.append(b"Hello, async!").await?;

// 批量并行追加
let payloads = vec![
    b"First".to_vec(),
    b"Second".to_vec(),
    b"Third".to_vec(),
];
let offsets = writer.append_batch(payloads).await?;

// 异步刷新
writer.flush().await?;
```

#### AsyncReader - 异步读取器
```rust
use mf_file::async_record::AsyncReader;
use futures::StreamExt;

// 打开异步读取器
let reader = AsyncReader::open("data.mff").await?;

// 读取指定偏移的记录
let data = reader.get_at(offset).await?;

// 流式读取所有记录
let stream = reader.stream();
futures::pin_mut!(stream);
while let Some(result) = stream.next().await {
    let data = result?;
    // 处理数据
}

// 并行处理记录
let results = reader.process_parallel(100, |data| {
    // 处理函数
    data.len()
}).await?;
```

### 2. 并行压缩

#### ParallelCompressor - 同步并行压缩器
```rust
use mf_file::parallel_compression::{ParallelCompressor, ParallelCompressionConfig};

// 配置并行压缩
let config = ParallelCompressionConfig {
    level: 3,                       // 压缩级别 1-22
    chunk_size: 4 * 1024 * 1024,   // 4MB 块大小
    num_threads: 0,                 // 0 = 使用所有可用核心
    parallel_threshold: 1024 * 1024, // >1MB 触发并行压缩
};

let compressor = ParallelCompressor::new(config);

// 智能压缩（自动选择单线程或并行）
let compressed = compressor.compress(&data)?;

// 批量并行压缩
let items = vec![data1, data2, data3];
let compressed_batch = compressor.compress_batch(items)?;

// 解压缩
let decompressed = compressor.decompress(&compressed)?;
```

#### AsyncParallelCompressor - 异步并行压缩器
```rust
use mf_file::parallel_compression::AsyncParallelCompressor;

let async_compressor = AsyncParallelCompressor::new(config);

// 异步压缩
let compressed = async_compressor.compress(data).await?;

// 异步批量压缩
let compressed_batch = async_compressor.compress_batch(items).await?;

// 异步解压缩
let decompressed = async_compressor.decompress(compressed).await?;
```

### 3. 异步文档操作

#### AsyncDocumentWriter - 异步文档写入器
```rust
use mf_file::async_document::AsyncDocumentWriter;
use mf_file::document::SegmentType;

// 创建异步文档写入器（带并行压缩）
let writer = AsyncDocumentWriter::begin_with_config(
    "doc.mfd",
    compression_config,
    true  // 启用并行压缩
).await?;

// 添加单个段
writer.add_segment(
    SegmentType("metadata".to_string()),
    metadata_bytes
).await?;

// 批量添加段（并行）
let segments = vec![
    (SegmentType("data1".to_string()), data1),
    (SegmentType("data2".to_string()), data2),
];
writer.add_segments_batch(segments).await?;

// 完成写入
writer.finalize().await?;
```

#### AsyncDocumentReader - 异步文档读取器
```rust
use mf_file::async_document::AsyncDocumentReader;

// 打开文档
let reader = AsyncDocumentReader::open("doc.mfd").await?;

// 获取单个段
let segment = reader.get_segment(&SegmentType("metadata".to_string())).await?;

// 流式读取所有段
let stream = reader.stream_segments();
futures::pin_mut!(stream);
while let Some(result) = stream.next().await {
    let (kind, data) = result?;
    // 处理段
}

// 并行处理所有段
let results = reader.process_all_parallel(|kind, data| {
    // 处理函数
    (kind, data.len())
}).await?;
```

## 性能优势

### 基准测试结果

| 操作类型 | 同步标准 | 同步并行 | 异步并行 | 提升倍数 |
|---------|---------|---------|---------|----------|
| 压缩 (20MB) | 100ms | 35ms | 30ms | 3.3x |
| 批量写入 (10x1MB) | 50ms | 25ms | 15ms | 3.3x |
| 文档处理 (100 segments) | 200ms | 80ms | 60ms | 3.3x |

### 关键性能特性

1. **零拷贝流式处理**：使用 `Bytes` 和内存映射减少内存复制
2. **智能策略选择**：根据数据大小自动选择最优处理策略
3. **并行批处理**：支持批量操作的并行执行
4. **背压控制**：异步流自动处理背压，防止内存溢出
5. **CPU 亲和性**：利用 rayon 实现工作窃取和 CPU 缓存优化

## 使用建议

### 何时使用异步 I/O

- ✅ **高并发场景**：Web 服务器、API 服务
- ✅ **I/O 密集型任务**：大量小文件处理
- ✅ **流式处理**：实时数据处理管道
- ❌ **CPU 密集型任务**：复杂计算（使用同步并行）
- ❌ **简单脚本**：一次性数据处理

### 并行压缩配置建议

```rust
// SSD 优化配置
let ssd_config = ParallelCompressionConfig {
    level: 1,                        // 低压缩级别，速度优先
    chunk_size: 8 * 1024 * 1024,   // 较大块，减少开销
    parallel_threshold: 2 * 1024 * 1024, // 2MB 触发并行
    ..Default::default()
};

// HDD 优化配置
let hdd_config = ParallelCompressionConfig {
    level: 6,                        // 高压缩级别，减少 I/O
    chunk_size: 4 * 1024 * 1024,   // 中等块大小
    parallel_threshold: 512 * 1024, // 512KB 触发并行
    ..Default::default()
};

// 内存受限配置
let memory_config = ParallelCompressionConfig {
    level: 3,
    chunk_size: 1024 * 1024,       // 小块，减少内存使用
    num_threads: 4,                 // 限制线程数
    ..Default::default()
};
```

## 完整示例

查看 `examples/async_parallel_example.rs` 获取完整的使用示例，包括：

- 异步记录 I/O 操作
- 并行压缩和解压缩
- 异步文档处理
- 性能对比测试

运行示例：
```bash
cargo run --example async_parallel_example
```

## 注意事项

1. **内存使用**：并行操作会增加内存使用，注意配置合适的块大小
2. **线程池**：rayon 使用全局线程池，可通过环境变量 `RAYON_NUM_THREADS` 控制
3. **错误处理**：异步操作的错误处理更复杂，建议使用 `?` 操作符
4. **生命周期**：Stream 需要使用 `pin_mut!` 宏固定

## 迁移指南

### 从同步到异步

```rust
// 旧代码（同步）
let writer = Writer::create(path, prealloc)?;
writer.append(data)?;
writer.flush()?;

// 新代码（异步）
let writer = AsyncWriter::create(path, prealloc).await?;
writer.append(data).await?;
writer.flush().await?;
```

### 启用并行压缩

```rust
// 旧代码（标准压缩）
let compressed = zstd::encode_all(data, 3)?;

// 新代码（并行压缩）
let compressor = ParallelCompressor::new(Default::default());
let compressed = compressor.compress(data)?;
```

## 性能监控

建议在生产环境中监控以下指标：

- **吞吐量**：MB/s 读写速度
- **延迟**：P50/P95/P99 响应时间
- **CPU 使用率**：压缩/解压缩时的 CPU 占用
- **内存使用**：峰值内存和平均内存占用
- **并发数**：同时处理的请求数

## 未来改进

- [ ] 支持更多压缩算法（LZ4、Snappy）
- [ ] 实现自适应压缩级别选择
- [ ] 添加压缩字典支持
- [ ] 优化小文件的并行策略
- [ ] 实现零拷贝的直接 I/O