use std::fs::File;
use std::io::Result;
use mf_file::{ZipDocumentWriter, ZipDocumentReader, MmapConfig};

fn main() -> Result<()> {
    println!("=== 智能回调处理演示 ===\n");

    // 创建包含不同大小文件的测试ZIP
    create_test_files()?;

    // 演示智能回调处理
    demo_smart_callback_processing()?;

    // 演示批量智能处理
    demo_batch_smart_processing()?;

    // 清理
    let _ = std::fs::remove_file("smart_callback_test.ysf");
    println!("演示完成！");
    Ok(())
}

fn create_test_files() -> Result<()> {
    println!("📁 创建测试文件...");

    let file = File::create("smart_callback_test.ysf")?;
    let mut writer = ZipDocumentWriter::new(file)?;

    // 小文件 (1KB) - 预期：一次性回调
    let tiny_data = vec![1u8; 1024];
    writer.add_stored("tiny.bin", &tiny_data)?;

    // 中等文件 (3MB) - 预期：mmap 零拷贝回调
    let medium_data = vec![2u8; 3 * 1024 * 1024];
    writer.add_deflated("medium.data", &medium_data)?;

    // 超大文件 (25MB) - 预期：流式回调处理
    let large_data = vec![3u8; 25 * 1024 * 1024];
    writer.add_stored("large.blob", &large_data)?;

    writer.finalize()?;
    println!("✅ 测试文件创建完成\n");
    Ok(())
}

fn demo_smart_callback_processing() -> Result<()> {
    // 配置智能处理
    let config = MmapConfig {
        threshold: 1024 * 1024,                // 1MB - mmap阈值
        huge_file_threshold: 20 * 1024 * 1024, // 20MB - 流式阈值
        stream_chunk_size: 4 * 1024 * 1024,    // 4MB - 流式块大小
        enable_streaming: true,
        ..Default::default()
    };

    let file = File::open("smart_callback_test.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config.clone())?;

    println!("🧠 智能回调处理演示:");
    println!(
        "{:<12} {:<10} {:<12} {:<15} {:<20}",
        "文件名", "原始大小", "推荐策略", "实际处理方式", "回调统计"
    );
    println!("{}", "-".repeat(75));

    let files = ["tiny.bin", "medium.data", "large.blob"];

    for filename in &files {
        demonstrate_smart_callback(&mut reader, filename)?;
    }

    println!("\n💡 智能回调的优势:");
    println!("   🚀 小文件：一次性回调，减少函数调用开销");
    println!("   ⚡ 中等文件：mmap 零拷贝，高性能直接访问");
    println!("   💾 超大文件：流式回调，恒定内存占用");
    println!("   🔄 自动回退：失败时智能选择备用策略");

    Ok(())
}

fn demonstrate_smart_callback(
    reader: &mut ZipDocumentReader<File>,
    filename: &str,
) -> Result<()> {
    let file_info = reader.get_file_info(filename)?;

    let size_str = format_bytes(file_info.size);
    let strategy_str = format_strategy(&file_info.recommended_strategy);

    // 回调计数器和统计
    let mut callback_count = 0;
    let mut total_bytes = 0;
    let mut min_chunk = usize::MAX;
    let mut max_chunk = 0;

    let start = std::time::Instant::now();

    // 使用智能回调处理
    let result = reader.process_smart(filename, |chunk| {
        callback_count += 1;
        total_bytes += chunk.len();
        min_chunk = min_chunk.min(chunk.len());
        max_chunk = max_chunk.max(chunk.len());

        // 这里可以进行实际的数据处理
        // 例如：计算哈希、写入另一个文件、网络传输等
        verify_chunk_data(chunk, &file_info)?;

        Ok(())
    });

    let duration = start.elapsed();

    let (method_str, callback_stats) = match result {
        Ok(()) => {
            let method = match (file_info.recommended_strategy, callback_count)
            {
                (mf_file::ProcessingStrategy::Standard, 1) => "一次性回调",
                (mf_file::ProcessingStrategy::MemoryMap, 1) => "mmap零拷贝",
                (mf_file::ProcessingStrategy::Streaming, n) if n > 1 => {
                    "流式回调"
                },
                _ => "混合策略",
            };

            let stats = if callback_count == 1 {
                format!("1次回调 {}字节", total_bytes)
            } else {
                format!("{}次回调 {}字节", callback_count, total_bytes)
            };

            (method, stats)
        },
        Err(e) => ("错误", format!("❌ {}", e)),
    };

    println!(
        "{:<12} {:<10} {:<12} {:<15} {:<20}",
        filename, size_str, strategy_str, method_str, callback_stats
    );

    // 显示详细统计
    if callback_count > 1 {
        println!(
            "             └─ 块大小: {}B - {}B, 耗时: {:.2}ms",
            min_chunk,
            max_chunk,
            duration.as_secs_f64() * 1000.0
        );
    } else if duration.as_millis() > 0 {
        println!(
            "             └─ 耗时: {:.2}ms",
            duration.as_secs_f64() * 1000.0
        );
    }

    Ok(())
}

fn demo_batch_smart_processing() -> Result<()> {
    println!("\n🚀 批量智能处理演示:");

    let config = MmapConfig {
        threshold: 1024 * 1024,
        huge_file_threshold: 20 * 1024 * 1024,
        stream_chunk_size: 4 * 1024 * 1024,
        enable_streaming: true,
        ..Default::default()
    };

    let file = File::open("smart_callback_test.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config)?;

    let files = ["tiny.bin", "medium.data", "large.blob"];
    let start = std::time::Instant::now();

    // 使用批量智能处理
    reader.process_files_smart(&files, |filename, data| {
        println!("   处理 {}: {} 字节", filename, data.len());

        // 这里可以进行批量处理逻辑
        // 例如：数据验证、格式转换、存储等
        perform_batch_processing(filename, data)?;

        Ok(())
    })?;

    let duration = start.elapsed();
    println!("✅ 批量处理完成，耗时: {:.2}ms", duration.as_secs_f64() * 1000.0);

    println!("\n📊 批量处理的智能优化:");
    println!("   🎯 自动识别每个文件的最优处理策略");
    println!("   💾 大文件流式处理，避免内存溢出");
    println!("   ⚡ 中等文件零拷贝，最大化性能");
    println!("   🔄 统一接口，简化批量操作代码");

    Ok(())
}

fn verify_chunk_data(
    chunk: &[u8],
    file_info: &mf_file::FileInfo,
) -> Result<()> {
    // 简单的数据验证示例
    if chunk.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Empty chunk received",
        ));
    }

    // 根据文件大小验证预期的数据模式
    let expected_byte = match file_info.category {
        mf_file::FileSizeCategory::Small => 1u8, // tiny.bin
        mf_file::FileSizeCategory::Large => 2u8, // medium.data
        mf_file::FileSizeCategory::Huge => 3u8,  // large.blob
    };

    // 检查数据的一致性（这只是示例，实际应用中可能有更复杂的验证）
    if !chunk.iter().all(|&b| b == expected_byte) {
        // 注意：压缩文件可能不会保持原始字节模式
        // 这里只是演示验证逻辑的可能性
    }

    Ok(())
}

