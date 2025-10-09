use std::io::{self, Read, Seek, Write};
use std::path::Path;
use rayon::prelude::*;
use serde::{Serialize, Deserialize, de::DeserializeOwned};

use super::writer::ZipDocumentWriter;
use super::reader::ZipDocumentReader;

// =============== 分片快照辅助函数 ===============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotShardMeta {
    pub root_id: String,
    pub num_shards: usize,
    pub counts: Vec<usize>,
}

/// 写入分片快照：
/// - snapshot/meta.json（分片元数据，JSON）
/// - snapshot/shard-XXX.bin.zst（每个分片的压缩数据）
pub fn write_snapshot_shards<W, F>(
    zw: &mut ZipDocumentWriter<W>,
    meta: &SnapshotShardMeta,
    mut get_shard_bytes: F,
    zstd_level: i32,
) -> io::Result<()>
where
    W: Write + Seek,
    F: FnMut(usize) -> io::Result<Vec<u8>>,
{
    // 写 meta
    let meta_val = serde_json::to_value(meta).map_err(io::Error::other)?;
    zw.add_json("snapshot/meta.json", &meta_val)?;
    // 写每个分片
    for i in 0..meta.num_shards {
        let raw = get_shard_bytes(i)?;
        let zst = zstd::stream::encode_all(&raw[..], zstd_level)
            .map_err(io::Error::other)?;
        let name = format!("snapshot/shard-{i:03}.bin.zst");
        zw.add_stored(&name, &zst)?;
    }
    Ok(())
}

/// 读取分片快照：返回元数据和解压后的每个分片字节
pub fn read_snapshot_shards<R: Read + Seek>(
    zr: &mut ZipDocumentReader<R>
) -> io::Result<(SnapshotShardMeta, Vec<Vec<u8>>)> {
    let meta_bytes = zr.read_all("snapshot/meta.json")?;
    let meta: SnapshotShardMeta =
        serde_json::from_slice(&meta_bytes).map_err(io::Error::other)?;
    // 顺序读取 ZIP 条目（ZIP 读取器不支持并发），但解压并行
    let mut compressed: Vec<Vec<u8>> = Vec::with_capacity(meta.num_shards);
    for i in 0..meta.num_shards {
        let name = format!("snapshot/shard-{i:03}.bin.zst");
        let zst = zr.read_all(&name)?;
        compressed.push(zst);
    }
    let shards: Vec<Vec<u8>> = compressed
        .into_par_iter()
        .map(|zst| zstd::stream::decode_all(&zst[..]).map_err(io::Error::other))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((meta, shards))
}

/// 读取并用 bincode 反序列化每个分片
pub fn read_and_decode_snapshot_shards<R: Read + Seek, T: DeserializeOwned>(
    zr: &mut ZipDocumentReader<R>
) -> io::Result<(SnapshotShardMeta, Vec<T>)> {
    let (meta, shards_raw) = read_snapshot_shards(zr)?;
    // 为避免对 T 施加 Send 约束，这里顺序反序列化
    let mut out: Vec<T> = Vec::with_capacity(shards_raw.len());
    for raw in shards_raw.iter() {
        let (val, _): (T, _) =
            bincode::serde::decode_from_slice(raw, bincode::config::standard())
                .map_err(io::Error::other)?;
        out.push(val);
    }
    Ok((meta, out))
}

/// 流式：逐个分片解压为原始字节并回调处理，避免一次性加载内存
pub fn for_each_snapshot_shard_raw<R: Read + Seek, F>(
    zr: &mut ZipDocumentReader<R>,
    mut on_shard: F,
) -> io::Result<SnapshotShardMeta>
where
    F: FnMut(usize, Vec<u8>) -> io::Result<()>,
{
    let meta_bytes = zr.read_all("snapshot/meta.json")?;
    let meta: SnapshotShardMeta =
        serde_json::from_slice(&meta_bytes).map_err(io::Error::other)?;
    for i in 0..meta.num_shards {
        let name = format!("snapshot/shard-{i:03}.bin.zst");
        let zst = zr.read_all(&name)?;
        let raw =
            zstd::stream::decode_all(&zst[..]).map_err(io::Error::other)?;
        on_shard(i, raw)?;
    }
    Ok(meta)
}

/// 高层接口：导出包含 meta.json、schema.xml 和分片快照的 ZIP 文档
pub fn export_zip_with_shards<P, F>(
    path: P,
    meta_json: &serde_json::Value,
    schema_xml: &[u8],
    shard_meta: &SnapshotShardMeta,
    get_shard_bytes: F,
    zstd_level: i32,
) -> io::Result<()>
where
    P: AsRef<Path>,
    F: FnMut(usize) -> io::Result<Vec<u8>>,
{
    let file = std::fs::File::create(path)?;
    let mut zw = ZipDocumentWriter::new(file)?;
    zw.add_json("meta.json", meta_json)?;
    zw.add_deflated("schema.xml", schema_xml)?;
    write_snapshot_shards(&mut zw, shard_meta, get_shard_bytes, zstd_level)?;
    let _ = zw.finalize()?;
    Ok(())
}

/// 高层接口：导入 ZIP 文档，返回（meta.json、schema.xml、分片元数据、解码后的分片）
pub fn import_zip_with_shards<P, T>(
    path: P
) -> io::Result<(serde_json::Value, Vec<u8>, SnapshotShardMeta, Vec<T>)>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let file = std::fs::File::open(path)?;
    let mut zr = ZipDocumentReader::new(file)?;
    let meta_json = zr.read_all("meta.json")?;
    let meta_val: serde_json::Value =
        serde_json::from_slice(&meta_json).map_err(io::Error::other)?;
    let schema_xml = zr.read_all("schema.xml")?;
    let (shard_meta, decoded) =
        read_and_decode_snapshot_shards::<_, T>(&mut zr)?;
    Ok((meta_val, schema_xml, shard_meta, decoded))
}
