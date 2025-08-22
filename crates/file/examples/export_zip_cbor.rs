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
    use std::sync::Arc;

    #[tokio::test]
    async fn roundtrip_cbor() -> ForgeResult<()> {
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

        std::fs::create_dir_all("./data").ok();
        let zip_path = "./data/demo_doc_cbor.ysf";
        {
            let num_shards = tree.nodes.len();
            let shard_counts: Vec<usize> =
                tree.nodes.iter().map(|m| m.len()).collect();
            let meta_json = serde_json::json!({"title":"demo document","version":state.version});
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
                1,
                SnapshotFormat::Cbor,
            )
            .map_err(|e| anyhow!(e))?;
        }

        // read back via strategy
        {
            let (_meta_json, _schema_xml, meta, maps, parent_map): (
                serde_json::Value,
                Vec<u8>,
                SnapshotShardMeta,
                Vec<ImHashMap<String, Arc<Node>>>,
                Option<ImHashMap<String, String>>,
            ) = import_zip_with_format(zip_path, SnapshotFormat::Cbor, true)
                .map_err(|e| anyhow!(e))?;
            let meta_len = _meta_json.to_string().len();
            let schema_len = _schema_xml.len();
            let total_nodes: usize =
                maps.iter().map(|m| m.len()).sum::<usize>();
            println!(
                "read zip (cbor): meta={}B, schema={}B, shards={}, nodes={}, parent_map_entries={}",
                meta_len,
                schema_len,
                meta.num_shards,
                total_nodes,
                parent_map.unwrap().len()
            );
        }
        Ok(())
    }
}
