# moduforge-file 文档

`moduforge-file` 提供 ModuForge-RS 的文件持久化层，包括追加式记录存储、文档容器、ZIP 归档和并行压缩功能。

## 概述

File 层提供高效的文件存储格式，支持增量写入、数据完整性校验和多种压缩策略。该模块实现了同步和异步两套 API，确保跨平台兼容性。

## 核心功能

- **追加式记录存储**：基于 append-only 的低级存储格式
- **文档容器**：支持多段数据的高级容器格式
- **数据完整性**：CRC32 校验和 Blake3 哈希验证
- **压缩支持**：Zstandard 压缩和并行压缩优化
- **ZIP 归档**：文档打包、快照和插件状态导出
- **内存映射**：高效的大文件读取和尾指针优化
- **异步支持**：完整的异步 API 和同步/异步互操作性

## 文件格式

### 记录格式（Record Format）

ModuForge 使用自定义的二进制记录格式：

```
文件结构:
┌─────────────────────────────────────────┐
│ 文件头 (16 bytes)                        │
│ - 魔数: "MFFILE01" (8 bytes)            │
│ - 预留: 0x00 * 8 (8 bytes)              │
├─────────────────────────────────────────┤
│ 记录 1                                   │
│ - 长度: u32 LE (4 bytes)                │
│ - CRC32: u32 LE (4 bytes)               │
│ - 数据: [u8; 长度]                       │
├─────────────────────────────────────────┤
│ 记录 2                                   │
│ ...                                      │
├─────────────────────────────────────────┤
│ 记录 N                                   │
└─────────────────────────────────────────┘
```

### 文档格式（Document Format）

文档格式在记录格式之上构建，支持多段数据和目录索引：

```
文档结构:
┌─────────────────────────────────────────┐
│ 文件头 (16 bytes)                        │
├─────────────────────────────────────────┤
│ 段 1 (压缩的数据段)                      │
├─────────────────────────────────────────┤
│ 段 2                                     │
├─────────────────────────────────────────┤
│ ...                                      │
├─────────────────────────────────────────┤
│ 段 N                                     │
├─────────────────────────────────────────┤
│ 目录记录 (bincode 序列化)                │
│ - entries: Vec<SegmentEntry>            │
│ - flags: u32                            │
│ - file_hash: [u8; 32]                   │
├─────────────────────────────────────────┤
│ 尾指针 (16 bytes)                        │
│ - 魔数: "TAIL" (4 bytes)                │
│ - 目录偏移: u64 LE (8 bytes)            │
│ - CRC32: u32 LE (4 bytes)               │
└─────────────────────────────────────────┘
```

## 记录层 API

### Writer - 追加式写入器

```rust
use mf_file::{Writer, HEADER_LEN};

// 创建写入器（预分配 64MB）
let mut writer = Writer::create("data.mff", 64 * 1024 * 1024)?;

// 追加数据，返回偏移量
let offset1 = writer.append(b"hello")?;
let offset2 = writer.append(b"world")?;

// 追加大数据
let big_data = vec![42u8; 128 * 1024];
let offset3 = writer.append(&big_data)?;

// 刷新缓冲区
writer.flush()?;

// 获取逻辑文件末尾位置
let end_position = writer.logical_end;
```

### Reader - 内存映射读取器

```rust
use mf_file::{Reader, Iter};

// 打开文件进行读取
let reader = Reader::open("data.mff")?;

// 通过偏移量读取数据
let data1 = reader.get_at(offset1)?;
assert_eq!(data1, b"hello");

// 迭代所有记录
for record in reader.iter() {
    println!("Record: {} bytes", record.len());
}

// 获取记录数量
let count = reader.iter().count();
```

## 文档层 API

### DocumentWriter - 文档写入器

