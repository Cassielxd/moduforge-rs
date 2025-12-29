use mf_file::*;
use std::time::Instant;
use std::path::PathBuf;
use serde_json;

#[tokio::main]
async fn main() -> Result<()> {
    use mf_file::{
        async_document::{AsyncDocumentWriter, AsyncDocumentReader},
        parallel_compression::ParallelCompressionConfig,
        document::SegmentType,
    };

    println!("=== 异步文档读写演示 ===\n");

    // 使用真实目录 - 在当前目录下创建 test_data 文件夹
    let dir = PathBuf::from("test_data");
    tokio::fs::create_dir_all(&dir).await.ok();

    let path = dir.join("async_doc.mff");
    let start = Instant::now();

    // 准备测试数据
    let test_metadata = serde_json::json!({
        "version": "1.0.0",
        "author": "异步示例",
        "created": "2024-01-01T00:00:00Z",
        "description": "带并行压缩的测试文档"
    });

    let test_config = serde_json::json!({
        "compression": "并行",
        "chunk_size": 512 * 1024,
        "level": 3
    });

    // 生成真实数据
    let mut large_json_data = Vec::new();
    for i in 0..10000 {
        let record = serde_json::json!({
            "id": i,
            "名称": format!("记录 {}", i),
            "数值": i as f64 * 3.14159,
            "时间戳": 1704067200 + i,
            "标签": vec![format!("标签{}", i % 10), format!("分类{}", i % 5)]
        });
        large_json_data.push(record);
    }
    let large_json_bytes =
        serde_json::to_vec(&large_json_data).map_err(|e| {
            FileError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })?;

    // 生成带模式的二进制数据
    let mut binary_data = Vec::with_capacity(2 * 1024 * 1024);
    for i in 0..2 * 1024 * 1024 {
        binary_data.push(((i * 7 + 13) % 256) as u8);
    }

    println!("写入文档到: {}", path.display());

    // 使用并行压缩写入文档
    let bytes_written = {
        let compression_config = ParallelCompressionConfig {
            level: 3,
            chunk_size: 512 * 1024,
            parallel_threshold: 100 * 1024, // 100KB 阈值
            ..Default::default()
        };

        let writer = AsyncDocumentWriter::begin_with_config(
            &path,
            compression_config,
            true, // 启用并行压缩
        )
        .await?;

        // 添加各种类型的段
        let segments = vec![
            (
                SegmentType("元数据".to_string()),
                serde_json::to_vec(&test_metadata).map_err(|e| {
                    FileError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e,
                    ))
                })?,
            ),
            (
                SegmentType("配置".to_string()),
                serde_json::to_vec(&test_config).map_err(|e| {
                    FileError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e,
                    ))
                })?,
            ),
            (SegmentType("JSON数据".to_string()), large_json_bytes.clone()),
            (SegmentType("二进制数据".to_string()), binary_data.clone()),
            (
                SegmentType("文本数据".to_string()),
                "这是一个将被压缩的示例文本段。"
                    .repeat(1000)
                    .as_bytes()
                    .to_vec(),
            ),
        ];

        let total_size: usize =
            segments.iter().map(|(_, data)| data.len()).sum();
        println!(
            "- 未压缩总大小: {:.2} MB",
            total_size as f64 / 1024.0 / 1024.0
        );

        // 批量添加（并行）
        writer.add_segments_batch(segments).await?;
        writer.finalize().await?;

        total_size
    };

    // 获取压缩后的文件大小
    let file_metadata = tokio::fs::metadata(&path).await?;
    let compressed_size = file_metadata.len();
    println!(
        "- 压缩后文件大小: {:.2} MB",
        compressed_size as f64 / 1024.0 / 1024.0
    );
    println!(
        "- 压缩率: {:.1}%",
        compressed_size as f64 / bytes_written as f64 * 100.0
    );

    // 读取文档
    println!("\n正在读取文档...");
    {
        let reader = AsyncDocumentReader::open(&path).await?;

        // 获取文档信息
        let dir = reader.directory();
        println!("- 文档包含 {} 个段", dir.entries.len());
        println!("- 压缩标志: 0x{:04X}", dir.flags);

        // 获取并验证元数据
        let metadata_bytes = reader
            .get_segment(&SegmentType("元数据".to_string()))
            .await?
            .unwrap();
        let read_metadata: serde_json::Value =
            serde_json::from_slice(&metadata_bytes).map_err(|e| {
                FileError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?;
        println!("- 元数据验证通过: 版本={}", read_metadata["version"]);

        // 获取并验证配置
        let config_bytes = reader
            .get_segment(&SegmentType("配置".to_string()))
            .await?
            .unwrap();
        let read_config: serde_json::Value =
            serde_json::from_slice(&config_bytes).map_err(|e| {
                FileError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?;
        println!("- 配置验证通过: 压缩={}", read_config["compression"]);

        // 验证 JSON 数据
        let json_bytes = reader
            .get_segment(&SegmentType("JSON数据".to_string()))
            .await?
            .unwrap();
        let read_json_data: Vec<serde_json::Value> =
            serde_json::from_slice(&json_bytes).map_err(|e| {
                FileError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?;
        println!("- JSON 数据验证通过: {} 条记录", read_json_data.len());

        // 验证二进制数据
        let binary_bytes = reader
            .get_segment(&SegmentType("二进制数据".to_string()))
            .await?
            .unwrap();
        assert_eq!(binary_bytes, binary_data);
        println!("- 二进制数据验证通过: {} 字节", binary_bytes.len());

        // 流式读取所有段并计算大小
        use futures::StreamExt;
        let stream = reader.stream_segments();
        futures::pin_mut!(stream);
        let mut total_decompressed = 0;
        let mut segment_info = Vec::new();
        while let Some(result) = stream.next().await {
            let (kind, data) = result?;
            total_decompressed += data.len();
            segment_info.push((kind.0.clone(), data.len()));
        }
        println!("\n解压后的段大小:");
        for (name, size) in &segment_info {
            println!("  - {}: {:.2} KB", name, *size as f64 / 1024.0);
        }
        println!(
            "- 解压后总大小: {:.2} MB",
            total_decompressed as f64 / 1024.0 / 1024.0
        );

        // 并行处理所有段进行分析
        let analysis_start = Instant::now();
        let results = reader
            .process_all_parallel(|kind, data| {
                // 模拟数据处理
                let checksum = data
                    .iter()
                    .fold(0u64, |acc, &b| acc.wrapping_add(b as u64));
                (kind.0, data.len(), checksum)
            })
            .await?;
        println!("\n并行分析完成，耗时 {:?}", analysis_start.elapsed());
        println!("分析结果:");
        for (name, size, checksum) in &results {
            println!("  - {}: 大小={}, 校验和={}", name, size, checksum);
        }

        // 获取文档长度
        let doc_len = reader.logical_len().await;
        println!("\n文档逻辑长度: {} 字节", doc_len);
    }

    println!("\n演示总耗时: {:?}", start.elapsed());

    // 显示最终文件信息
    println!("\n最终文件信息:");
    println!("- 路径: {}", path.display());
    println!("- 大小: {} 字节", compressed_size);
    println!("- 成功写入并验证 ✓");

    println!("\n✅ 演示成功完成！");
    println!("📁 文件保存在: {}", path.display());

    Ok(())
}
