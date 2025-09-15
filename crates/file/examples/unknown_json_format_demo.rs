use std::fs::File;
use std::io::Result;
use mf_file::{ZipDocumentWriter, ZipDocumentReader, MmapConfig};
use serde_json::Value;

fn main() -> Result<()> {
    println!("=== 未知JSON格式探索性解析演示 ===\n");

    // 创建包含不同格式JSON的测试文件
    create_mixed_json_formats()?;

    // 演示探索性解析策略
    demo_format_discovery()?;

    // 清理
    let _ = std::fs::remove_file("mixed_json_test.ysf");
    println!("演示完成！");
    Ok(())
}

fn create_mixed_json_formats() -> Result<()> {
    println!("📁 创建包含多种JSON格式的测试文件...");
    
    let file = File::create("mixed_json_test.ysf")?;
    let mut writer = ZipDocumentWriter::new(file)?;

    // 格式1: JSON对象
    let object_json = r#"{
  "type": "user_data",
  "users": [
    {"id": 1, "name": "张三", "age": 25},
    {"id": 2, "name": "李四", "age": 30}
  ],
  "metadata": {
    "version": "1.0",
    "created": "2024-01-01"
  }
}"#;
    writer.add_stored("object.json", object_json.as_bytes())?;

    // 格式2: JSON数组
    let array_json = r#"[
  {"event": "login", "user": "user1", "time": "2024-01-01T10:00:00Z"},
  {"event": "logout", "user": "user1", "time": "2024-01-01T11:00:00Z"},
  {"event": "login", "user": "user2", "time": "2024-01-01T10:30:00Z"}
]"#;
    writer.add_stored("array.json", array_json.as_bytes())?;

    // 格式3: JSONL (每行一个JSON)
    let jsonl_data = r#"{"level": "info", "message": "服务启动", "timestamp": "2024-01-01T09:00:00Z"}
{"level": "warn", "message": "内存使用率高", "timestamp": "2024-01-01T09:15:00Z"}
{"level": "error", "message": "数据库连接失败", "timestamp": "2024-01-01T09:30:00Z"}
{"level": "info", "message": "服务恢复", "timestamp": "2024-01-01T09:35:00Z"}"#;
    writer.add_stored("logs.jsonl", jsonl_data.as_bytes())?;

    // 格式4: 嵌套复杂结构
    let complex_json = r#"{
  "schema": "analytics_data",
  "data": {
    "daily_stats": [
      {
        "date": "2024-01-01",
        "metrics": {
          "pageviews": 1500,
          "users": 234,
          "sessions": 456
        },
        "breakdown": [
          {"source": "direct", "count": 800},
          {"source": "search", "count": 500},
          {"source": "social", "count": 200}
        ]
      }
    ]
  }
}"#;
    writer.add_stored("complex.json", complex_json.as_bytes())?;

    writer.finalize()?;
    println!("✅ 创建完成：包含4种不同JSON格式\n");
    Ok(())
}

fn demo_format_discovery() -> Result<()> {
    let config = MmapConfig {
        huge_file_threshold: 1024, // 很小的阈值，强制使用流式
        stream_chunk_size: 512,    // 小块读取用于探索
        enable_streaming: true,
        ..Default::default()
    };

    let file = File::open("mixed_json_test.ysf")?;
    let mut reader = ZipDocumentReader::with_mmap_config(file, config)?;

    // 分析每个文件的格式
    let files = ["object.json", "array.json", "logs.jsonl", "complex.json"];
    
    for filename in &files {
        println!("🔍 分析文件: {}", filename);
        analyze_json_format(&mut reader, filename)?;
        println!();
    }

    Ok(())
}

fn analyze_json_format(reader: &mut ZipDocumentReader<File>, filename: &str) -> Result<()> {
    // 策略1: 读取足够多的内容来判断格式（包括多行检测）
    let preview = peek_file_start(reader, filename, 500)?; // 增加到500字节
    
    let format_hint = detect_basic_format(&preview);
    println!("   基本格式检测: {}", format_hint);

    match format_hint.as_str() {
        "JSON对象" => handle_json_object(reader, filename)?,
        "JSON数组" => handle_json_array(reader, filename)?,
        "JSONL" => handle_jsonl_format(reader, filename)?,
        _ => handle_unknown_format(reader, filename)?,
    }

    Ok(())
}

fn peek_file_start(reader: &mut ZipDocumentReader<File>, filename: &str, bytes: usize) -> Result<String> {
    // 使用流式读取获取文件开头
    reader.process_huge_file(filename, |chunk| {
        let preview = String::from_utf8_lossy(&chunk[..bytes.min(chunk.len())]);
        println!("   预览: {:?}", preview.chars().take(50).collect::<String>());
        Ok(())
    })?;

    // 实际获取预览数据（简化实现）
    let full_data = reader.read_all(filename)?;
    let preview_bytes = &full_data[..bytes.min(full_data.len())];
    Ok(String::from_utf8_lossy(preview_bytes).to_string())
}