fn perform_batch_processing(
    filename: &str,
    data: &[u8],
) -> Result<()> {
    // 模拟批量处理操作
    match filename {
        name if name.ends_with(".bin") => {
            // 二进制文件处理
            println!(
                "       └─ 二进制文件处理: 校验和 = {}",
                calculate_simple_checksum(data)
            );
        },
        name if name.ends_with(".data") => {
            // 数据文件处理
            println!(
                "       └─ 数据文件处理: 压缩率 = {:.1}%",
                estimate_compression_ratio(data)
            );
        },
        name if name.ends_with(".blob") => {
            // 大型对象处理
            println!(
                "       └─ 大型对象处理: 分块数 = {}",
                (data.len() + 4095) / 4096
            );
        },
        _ => {
            println!("       └─ 通用处理");
        },
    }

    Ok(())
}

fn calculate_simple_checksum(data: &[u8]) -> u32 {
    data.iter().map(|&b| b as u32).sum()
}

fn estimate_compression_ratio(data: &[u8]) -> f64 {
    // 简单估算：计算数据的重复程度
    let unique_bytes =
        data.iter().collect::<std::collections::HashSet<_>>().len();
    (unique_bytes as f64 / 256.0) * 100.0
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

fn format_strategy(strategy: &mf_file::ProcessingStrategy) -> &'static str {
    match strategy {
        mf_file::ProcessingStrategy::Standard => "标准读取",
        mf_file::ProcessingStrategy::MemoryMap => "内存映射",
        mf_file::ProcessingStrategy::Streaming => "流式处理",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_callback_processing() -> Result<()> {
        // 创建简单测试文件
        let file = File::create("test_callback.ysf")?;
        let mut writer = ZipDocumentWriter::new(file)?;

        let test_data = vec![42u8; 2048];
        writer.add_stored("test.bin", &test_data)?;
        writer.finalize()?;

        // 测试智能回调处理
        let file = File::open("test_callback.ysf")?;
        let mut reader = ZipDocumentReader::new(file)?;

        let mut callback_count = 0;
        let mut total_bytes = 0;

        reader.process_smart("test.bin", |chunk| {
            callback_count += 1;
            total_bytes += chunk.len();
            assert!(!chunk.is_empty());
            Ok(())
        })?;

        assert_eq!(callback_count, 1); // 小文件应该只回调一次
        assert_eq!(total_bytes, 2048);

        // 清理
        let _ = std::fs::remove_file("test_callback.ysf");
        Ok(())
    }

    #[test]
    fn test_batch_smart_processing() -> Result<()> {
        // 创建多文件测试
        let file = File::create("test_batch.ysf")?;
        let mut writer = ZipDocumentWriter::new(file)?;

        writer.add_stored("file1.txt", b"data1")?;
        writer.add_stored("file2.txt", b"data2")?;
        writer.finalize()?;

        // 测试批量处理
        let file = File::open("test_batch.ysf")?;
        let mut reader = ZipDocumentReader::new(file)?;

        let mut processed_files = Vec::new();

        reader.process_files_smart(
            &["file1.txt", "file2.txt"],
            |filename, data| {
                processed_files.push((filename.to_string(), data.to_vec()));
                Ok(())
            },
        )?;

        assert_eq!(processed_files.len(), 2);
        assert_eq!(processed_files[0].1, b"data1");
        assert_eq!(processed_files[1].1, b"data2");

        // 清理
        let _ = std::fs::remove_file("test_batch.ysf");
        Ok(())
    }
}
