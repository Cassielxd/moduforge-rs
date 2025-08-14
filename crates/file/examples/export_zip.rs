
fn main(){
    
}




#[cfg(test)]
mod tests {
    use anyhow::anyhow;
use mf_core::{EditorOptionsBuilder, ForgeAsyncRuntime, ForgeResult};
use mf_file::zipdoc::{SnapshotShardMeta, ZipDocumentWriter, ZipDocumentReader};
use mf_file::zipdoc::formats::strategy::{
    SnapshotFormat, export_zip_with_format, import_zip_with_format,
};
use std::collections::HashMap as StdHashMap;
use mf_model::node::Node;
use mf_model::imbl::HashMap as ImHashMap;
use std::sync::Arc;
    #[tokio::test]
    async fn roundtrip() -> ForgeResult<()> {
        let options = EditorOptionsBuilder::new().build();
    let xml_path = "../../schema/main.xml";
    let mut editor = ForgeAsyncRuntime::from_xml_schema_path(xml_path, Some(options), None).await?;
    let state = editor.get_state();
    let tree = state.doc().get_inner().clone();
    let schema_bytes = std::fs::read(xml_path).map_err(|e| anyhow!(e))?;

    std::fs::create_dir_all("./data").ok();
    let zip_path = "./data/demo_doc.ysf";
    {
        let num_shards = tree.nodes.len();
        let shard_counts: Vec<usize> = tree.nodes.iter().map(|m| m.len()).collect();
        let meta_json = serde_json::json!({"title":"demo document","version":state.version});
        let shard_meta = SnapshotShardMeta { root_id: tree.root_id.clone(), num_shards, counts: shard_counts };
        export_zip_with_format(
            zip_path,
            &meta_json,
            &schema_bytes,
            &shard_meta,
            |i| Ok(tree.nodes[i].clone()),
            Some(&tree.parent_map),
            1,
            SnapshotFormat::MsgPack,
        ).map_err(|e| anyhow!(e))?;
    }
    println!("exported zip: {}", zip_path);

    // read back (MessagePack + zstd) 直接反序列化为 imbl::HashMap<String, Arc<Node>>
    {
        let file = std::fs::File::open(zip_path).map_err(|e| anyhow!(e))?;
        let mut zr = ZipDocumentReader::new(file).map_err(|e| anyhow!(e))?;
        let meta_len = zr.read_all("meta.json").map_err(|e| anyhow!(e))?.len();
        let schema_len = zr.read_all("schema.xml").map_err(|e| anyhow!(e))?.len();
        // 也可用高层导入封装
        let (_meta_json, _schema_xml, meta, maps, parent_map): (
            serde_json::Value,
            Vec<u8>,
            SnapshotShardMeta,
            Vec<ImHashMap<String, Arc<Node>>>,
            Option<ImHashMap<String, String>>,
        ) = import_zip_with_format(zip_path, SnapshotFormat::MsgPack, true).map_err(|e| anyhow!(e))?;
        let total_nodes: usize = maps.iter().map(|m| m.len()).sum::<usize>();
        println!(
            "read zip: meta={}B, schema={}B, shards={}, nodes={}, parent_map_entries={}",
            meta_len, schema_len, meta.num_shards, total_nodes, parent_map.unwrap().len()
        );
    }
        Ok(())
    }
}

// moved implementations to modules: document.rs, history.rs, zipdoc.rs



