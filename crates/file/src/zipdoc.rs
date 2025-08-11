use std::io::{self, Read, Seek, Write};
use std::path::Path;
use zip::{ZipWriter, ZipArchive, write::SimpleFileOptions, CompressionMethod};
use serde::{Serialize, Deserialize, de::DeserializeOwned};

// 基于 ZIP 的文档写入器（docx 风格容器）
pub struct ZipDocumentWriter<W: Write + Seek> {
    zip: ZipWriter<W>,
    manifest: serde_json::Value,
}

impl<W: Write + Seek> ZipDocumentWriter<W> {
    // 创建写入器
    pub fn new(w: W) -> io::Result<Self> {
        let zip = ZipWriter::new(w);
        let manifest = serde_json::json!({ "version": 1, "entries": [] });
        Ok(Self { zip, manifest })
    }
    // 写入 JSON 文件（deflate 压缩）
    pub fn add_json(&mut self, name: &str, value: &serde_json::Value) -> io::Result<()> {
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        self.zip.start_file(name, opts)?;
        let data = serde_json::to_vec(value).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        self.zip.write_all(&data)
    }
    // 写入原样存储的条目（不压缩）
    pub fn add_stored(&mut self, name: &str, bytes: &[u8]) -> io::Result<()> {
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        self.zip.start_file(name, opts)?;
        self.zip.write_all(bytes)
    }
    // 写入 deflate 压缩条目
    pub fn add_deflated(&mut self, name: &str, bytes: &[u8]) -> io::Result<()> {
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        self.zip.start_file(name, opts)?;
        self.zip.write_all(bytes)
    }
    // 完成写入，附带 manifest.json
    pub fn finalize(mut self) -> io::Result<W> {
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        self.zip.start_file("manifest.json", opts)?;
        let data = serde_json::to_vec(&self.manifest).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        self.zip.write_all(&data)?;
        self.zip.finish().map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

// 基于 ZIP 的文档读取器
pub struct ZipDocumentReader<R: Read + Seek> { zip: ZipArchive<R> }

impl<R: Read + Seek> ZipDocumentReader<R> {
    // 打开读取器
    pub fn new(r: R) -> io::Result<Self> { Ok(Self { zip: ZipArchive::new(r)? }) }
    // 读取指定文件完整内容
    pub fn read_all(&mut self, name: &str) -> io::Result<Vec<u8>> {
        let mut f = self.zip.by_name(name)?;
        let mut buf = Vec::with_capacity(f.size() as usize);
        std::io::copy(&mut f, &mut buf)?;
        Ok(buf)
    }
}

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
    let meta_val = serde_json::to_value(meta).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    zw.add_json("snapshot/meta.json", &meta_val)?;
    // 写每个分片
    for i in 0..meta.num_shards {
        let raw = get_shard_bytes(i)?;
        let zst = zstd::stream::encode_all(&raw[..], zstd_level)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let name = format!("snapshot/shard-{:03}.bin.zst", i);
        zw.add_stored(&name, &zst)?;
    }
    Ok(())
}

/// 读取分片快照：返回元数据和解压后的每个分片字节
pub fn read_snapshot_shards<R: Read + Seek>(
    zr: &mut ZipDocumentReader<R>,
) -> io::Result<(SnapshotShardMeta, Vec<Vec<u8>>)> {
    let meta_bytes = zr.read_all("snapshot/meta.json")?;
    let meta: SnapshotShardMeta = serde_json::from_slice(&meta_bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut shards: Vec<Vec<u8>> = Vec::with_capacity(meta.num_shards);
    for i in 0..meta.num_shards {
        let name = format!("snapshot/shard-{:03}.bin.zst", i);
        let zst = zr.read_all(&name)?;
        let raw = zstd::stream::decode_all(&zst[..])
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        shards.push(raw);
    }
    Ok((meta, shards))
}

/// 读取并用 bincode 反序列化每个分片
pub fn read_and_decode_snapshot_shards<R: Read + Seek, T: DeserializeOwned>(
    zr: &mut ZipDocumentReader<R>,
) -> io::Result<(SnapshotShardMeta, Vec<T>)> {
    let (meta, shards_raw) = read_snapshot_shards(zr)?;
    let mut out: Vec<T> = Vec::with_capacity(shards_raw.len());
    for raw in shards_raw.iter() {
        let (val, _): (T, _) = bincode::serde::decode_from_slice(raw, bincode::config::standard())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        out.push(val);
    }
    Ok((meta, out))
}

/// JSON 变体：用 serde_json 反序列化每个分片（对含有 JSON Value 的结构更稳健）
pub fn read_and_decode_snapshot_shards_json<R: Read + Seek, T: DeserializeOwned>(
    zr: &mut ZipDocumentReader<R>,
) -> io::Result<(SnapshotShardMeta, Vec<T>)> {
    let (meta, shards_raw) = read_snapshot_shards(zr)?;
    let mut out: Vec<T> = Vec::with_capacity(shards_raw.len());
    for raw in shards_raw.iter() {
        let val: T = serde_json::from_slice(raw)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        out.push(val);
    }
    Ok((meta, out))
}

/// JSON 变体：将每个分片用 serde_json 序列化后 zstd 压缩写入
pub fn write_snapshot_shards_json<W, F, T>(
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
    let meta_val = serde_json::to_value(meta).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    zw.add_json("snapshot/meta.json", &meta_val)?;
    for i in 0..meta.num_shards {
        let v = get_shard_value(i)?;
        let bytes = serde_json::to_vec(&v).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let zst = zstd::stream::encode_all(&bytes[..], zstd_level)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let name = format!("snapshot/shard-{:03}.bin.zst", i);
        zw.add_stored(&name, &zst)?;
    }
    Ok(())
}

/// MessagePack 变体：用 rmp-serde 序列化每个分片后 zstd 压缩
pub fn write_snapshot_shards_msgpack<W, F, T>(
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
    let meta_val = serde_json::to_value(meta).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    zw.add_json("snapshot/meta.json", &meta_val)?;
    for i in 0..meta.num_shards {
        let v = get_shard_value(i)?;
        let bytes = rmp_serde::to_vec(&v).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let zst = zstd::stream::encode_all(&bytes[..], zstd_level)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let name = format!("snapshot/shard-{:03}.bin.zst", i);
        zw.add_stored(&name, &zst)?;
    }
    Ok(())
}

/// CBOR 变体：用 serde_cbor 序列化每个分片后 zstd 压缩
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
    let meta_val = serde_json::to_value(meta).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    zw.add_json("snapshot/meta.json", &meta_val)?;
    for i in 0..meta.num_shards {
        let v = get_shard_value(i)?;
        let bytes = serde_cbor::to_vec(&v).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let zst = zstd::stream::encode_all(&bytes[..], zstd_level)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let name = format!("snapshot/shard-{:03}.bin.zst", i);
        zw.add_stored(&name, &zst)?;
    }
    Ok(())
}

/// 读取并用 MessagePack 反序列化每个分片
pub fn read_and_decode_snapshot_shards_msgpack<R: Read + Seek, T: DeserializeOwned>(
    zr: &mut ZipDocumentReader<R>,
) -> io::Result<(SnapshotShardMeta, Vec<T>)> {
    let (meta, shards_raw) = read_snapshot_shards(zr)?;
    let mut out: Vec<T> = Vec::with_capacity(shards_raw.len());
    for raw in shards_raw.iter() {
        let val: T = rmp_serde::from_slice(raw)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        out.push(val);
    }
    Ok((meta, out))
}

/// 读取并用 CBOR 反序列化每个分片
pub fn read_and_decode_snapshot_shards_cbor<R: Read + Seek, T: DeserializeOwned>(
    zr: &mut ZipDocumentReader<R>,
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
    path: P,
) -> io::Result<(serde_json::Value, Vec<u8>, SnapshotShardMeta, Vec<T>)>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let file = std::fs::File::open(path)?;
    let mut zr = ZipDocumentReader::new(file)?;
    let meta_json = zr.read_all("meta.json")?;
    let meta_val: serde_json::Value = serde_json::from_slice(&meta_json)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let schema_xml = zr.read_all("schema.xml")?;
    let (shard_meta, decoded) = read_and_decode_snapshot_shards::<_, T>(&mut zr)?;
    Ok((meta_val, schema_xml, shard_meta, decoded))
}


