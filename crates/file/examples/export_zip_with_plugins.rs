fn main() {}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use mf_core::{EditorOptionsBuilder, ForgeAsyncRuntime, ForgeResult};
    use mf_file::zipdoc::SnapshotShardMeta;
    use mf_file::zipdoc::formats::strategy::{
        SnapshotFormat, export_zip_with_format, import_zip_with_format,
    };
    use mf_model::node::Node;
    use mf_model::imbl::HashMap as ImHashMap;
    use std::collections::HashMap;
    use std::sync::Arc;

    // 模拟插件状态数据
    fn create_mock_plugin_states() -> HashMap<String, Vec<u8>> {
        let mut plugin_states = HashMap::new();
        
        // 模拟各种插件状态数据
        plugin_states.insert(
            "editor_config".to_string(),
            b"font_size=14,theme=dark,word_wrap=true".to_vec()
        );
        plugin_states.insert(
            "vim_mode".to_string(), 
            b"enabled=true,leader_key=space,relative_numbers=true".to_vec()
        );
        plugin_states.insert(
            "collaboration".to_string(),
            serde_json::to_vec(&serde_json::json!({
                "user_id": "user123",
                "room_id": "room456", 
                "cursor_position": {"line": 42, "column": 15},
                "selections": [{"start": [1, 0], "end": [1, 10]}]
            })).unwrap()
        );
        plugin_states.insert(
            "git_status".to_string(),
            b"branch=main,status=clean,ahead=0,behind=0".to_vec()
        );
        
        plugin_states
    }

    #[tokio::test]
    async fn roundtrip_json_with_plugins() -> ForgeResult<()> {
        let options = EditorOptionsBuilder::new().build();
        let xml_path = "../../schema/main.xml";
        let editor = ForgeAsyncRuntime::from_xml_schema_path(
            xml_path,
            Some(options),
            None,
        )
        .await?;
        let state = editor.get_state();
        let tree = state.doc().get_inner().clone();
        let schema_bytes = std::fs::read(xml_path).map_err(|e| anyhow!(e))?;

        // 准备插件状态
        let plugin_states = create_mock_plugin_states();
        println!("创建插件状态: {} 个插件", plugin_states.len());
        for (name, data) in &plugin_states {
            println!("  {}: {} bytes", name, data.len());
        }

        std::fs::create_dir_all("./plugin_export_examples").ok();
        let zip_path = "./plugin_export_examples/demo_doc_json_with_plugins.ysf";
        
        // 导出 - 使用 JSON 格式
        {
            let num_shards = tree.nodes.len();
            let shard_counts: Vec<usize> =
                tree.nodes.iter().map(|m| m.len()).collect();
            let meta_json = serde_json::json!({
                "title": "demo document with plugins",
                "version": state.version,
                "plugin_count": plugin_states.len(),
                "format": "json"
            });
            let shard_meta = SnapshotShardMeta {
                root_id: tree.root_id.clone(),
                num_shards,
                counts: shard_counts,
            };
            
            export_zip_with_format(
                zip_path,
                &meta_json,
                &schema_bytes,
                &shard_meta,
                |i| Ok(tree.nodes[i].clone()),
                Some(&tree.parent_map),
                Some(plugin_states.clone()),  // 添加插件状态
                1,
                SnapshotFormat::Json,
            )
            .map_err(|e| anyhow!(e))?;
        }
        println!("✅ JSON 格式导出完成: {}", zip_path);

        // 导入并验证
        {
            let (_meta_json, _schema_xml, meta, maps, parent_map, imported_plugin_states): (
                serde_json::Value,
                Vec<u8>,
                SnapshotShardMeta,
                Vec<ImHashMap<String, Arc<Node>>>,
                Option<ImHashMap<String, String>>,
                Option<HashMap<String, Vec<u8>>>,
            ) = import_zip_with_format(zip_path, SnapshotFormat::Json, true, true)
                .map_err(|e| anyhow!(e))?;
                
            let meta_len = _meta_json.to_string().len();
            let schema_len = _schema_xml.len();
            let total_nodes: usize = maps.iter().map(|m| m.len()).sum::<usize>();
            
            println!(
                "读取 ZIP (JSON): meta={}B, schema={}B, shards={}, nodes={}, parent_map_entries={}",
                meta_len,
                schema_len,
                meta.num_shards,
                total_nodes,
                parent_map.as_ref().map(|pm| pm.len()).unwrap_or(0)
            );

            // 验证插件状态
            if let Some(imported_states) = imported_plugin_states {
                println!("导入的插件状态: {} 个", imported_states.len());
                let mut verified = 0;
                for (name, expected_data) in &plugin_states {
                    if let Some(actual_data) = imported_states.get(name) {
                        if actual_data == expected_data {
                            verified += 1;
                            println!("  ✅ {}: 验证通过", name);
                        } else {
                            println!("  ❌ {}: 数据不匹配", name);
                        }
                    } else {
                        println!("  ❌ {}: 缺失", name);
                    }
                }
                println!("插件状态验证: {}/{} 通过", verified, plugin_states.len());
            } else {
                println!("⚠️ 未找到插件状态数据");
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn roundtrip_cbor_with_plugins() -> ForgeResult<()> {
        let options = EditorOptionsBuilder::new().build();
        let xml_path = "../../schema/main.xml";
        let editor = ForgeAsyncRuntime::from_xml_schema_path(
            xml_path,
            Some(options),
            None,
        )
        .await?;
        let state = editor.get_state();
        let tree = state.doc().get_inner().clone();
        let schema_bytes = std::fs::read(xml_path).map_err(|e| anyhow!(e))?;

        let plugin_states = create_mock_plugin_states();
        std::fs::create_dir_all("./plugin_export_examples").ok();
        let zip_path = "./plugin_export_examples/demo_doc_cbor_with_plugins.ysf";
        
        // 导出 - 使用 CBOR 格式
        {
            let num_shards = tree.nodes.len();
            let shard_counts: Vec<usize> =
                tree.nodes.iter().map(|m| m.len()).collect();
            let meta_json = serde_json::json!({
                "title": "demo document with plugins",
                "version": state.version,
                "plugin_count": plugin_states.len(),
                "format": "cbor"
            });
            let shard_meta = SnapshotShardMeta {
                root_id: tree.root_id.clone(),
                num_shards,
                counts: shard_counts,
            };
            
            export_zip_with_format(
                zip_path,
                &meta_json,
                &schema_bytes,
                &shard_meta,
                |i| Ok(tree.nodes[i].clone()),
                Some(&tree.parent_map),
                Some(plugin_states.clone()),  // 添加插件状态
                1,
                SnapshotFormat::Cbor,
            )
            .map_err(|e| anyhow!(e))?;
        }
        println!("✅ CBOR 格式导出完成: {}", zip_path);

        // 导入并验证
        {
            let (_meta_json, _schema_xml, meta, maps, parent_map, imported_plugin_states): (
                serde_json::Value,
                Vec<u8>,
                SnapshotShardMeta,
                Vec<ImHashMap<String, Arc<Node>>>,
                Option<ImHashMap<String, String>>,
                Option<HashMap<String, Vec<u8>>>,
            ) = import_zip_with_format(zip_path, SnapshotFormat::Cbor, true, true)
                .map_err(|e| anyhow!(e))?;
                
            println!(
                "读取 ZIP (CBOR): meta={}B, schema={}B, shards={}, nodes={}",
                _meta_json.to_string().len(),
                _schema_xml.len(),
                meta.num_shards,
                maps.iter().map(|m| m.len()).sum::<usize>()
            );

            // 验证插件状态
            if let Some(imported_states) = imported_plugin_states {
                let verified = plugin_states.iter()
                    .filter(|(name, data)| imported_states.get(*name) == Some(data))
                    .count();
                println!("CBOR 插件状态验证: {}/{} 通过", verified, plugin_states.len());
            }
        }
        Ok(())
    }

    #[tokio::test] 
    async fn compare_formats_with_plugins() -> ForgeResult<()> {
        let options = EditorOptionsBuilder::new().build();
        let xml_path = "../../schema/main.xml";
        let editor = ForgeAsyncRuntime::from_xml_schema_path(
            xml_path,
            Some(options),
            None,
        )
        .await?;
        let state = editor.get_state();
        let tree = state.doc().get_inner().clone();
        let schema_bytes = std::fs::read(xml_path).map_err(|e| anyhow!(e))?;

        let plugin_states = create_mock_plugin_states();
        std::fs::create_dir_all("./plugin_export_examples").ok();
        
        let formats = vec![
            (SnapshotFormat::Json, "demo_format_comparison_json.ysf"),
            (SnapshotFormat::Cbor, "demo_format_comparison_cbor.ysf"),
            (SnapshotFormat::MsgPack, "demo_format_comparison_msgpack.ysf"),
        ];

        println!("=== 格式对比测试 ===");
        
        for (format, filename) in formats {
            let zip_path = format!("./plugin_export_examples/{}", filename);
            
            // 导出
            {
                let num_shards = tree.nodes.len();
                let shard_counts: Vec<usize> =
                    tree.nodes.iter().map(|m| m.len()).collect();
                let meta_json = serde_json::json!({
                    "title": "format comparison test",
                    "version": state.version,
                    "plugin_count": plugin_states.len(),
                    "format": format.as_str()
                });
                let shard_meta = SnapshotShardMeta {
                    root_id: tree.root_id.clone(),
                    num_shards,
                    counts: shard_counts,
                };
                
                export_zip_with_format(
                    &zip_path,
                    &meta_json,
                    &schema_bytes,
                    &shard_meta,
                    |i| Ok(tree.nodes[i].clone()),
                    Some(&tree.parent_map),
                    Some(plugin_states.clone()),
                    1,
                    format,
                )
                .map_err(|e| anyhow!(e))?;
            }

            // 检查文件大小
            let file_size = std::fs::metadata(&zip_path)
                .map(|m| m.len())
                .unwrap_or(0);
                
            println!("✅ {}: {} bytes", format.as_str().to_uppercase(), file_size);
        }
        
        Ok(())
    }
}