fn detect_basic_format(preview: &str) -> String {
    let trimmed = preview.trim();
    let lines: Vec<&str> = trimmed.lines().collect();
    
    // 先检查是否是JSONL格式（多行，每行都是JSON对象）
    if lines.len() > 1 && lines.iter().all(|line| {
        let line_trimmed = line.trim();
        !line_trimmed.is_empty() && line_trimmed.starts_with('{') && line_trimmed.ends_with('}')
    }) {
        return "JSONL".to_string();
    }
    
    // 然后检查单个JSON
    if trimmed.starts_with('{') {
        "JSON对象".to_string()
    } else if trimmed.starts_with('[') {
        "JSON数组".to_string()
    } else {
        "未知格式".to_string()
    }
}

fn handle_json_object(reader: &mut ZipDocumentReader<File>, filename: &str) -> Result<()> {
    println!("   处理策略: JSON对象 - 一次性解析");
    
    let data = reader.read_all(filename)?;
    let json: Value = serde_json::from_slice(&data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    
    // 分析结构
    analyze_json_structure(&json, "   ");
    
    Ok(())
}

fn handle_json_array(reader: &mut ZipDocumentReader<File>, filename: &str) -> Result<()> {
    println!("   处理策略: JSON数组 - 可考虑流式解析");
    
    let data = reader.read_all(filename)?;
    let json: Value = serde_json::from_slice(&data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    
    if let Value::Array(arr) = &json {
        println!("   数组长度: {}", arr.len());
        if !arr.is_empty() {
            println!("   数组元素类型:");
            analyze_json_structure(&arr[0], "     ");
        }
    }
    
    Ok(())
}

fn handle_jsonl_format(reader: &mut ZipDocumentReader<File>, filename: &str) -> Result<()> {
    println!("   处理策略: JSONL - 逐行流式解析");
    
    let mut line_count = 0;
    let mut line_buffer = String::new();
    
    reader.process_huge_file(filename, |chunk| {
        let chunk_str = String::from_utf8_lossy(chunk);
        
        for ch in chunk_str.chars() {
            if ch == '\n' {
                if !line_buffer.trim().is_empty() {
                    line_count += 1;
                    
                    // 分析第一行的结构
                    if line_count == 1 {
                        if let Ok(json) = serde_json::from_str::<Value>(&line_buffer) {
                            println!("   每行JSON结构:");
                            analyze_json_structure(&json, "     ");
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
    
    // 处理最后一行
    if !line_buffer.trim().is_empty() {
        line_count += 1;
    }
    
    println!("   总行数: {}", line_count);
    
    Ok(())
}

fn handle_unknown_format(reader: &mut ZipDocumentReader<File>, filename: &str) -> Result<()> {
    println!("   处理策略: 未知格式 - 尝试多种解析方式");
    
    let data = reader.read_all(filename)?;
    let content = String::from_utf8_lossy(&data);
    
    // 尝试不同的解析策略
    
    // 1. 尝试作为单个JSON解析
    if let Ok(json) = serde_json::from_str::<Value>(&content) {
        println!("   ✅ 成功解析为单个JSON");
        analyze_json_structure(&json, "   ");
        return Ok(());
    }
    
    // 2. 尝试作为JSONL解析
    let lines: Vec<&str> = content.lines().collect();
    let mut valid_json_lines = 0;
    
    for line in &lines {
        if serde_json::from_str::<Value>(line.trim()).is_ok() {
            valid_json_lines += 1;
        }
    }
    
    if valid_json_lines > 0 {
        println!("   ✅ 可能是JSONL格式，{}行有效JSON", valid_json_lines);
        return Ok(());
    }
    
    // 3. 尝试部分内容解析
    println!("   ⚠️  无法识别的JSON格式");
    println!("   内容预览: {:?}", content.chars().take(100).collect::<String>());
    
    Ok(())
}

fn analyze_json_structure(value: &Value, indent: &str) -> () {
    match value {
        Value::Object(obj) => {
            println!("{}对象，包含 {} 个字段:", indent, obj.len());
            for (key, val) in obj.iter().take(3) { // 只显示前3个字段
                print!("{}  {}: ", indent, key);
                match val {
                    Value::String(_) => println!("字符串"),
                    Value::Number(_) => println!("数字"),
                    Value::Bool(_) => println!("布尔值"),
                    Value::Array(arr) => println!("数组[{}个元素]", arr.len()),
                    Value::Object(sub_obj) => println!("对象{{{}个字段}}", sub_obj.len()),
                    Value::Null => println!("null"),
                }
            }
            if obj.len() > 3 {
                println!("{}  ... (还有{}个字段)", indent, obj.len() - 3);
            }
        },
        Value::Array(arr) => {
            println!("{}数组，包含 {} 个元素", indent, arr.len());
            if !arr.is_empty() {
                print!("{}  元素类型: ", indent);
                match &arr[0] {
                    Value::String(_) => println!("字符串"),
                    Value::Number(_) => println!("数字"),
                    Value::Bool(_) => println!("布尔值"),
                    Value::Array(_) => println!("嵌套数组"),
                    Value::Object(_) => println!("对象"),
                    Value::Null => println!("null"),
                }
            }
        },
        _ => {
            println!("{}基础类型: {:?}", indent, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(detect_basic_format("{ \"key\": \"value\" }"), "JSON对象");
        assert_eq!(detect_basic_format("[ {\"a\": 1}, {\"b\": 2} ]"), "JSON数组");
        assert_eq!(detect_basic_format("{\"line1\": true}\n{\"line2\": false}"), "JSONL");
    }
}