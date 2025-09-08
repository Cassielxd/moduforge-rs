// SnapshotFormat 插件状态导出演示 - 对比不同序列化格式的效果

use std::collections::HashMap;
use std::path::Path;
use serde::{Serialize, Deserialize};
use mf_file::{
    ZipDocumentWriter, ZipDocumentReader, SnapshotFormat,
    export_zip_with_format, import_zip_with_format
};
use mf_file::zipdoc::SnapshotShardMeta;

// 模拟文档节点数据
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockNode {
    id: String,
    node_type: String,
    content: String,
    attributes: HashMap<String, String>,
}

// 模拟分片数据
#[derive(Debug, Clone, Serialize, Deserialize)]  
struct MockShard {
    nodes: Vec<MockNode>,
    metadata: HashMap<String, String>,
}

fn create_mock_data() -> (Vec<MockShard>, HashMap<String, Vec<u8>>) {
    // 创建模拟文档数据
    let mut shards = Vec::new();
    for i in 0..3 {
        let mut nodes = Vec::new();
        for j in 0..5 {
            let mut attributes = HashMap::new();
            attributes.insert("level".to_string(), i.to_string());
            attributes.insert("index".to_string(), j.to_string());
            
            nodes.push(MockNode {
                id: format!("node_{}_{}", i, j),
                node_type: if j == 0 { "paragraph".to_string() } else { "text".to_string() },
                content: format!("这是第{}个分片的第{}个节点内容", i, j),
                attributes,
            });
        }
        
        let mut metadata = HashMap::new();
        metadata.insert("shard_id".to_string(), i.to_string());
        metadata.insert("created_at".to_string(), "2025-09-03".to_string());
        
        shards.push(MockShard { nodes, metadata });
    }

    // 创建模拟插件状态  
    let mut plugin_states = HashMap::new();
    plugin_states.insert(
        "editor_settings".to_string(),
        serde_json::to_vec(&serde_json::json!({
            "font_family": "JetBrains Mono",
            "font_size": 14,
            "theme": "monokai-pro",
            "word_wrap": true,
            "line_numbers": true
        })).unwrap()
    );
    
    plugin_states.insert(
        "vim_mode".to_string(),
        serde_json::to_vec(&serde_json::json!({
            "enabled": true,
            "leader_key": "<Space>",
            "relative_line_numbers": true,
            "key_bindings": {
                "jj": "<Esc>",
                "kk": "<Esc>"
            }
        })).unwrap()
    );
    
    plugin_states.insert(
        "collaboration".to_string(),
        serde_json::to_vec(&serde_json::json!({
            "user_id": "user_12345", 
            "room_id": "room_abcdef",
            "cursor_position": [42, 15],
            "active_selections": [
                {"start": [10, 0], "end": [10, 25]},
                {"start": [20, 5], "end": [20, 15]}
            ],
            "user_color": "#ff6b6b"
        })).unwrap()
    );

    plugin_states.insert(
        "git_integration".to_string(),
        b"branch=feature/plugin-export\nstatus=modified\nahead=2\nbehind=0\nmodified_files=3\n".to_vec()
    );

    (shards, plugin_states)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SnapshotFormat 插件状态导出演示 ===\n");
    
    let output_dir = Path::new("plugin_export_examples");
    std::fs::create_dir_all(output_dir)?;

    let (shards, plugin_states) = create_mock_data();
    
    println!("📦 模拟数据准备:");
    println!("  文档分片: {} 个", shards.len());
    println!("  插件状态: {} 个", plugin_states.len());
    for (name, data) in &plugin_states {
        println!("    {}: {} bytes", name, data.len());
    }
    println!();

    // 准备公共数据
    let meta_json = serde_json::json!({
        "title": "SnapshotFormat 插件演示文档",
        "version": "1.0.0",
        "author": "ModuForge-RS",
        "created_at": "2025-09-03T17:00:00Z",
        "plugin_count": plugin_states.len(),
        "document_nodes": shards.iter().map(|s| s.nodes.len()).sum::<usize>()
    });

    let schema_xml = b"<?xml version=\"1.0\"?><schema><node type=\"paragraph\"/><node type=\"text\"/></schema>";
    
    let shard_meta = SnapshotShardMeta {
        root_id: "root_node".to_string(),
        num_shards: shards.len(),
        counts: shards.iter().map(|s| s.nodes.len()).collect(),
    };

    // 测试不同格式
    let formats = vec![
        (SnapshotFormat::Json, "json"),
        (SnapshotFormat::Cbor, "cbor"),
        (SnapshotFormat::MsgPack, "msgpack"),
    ];

    let mut results = Vec::new();

    for (format, format_name) in formats {
        let file_path = output_dir.join(format!("snapshot_format_demo_{}.ysf", format_name));
        
        println!("🔄 测试 {} 格式...", format_name.to_uppercase());
        
        // 导出
        let start_time = std::time::Instant::now();
        export_zip_with_format(
            &file_path,
            &meta_json,
            schema_xml,
            &shard_meta,
            |i| Ok(shards[i].clone()),
            None::<&HashMap<String, String>>,  // 不使用 parent_map
            Some(plugin_states.clone()),
            1, // zstd压缩级别
            format,
        )?;
        let export_duration = start_time.elapsed();
        
        // 检查文件大小
        let file_size = std::fs::metadata(&file_path)?.len();
        
        // 导入验证
        let import_start = std::time::Instant::now();
        let (imported_meta, imported_schema, imported_shard_meta, imported_shards, _parent_map, imported_plugin_states): (
            serde_json::Value,
            Vec<u8>,
            SnapshotShardMeta,
            Vec<MockShard>,
            Option<HashMap<String, String>>,
            Option<HashMap<String, Vec<u8>>>,
        ) = import_zip_with_format(&file_path, format, false, true)?;
        let import_duration = import_start.elapsed();

        // 验证数据完整性
        let mut plugin_verified = 0;
        if let Some(imported_states) = imported_plugin_states {
            for (name, expected_data) in &plugin_states {
                if imported_states.get(name) == Some(expected_data) {
                    plugin_verified += 1;
                }
            }
        }

        let shard_verified = imported_shards.len() == shards.len() && 
            imported_shards.iter().zip(shards.iter()).all(|(a, b)| a.nodes.len() == b.nodes.len());

        results.push((
            format_name.to_uppercase().to_string(),
            file_size,
            export_duration,
            import_duration,
            plugin_verified,
            shard_verified
        ));

        println!("  ✅ 导出: {:?}", export_duration);
        println!("  ✅ 导入: {:?}", import_duration);  
        println!("  📁 文件大小: {} bytes", file_size);
        println!("  🔍 插件验证: {}/{}", plugin_verified, plugin_states.len());
        println!("  🔍 分片验证: {}", if shard_verified { "通过" } else { "失败" });
        println!();
    }

    // 性能对比总结
    println!("📊 格式对比总结:");
    println!("{:<10} {:<12} {:<12} {:<12} {:<12} {:<8}", "格式", "文件大小", "导出时间", "导入时间", "插件验证", "分片验证");
    println!("{}", "-".repeat(70));
    
    for (format_name, file_size, export_time, import_time, plugin_verified, shard_verified) in results {
        println!("{:<10} {:<12} {:<12} {:<12} {:<12} {:<8}",
            format_name,
            format!("{}B", file_size),
            format!("{:.2?}", export_time),
            format!("{:.2?}", import_time),
            format!("{}/{}", plugin_verified, plugin_states.len()),
            if shard_verified { "✅" } else { "❌" }
        );
    }

    println!("\n📂 生成的文件:");
    for entry in std::fs::read_dir(output_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let metadata = entry.metadata()?;
        if file_name.to_string_lossy().contains("snapshot_format_demo") {
            println!("  {} - {} bytes", file_name.to_string_lossy(), metadata.len());
        }
    }

    println!("\n🎯 关键发现:");
    println!("• JSON: 可读性最好，文件较大，兼容性最佳");  
    println!("• CBOR: 二进制格式，文件较小，解析速度快");
    println!("• MessagePack: 紧凑高效，在大小和速度间平衡");
    println!("• 所有格式都完整支持插件状态的序列化和反序列化");
    
    println!("\n💡 您可以在 {:?} 目录中查看生成的文件", output_dir.canonicalize()?);
    
    Ok(())
}