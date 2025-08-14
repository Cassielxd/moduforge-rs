use std::io::{self, Read, Seek, Write};
use std::path::Path;
use serde::{Serialize, de::DeserializeOwned};

use crate::zipdoc::{ZipDocumentReader, ZipDocumentWriter};
use crate::zipdoc::snapshot::SnapshotShardMeta;

use super::{json, cbor, msgpack};

#[derive(Debug, Clone, Copy)]
pub enum SnapshotFormat {
    Json,
    Cbor,
    MsgPack,
}

impl SnapshotFormat {
    pub fn write_shards<W, F, T>(
        &self,
        zw: &mut ZipDocumentWriter<W>,
        meta: &SnapshotShardMeta,
        get_shard_value: F,
        zstd_level: i32,
    ) -> io::Result<()>
    where
        W: Write + Seek,
        F: FnMut(usize) -> io::Result<T>,
        T: Serialize,
    {
        match self {
            SnapshotFormat::Json => json::write_snapshot_shards_json(zw, meta, get_shard_value, zstd_level),
            SnapshotFormat::Cbor => cbor::write_snapshot_shards_cbor(zw, meta, get_shard_value, zstd_level),
            SnapshotFormat::MsgPack => msgpack::write_snapshot_shards_msgpack(zw, meta, get_shard_value, zstd_level),
        }
    }

    pub fn read_shards<R, T>(
        &self,
        zr: &mut ZipDocumentReader<R>,
    ) -> io::Result<(SnapshotShardMeta, Vec<T>)>
    where
        R: Read + Seek,
        T: DeserializeOwned,
    {
        match self {
            SnapshotFormat::Json => json::read_and_decode_snapshot_shards_json(zr),
            SnapshotFormat::Cbor => cbor::read_and_decode_snapshot_shards_cbor(zr),
            SnapshotFormat::MsgPack => msgpack::read_and_decode_snapshot_shards_msgpack(zr),
        }
    }

    pub fn for_each_shard<R, T, F>(
        &self,
        zr: &mut ZipDocumentReader<R>,
        on_shard: F,
    ) -> io::Result<SnapshotShardMeta>
    where
        R: Read + Seek,
        T: DeserializeOwned,
        F: FnMut(usize, T) -> io::Result<()>,
    {
        match self {
            SnapshotFormat::Json => json::for_each_snapshot_shard_json(zr, on_shard),
            SnapshotFormat::Cbor => cbor::for_each_snapshot_shard_cbor(zr, on_shard),
            SnapshotFormat::MsgPack => msgpack::for_each_snapshot_shard_msgpack(zr, on_shard),
        }
    }

    pub fn write_parent_map<W, T>(
        &self,
        zw: &mut ZipDocumentWriter<W>,
        parent_map: &T,
        zstd_level: i32,
    ) -> io::Result<()>
    where
        W: Write + Seek,
        T: Serialize,
    {
        match self {
            SnapshotFormat::Json => json::write_parent_map_json(zw, parent_map, zstd_level),
            SnapshotFormat::Cbor => cbor::write_parent_map_cbor(zw, parent_map, zstd_level),
            SnapshotFormat::MsgPack => msgpack::write_parent_map_msgpack(zw, parent_map, zstd_level),
        }
    }

    pub fn read_parent_map<R, T>(&self, zr: &mut ZipDocumentReader<R>) -> io::Result<T>
    where
        R: Read + Seek,
        T: DeserializeOwned,
    {
        match self {
            SnapshotFormat::Json => json::read_parent_map_json(zr),
            SnapshotFormat::Cbor => cbor::read_parent_map_cbor(zr),
            SnapshotFormat::MsgPack => msgpack::read_parent_map_msgpack(zr),
        }
    }
}

impl SnapshotFormat {
    pub fn as_str(&self) -> &'static str {
        match self { SnapshotFormat::Json => "json", SnapshotFormat::Cbor => "cbor", SnapshotFormat::MsgPack => "msgpack" }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "json" => Some(SnapshotFormat::Json),
            "cbor" => Some(SnapshotFormat::Cbor),
            "msgpack" | "rmp" | "msg" => Some(SnapshotFormat::MsgPack),
            _ => None,
        }
    }
    pub fn from_extension<P: AsRef<Path>>(path: P) -> Option<Self> {
        match path.as_ref().extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()) {
            Some(ext) if ext == "json" => Some(SnapshotFormat::Json),
            Some(ext) if ext == "cbor" || ext == "cbr" => Some(SnapshotFormat::Cbor),
            Some(ext) if ext == "msgpack" || ext == "rmp" || ext == "msg" => Some(SnapshotFormat::MsgPack),
            _ => None,
        }
    }
}

// 高层封装：根据策略导出 ZIP（meta.json + schema.xml + 分片 + 可选 parent_map）
pub fn export_zip_with_format<P, F, T, PM>(
    path: P,
    meta_json: &serde_json::Value,
    schema_xml: &[u8],
    shard_meta: &SnapshotShardMeta,
    get_shard_value: F,
    parent_map: Option<&PM>,
    zstd_level: i32,
    format: SnapshotFormat,
) -> io::Result<()>
where
    P: AsRef<Path>,
    F: FnMut(usize) -> io::Result<T>,
    T: Serialize,
    PM: Serialize,
{
    let file = std::fs::File::create(path)?;
    let mut zw = ZipDocumentWriter::new(file)?;
    zw.add_json("meta.json", meta_json)?;
    zw.add_deflated("schema.xml", schema_xml)?;
    format.write_shards(&mut zw, shard_meta, get_shard_value, zstd_level)?;
    if let Some(pm) = parent_map {
        format.write_parent_map(&mut zw, pm, zstd_level)?;
    }
    let _ = zw.finalize()?;
    Ok(())
}

// 高层封装：根据策略导入 ZIP（返回 meta.json, schema.xml, 分片, 可选 parent_map）
pub fn import_zip_with_format<P, T, PM>(
    path: P,
    format: SnapshotFormat,
    read_parent_map: bool,
) -> io::Result<(serde_json::Value, Vec<u8>, SnapshotShardMeta, Vec<T>, Option<PM>)>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
    PM: DeserializeOwned,
{
    let file = std::fs::File::open(path)?;
    let mut zr = ZipDocumentReader::new(file)?;
    let meta_json = zr.read_all("meta.json")?;
    let meta_val: serde_json::Value = serde_json::from_slice(&meta_json)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let schema_xml = zr.read_all("schema.xml")?;
    let (shard_meta, decoded) = format.read_shards::<_, T>(&mut zr)?;
    let parent_map = if read_parent_map {
        Some(format.read_parent_map::<_, PM>(&mut zr)?)
    } else {
        None
    };
    Ok((meta_val, schema_xml, shard_meta, decoded, parent_map))
}


