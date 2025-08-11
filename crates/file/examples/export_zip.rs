
fn main(){
    
}




#[cfg(test)]
mod tests {
    use anyhow::anyhow;
use mf_core::{EditorOptionsBuilder, ForgeAsyncRuntime, ForgeResult};
use mf_file::zipdoc::{SnapshotShardMeta, ZipDocumentWriter, ZipDocumentReader, write_snapshot_shards_msgpack, read_and_decode_snapshot_shards_msgpack};
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
        let file = std::fs::File::create(zip_path).map_err(|e| anyhow!(e))?;
        let mut zw = ZipDocumentWriter::new(file).map_err(|e| anyhow!(e))?;
        zw.add_json("meta.json", &meta_json).map_err(|e| anyhow!(e))?;
        zw.add_deflated("schema.xml", &schema_bytes).map_err(|e| anyhow!(e))?;
        write_snapshot_shards_msgpack(&mut zw, &shard_meta, |i| {
            // 直接序列化 imbl::HashMap<String, Arc<Node>>（serde 已启用 rc/serde）
            Ok(tree.nodes[i].clone())
        }, 1).map_err(|e| anyhow!(e))?;
        let _ = zw.finalize().map_err(|e| anyhow!(e))?;
    }
    println!("exported zip: {}", zip_path);

    // read back (MessagePack + zstd) 直接反序列化为 imbl::HashMap<String, Arc<Node>>
    {
        let file = std::fs::File::open(zip_path).map_err(|e| anyhow!(e))?;
        let mut zr = ZipDocumentReader::new(file).map_err(|e| anyhow!(e))?;
        let meta_len = zr.read_all("meta.json").map_err(|e| anyhow!(e))?.len();
        let schema_len = zr.read_all("schema.xml").map_err(|e| anyhow!(e))?.len();
        let (meta, maps): (SnapshotShardMeta, Vec<ImHashMap<String, Arc<Node>>>) = read_and_decode_snapshot_shards_msgpack(&mut zr).map_err(|e| anyhow!(e))?;
        let total_nodes: usize = maps.iter().map(|m| m.len()).sum::<usize>();
        println!("read zip: meta={}B, schema={}B, shards={}, nodes={}", meta_len, schema_len, meta.num_shards, total_nodes);
    }
        Ok(())
    }
}

// moved implementations to modules: document.rs, history.rs, zipdoc.rs



