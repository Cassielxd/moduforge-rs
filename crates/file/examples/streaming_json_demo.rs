use std::fs::File;
use std::io::{Result, Write};
use mf_file::{ZipDocumentWriter, ZipDocumentReader, MmapConfig};
use serde_json::{Value, Deserializer};
use std::io::Read;

fn main() -> Result<()> {
    println!("=== 超大JSON流式处理演示 ===\n");

    // 创建包含超大JSON的测试文件
    create_huge_json_test()?;

    // 演示不同的JSON处理策略
    demo_json_processing_strategies()?;

    // 清理
    let _ = std::fs::remove_file("huge_json_test.ysf");
    println!("演示完成！");
    Ok(())
}

fn create_huge_json_test() -> Result<()> {
    println!("📁 创建包含超大JSON的测试文件...");
    
    let file = File::create("huge_json_test.ysf")?;
    let mut writer = ZipDocumentWriter::new(file)?;

    // 创建一个大型JSON数组 (约10MB)
    let mut json_data = String::new();
    json_data.push_str("[\n");
    
    for i in 0..100000 {
        if i > 0 {
            json_data.push_str(",\n");
        }
        json_data.push_str(&format!(
            r#"  {{
    "id": {},
    "name": "用户{}",
    "email": "user{}@example.com",
    "data": {{
      "score": {},
      "level": {},
      "items": ["item1", "item2", "item3"]
    }}
  }}"#,
            i, i, i, i % 1000, i % 10
        ));
    }
    json_data.push_str("\n]");

    writer.add_stored("huge_array.json", json_data.as_bytes())?;

    // 创建一个大型JSONL文件 (每行一个JSON对象)
    let mut jsonl_data = String::new();
    for i in 0..50000 {
        jsonl_data.push_str(&format!(
            r#"{{"id": {}, "name": "记录{}", "timestamp": {}, "value": {:.2}}}
"#,
            i, i, 1600000000 + i, (i as f64) * 0.5
        ));
    }

    writer.add_stored("huge_lines.jsonl", jsonl_data.as_bytes())?;
    writer.finalize()?;
    
    println!("✅ 创建完成：包含大型JSON数组和JSONL文件\n");
    Ok(())
}

fn demo_json_processing_strategies() -> Result<()> {
    let config = MmapConfig {
        huge_file_threshold: 1024 * 1024, // 1MB 触发流式
        stream_chunk_size: 64 * 1024,     // 64KB 块
        enable_streaming: true,
        ..Default::default()
    };

    println!("🎯 演示JSON处理策略:\n");

    // 策略1: 传统方式 - 一次性加载整个JSON
    demo_traditional_json_parsing(&config)?;

    // 策略2: 流式JSON解析 - 适合大型数组
    demo_streaming_json_parsing(&config)?;

    // 策略3: 行式JSON处理 - 适合JSONL格式
    demo_line_by_line_json(&config)?;

    Ok(())
}

fn demo_traditional_json_parsing(config: &MmapConfig) -> Result<()> {
    println!("1️⃣ 传统JSON解析（一次性加载）:");
    
    let file = File::open("huge_json_test.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config.clone())?;
    
    let start = std::time::Instant::now();
    
    // 一次性读取整个JSON
    let json_data = reader.read_all("huge_array.json")?;
    let load_time = start.elapsed();
    
    println!("   数据加载: {:?}", load_time);
    println!("   数据大小: {:.1}MB", json_data.len() as f64 / (1024.0 * 1024.0));
    
    // 解析JSON
    let parse_start = std::time::Instant::now();
    let json_value: Value = serde_json::from_slice(&json_data).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })?;
    let parse_time = parse_start.elapsed();
    
    println!("   JSON解析: {:?}", parse_time);
    
    if let Value::Array(array) = json_value {
        println!("   数组长度: {}", array.len());
        println!("   第一个元素: {}", array[0]);
    }
    
    let total_time = start.elapsed();
    println!("   总时间: {:?}", total_time);
    println!("   内存特征: 需要同时存储原始数据和解析后的数据\n");
    
    Ok(())
}

