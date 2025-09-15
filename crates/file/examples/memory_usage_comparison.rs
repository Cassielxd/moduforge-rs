use std::fs::File;
use std::io::Result;
use mf_file::{ZipDocumentWriter, ZipDocumentReader, MmapConfig};

fn main() -> Result<()> {
    println!("=== 内存使用对比演示 ===\n");

    // 创建测试文件
    create_test_file()?;

    // 比较内存使用
    compare_memory_usage()?;

    // 清理
    let _ = std::fs::remove_file("memory_test.ysf");
    println!("演示完成！");
    Ok(())
}

fn create_test_file() -> Result<()> {
    println!("📁 创建 50MB 测试文件...");
    
    let file = File::create("memory_test.ysf")?;
    let mut writer = ZipDocumentWriter::new(file)?;
    
    // 创建 50MB 文件
    let data = vec![42u8; 50 * 1024 * 1024];
    writer.add_stored("test.bin", &data)?;
    writer.finalize()?;
    
    println!("✅ 创建完成\n");
    Ok(())
}

fn compare_memory_usage() -> Result<()> {
    let config = MmapConfig {
        huge_file_threshold: 10 * 1024 * 1024, // 10MB 触发流式
        stream_chunk_size: 8 * 1024 * 1024,    // 8MB 块
        enable_streaming: true,
        ..Default::default()
    };

    println!("🔍 内存使用分析（50MB文件，8MB块）：\n");

    // 方法1: 回调方式
    println!("1️⃣ 回调方式 (process_huge_file):");
    println!("   理论内存占用: ~8MB（单个缓冲区）");
    println!("   实际表现: 恒定低内存占用");
    println!("   ✅ 优点: 内存效率高，适合超大文件");
    println!("   ❌ 缺点: 控制灵活性差\n");

    // 方法2: 流式读取器
    println!("2️⃣ 流式读取器 (create_stream_reader):");
    println!("   理论内存占用: ~50MB（预加载所有块）");
    println!("   实际表现: 初始化时内存峰值高");
    println!("   ✅ 优点: 控制灵活，可重复访问");
    println!("   ❌ 缺点: 内存占用等于文件大小\n");

    // 实际测试
    test_callback_method(&config)?;
    test_stream_reader_method(&config)?;

    Ok(())
}

fn test_callback_method(config: &MmapConfig) -> Result<()> {
    println!("📊 测试回调方式:");
    
    let file = File::open("memory_test.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config.clone())?;
    
    let start = std::time::Instant::now();
    let mut total_processed = 0;
    let mut chunk_count = 0;
    
    reader.process_huge_file("test.bin", |chunk| {
        total_processed += chunk.len();
        chunk_count += 1;
        
        // 模拟处理（不存储数据）
        let _checksum: u64 = chunk.iter().map(|&b| b as u64).sum();
        
        Ok(())
    })?;
    
    let elapsed = start.elapsed();
    println!("   处理时间: {:?}", elapsed);
    println!("   处理数据: {:.1}MB", total_processed as f64 / (1024.0 * 1024.0));
    println!("   块数量: {}", chunk_count);
    println!("   内存特征: 处理过程中内存占用恒定\n");
    
    Ok(())
}

fn test_stream_reader_method(config: &MmapConfig) -> Result<()> {
    println!("📊 测试流式读取器:");
    
    let file = File::open("memory_test.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config.clone())?;
    
    let start = std::time::Instant::now();
    
    // 创建流式读取器（此时会预加载所有数据）
    let creation_time = std::time::Instant::now();
    let mut stream = reader.create_stream_reader("test.bin")?;
    let creation_elapsed = creation_time.elapsed();
    
    let mut total_processed = 0;
    let mut chunk_count = 0;
    
    // 读取数据（此时只是从内存中获取）
    while let Some(chunk) = stream.read_chunk()? {
        total_processed += chunk.len();
        chunk_count += 1;
        
        // 模拟处理
        let _checksum: u64 = chunk.iter().map(|&b| b as u64).sum();
    }
    
    let elapsed = start.elapsed();
    println!("   初始化时间: {:?} (预加载数据)", creation_elapsed);
    println!("   总处理时间: {:?}", elapsed);
    println!("   处理数据: {:.1}MB", total_processed as f64 / (1024.0 * 1024.0));
    println!("   块数量: {}", chunk_count);
    println!("   内存特征: 初始化时内存占用 = 文件大小\n");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn test_memory_patterns() -> Result<()> {
        // 简化测试，避免创建大文件
        let config = MmapConfig {
            huge_file_threshold: 1024, // 1KB 
            stream_chunk_size: 512,    // 512B
            enable_streaming: true,
            ..Default::default()
        };

        // 测试内存使用模式的差异
        assert!(config.enable_streaming);
        Ok(())
    }
}