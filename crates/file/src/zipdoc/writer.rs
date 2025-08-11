use std::io::{self, Write, Seek};
use zip::{ZipWriter, write::SimpleFileOptions, CompressionMethod};

// 基于 ZIP 的文档写入器（docx 风格容器）
pub struct ZipDocumentWriter<W: Write + Seek> {
    pub(crate) zip: ZipWriter<W>,
    pub(crate) manifest: serde_json::Value,
}

impl<W: Write + Seek> ZipDocumentWriter<W> {
    // 创建写入器
    pub fn new(w: W) -> io::Result<Self> {
        let zip = ZipWriter::new(w);
        let manifest = serde_json::json!({ "version": 1, "entries": [] });
        Ok(Self { zip, manifest })
    }
    // 读取当前 manifest 的不可变引用
    pub fn manifest(&self) -> &serde_json::Value { &self.manifest }
    // 读取当前 manifest 的可变引用（可自由添加自定义字段）
    pub fn manifest_mut(&mut self) -> &mut serde_json::Value { &mut self.manifest }
    // 替换 manifest（链式调用）
    pub fn set_manifest(&mut self, manifest: serde_json::Value) -> &mut Self { self.manifest = manifest; self }
    // 写入 JSON 文件（deflate 压缩）
    pub fn add_json(&mut self, name: &str, value: &serde_json::Value) -> io::Result<()> {
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        self.zip.start_file(name, opts)?;
        let data = serde_json::to_vec(value).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        // 记录到 manifest.entries（若存在且为数组）
        if let Some(entries) = self.manifest.get_mut("entries").and_then(|v| v.as_array_mut()) {
            entries.push(serde_json::json!({
                "name": name,
                "kind": "json",
                "logical_len": data.len(),
                "compression": "deflate"
            }));
        }
        self.zip.write_all(&data)
    }
    // 写入原样存储的条目（不压缩）
    pub fn add_stored(&mut self, name: &str, bytes: &[u8]) -> io::Result<()> {
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        self.zip.start_file(name, opts)?;
        if let Some(entries) = self.manifest.get_mut("entries").and_then(|v| v.as_array_mut()) {
            entries.push(serde_json::json!({
                "name": name,
                "kind": "binary",
                "logical_len": bytes.len(),
                "compression": "stored"
            }));
        }
        self.zip.write_all(bytes)
    }
    // 写入 deflate 压缩条目
    pub fn add_deflated(&mut self, name: &str, bytes: &[u8]) -> io::Result<()> {
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        self.zip.start_file(name, opts)?;
        if let Some(entries) = self.manifest.get_mut("entries").and_then(|v| v.as_array_mut()) {
            entries.push(serde_json::json!({
                "name": name,
                "kind": "binary",
                "logical_len": bytes.len(),
                "compression": "deflate"
            }));
        }
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


