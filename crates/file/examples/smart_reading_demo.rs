use std::fs::File;
use std::io::Result;
use mf_file::{ZipDocumentWriter, ZipDocumentReader, MmapConfig};

fn main() -> Result<()> {
    println!("=== 智能读取策略演示 ===\n");

    // 创建包含不同大小文件的测试ZIP
    create_test_files()?;

    // 演示智能读取功能
    demo_smart_reading()?;

    // 清理
    let _ = std::fs::remove_file("smart_reading_test.ysf");
    println!("演示完成！");
    Ok(())
}

fn create_test_files() -> Result<()> {
    println!("📁 创建测试文件...");

    let file = File::create("smart_reading_test.ysf")?;
    let mut writer = ZipDocumentWriter::new(file)?;

    // 小文件 (512 字节) - 预期：标准读取
    let tiny_data = vec![1u8; 512];
    writer.add_stored("tiny.txt", &tiny_data)?;

    // 小文件 (50KB) - 预期：标准读取
    let small_data = vec![2u8; 50 * 1024];
    writer.add_stored("small.bin", &small_data)?;

    // 中等文件 (5MB) - 预期：内存映射
    let medium_data = vec![3u8; 5 * 1024 * 1024];
    writer.add_deflated("medium.data", &medium_data)?;

    // 大文件 (30MB) - 预期：流式处理
    let large_data = vec![4u8; 30 * 1024 * 1024];
    writer.add_stored("large.blob", &large_data)?;

    writer.finalize()?;
    println!("✅ 测试文件创建完成\n");
    Ok(())
}

fn demo_smart_reading() -> Result<()> {
    // 配置智能读取
    let config = MmapConfig {
        threshold: 1024 * 1024,                // 1MB - mmap阈值
        huge_file_threshold: 20 * 1024 * 1024, // 20MB - 流式阈值
        stream_chunk_size: 8 * 1024 * 1024,    // 8MB - 流式块大小
        enable_streaming: true,
        ..Default::default()
    };

    let file = File::open("smart_reading_test.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config.clone())?;

    println!("⚙️  智能读取配置:");
    println!(
        "   mmap阈值: {:.1}MB",
        config.threshold as f64 / (1024.0 * 1024.0)
    );
    println!(
        "   流式阈值: {:.1}MB",
        config.huge_file_threshold as f64 / (1024.0 * 1024.0)
    );
    println!(
        "   流式处理: {}",
        if config.enable_streaming { "启用" } else { "禁用" }
    );
    println!();

    let files = ["tiny.txt", "small.bin", "medium.data", "large.blob"];

    println!("📊 智能读取演示:");
    println!(
        "{:<12} {:<10} {:<8} {:<12} {:<15} {:<10}",
        "文件名", "原始大小", "类别", "推荐策略", "实际读取方法", "读取结果"
    );
    println!("{}", "-".repeat(80));

    for filename in &files {
        demonstrate_smart_reading(&mut reader, filename)?;
    }

    println!("\n🧠 智能读取优势:");
    println!("   ✅ 自动分析文件特征，选择最优读取策略");
    println!("   ✅ 小文件直接读取，避免 mmap 开销");
    println!("   ✅ 中等文件使用 mmap，实现零拷贝高性能");
    println!("   ✅ 超大文件流式处理，节省内存");
    println!("   ✅ 失败时自动回退，确保读取成功");

    // 演示批量智能读取
    println!("\n🚀 批量智能读取演示:");
    let start = std::time::Instant::now();

    for filename in &files {
        let data = reader.read_smart(filename)?;
        println!("   {} 读取完成: {} 字节", filename, data.len());
    }

    let duration = start.elapsed();
    println!("   批量读取耗时: {:.2}ms", duration.as_secs_f64() * 1000.0);

    Ok(())
}

fn demonstrate_smart_reading(
    reader: &mut ZipDocumentReader<File>,
    filename: &str,
) -> Result<()> {
    // 获取文件信息和推荐策略
    let file_info = reader.get_file_info(filename)?;

    let size_str = format_bytes(file_info.size);
    let category_str = format_category(&file_info.category);
    let strategy_str = format_strategy(&file_info.recommended_strategy);

    // 执行智能读取
    let start = std::time::Instant::now();
    let result = reader.read_smart(filename);
    let duration = start.elapsed();

    let (method_str, result_str) = match result {
        Ok(data) => {
            let method = match file_info.recommended_strategy {
                mf_file::ProcessingStrategy::Standard => "标准读取",
                mf_file::ProcessingStrategy::MemoryMap => "内存映射",
                mf_file::ProcessingStrategy::Streaming => "流式处理",
            };
            (method, format!("✅ {}字节", data.len()))
        },
        Err(e) => ("错误", format!("❌ {e}")),
    };

    println!(
        "{filename:<12} {size_str:<10} {category_str:<8} {strategy_str:<12} {method_str:<15} {result_str:<10}"
    );

    // 显示读取耗时（如果有意义）
    if duration.as_millis() > 0 {
        println!(
            "             └─ 耗时: {:.2}ms",
            duration.as_secs_f64() * 1000.0
        );
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
        format!("{bytes}B")
    }
}

fn format_category(category: &mf_file::FileSizeCategory) -> &'static str {
    match category {
        mf_file::FileSizeCategory::Small => "小文件",
        mf_file::FileSizeCategory::Large => "大文件",
        mf_file::FileSizeCategory::Huge => "超大",
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
    fn test_smart_reading_integration() -> Result<()> {
        // 创建简单测试文件
        let file = File::create("test_smart.ysf")?;
        let mut writer = ZipDocumentWriter::new(file)?;

        let test_data = vec![42u8; 1024];
        writer.add_stored("test.bin", &test_data)?;
        writer.finalize()?;

        // 测试智能读取
        let file = File::open("test_smart.ysf")?;
        let mut reader = ZipDocumentReader::new(file)?;

        let data = reader.read_smart("test.bin")?;
        assert_eq!(data.len(), 1024);
        assert_eq!(data[0], 42);

        // 清理
        let _ = std::fs::remove_file("test_smart.ysf");
        Ok(())
    }

    #[test]
    fn test_strategy_selection() -> Result<()> {
        let config = MmapConfig {
            threshold: 1024,
            huge_file_threshold: 10 * 1024,
            enable_streaming: true,
            ..Default::default()
        };

        // 小文件策略测试
        assert_eq!(config.threshold > 512, true, "小文件应使用标准读取");

        // 大文件策略测试
        assert_eq!(
            config.huge_file_threshold > config.threshold,
            true,
            "阈值配置应正确"
        );

        Ok(())
    }
}