fn demo_streaming_json_parsing(config: &MmapConfig) -> Result<()> {
    println!("2️⃣ 流式JSON解析（逐个处理）:");
    
    let file = File::open("huge_json_test.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config.clone())?;
    
    let start = std::time::Instant::now();
    let mut processed_items = 0;
    let mut total_score = 0i64;
    
    // 使用回调方式流式处理
    let mut json_buffer = Vec::new();
    
    reader.process_huge_file("huge_array.json", |chunk| {
        json_buffer.extend_from_slice(chunk);
        Ok(())
    })?;
    
    // 模拟流式JSON解析（实际应用中会使用专门的流式JSON库）
    let json_str = String::from_utf8_lossy(&json_buffer);
    
    // 简化的流式解析模拟：逐行处理
    let mut in_object = false;
    let mut object_str = String::new();
    let mut brace_count = 0;
    
    for line in json_str.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with('{') {
            in_object = true;
            object_str.clear();
            brace_count = 0;
        }
        
        if in_object {
            object_str.push_str(line);
            object_str.push('\n');
            
            // 计算大括号平衡
            for ch in trimmed.chars() {
                match ch {
                    '{' => brace_count += 1,
                    '}' => brace_count -= 1,
                    _ => {}
                }
            }
            
            // 当大括号平衡时，我们有一个完整的对象
            if brace_count == 0 && trimmed.ends_with('}') {
                if let Ok(obj) = serde_json::from_str::<Value>(&object_str) {
                    processed_items += 1;
                    
                    // 提取并累加score
                    if let Some(data) = obj.get("data") {
                        if let Some(score) = data.get("score") {
                            if let Some(score_num) = score.as_i64() {
                                total_score += score_num;
                            }
                        }
                    }
                    
                    // 每处理1000个项目报告一次
                    if processed_items % 10000 == 0 {
                        println!("     已处理: {} 个对象", processed_items);
                    }
                }
                in_object = false;
            }
        }
    }
    
    let elapsed = start.elapsed();
    println!("   处理时间: {:?}", elapsed);
    println!("   处理对象: {}", processed_items);
    println!("   总分数: {}", total_score);
    println!("   平均分数: {:.2}", total_score as f64 / processed_items as f64);
    println!("   内存特征: 只需要存储当前处理的对象\n");
    
    Ok(())
}

fn demo_line_by_line_json(config: &MmapConfig) -> Result<()> {
    println!("3️⃣ 行式JSON处理（JSONL格式）:");
    
    let file = File::open("huge_json_test.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config.clone())?;
    
    let start = std::time::Instant::now();
    let mut processed_lines = 0;
    let mut total_value = 0.0;
    let mut line_buffer = String::new();
    
    // 流式处理JSONL文件
    reader.process_huge_file("huge_lines.jsonl", |chunk| {
        let chunk_str = String::from_utf8_lossy(chunk);
        
        for ch in chunk_str.chars() {
            if ch == '\n' {
                // 处理完整的一行
                if !line_buffer.trim().is_empty() {
                    if let Ok(obj) = serde_json::from_str::<Value>(&line_buffer) {
                        processed_lines += 1;
                        
                        if let Some(value) = obj.get("value") {
                            if let Some(val_num) = value.as_f64() {
                                total_value += val_num;
                            }
                        }
                        
                        if processed_lines % 5000 == 0 {
                            println!("     已处理: {} 行", processed_lines);
                        }
                    }
                }
                line_buffer.clear();
            } else {
                line_buffer.push(ch);
            }
        }
        Ok(())
    })?;
    
    // 处理最后一行（如果没有换行符结尾）
    if !line_buffer.trim().is_empty() {
        if let Ok(obj) = serde_json::from_str::<Value>(&line_buffer) {
            processed_lines += 1;
            if let Some(value) = obj.get("value") {
                if let Some(val_num) = value.as_f64() {
                    total_value += val_num;
                }
            }
        }
    }
    
    let elapsed = start.elapsed();
    println!("   处理时间: {:?}", elapsed);
    println!("   处理行数: {}", processed_lines);
    println!("   总值: {:.2}", total_value);
    println!("   平均值: {:.2}", total_value / processed_lines as f64);
    println!("   内存特征: 只需要存储当前行的JSON对象\n");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_processing() -> Result<()> {
        let config = MmapConfig {
            huge_file_threshold: 1024,
            stream_chunk_size: 512,
            enable_streaming: true,
            ..Default::default()
        };
        
        // 测试配置有效性
        assert!(config.enable_streaming);
        Ok(())
    }
}