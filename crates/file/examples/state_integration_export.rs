// 展示与State模块集成的完整导出示例
// 注意：此示例需要 mf-state 依赖才能完全运行

use std::fs::File;
use std::collections::HashMap;
use tempfile::tempdir;
use mf_file::{ZipDocumentWriter, ZipDocumentReader};

// 模拟 State 序列化结构（实际使用时导入 mf_state::StateSerialize）
#[derive(Debug)]
struct MockStateSerialize {
    state_fields: Vec<u8>,  // 插件状态字段的序列化数据
    node_pool: Vec<u8>,     // 文档节点池的序列化数据
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("state_integrated_export.zip");

    println!("创建模拟状态数据...");

    // 模拟状态序列化数据
    let mock_state = MockStateSerialize {
        state_fields: serde_json::to_vec(&serde_json::json!({
            "editor_config": "编辑器配置状态",
            "collaboration": "协作状态数据",
            "theme_manager": "主题管理状态"
        }))?,
        node_pool: serde_json::to_vec(&serde_json::json!({
            "nodes": [
                {"id": "1", "type": "doc", "content": []},
                {"id": "2", "type": "paragraph", "content": ["Hello World"]}
            ]
        }))?,
    };

    // 解析插件状态字段
    let plugin_states: HashMap<String, serde_json::Value> = 
        serde_json::from_slice(&mock_state.state_fields)?;

    println!("写入完整文档状态...");
    {
        let file = File::create(&file_path)?;
        let mut writer = ZipDocumentWriter::new(file)?;

        // 添加文档节点池
        writer.add_deflated("document.bin", &mock_state.node_pool)?;

        // 为每个插件创建单独的状态文件
        for (plugin_name, plugin_data) in plugin_states {
            let serialized_data = serde_json::to_vec(&plugin_data)?;
            writer.add_plugin_state(&plugin_name, &serialized_data)?;
        }

        // 添加元数据
        writer.add_json("metadata.json", &serde_json::json!({
            "export_version": "1.0",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "plugin_count": 3,
            "document_format": "binary"
        }))?;

        writer.finalize()?;
    }

    println!("文档状态写入完成！");

    println!("\n读取并重构状态...");
    {
        let file = File::open(&file_path)?;
        let mut reader = ZipDocumentReader::new(file)?;

        // 读取文档数据
        let document_data = reader.read_all("document.bin")?;
        let document: serde_json::Value = serde_json::from_slice(&document_data)?;
        println!("文档节点数: {}", document["nodes"].as_array().unwrap().len());

        // 读取所有插件状态
        let all_plugin_states = reader.read_all_plugin_states()?;
        println!("恢复的插件状态:");

        // 重构插件状态数据
        let mut reconstructed_state_fields = HashMap::new();
        for (plugin_name, state_data) in all_plugin_states {
            let plugin_state: serde_json::Value = serde_json::from_slice(&state_data)?;
            reconstructed_state_fields.insert(plugin_name.clone(), plugin_state);
            println!("  {}: 已恢复", plugin_name);
        }

        // 模拟重构 StateSerialize
        let reconstructed_state = MockStateSerialize {
            state_fields: serde_json::to_vec(&reconstructed_state_fields)?,
            node_pool: document_data,
        };

        println!("\n状态重构完成:");
        println!("  插件状态大小: {} bytes", reconstructed_state.state_fields.len());
        println!("  文档数据大小: {} bytes", reconstructed_state.node_pool.len());

        // 读取元数据
        let metadata_data = reader.read_all("metadata.json")?;
        let metadata: serde_json::Value = serde_json::from_slice(&metadata_data)?;
        println!("  导出版本: {}", metadata["export_version"]);
        println!("  导出时间: {}", metadata["timestamp"]);
    }

    println!("\n状态集成导出示例完成！");

    // 实际使用时的伪代码注释
    println!("\n// 实际使用示例:");
    println!("// let state_serialize = state.serialize().await?;");
    println!("// let plugin_states: HashMap<String, Vec<u8>> = parse_plugin_states(&state_serialize.state_fields);");
    println!("// writer.add_deflated(\"document.bin\", &state_serialize.node_pool)?;");
    println!("// writer.add_plugin_states(plugin_states)?;");
    
    Ok(())
}