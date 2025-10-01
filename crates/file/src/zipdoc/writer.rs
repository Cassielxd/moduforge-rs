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
    pub fn manifest(&self) -> &serde_json::Value {
        &self.manifest
    }
    // 读取当前 manifest 的可变引用（可自由添加自定义字段）
    pub fn manifest_mut(&mut self) -> &mut serde_json::Value {
        &mut self.manifest
    }
    // 替换 manifest（链式调用）
    pub fn set_manifest(
        &mut self,
        manifest: serde_json::Value,
    ) -> &mut Self {
        self.manifest = manifest;
        self
    }
    // 写入 JSON 文件（deflate 压缩）
    pub fn add_json(
        &mut self,
        name: &str,
        value: &serde_json::Value,
    ) -> io::Result<()> {
        let opts = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated);
        self.zip.start_file(name, opts)?;
        let data = serde_json::to_vec(value)
            .map_err(io::Error::other)?;
        // 记录到 manifest.entries（若存在且为数组）
        if let Some(entries) =
            self.manifest.get_mut("entries").and_then(|v| v.as_array_mut())
        {
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
    pub fn add_stored(
        &mut self,
        name: &str,
        bytes: &[u8],
    ) -> io::Result<()> {
        let opts = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored);
        self.zip.start_file(name, opts)?;
        if let Some(entries) =
            self.manifest.get_mut("entries").and_then(|v| v.as_array_mut())
        {
            entries.push(serde_json::json!({
                "name": name,
                "kind": "binary",
                "logical_len": bytes.len(),
                "compression": "stored"
            }));
        }
        self.zip.write_all(bytes)
    }

    // 添加插件状态目录和文件（二进制存储）
    pub fn add_plugin_state(
        &mut self,
        plugin_name: &str,
        state_data: &[u8],
    ) -> io::Result<()> {
        let plugin_file_path = format!("plugins/{plugin_name}");
        let opts = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated);
        self.zip.start_file(&plugin_file_path, opts)?;

        if let Some(entries) =
            self.manifest.get_mut("entries").and_then(|v| v.as_array_mut())
        {
            entries.push(serde_json::json!({
                "name": plugin_file_path,
                "kind": "plugin_state",
                "plugin": plugin_name,
                "logical_len": state_data.len(),
                "compression": "deflate"
            }));
        }
        self.zip.write_all(state_data)
    }

    // 批量添加插件状态
    pub fn add_plugin_states<I>(
        &mut self,
        plugin_states: I,
    ) -> io::Result<()>
    where
        I: IntoIterator<Item = (String, Vec<u8>)>,
    {
        for (plugin_name, state_data) in plugin_states {
            self.add_plugin_state(&plugin_name, &state_data)?;
        }
        Ok(())
    }
    // 写入 deflate 压缩条目
    pub fn add_deflated(
        &mut self,
        name: &str,
        bytes: &[u8],
    ) -> io::Result<()> {
        let opts = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated);
        self.zip.start_file(name, opts)?;
        if let Some(entries) =
            self.manifest.get_mut("entries").and_then(|v| v.as_array_mut())
        {
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
        let opts = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated);
        self.zip.start_file("manifest.json", opts)?;
        let data = serde_json::to_vec(&self.manifest)
            .map_err(io::Error::other)?;
        self.zip.write_all(&data)?;
        self.zip.finish().map_err(io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_plugin_state_export() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = ZipDocumentWriter::new(cursor).unwrap();

        // 添加插件状态
        writer.add_plugin_state("test_plugin", b"test state data").unwrap();
        writer
            .add_plugin_state("another_plugin", b"another state data")
            .unwrap();

        let result = writer.finalize().unwrap();
        let final_data = result.into_inner();

        // 验证数据已写入
        assert!(!final_data.is_empty());

        // 验证 manifest 包含插件条目
        let cursor = Cursor::new(&final_data);
        let mut reader = crate::zipdoc::ZipDocumentReader::new(cursor).unwrap();

        let plugins = reader.list_plugins().unwrap();
        assert_eq!(plugins.len(), 2);
        assert!(plugins.contains(&"test_plugin".to_string()));
        assert!(plugins.contains(&"another_plugin".to_string()));

        // 验证插件状态可以读取
        let test_state =
            reader.read_plugin_state("test_plugin").unwrap().unwrap();
        assert_eq!(test_state, b"test state data");
    }

    #[test]
    fn test_batch_plugin_states() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = ZipDocumentWriter::new(cursor).unwrap();

        let plugin_states = vec![
            ("plugin1".to_string(), b"state1".to_vec()),
            ("plugin2".to_string(), b"state2".to_vec()),
            ("plugin3".to_string(), b"state3".to_vec()),
        ];

        writer.add_plugin_states(plugin_states.clone()).unwrap();
        let result = writer.finalize().unwrap();
        let final_data = result.into_inner();

        let cursor = Cursor::new(&final_data);
        let mut reader = crate::zipdoc::ZipDocumentReader::new(cursor).unwrap();

        let all_states = reader.read_all_plugin_states().unwrap();
        assert_eq!(all_states.len(), 3);

        for (name, expected_data) in plugin_states {
            let actual_data = all_states.get(&name).unwrap();
            assert_eq!(actual_data, &expected_data);
        }
    }
}
