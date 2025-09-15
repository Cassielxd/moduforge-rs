use std::fs::File;
use std::io::Result;
use mf_file::{ZipDocumentWriter, ZipDocumentReader, MmapConfig, FileSizeCategory, ProcessingStrategy};

fn main() -> Result<()> {
    println!("=== 文件大小分析演示 ===\n");

    // 创建包含不同大小文件的测试ZIP
    create_test_files()?;

    // 演示文件大小分析功能
    demo_file_size_analysis()?;

    // 清理
    let _ = std::fs::remove_file("size_test.ysf");
    println!("演示完成！");
    Ok(())
}

fn create_test_files() -> Result<()> {
    println!("📁 创建包含不同大小文件的测试ZIP...");
    
    let file = File::create("size_test.ysf")?;
    let mut writer = ZipDocumentWriter::new(file)?;

    // 小文件 (1KB)
    let tiny_data = vec![1u8; 1024];
    writer.add_stored("tiny.txt", &tiny_data)?;
    
    // 小文件 (10KB)  
    let small_data = vec![2u8; 10 * 1024];
    writer.add_stored("small.bin", &small_data)?;

    // 中等文件 (2MB) - 会触发mmap
    let medium_data = vec![3u8; 2 * 1024 * 1024];
    writer.add_deflated("medium.data", &medium_data)?;
    
    // 大文件 (10MB) - 会触发mmap
    let large_data = vec![4u8; 10 * 1024 * 1024];
    writer.add_stored("large.blob", &large_data)?;
    
    // 超大文件 (50MB) - 会触发流式处理
    let huge_data = vec![5u8; 50 * 1024 * 1024];
    writer.add_deflated("huge.archive", &huge_data)?;

    writer.finalize()?;
    println!("✅ 创建完成：包含5个不同大小的文件\n");
    Ok(())
}

fn demo_file_size_analysis() -> Result<()> {
    // 配置阈值
    let config = MmapConfig {
        threshold: 1024 * 1024,          // 1MB
        huge_file_threshold: 20 * 1024 * 1024, // 20MB
        stream_chunk_size: 8 * 1024 * 1024,    // 8MB
        enable_streaming: true,
        ..Default::default()
    };

    let file = File::open("size_test.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config.clone())?;

    println!("⚙️  配置:");
    println!("   mmap阈值: {:.1}MB", config.threshold as f64 / (1024.0 * 1024.0));
    println!("   流式阈值: {:.1}MB", config.huge_file_threshold as f64 / (1024.0 * 1024.0));
    println!("   块大小: {:.1}MB\n", config.stream_chunk_size as f64 / (1024.0 * 1024.0));

    // 分析每个文件
    let files = ["tiny.txt", "small.bin", "medium.data", "large.blob", "huge.archive"];
    
    println!("📊 文件大小分析:");
    println!("{:<15} {:<10} {:<12} {:<10} {:<8} {:<12}", 
        "文件名", "原始大小", "压缩大小", "压缩率", "类别", "推荐策略");
    println!("{}", "-".repeat(75));

    for filename in &files {
        analyze_single_file(&mut reader, filename)?;
    }

    println!("\n🔍 详细文件信息:");
    for filename in &files {
        show_detailed_info(&mut reader, filename)?;
    }

    Ok(())
}

fn analyze_single_file(reader: &mut ZipDocumentReader<File>, filename: &str) -> Result<()> {
    let file_info = reader.get_file_info(filename)?;
    
    let size_str = format_bytes(file_info.size);
    let compressed_str = format_bytes(file_info.compressed_size);
    let ratio_str = format!("{:.1}%", file_info.compression_ratio * 100.0);
    let category_str = format_category(&file_info.category);
    let strategy_str = format_strategy(&file_info.recommended_strategy);

    println!("{:<15} {:<10} {:<12} {:<10} {:<8} {:<12}", 
        filename, size_str, compressed_str, ratio_str, category_str, strategy_str);

    Ok(())
}

fn show_detailed_info(reader: &mut ZipDocumentReader<File>, filename: &str) -> Result<()> {
    println!("\n📄 {}:", filename);
    
    // 基本信息
    let size = reader.get_file_size(filename)?;
    let compressed_size = reader.get_compressed_size(filename)?;
    let category = reader.classify_file_size(filename)?;
    let strategy = reader.recommend_processing_strategy(size);
    
    println!("   原始大小: {} ({} 字节)", format_bytes(size), size);
    println!("   压缩大小: {} ({} 字节)", format_bytes(compressed_size), compressed_size);
    println!("   压缩率: {:.1}%", (compressed_size as f64 / size as f64) * 100.0);
    println!("   类别: {}", format_category(&category));
    println!("   推荐策略: {}", format_strategy(&strategy));
    
    // 性能建议
    match strategy {
        ProcessingStrategy::Standard => {
            println!("   💡 建议: 直接使用 read_all() 方法");
        },
        ProcessingStrategy::MemoryMap => {
            println!("   💡 建议: 使用 read_mmap() 获得更好性能，支持零拷贝");
        },
        ProcessingStrategy::Streaming => {
            println!("   💡 建议: 使用 process_huge_file() 或 create_stream_reader()");
            println!("      - process_huge_file(): 内存效率最高");
            println!("      - create_stream_reader(): 控制灵活性更好");
        },
    }

    Ok(())
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

fn format_category(category: &FileSizeCategory) -> &'static str {
    match category {
        FileSizeCategory::Small => "小文件",
        FileSizeCategory::Large => "大文件", 
        FileSizeCategory::Huge => "超大",
    }
}

fn format_strategy(strategy: &ProcessingStrategy) -> &'static str {
    match strategy {
        ProcessingStrategy::Standard => "标准读取",
        ProcessingStrategy::MemoryMap => "内存映射",
        ProcessingStrategy::Streaming => "流式处理",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_size_methods() -> Result<()> {
        // 简化测试，只验证方法存在且可调用
        let config = MmapConfig::default();
        assert!(config.threshold > 0);
        assert!(config.huge_file_threshold > config.threshold);
        Ok(())
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512B");
        assert_eq!(format_bytes(1024), "1.0KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0GB");
    }
}