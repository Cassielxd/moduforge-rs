use std::fs::File;
use std::io::Result;
use mf_file::{ZipDocumentWriter, ZipDocumentReader, MmapConfig};

fn main() -> Result<()> {
    println!("=== 超大文件处理演示（简化版）===\n");

    // 创建测试文件
    create_test_zip()?;

    // 演示三种处理策略
    demo_processing_strategies()?;

    // 清理
    let _ = std::fs::remove_file("test_huge.ysf");
    println!("演示完成！");
    Ok(())
}

fn create_test_zip() -> Result<()> {
    println!("📁 创建测试文件...");
    
    let file = File::create("test_huge.ysf")?;
    let mut writer = ZipDocumentWriter::new(file)?;

    // 小文件 (10KB)
    let small_data = vec![1u8; 10 * 1024];
    writer.add_stored("small.bin", &small_data)?;
    
    // 中等文件 (5MB)
    let medium_data = vec![2u8; 5 * 1024 * 1024];
    writer.add_deflated("medium.bin", &medium_data)?;
    
    // 超大文件 (20MB) - 在演示中已经足够展示差异
    let huge_data = vec![3u8; 20 * 1024 * 1024];
    writer.add_stored("huge.bin", &huge_data)?;

    writer.finalize()?;
    println!("✅ 创建完成：test_huge.ysf (~25MB)\n");
    
    Ok(())
}

fn demo_processing_strategies() -> Result<()> {
    println!("🎯 演示三种处理策略:\n");

    // 策略1: 标准处理（小文件）
    demo_small_file_processing()?;
    
    // 策略2: mmap 处理（中等文件）
    demo_mmap_processing()?;
    
    // 策略3: 流式处理（超大文件）  
    demo_streaming_processing()?;

    Ok(())
}

fn demo_small_file_processing() -> Result<()> {
    println!("1️⃣ 小文件标准处理:");
    
    let file = File::open("test_huge.ysf")?;
    let mut reader = ZipDocumentReader::new(file)?;
    
    let start = std::time::Instant::now();
    let data = reader.read_all("small.bin")?;
    let time = start.elapsed();
    
    println!("   文件大小: {:.1}KB", data.len() as f64 / 1024.0);
    println!("   处理时间: {:?}", time);
    println!("   策略: 直接内存读取");
    
    let stats = reader.mmap_stats();
    println!("   mmap缓存: {} 条目\n", stats.cached_entries);
    
    Ok(())
}

fn demo_mmap_processing() -> Result<()> {
    println!("2️⃣ 中等文件 mmap 处理:");
    
    let file = File::open("test_huge.ysf")?;
    let mut reader = ZipDocumentReader::new(file)?;
    
    let start = std::time::Instant::now();
    let data = reader.read_all("medium.bin")?;
    let first_time = start.elapsed();
    
    // 再次读取测试缓存效果
    let start = std::time::Instant::now();
    let _data2 = reader.read_all("medium.bin")?;
    let cache_time = start.elapsed();
    
    println!("   文件大小: {:.1}MB", data.len() as f64 / (1024.0 * 1024.0));
    println!("   首次读取: {:?}", first_time);
    println!("   缓存读取: {:?}", cache_time);
    println!("   策略: 内存映射 + 临时文件");
    
    let speedup = first_time.as_nanos() as f64 / cache_time.as_nanos() as f64;
    println!("   缓存加速: {:.1}x", speedup);
    
    let stats = reader.mmap_stats();
    println!("   mmap缓存: {} 条目\n", stats.cached_entries);
    
    Ok(())
}

fn demo_streaming_processing() -> Result<()> {
    println!("3️⃣ 超大文件流式处理:");
    
    // 配置超大文件阈值为 15MB
    let config = MmapConfig {
        huge_file_threshold: 15 * 1024 * 1024, // 15MB
        stream_chunk_size: 4 * 1024 * 1024,    // 4MB chunks
        enable_streaming: true,
        ..Default::default()
    };
    
    let file = File::open("test_huge.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config.clone())?;
    
    // 方法1: 回调处理（最节省内存）
    println!("   方法1: 流式回调处理");
    let start = std::time::Instant::now();
    let mut processed_chunks = 0;
    let mut total_bytes = 0;
    
    reader.process_huge_file("huge.bin", |chunk| {
        processed_chunks += 1;
        total_bytes += chunk.len();
        
        // 模拟一些处理工作
        let _checksum: u32 = chunk.iter().map(|&b| b as u32).sum();
        
        if processed_chunks % 2 == 0 {
            println!("     已处理: {} 块 ({:.1}MB)", 
                processed_chunks, 
                total_bytes as f64 / (1024.0 * 1024.0));
        }
        
        Ok(())
    })?;
    
    let callback_time = start.elapsed();
    println!("   文件大小: {:.1}MB", total_bytes as f64 / (1024.0 * 1024.0));
    println!("   处理时间: {:?}", callback_time);
    println!("   处理块数: {}", processed_chunks);
    println!("   策略: 流式回调（低内存占用）");
    
    // 方法2: 创建流式读取器
    println!("\n   方法2: 流式读取器");
    let file2 = File::open("test_huge.ysf")?;
    let mut reader2 = ZipDocumentReader::with_mmap_config(file2, config)?;
    
    let start = std::time::Instant::now();
    let mut stream = reader2.create_stream_reader("huge.bin")?;
    
    let mut read_chunks = 0;
    while let Some(chunk) = stream.read_chunk()? {
        read_chunks += 1;
        // 模拟处理
        let _sum: usize = chunk.iter().map(|&b| b as usize).sum();
        
        if read_chunks % 2 == 0 {
            println!("     读取进度: {:.1}% ({:.1}MB / {:.1}MB)", 
                (stream.position() as f64 / stream.total_size() as f64) * 100.0,
                stream.position() as f64 / (1024.0 * 1024.0),
                stream.total_size() as f64 / (1024.0 * 1024.0));
        }
    }
    
    let stream_time = start.elapsed();
    println!("   流式读取: {:?}", stream_time);
    println!("   读取块数: {}", read_chunks);
    println!("   策略: 分块流式读取器\n");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_read() -> Result<()> {
        create_test_zip()?;
        
        let file = File::open("test_huge.ysf")?;
        let mut reader = ZipDocumentReader::new(file)?;
        
        // 验证能读取各种大小的文件
        let small = reader.read_all("small.bin")?;
        assert_eq!(small.len(), 10 * 1024);
        
        let medium = reader.read_all("medium.bin")?;
        assert_eq!(medium.len(), 5 * 1024 * 1024);
        
        let huge = reader.read_all("huge.bin")?;
        assert_eq!(huge.len(), 20 * 1024 * 1024);
        
        let _ = std::fs::remove_file("test_huge.ysf");
        Ok(())
    }
}