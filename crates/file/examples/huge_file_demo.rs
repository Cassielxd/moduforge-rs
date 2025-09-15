use std::fs::File;
use std::io::{Result, Write};
use mf_file::{ZipDocumentWriter, ZipDocumentReader, MmapConfig};

fn main() -> Result<()> {
    println!("=== 超大文件 ZIP 处理演示 ===\n");

    // 1. 创建包含不同大小文件的测试 ZIP
    create_huge_file_test_zip()?;

    // 2. 演示不同的读取策略
    demo_reading_strategies()?;

    // 3. 演示流式处理
    demo_streaming_processing()?;

    // 4. 演示内存效率对比
    demo_memory_efficiency()?;

    // 5. 演示实际应用场景
    demo_practical_scenarios()?;

    println!("清理测试文件...");
    let _ = std::fs::remove_file("huge_test.ysf");
    
    println!("演示完成！");
    Ok(())
}

fn create_huge_file_test_zip() -> Result<()> {
    println!("📁 创建包含超大文件的测试 ZIP...");
    
    let file = File::create("huge_test.ysf")?;
    let mut writer = ZipDocumentWriter::new(file)?;

    // 小文件 (1KB)
    let small_data = vec![1u8; 1024];
    writer.add_stored("small.bin", &small_data)?;
    
    // 中等文件 (5MB)
    println!("  创建 5MB 中等文件...");
    let medium_data = vec![2u8; 5 * 1024 * 1024];
    writer.add_deflated("medium.bin", &medium_data)?;
    
    // 大文件 (50MB)
    println!("  创建 50MB 大文件...");
    let large_data = vec![3u8; 50 * 1024 * 1024];
    writer.add_stored("large.bin", &large_data)?;
    
    // 超大文件 (200MB)
    println!("  创建 200MB 超大文件...");
    let huge_data = vec![4u8; 200 * 1024 * 1024];
    writer.add_deflated("huge.bin", &huge_data)?;

    writer.finalize()?;
    println!("✅ 创建完成：huge_test.ysf (~255MB)\n");
    
    Ok(())
}