```rust
use mf_file::{DocumentWriter, SegmentType};

// 开始写入新文档
let mut writer = DocumentWriter::begin("document.mfd")?;

// 添加各种类型的段
writer.add_segment(
    SegmentType("metadata".to_string()),
    b"Document metadata"
)?;

writer.add_segment(
    SegmentType("content".to_string()),
    b"Main document content"
)?;

writer.add_segment(
    SegmentType("index".to_string()),
    br#"{"entries": ["item1", "item2"]}"#
)?;

// 完成写入（生成目录和文件哈希）
writer.finalize()?;
```

### DocumentReader - 文档读取器

```rust
use mf_file::{DocumentReader, SegmentType};

// 打开文档
let reader = DocumentReader::open("document.mfd")?;

// 获取所有段信息
let segments = reader.segments();
println!("文档包含 {} 个段", segments.len());

// 读取特定类型的段
reader.read_segments(
    SegmentType("content".to_string()),
    |index, data| {
        println!("段 {}: {} bytes", index, data.len());
        Ok(())
    }
)?;

// 获取段的原始数据
let payload = reader.segment_payload(0)?;

// 获取文件哈希
let hash = reader.file_hash();
println!("文件 Blake3: {:x?}", hash);
```

## ZIP 归档功能

### ZipDocumentWriter - ZIP 文档写入

```rust
use mf_file::{ZipDocumentWriter, SegmentType};
use mf_file::zipdoc::formats::strategy::SnapshotFormat;

// 创建 ZIP 文档写入器
let mut writer = ZipDocumentWriter::new("archive.zip")?;

// 添加段到 ZIP
writer.add_segment(
    SegmentType("doc".to_string()),
    b"Document content"
)?;

// 添加插件状态
writer.add_plugin_state("history", b"plugin state data")?;

// 完成 ZIP 创建
writer.finalize()?;

// 导出带格式的快照
export_zip_with_format(
    "data.mfd",
    "snapshot.zip",
    SnapshotFormat::Full
)?;
```

### ZipDocumentReader - ZIP 文档读取

```rust
use mf_file::{ZipDocumentReader, MmapConfig};

// 配置内存映射策略
let config = MmapConfig::auto_detect("archive.zip")?;

// 打开 ZIP 文档
let reader = ZipDocumentReader::open_with_config("archive.zip", config)?;

// 读取段
let content = reader.get_segment(&SegmentType("doc".to_string()))?;

// 获取插件状态
let plugin_states = reader.get_plugin_states()?;
for (name, data) in plugin_states {
    println!("插件 {}: {} bytes", name, data.len());
}

// 流式读取（处理大文件）
let stream_reader = ZipStreamReader::new("large.zip")?;
for entry in stream_reader.entries() {
    println!("处理: {}", entry.name);
}
```

## 并行压缩

### ParallelCompressor - 并行压缩器

```rust
use mf_file::{ParallelCompressor, ParallelCompressionConfig};

// 配置并行压缩
let config = ParallelCompressionConfig {
    num_threads: 4,
    chunk_size: 1024 * 1024, // 1MB chunks
    compression_level: 3,
};

// 创建压缩器
let compressor = ParallelCompressor::new(config);

// 压缩大数据
let large_data = vec![0u8; 10_000_000];
let compressed = compressor.compress(&large_data)?;

// 解压
let decompressed = compressor.decompress(&compressed)?;
```

## 异步 API

### AsyncDocumentWriter - 异步文档写入

```rust
use mf_file::{AsyncDocumentWriter, SegmentType};

#[tokio::main]
async fn main() -> Result<()> {
    // 创建异步写入器（支持并行压缩）
    let writer = AsyncDocumentWriter::begin("async_doc.mfd").await?;

    // 异步添加段
    writer.add_segment(
        SegmentType("data".to_string()),
        b"Async data".to_vec()
    ).await?;

    // 批量添加段（并行处理）
    let segments = vec![
        (SegmentType("s1".to_string()), b"Data 1".to_vec()),
        (SegmentType("s2".to_string()), b"Data 2".to_vec()),
        (SegmentType("s3".to_string()), b"Data 3".to_vec()),
    ];
    writer.add_segments_batch(segments).await?;

    // 完成写入
    writer.finalize().await?;
    Ok(())
}
```

