use std::collections::HashMap;
use std::path::Path;
use mf_file::zipdoc::formats::strategy::{
    export_plugin_states_only, 
    import_plugin_states_only,
    has_plugin_states,
    list_zip_plugins
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = Path::new("plugin_export_examples");
    std::fs::create_dir_all(output_dir)?;
    let plugin_backup_path = output_dir.join("plugin_backup.zip");

    println!("=== 插件状态备份和恢复示例 ===\n");

    // 准备插件状态数据
    let mut plugin_states = HashMap::new();
    plugin_states.insert("editor_settings".to_string(), b"editor preferences and shortcuts".to_vec());
    plugin_states.insert("theme_config".to_string(), b"dark theme with custom colors".to_vec());
    plugin_states.insert("extension_data".to_string(), b"installed extensions and settings".to_vec());
    plugin_states.insert("workspace_layout".to_string(), b"panel arrangements and sizes".to_vec());

    println!("准备插件状态数据:");
    for (name, data) in &plugin_states {
        println!("  - {}: {} bytes", name, data.len());
    }

    // 导出插件状态到ZIP
    println!("\n导出插件状态到 ZIP 文件...");
    export_plugin_states_only(&plugin_backup_path, plugin_states.clone(), None)?;
    println!("✅ 插件状态已导出到: {:?}", plugin_backup_path);

    // 验证ZIP文件包含插件状态
    println!("\n验证 ZIP 文件...");
    let has_states = has_plugin_states(&plugin_backup_path)?;
    println!("包含插件状态: {}", has_states);

    let plugins_in_zip = list_zip_plugins(&plugin_backup_path)?;
    println!("ZIP 中的插件数量: {}", plugins_in_zip.len());
    for plugin in &plugins_in_zip {
        println!("  - {}", plugin);
    }

    // 导入插件状态
    println!("\n导入插件状态...");
    let (meta, imported_states) = import_plugin_states_only(&plugin_backup_path)?;
    
    println!("元数据信息:");
    println!("  类型: {}", meta["type"]);
    println!("  版本: {}", meta["version"]);
    println!("  插件数量: {}", meta["plugin_count"]);
    println!("  时间戳: {}", meta["timestamp"]);

    println!("\n导入的插件状态:");
    for (name, data) in &imported_states {
        let content = String::from_utf8_lossy(data);
        println!("  - {}: \"{}\"", name, content);
    }

    // 验证数据完整性
    println!("\n数据完整性检查:");
    let mut verified_count = 0;
    for (name, expected_data) in &plugin_states {
        if let Some(actual_data) = imported_states.get(name) {
            if actual_data == expected_data {
                verified_count += 1;
                println!("  ✅ {}: 匹配", name);
            } else {
                println!("  ❌ {}: 不匹配", name);
            }
        } else {
            println!("  ❌ {}: 缺失", name);
        }
    }

    let total_plugins = plugin_states.len();
    println!("\n总结: {}/{} 插件状态验证通过 ({}%)", 
        verified_count, 
        total_plugins,
        (verified_count * 100) / total_plugins
    );

    if verified_count == total_plugins {
        println!("🎉 所有插件状态备份和恢复成功！");
    } else {
        println!("⚠️  部分插件状态验证失败");
    }

    // 演示自定义元数据
    let custom_backup_path = output_dir.join("custom_plugin_backup.zip");
    println!("\n=== 自定义元数据示例 ===");
    
    let custom_meta = serde_json::json!({
        "type": "user_plugin_backup",
        "version": "2.0", 
        "user_id": "user123",
        "backup_name": "我的插件配置",
        "description": "包含编辑器和主题设置的完整备份",
        "created_by": "ModuForge-RS",
        "plugin_count": plugin_states.len(),
        "tags": ["editor", "theme", "workspace", "settings"]
    });

    export_plugin_states_only(&custom_backup_path, plugin_states, Some(&custom_meta))?;
    println!("✅ 自定义元数据备份已创建");

    let (custom_meta_imported, _) = import_plugin_states_only(&custom_backup_path)?;
    println!("自定义备份信息:");
    println!("  备份名称: {}", custom_meta_imported["backup_name"]);
    println!("  描述: {}", custom_meta_imported["description"]);
    println!("  创建者: {}", custom_meta_imported["created_by"]);
    println!("  标签: {:?}", custom_meta_imported["tags"]);

    println!("\n📂 生成的文件:");
    for entry in std::fs::read_dir(output_dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let file_name = entry.file_name();
        let file_size = metadata.len();
        println!("  {} - {} bytes", file_name.to_string_lossy(), file_size);
    }

    println!("\n🎉 strategy 插件导出示例完成！");
    println!("💡 您可以在 {:?} 目录中查看生成的文件", output_dir.canonicalize()?);
    Ok(())
}