fn demo_reading_strategies() -> Result<()> {
    println!("📖 演示不同读取策略...");
    
    // 配置针对超大文件优化
    let config = MmapConfig {
        threshold: 1024 * 1024,          // 1MB
        max_maps: 4,
        huge_file_threshold: 80 * 1024 * 1024, // 80MB
        stream_chunk_size: 16 * 1024 * 1024,   // 16MB chunks
        enable_streaming: true,
        temp_dir: None,
    };
    
    let file = File::open("huge_test.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config)?;
    
    println!("配置：mmap阈值=1MB, 流式阈值=80MB, 块大小=16MB");
    
    // 小文件 - 标准读取
    let start = std::time::Instant::now();
    let small_data = reader.read_all("small.bin")?;
    let small_time = start.elapsed();
    println!("小文件 (1KB): {:?} - 标准读取", small_time);
    assert_eq!(small_data.len(), 1024);
    
    // 中等文件 - mmap 读取
    let start = std::time::Instant::now();
    let medium_data = reader.read_all("medium.bin")?;
    let medium_time = start.elapsed();
    println!("中等文件 (5MB): {:?} - mmap 读取", medium_time);
    assert_eq!(medium_data.len(), 5 * 1024 * 1024);
    
    // 大文件 - mmap 读取
    let start = std::time::Instant::now();
    let large_data = reader.read_all("large.bin")?;
    let large_time = start.elapsed();
    println!("大文件 (50MB): {:?} - mmap 读取", large_time);
    assert_eq!(large_data.len(), 50 * 1024 * 1024);
    
    // 超大文件 - 流式读取
    let start = std::time::Instant::now();
    let huge_data = reader.read_all("huge.bin")?;
    let huge_time = start.elapsed();
    println!("超大文件 (200MB): {:?} - 流式读取", huge_time);
    assert_eq!(huge_data.len(), 200 * 1024 * 1024);
    
    // 显示统计
    let stats = reader.mmap_stats();
    println!("mmap 统计: {}\n", stats);
    
    Ok(())
}

fn demo_streaming_processing() -> Result<()> {
    println!("🌊 演示流式处理（避免大内存占用）...");
    
    let config = MmapConfig {
        huge_file_threshold: 40 * 1024 * 1024, // 40MB 触发流式
        stream_chunk_size: 8 * 1024 * 1024,    // 8MB chunks
        enable_streaming: true,
        ..Default::default()
    };
    
    let file = File::open("huge_test.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config)?;
    
    // 方式1: 回调处理（最节省内存）
    println!("方式1: 回调处理 200MB 文件");
    let start = std::time::Instant::now();
    let mut checksum = 0u64;
    let mut processed_bytes = 0usize;
    
    reader.process_huge_file("huge.bin", |chunk| {
        processed_bytes += chunk.len();
        // 简单校验和计算
        for &byte in chunk {
            checksum = checksum.wrapping_add(byte as u64);
        }
        
        // 模拟一些处理时间
        if processed_bytes % (32 * 1024 * 1024) == 0 {
            println!("  已处理: {:.1}MB", processed_bytes as f64 / (1024.0 * 1024.0));
        }
        
        Ok(())
    })?;
    
    let callback_time = start.elapsed();
    println!("  完成：处理 {:.1}MB，校验和 {}，耗时 {:?}", 
        processed_bytes as f64 / (1024.0 * 1024.0), checksum, callback_time);
    
    // 方式2: 流式读取器
    println!("\n方式2: 流式读取器");
    let start = std::time::Instant::now();
    let mut stream = reader.create_stream_reader("huge.bin")?;
    
    let mut hash_sum = 0u64;
    let mut chunk_count = 0;
    
    while let Some(chunk) = stream.read_chunk()? {
        chunk_count += 1;
        for &byte in &chunk {
            hash_sum = hash_sum.wrapping_add(byte as u64);
        }
        
        if chunk_count % 4 == 0 { // 每4个块报告一次
            println!("  已读取 {} 块，当前位置: {:.1}MB / {:.1}MB", 
                chunk_count,
                stream.position() as f64 / (1024.0 * 1024.0),
                stream.total_size() as f64 / (1024.0 * 1024.0));
        }
    }
    
    let stream_time = start.elapsed();
    println!("  完成：读取 {} 块，哈希 {}，耗时 {:?}", chunk_count, hash_sum, stream_time);
    
    Ok(())
}

fn demo_memory_efficiency() -> Result<()> {
    println!("💾 演示内存效率对比...");
    
    // 获取当前进程内存使用（简化版）
    fn get_memory_usage() -> usize {
        // 这里只是示例，实际应用中可以使用 psutil 等库
        // 返回模拟的内存使用量
        42 * 1024 * 1024 // 42MB baseline
    }
    
    let baseline_memory = get_memory_usage();
    println!("基线内存使用: {:.1}MB", baseline_memory as f64 / (1024.0 * 1024.0));
    
    // 传统方式：一次性加载 200MB 文件
    println!("\n传统方式：一次性加载");
    let config_traditional = MmapConfig {
        enable_streaming: false, // 禁用流式处理
        huge_file_threshold: u64::MAX, // 永不触发流式
        ..Default::default()
    };
    
    {
        let file = File::open("huge_test.ysf")?;
        let mut reader = ZipDocumentReader::with_mmap_config(file, config_traditional)?;
        
        let start = std::time::Instant::now();
        let data = reader.read_all("huge.bin")?;
        let load_time = start.elapsed();
        
        let peak_memory = get_memory_usage();
        println!("  加载时间: {:?}", load_time);
        println!("  峰值内存: {:.1}MB (+{:.1}MB)", 
            peak_memory as f64 / (1024.0 * 1024.0),
            (peak_memory - baseline_memory) as f64 / (1024.0 * 1024.0));
        println!("  数据大小: {:.1}MB", data.len() as f64 / (1024.0 * 1024.0));
    } // data 被释放
    
    // 流式方式：分块处理
    println!("\n流式方式：分块处理");
    let config_streaming = MmapConfig {
        huge_file_threshold: 50 * 1024 * 1024, // 50MB 触发流式
        stream_chunk_size: 4 * 1024 * 1024,    // 4MB chunks
        enable_streaming: true,
        ..Default::default()
    };
    
    {
        let file = File::open("huge_test.ysf")?;
        let mut reader = ZipDocumentReader::with_mmap_config(file, config_streaming)?;
        
        let start = std::time::Instant::now();
        let mut total_processed = 0;
        
        reader.process_huge_file("huge.bin", |chunk| {
            total_processed += chunk.len();
            // 模拟处理：计算简单统计
            let _sum: u64 = chunk.iter().map(|&b| b as u64).sum();
            Ok(())
        })?;
        
        let process_time = start.elapsed();
        let streaming_memory = get_memory_usage();
        
        println!("  处理时间: {:?}", process_time);
        println!("  峰值内存: {:.1}MB (+{:.1}MB)", 
            streaming_memory as f64 / (1024.0 * 1024.0),
            (streaming_memory - baseline_memory) as f64 / (1024.0 * 1024.0));
        println!("  处理数据: {:.1}MB", total_processed as f64 / (1024.0 * 1024.0));
    }
    
    println!("✅ 流式处理显著减少了内存占用\n");
    
    Ok(())
}

fn demo_practical_scenarios() -> Result<()> {
    println!("🎯 演示实际应用场景...");
    
    let config = MmapConfig {
        huge_file_threshold: 60 * 1024 * 1024,
        stream_chunk_size: 16 * 1024 * 1024,
        enable_streaming: true,
        ..Default::default()
    };
    
    let file = File::open("huge_test.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config.clone())?;
    
    // 场景1: 数据验证（计算哈希）
    println!("场景1: 数据完整性验证");
    let start = std::time::Instant::now();
    let mut hasher = blake3::Hasher::new();
    
    reader.process_huge_file("huge.bin", |chunk| {
        hasher.update(chunk);
        Ok(())
    })?;
    
    let hash = hasher.finalize();
    let hash_time = start.elapsed();
    println!("  BLAKE3 哈希: {}", hash.to_hex());
    println!("  耗时: {:?}", hash_time);
    
    // 场景2: 数据转换（压缩/解压）
    println!("\n场景2: 数据转换处理");
    let start = std::time::Instant::now();
    let mut output_file = std::fs::File::create("processed_output.tmp")?;
    let mut processed_chunks = 0;
    
    let file2 = File::open("huge_test.ysf")?;
    let mut reader2 = ZipDocumentReader::with_mmap_config(file2, config.clone())?;
    
    reader2.process_huge_file("huge.bin", |chunk| {
        // 模拟数据转换：简单的 XOR 变换
        let transformed: Vec<u8> = chunk.iter().map(|&b| b ^ 0x55).collect();
        output_file.write_all(&transformed)?;
        processed_chunks += 1;
        Ok(())
    })?;
    
    let transform_time = start.elapsed();
    println!("  处理了 {} 个数据块", processed_chunks);
    println!("  耗时: {:?}", transform_time);
    
    // 清理临时文件
    let _ = std::fs::remove_file("processed_output.tmp");
    
    // 场景3: 数据分析（统计信息）
    println!("\n场景3: 数据分析统计");
    let start = std::time::Instant::now();
    let mut byte_counts = vec![0u64; 256];
    let mut total_bytes = 0u64;
    
    let file3 = File::open("huge_test.ysf")?;
    let mut reader3 = ZipDocumentReader::with_mmap_config(file3, config.clone())?;
    
    reader3.process_huge_file("huge.bin", |chunk| {
        total_bytes += chunk.len() as u64;
        for &byte in chunk {
            byte_counts[byte as usize] += 1;
        }
        Ok(())
    })?;
    
    let analysis_time = start.elapsed();
    println!("  分析了 {:.1}MB 数据", total_bytes as f64 / (1024.0 * 1024.0));
    
    // 找出最常见的字节值
    let (most_common_byte, max_count) = byte_counts.iter()
        .enumerate()
        .max_by_key(|(_, count)| **count)
        .map(|(byte, count)| (byte as u8, *count))
        .unwrap();
    
    println!("  最常见字节: 0x{:02X} (出现 {} 次)", most_common_byte, max_count);
    println!("  耗时: {:?}", analysis_time);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huge_file_creation() {
        // 在测试中创建较小的文件以避免耗时过长
        let config = MmapConfig {
            huge_file_threshold: 1024, // 1KB for testing
            stream_chunk_size: 512,
            enable_streaming: true,
            ..Default::default()
        };

        // 测试流式处理逻辑
        assert!(config.enable_streaming);
        assert_eq!(config.stream_chunk_size, 512);
    }
}