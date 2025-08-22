use std::io::{self, Read, Seek, Write};
use serde::{Serialize, de::DeserializeOwned};

use crate::zipdoc::{ZipDocumentReader, ZipDocumentWriter};
use crate::zipdoc::snapshot::{
    SnapshotShardMeta, read_snapshot_shards, for_each_snapshot_shard_raw,
};

pub fn write_snapshot_shards_cbor<W, F, T>(
    zw: &mut ZipDocumentWriter<W>,
    meta: &SnapshotShardMeta,
    mut get_shard_value: F,
    zstd_level: i32,
) -> io::Result<()>
where
    W: Write + Seek,
    F: FnMut(usize) -> io::Result<T>,
    T: Serialize,
{
    let meta_val = serde_json::to_value(meta)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    zw.add_json("snapshot/meta.json", &meta_val)?;
    for i in 0..meta.num_shards {
        let v = get_shard_value(i)?;
        let bytes = serde_cbor::to_vec(&v)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let zst = zstd::stream::encode_all(&bytes[..], zstd_level)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let name = format!("snapshot/shard-{:03}.bin.zst", i);
        zw.add_stored(&name, &zst)?;
    }
    Ok(())
}

pub fn read_and_decode_snapshot_shards_cbor<
    R: Read + Seek,
    T: DeserializeOwned,
>(
    zr: &mut ZipDocumentReader<R>
) -> io::Result<(SnapshotShardMeta, Vec<T>)> {
    let (meta, shards_raw) = read_snapshot_shards(zr)?;
    let mut out: Vec<T> = Vec::with_capacity(shards_raw.len());
    for raw in shards_raw.iter() {
        let val: T = serde_cbor::from_slice(raw)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        out.push(val);
    }
    Ok((meta, out))
}

pub fn for_each_snapshot_shard_cbor<R: Read + Seek, T, F>(
    zr: &mut ZipDocumentReader<R>,
    mut on_shard: F,
) -> io::Result<SnapshotShardMeta>
where
    T: DeserializeOwned,
    F: FnMut(usize, T) -> io::Result<()>,
{
    for_each_snapshot_shard_raw(zr, |i, raw| {
        let val: T = serde_cbor::from_slice(&raw)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        on_shard(i, val)
    })
}

pub fn write_parent_map_cbor<W, T>(
    zw: &mut ZipDocumentWriter<W>,
    parent_map: &T,
    zstd_level: i32,
) -> io::Result<()>
where
    W: Write + Seek,
    T: Serialize,
{
    let bytes = serde_cbor::to_vec(parent_map)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let zst = zstd::stream::encode_all(&bytes[..], zstd_level)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    zw.add_stored("snapshot/parent_map.cbor.zst", &zst)
}

pub fn read_parent_map_cbor<R, T>(
    zr: &mut ZipDocumentReader<R>
) -> io::Result<T>
where
    R: Read + Seek,
    T: DeserializeOwned,
{
    let zst = zr.read_all("snapshot/parent_map.cbor.zst")?;
    let raw = zstd::stream::decode_all(&zst[..])
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let val: T = serde_cbor::from_slice(&raw)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(val)
}
