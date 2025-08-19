fn main() {}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use mf_core::{EditorOptionsBuilder, ForgeAsyncRuntime, ForgeResult};
    use mf_file::document::{DocumentWriter, DocumentReader, SegmentType};
    use mf_model::node::Node;
    use mf_model::imbl::HashMap as ImHashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn roundtrip_single_file() -> ForgeResult<()> {
        // prepare editor and data
        let options = EditorOptionsBuilder::new().build();
        let xml_path = "../../schema/main.xml";
        let mut editor = ForgeAsyncRuntime::from_xml_schema_path(
            xml_path,
            Some(options),
            None,
        )
        .await?;
        let state = editor.get_state();
        let tree = state.doc().get_inner().clone();
        let schema_bytes = std::fs::read(xml_path).map_err(|e| anyhow!(e))?;

        // export to single append-only file
        std::fs::create_dir_all("./data").ok();
        let file_path = "./data/demo_doc.mff";
        {
            let mut w =
                DocumentWriter::begin(file_path).map_err(|e| anyhow!(e))?;
            let meta_json = serde_json::json!({
                "title": "demo document",
                "version": state.version,
                "root_id": tree.root_id,
                "num_shards": tree.nodes.len(),
                "counts": tree.nodes.iter().map(|m| m.len()).collect::<Vec<_>>()
            });
            let meta_bytes =
                serde_json::to_vec(&meta_json).map_err(|e| anyhow!(e))?;
            w.add_segment(SegmentType::Meta, &meta_bytes)
                .map_err(|e| anyhow!(e))?;
            w.add_segment(SegmentType::Schema, &schema_bytes)
                .map_err(|e| anyhow!(e))?;

            // snapshot: MessagePack + zstd of Vec<ImHashMap<String, Arc<Node>>>
            // convert imbl::Vector -> Vec for compact serialization here
            let maps: Vec<ImHashMap<String, Arc<Node>>> =
                tree.nodes.iter().cloned().collect();
            let packed = rmp_serde::to_vec(&maps).map_err(|e| anyhow!(e))?;
            let compressed = zstd::stream::encode_all(&packed[..], 1)
                .map_err(|e| anyhow!(e))?;
            w.add_segment(SegmentType::Snapshot, &compressed)
                .map_err(|e| anyhow!(e))?;

            w.finalize().map_err(|e| anyhow!(e))?;
        }

        // read back
        {
            let r = DocumentReader::open(file_path).map_err(|e| anyhow!(e))?;
            let meta_len = r
                .read_segment(SegmentType::Meta)
                .map_err(|e| anyhow!(e))?
                .unwrap()
                .len();
            let schema_len = r
                .read_segment(SegmentType::Schema)
                .map_err(|e| anyhow!(e))?
                .unwrap()
                .len();
            let snapshot_bytes = r
                .read_segment(SegmentType::Snapshot)
                .map_err(|e| anyhow!(e))?
                .unwrap();
            let decoded = zstd::stream::decode_all(snapshot_bytes)
                .map_err(|e| anyhow!(e))?;
            let maps: Vec<ImHashMap<String, Arc<Node>>> =
                rmp_serde::from_slice(&decoded).map_err(|e| anyhow!(e))?;
            let total_nodes: usize =
                maps.iter().map(|m| m.len()).sum::<usize>();
            println!(
                "read single-file: meta={}B, schema={}B, shards={}, nodes={}",
                meta_len,
                schema_len,
                maps.len(),
                total_nodes
            );
        }

        Ok(())
    }
}