### AsyncDocumentReader - 异步文档读取

```rust
use mf_file::{AsyncDocumentReader, SegmentType};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    // 打开文档
    let reader = AsyncDocumentReader::open("async_doc.mfd").await?;

    // 异步读取段
    let data = reader.get_segment(&SegmentType("data".to_string())).await?;

    // 流式读取所有段
    let stream = reader.stream_segments();
    futures::pin_mut!(stream);

    while let Some(result) = stream.next().await {
        let (kind, data) = result?;
        println!("段 {}: {} bytes", kind.0, data.len());
    }

    Ok(())
}
```

## 跨兼容性测试

文件模块确保同步和异步实现之间的完全兼容性：

```rust
#[cfg(test)]
mod cross_compatibility_tests {
    use super::*;

    #[tokio::test]
    async fn sync_write_async_read() -> Result<()> {
        let path = "test.mfd";

        // 同步写入
        {
            let mut writer = DocumentWriter::begin(path)?;
            writer.add_segment(SegmentType("test".to_string()), b"Hello")?;
            writer.finalize()?;
        }

        // 异步读取
        {
            let reader = AsyncDocumentReader::open(path).await?;
            let data = reader.get_segment(&SegmentType("test".to_string())).await?;
            assert_eq!(data.as_deref(), Some(b"Hello".as_slice()));
        }

        Ok(())
    }

    #[tokio::test]
    async fn async_write_sync_read() -> Result<()> {
        let path = "test2.mfd";

        // 异步写入（标准模式，兼容同步）
        {
            let writer = AsyncDocumentWriter::begin_standard(path).await?;
            writer.add_segment(
                SegmentType("async".to_string()),
                b"From async".to_vec()
            ).await?;
            writer.finalize().await?;
        }

        // 同步读取
        {
            let reader = DocumentReader::open(path)?;
            let mut found = false;
            reader.read_segments(SegmentType("async".to_string()), |_, data| {
                assert_eq!(data, b"From async");
                found = true;
                Ok(())
            })?;
            assert!(found);
        }

        Ok(())
    }

    #[tokio::test]
    async fn tail_pointer_optimization() -> Result<()> {
        let path = "tail_test.mfd";

        // 写入多个段
        let mut writer = DocumentWriter::begin(path)?;
        for i in 0..100 {
            writer.add_segment(
                SegmentType(format!("seg{}", i)),
                format!("Segment {}", i).as_bytes()
            )?;
        }
        writer.finalize()?;

        // 验证快速打开（使用尾指针）
        let start = std::time::Instant::now();
        let reader = AsyncDocumentReader::open(path).await?;
        let duration = start.elapsed();

        // 应该在 100ms 内打开
        assert!(duration.as_millis() < 100);

        // 验证可以读取段
        let seg50 = reader.get_segment(&SegmentType("seg50".to_string())).await?;
        assert_eq!(seg50.as_deref(), Some(b"Segment 50".as_slice()));

        Ok(())
    }
}
```

## 性能优化

### 1. 预分配策略

```rust
// 为大文件预分配空间，减少文件系统碎片
let mut writer = Writer::create("large.mff", 1024 * 1024 * 1024)?; // 1GB
```

### 2. 内存映射

```rust
// 自动选择最佳内存映射策略
let config = MmapConfig::auto_detect("file.zip")?;
match config.file_size_category() {
    FileSizeCategory::Small => { /* 完全映射 */ }
    FileSizeCategory::Medium => { /* 分段映射 */ }
    FileSizeCategory::Large => { /* 流式处理 */ }
}
```

### 3. 并行压缩

```rust
// 大数据使用并行压缩
let writer = AsyncDocumentWriter::begin("large.mfd").await?;
// 自动检测并使用并行压缩
```

