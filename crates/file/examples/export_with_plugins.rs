use std::fs::File;
use std::io::Cursor;
use std::collections::HashMap;
use std::path::Path;
use mf_file::{ZipDocumentWriter, ZipDocumentReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = Path::new("plugin_export_examples");
    std::fs::create_dir_all(output_dir)?;
    let file_path = output_dir.join("document_with_plugins.zip");

    // 模拟插件状态数据
    let mut plugin_states: HashMap<String, Vec<u8>> = HashMap::new();
    plugin_states.insert("editor_config".to_string(), b"editor config state data".to_vec());
    plugin_states.insert("collaboration".to_string(), b"collaboration state data".to_vec());
    plugin_states.insert("version_control".to_string(), b"version control state data".to_vec());

    // 写入文档和插件状态
    println!("正在写入文档和插件状态...");
    {
        let file = File::create(&file_path)?;
        let mut writer = ZipDocumentWriter::new(file)?;

        // 添加主文档内容
        writer.add_json("document.json", &serde_json::json!({
            "title": "测试文档",
            "content": "这是一个包含插件状态的文档示例",
            "version": "1.0"
        }))?;

        // 添加文档结构定义
        writer.add_deflated("schema.bin", b"binary schema data")?;

        // 批量添加插件状态
        writer.add_plugin_states(plugin_states.clone())?;

        // 添加一些额外的插件
        writer.add_plugin_state("theme_manager", b"theme manager state")?;
        writer.add_plugin_state("search_index", b"search index data")?;

        writer.finalize()?;
    }

    println!("文档写入完成！");

    // 读取并验证文档和插件状态
    println!("\n正在读取文档和插件状态...");
    {
        let file = File::open(&file_path)?;
        let mut reader = ZipDocumentReader::new(file)?;

        // 读取主文档
        let doc_data = reader.read_all("document.json")?;
        let doc: serde_json::Value = serde_json::from_slice(&doc_data)?;
        println!("文档标题: {}", doc["title"]);

        // 列出所有插件
        let plugins = reader.list_plugins()?;
        println!("发现 {} 个插件:", plugins.len());
        for plugin in &plugins {
            println!("  - {}", plugin);
        }

        // 读取所有插件状态
        let all_plugin_states = reader.read_all_plugin_states()?;
        println!("\n插件状态详情:");
        for (name, data) in &all_plugin_states {
            println!("  {}: {} bytes", name, data.len());
        }

        // 读取特定插件状态
        if let Some(editor_state) = reader.read_plugin_state("editor_config")? {
            println!("\n编辑器配置状态: {}", String::from_utf8_lossy(&editor_state));
        }

        // 检查插件是否存在
        println!("\n插件存在性检查:");
        println!("  collaboration: {}", reader.has_plugin_state("collaboration"));
        println!("  non_existent: {}", reader.has_plugin_state("non_existent"));

        // 验证数据完整性
        let mut verified_count = 0;
        for (expected_name, expected_data) in &plugin_states {
            if let Some(actual_data) = reader.read_plugin_state(expected_name)? {
                if &actual_data == expected_data {
                    verified_count += 1;
                }
            }
        }
        
        println!("\n数据完整性验证: {}/{} 插件状态匹配", verified_count, plugin_states.len());
    }

    println!("\n✅ 文件已保存到: {:?}", file_path);
    println!("示例完成！");
    Ok(())
}