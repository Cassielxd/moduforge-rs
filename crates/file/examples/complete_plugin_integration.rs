// 完整的插件状态集成示例：展示从低级API到高级便利函数的所有功能

use std::collections::HashMap;
use std::path::Path;
use mf_file::{
    ZipDocumentWriter, ZipDocumentReader,
    export_plugin_states_only, import_plugin_states_only,
    has_plugin_states, list_zip_plugins
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ModuForge-RS 完整插件状态集成示例 ===\n");
    
    // 创建输出目录
    let output_dir = Path::new("plugin_export_examples");
    std::fs::create_dir_all(output_dir)?;
    println!("📁 输出目录: {:?}", output_dir.canonicalize()?);
    println!();
    
    // 准备测试数据
    let mut plugin_states = HashMap::new();
    plugin_states.insert("vim_mode".to_string(), b"vim keybindings enabled".to_vec());
    plugin_states.insert("auto_save".to_string(), b"auto save every 30 seconds".to_vec());
    plugin_states.insert("git_integration".to_string(), b"git status in sidebar".to_vec());
    plugin_states.insert("color_scheme".to_string(), b"monokai pro theme".to_vec());
    
    println!("📦 准备的插件状态:");
    for (name, data) in &plugin_states {
        println!("  {} - {} bytes: \"{}\"", name, data.len(), String::from_utf8_lossy(data));
    }

    // ========== 方法1: 低级API - 直接使用 ZipDocumentWriter/Reader ==========
    println!("\n🔧 方法1: 使用低级API (ZipDocumentWriter/Reader)");
    let low_level_path = output_dir.join("low_level_export.zip");
    
    // 写入
    {
        let file = std::fs::File::create(&low_level_path)?;
        let mut writer = ZipDocumentWriter::new(file)?;
        
        // 添加主要内容
        writer.add_json("document.json", &serde_json::json!({
            "title": "我的文档",
            "content": "这是一个完整的文档示例"
        }))?;
        
        // 逐个添加插件状态
        for (name, data) in &plugin_states {
            writer.add_plugin_state(name, data)?;
        }
        
        writer.finalize()?;
    }
    println!("✅ 低级API导出完成 -> {:?}", low_level_path);
    
    // 读取验证
    {
        let file = std::fs::File::open(&low_level_path)?;
        let mut reader = ZipDocumentReader::new(file)?;
        
        let plugins = reader.list_plugins()?;
        println!("发现 {} 个插件: {:?}", plugins.len(), plugins);
        
        // 验证特定插件
        if let Some(vim_data) = reader.read_plugin_state("vim_mode")? {
            println!("vim_mode 状态: \"{}\"", String::from_utf8_lossy(&vim_data));
        }
    }

    // ========== 方法2: 高级便利函数 ==========
    println!("\n🚀 方法2: 使用高级便利函数");
    let high_level_path = output_dir.join("high_level_export.zip");
    
    // 使用便利函数导出
    export_plugin_states_only(&high_level_path, plugin_states.clone(), None)?;
    println!("✅ 高级API导出完成 -> {:?}", high_level_path);
    
    // 验证文件
    let has_states = has_plugin_states(&high_level_path)?;
    println!("包含插件状态: {}", has_states);
    
    let plugins_list = list_zip_plugins(&high_level_path)?;
    println!("插件列表: {:?}", plugins_list);
    
    // 导入验证
    let (meta, imported) = import_plugin_states_only(&high_level_path)?;
    println!("导入元数据: {}", serde_json::to_string_pretty(&meta)?);
    
    // ========== 方法3: 批量操作 ==========
    println!("\n⚡ 方法3: 批量操作");
    let batch_path = output_dir.join("batch_export.zip");
    
    {
        let file = std::fs::File::create(&batch_path)?;
        let mut writer = ZipDocumentWriter::new(file)?;
        
        // 添加文档数据
        writer.add_deflated("schema.bin", b"binary schema data")?;
        writer.add_json("config.json", &serde_json::json!({"version": "1.0"}))?;
        
        // 批量添加所有插件状态
        writer.add_plugin_states(plugin_states.clone())?;
        
        writer.finalize()?;
    }
    println!("✅ 批量操作完成 -> {:?}", batch_path);
    
    {
        let file = std::fs::File::open(&batch_path)?;
        let mut reader = ZipDocumentReader::new(file)?;
        
        let all_states = reader.read_all_plugin_states()?;
        println!("批量读取到 {} 个插件状态", all_states.len());
        
        // 检查特定插件
        println!("插件状态检查:");
        for name in ["vim_mode", "auto_save", "nonexistent"] {
            let exists = reader.has_plugin_state(name);
            println!("  {}: {}", name, if exists { "✅ 存在" } else { "❌ 不存在" });
        }
    }

    // ========== 数据完整性验证 ==========
    println!("\n🔍 数据完整性验证");
    
    let mut total_verified = 0;
    let total_methods = 3;
    
    for (method_name, path) in [
        ("低级API", &low_level_path),
        ("高级API", &high_level_path), 
        ("批量操作", &batch_path)
    ] {
        let file = std::fs::File::open(path)?;
        let mut reader = ZipDocumentReader::new(file)?;
        let imported_states = reader.read_all_plugin_states()?;
        
        let mut verified = 0;
        for (name, expected_data) in &plugin_states {
            if let Some(actual_data) = imported_states.get(name) {
                if actual_data == expected_data {
                    verified += 1;
                }
            }
        }
        
        let success_rate = (verified * 100) / plugin_states.len();
        println!("{}: {}/{} 验证通过 ({}%)", method_name, verified, plugin_states.len(), success_rate);
        
        if verified == plugin_states.len() {
            total_verified += 1;
        }
    }
    
    println!("\n📊 总体结果:");
    println!("方法验证通过: {}/{} ({}%)", total_verified, total_methods, (total_verified * 100) / total_methods);
    
    if total_verified == total_methods {
        println!("🎉 所有方法都成功完成插件状态的导出和导入！");
        println!("\n✨ 功能特性总结:");
        println!("  ✅ 支持多种导出方式 (低级API、高级便利函数、批量操作)");
        println!("  ✅ 插件状态以二进制格式存储在 plugins/ 目录");
        println!("  ✅ 支持检查插件存在性和列出所有插件");
        println!("  ✅ 支持自定义元数据和时间戳");
        println!("  ✅ 完整的数据完整性验证");
        println!("  ✅ 与现有ZIP文档格式完全兼容");
    } else {
        println!("⚠️  部分方法验证失败，需要检查实现");
    }

    println!("\n📂 生成的文件:");
    for entry in std::fs::read_dir(output_dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let file_name = entry.file_name();
        let file_size = metadata.len();
        println!("  {} - {} bytes", file_name.to_string_lossy(), file_size);
    }

    println!("\n🏁 完整插件状态集成示例结束");
    println!("💡 您可以在 {:?} 目录中查看生成的ZIP文件", output_dir.canonicalize()?);
    Ok(())
}