### 4. 尾指针优化

```rust
// 文档自动写入尾指针，实现 O(1) 目录查找
// 无需扫描整个文件
```

## 错误处理

```rust
use mf_file::{FileError, Result};

match operation() {
    Err(FileError::BadHeader) => {
        println!("文件头损坏或格式错误");
    }
    Err(FileError::BadCrc { offset, expected, actual }) => {
        println!("CRC 校验失败 @ {}: 期望 {} 实际 {}", offset, expected, actual);
    }
    Err(FileError::EmptyPayload) => {
        println!("不允许空数据段");
    }
    Err(FileError::Io(e)) => {
        println!("IO 错误: {}", e);
    }
    Ok(_) => {}
}
```

## 最佳实践

### 1. 选择合适的 API

```rust
// 简单场景：使用同步 API
let writer = DocumentWriter::begin("simple.mfd")?;

// 高并发场景：使用异步 API
let writer = AsyncDocumentWriter::begin("concurrent.mfd").await?;

// 需要兼容性：使用标准模式
let writer = AsyncDocumentWriter::begin_standard("compatible.mfd").await?;
```

### 2. 段类型命名

```rust
// ✅ 好：使用描述性名称
SegmentType("plugin.history.v1".to_string())
SegmentType("document.content".to_string())

// ❌ 差：使用模糊名称
SegmentType("data".to_string())
SegmentType("1".to_string())
```

### 3. 批量操作

```rust
// ✅ 好：批量添加段
writer.add_segments_batch(segments).await?;

// ❌ 差：逐个添加（性能差）
for (kind, data) in segments {
    writer.add_segment(kind, data).await?;
}
```

## 完整示例

### Price-RS 文件持久化

```rust
use mf_file::{DocumentWriter, DocumentReader, SegmentType};
use mf_state::State;

/// 保存 Price-RS 项目状态
pub fn save_project(state: &State, path: &str) -> Result<()> {
    let mut writer = DocumentWriter::begin(path)?;

    // 保存文档内容
    let doc_bytes = state.serialize_document()?;
    writer.add_segment(SegmentType("document".to_string()), &doc_bytes)?;

    // 保存插件状态
    for (plugin_name, plugin_state) in state.get_plugin_states() {
        let state_bytes = plugin_state.serialize()?;
        writer.add_segment(
            SegmentType(format!("plugin.{}", plugin_name)),
            &state_bytes
        )?;
    }

    // 保存元数据
    let metadata = format!(r#"{{
        "version": "{}",
        "timestamp": "{}",
        "node_count": {}
    }}"#, env!("CARGO_PKG_VERSION"), chrono::Utc::now(), state.node_count());

    writer.add_segment(SegmentType("metadata".to_string()), metadata.as_bytes())?;

    writer.finalize()?;
    Ok(())
}

/// 加载 Price-RS 项目
pub fn load_project(path: &str) -> Result<State> {
    let reader = DocumentReader::open(path)?;

    // 读取文档
    let mut doc_data = None;
    reader.read_segments(SegmentType("document".to_string()), |_, data| {
        doc_data = Some(data.to_vec());
        Ok(())
    })?;

    let doc = State::deserialize_document(&doc_data.unwrap())?;

    // 读取插件状态
    let mut plugin_states = HashMap::new();
    for entry in reader.segments() {
        if entry.kind.0.starts_with("plugin.") {
            let plugin_name = entry.kind.0.strip_prefix("plugin.").unwrap();
            let data = reader.segment_payload_for_entry(entry)?;
            plugin_states.insert(plugin_name.to_string(), data);
        }
    }

    // 恢复状态
    State::restore(doc, plugin_states)
}
```

## 下一步

- 查看 [moduforge-state](./state.md) 了解状态管理和持久化集成
- 查看 [moduforge-collaboration](./collaboration.md) 了解协作功能
- 浏览 [Price-RS 项目](https://github.com/LiRenTech/price-rs) 查看实际应用