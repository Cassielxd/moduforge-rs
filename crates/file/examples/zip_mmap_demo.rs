use std::fs::File;
use std::io::Result;
use mf_file::{ZipDocumentWriter, ZipDocumentReader, MmapConfig};

fn main() -> Result<()> {
    println!("=== ZIP memmap2 集成演示 ===\n");

    // 1. 创建包含不同大小文件的 ZIP 文档
    create_demo_zip()?;

    // 2. 使用默认配置读取
    demo_default_reading()?;

    // 3. 使用自定义 mmap 配置
    demo_custom_mmap_config()?;

    // 4. 性能对比演示
    demo_performance_comparison()?;

    // 5. 缓存管理演示
    demo_cache_management()?;

    println!("演示完成！");
    Ok(())
}

fn create_demo_zip() -> Result<()> {
    println!("📁 创建演示 ZIP 文件...");
    
    let file = File::create("demo.ysf")?;
    let mut writer = ZipDocumentWriter::new(file)?;

    // 添加小文件
    writer.add_stored("small.txt", "这是一个小文件的内容".as_bytes())?;
    
    // 添加中等文件 (500KB)
    let medium_data = vec![65u8; 500 * 1024]; // 'A' 字符
    writer.add_deflated("medium.bin", &medium_data)?;
    
    // 添加大文件 (5MB)
    let large_data = vec![66u8; 5 * 1024 * 1024]; // 'B' 字符
    writer.add_stored("large.bin", &large_data)?;
    
    // 添加超大文件 (10MB)
    let huge_data = vec![67u8; 10 * 1024 * 1024]; // 'C' 字符
    writer.add_deflated("huge.bin", &huge_data)?;

    // 添加插件状态
    writer.add_plugin_state("demo_plugin", "插件状态数据".as_bytes())?;

    writer.finalize()?;
    println!("✅ 创建了包含多种大小文件的 demo.ysf\n");
    
    Ok(())
}

fn demo_default_reading() -> Result<()> {
    println!("📖 使用默认配置读取文件...");
    
    let file = File::open("demo.ysf")?;
    let mut reader = ZipDocumentReader::new(file)?;
    
    // 读取小文件（不会使用 mmap）
    let small_data = reader.read_all("small.txt")?;
    println!("小文件内容: {}", String::from_utf8_lossy(&small_data));
    
    // 读取大文件（会自动使用 mmap）
    let large_data = reader.read_all("large.bin")?;
    println!("大文件大小: {:.2} MB", large_data.len() as f64 / (1024.0 * 1024.0));
    
    // 显示 mmap 统计
    let stats = reader.mmap_stats();
    println!("mmap 统计: {}\n", stats);
    
    Ok(())
}

fn demo_custom_mmap_config() -> Result<()> {
    println!("⚙️  使用自定义 mmap 配置...");
    
    let config = MmapConfig {
        threshold: 100 * 1024, // 100KB 阈值
        max_maps: 3,           // 最多3个映射
        temp_dir: None,
        huge_file_threshold: 50 * 1024 * 1024, // 50MB 触发流式处理
        stream_chunk_size: 8 * 1024 * 1024,    // 8MB 块大小
        enable_streaming: true, // 启用流式处理
    };
    
    let file = File::open("demo.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config)?;
    
    println!("配置: 阈值=100KB, 最大映射数=3");
    
    // 读取中等文件（现在会使用 mmap）
    let _medium_data = reader.read_all("medium.bin")?;
    println!("读取中等文件 (500KB) - 使用 mmap");
    
    let _large_data = reader.read_all("large.bin")?;
    println!("读取大文件 (5MB) - 使用 mmap");
    
    let _huge_data = reader.read_all("huge.bin")?;
    println!("读取超大文件 (10MB) - 使用 mmap");
    
    let stats = reader.mmap_stats();
    println!("当前缓存: {}\n", stats);
    
    // 再读取一个文件，应该触发缓存清理
    let _small_data = reader.read_all("small.txt")?;
    let stats_after = reader.mmap_stats();
    println!("缓存清理后: {}\n", stats_after);
    
    Ok(())
}

fn demo_performance_comparison() -> Result<()> {
    println!("🚀 性能对比演示...");
    
    let file = File::open("demo.ysf")?;
    let mut reader = ZipDocumentReader::new(file)?;
    
    // 使用标准读取
    let start = std::time::Instant::now();
    let _data1 = reader.read_standard("huge.bin")?;
    let standard_time = start.elapsed();
    
    // 使用 mmap 读取
    let start = std::time::Instant::now();
    let _data2 = reader.read_mmap("huge.bin")?;
    let mmap_time = start.elapsed();
    
    // 再次 mmap 读取（命中缓存）
    let start = std::time::Instant::now();
    let _data3 = reader.read_mmap("huge.bin")?;
    let cached_time = start.elapsed();
    
    println!("10MB 文件读取性能:");
    println!("  标准读取: {:?}", standard_time);
    println!("  mmap读取: {:?}", mmap_time);
    println!("  缓存命中: {:?}", cached_time);
    
    let speedup = standard_time.as_nanos() as f64 / cached_time.as_nanos() as f64;
    println!("  缓存加速比: {:.1}x\n", speedup);
    
    Ok(())
}

fn demo_cache_management() -> Result<()> {
    println!("🗂️  缓存管理演示...");
    
    let file = File::open("demo.ysf")?;
    let mut reader = ZipDocumentReader::new(file)?;
    
    // 预热缓存
    println!("预热缓存...");
    reader.preheat_mmap(&["large.bin", "huge.bin"])?;
    
    let stats = reader.mmap_stats();
    println!("预热后统计: {}", stats);
    
    // 读取已预热的文件（应该很快）
    let start = std::time::Instant::now();
    let _data = reader.read_mmap("large.bin")?;
    let preheated_time = start.elapsed();
    println!("预热文件读取时间: {:?}", preheated_time);
    
    // 清理缓存
    reader.clear_mmap_cache();
    let stats_after_clear = reader.mmap_stats();
    println!("清理后统计: {}", stats_after_clear);
    
    // 再次读取（需要重新建立 mmap）
    let start = std::time::Instant::now();
    let _data = reader.read_mmap("large.bin")?;
    let cold_time = start.elapsed();
    println!("冷启动读取时间: {:?}", cold_time);
    
    let cache_benefit = cold_time.as_nanos() as f64 / preheated_time.as_nanos() as f64;
    println!("缓存效益: {:.1}x 加速\n", cache_benefit);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_creation() {
        assert!(create_demo_zip().is_ok());
        assert!(std::path::Path::new("demo.ysf").exists());
        
        // 清理测试文件
        let _ = std::fs::remove_file("demo.ysf");
    }
}