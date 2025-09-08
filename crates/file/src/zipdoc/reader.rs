use std::io::{self, Read, Seek};
use zip::ZipArchive;

// 基于 ZIP 的文档读取器
pub struct ZipDocumentReader<R: Read + Seek> {
    pub(crate) zip: ZipArchive<R>,
}

impl<R: Read + Seek> ZipDocumentReader<R> {
    // 打开读取器
    pub fn new(r: R) -> io::Result<Self> {
        Ok(Self { zip: ZipArchive::new(r)? })
    }
    // 读取指定文件完整内容
    pub fn read_all(
        &mut self,
        name: &str,
    ) -> io::Result<Vec<u8>> {
        let mut f = self.zip.by_name(name)?;
        let mut buf = Vec::with_capacity(f.size() as usize);
        std::io::copy(&mut f, &mut buf)?;
        Ok(buf)
    }

    // 读取指定插件状态（二进制数据）
    pub fn read_plugin_state(
        &mut self,
        plugin_name: &str,
    ) -> io::Result<Option<Vec<u8>>> {
        let plugin_file_path = format!("plugins/{}", plugin_name);
        match self.read_all(&plugin_file_path) {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    // 读取所有插件状态
    pub fn read_all_plugin_states(&mut self) -> io::Result<std::collections::HashMap<String, Vec<u8>>> {
        let mut plugin_states = std::collections::HashMap::new();
        
        // 先收集所有插件文件名
        let mut plugin_files = Vec::new();
        for i in 0..self.zip.len() {
            let file = self.zip.by_index(i)?;
            let file_name = file.name().to_string();
            
            if file_name.starts_with("plugins/") && !file_name.ends_with('/') {
                let plugin_name = file_name
                    .strip_prefix("plugins/")
                    .unwrap()
                    .to_string();
                plugin_files.push((plugin_name, file_name));
            }
        }
        
        // 然后读取每个插件文件的数据
        for (plugin_name, file_name) in plugin_files {
            let data = self.read_all(&file_name)?;
            plugin_states.insert(plugin_name, data);
        }
        
        Ok(plugin_states)
    }

    // 列出所有插件名称
    pub fn list_plugins(&mut self) -> io::Result<Vec<String>> {
        let mut plugins = Vec::new();
        
        for i in 0..self.zip.len() {
            let file = self.zip.by_index(i)?;
            let file_name = file.name();
            
            if file_name.starts_with("plugins/") && !file_name.ends_with('/') {
                let plugin_name = file_name
                    .strip_prefix("plugins/")
                    .unwrap()
                    .to_string();
                plugins.push(plugin_name);
            }
        }
        
        Ok(plugins)
    }

    // 检查是否存在插件状态
    pub fn has_plugin_state(&mut self, plugin_name: &str) -> bool {
        let plugin_file_path = format!("plugins/{}", plugin_name);
        self.zip.by_name(&plugin_file_path).is_ok()
    }
}
