use std::io::{self, Read, Seek, Write, BufWriter};
use std::collections::HashMap;
use std::path::PathBuf;
use zip::ZipArchive;
use memmap2::{Mmap, MmapOptions};
use tempfile::NamedTempFile;

/// memmap2 优化配置
#[derive(Debug, Clone)]
pub struct MmapConfig {
    /// 使用 memmap 的最小文件大小阈值 (默认: 1MB)
    pub threshold: u64,
    /// 最大并发 mmap 映射数量 (默认: 8)
    pub max_maps: usize,
    /// 临时文件目录 (默认: 系统临时目录)
    pub temp_dir: Option<PathBuf>,
    /// 超大文件阈值 (默认: 100MB) - 超过此大小使用流式处理
    pub huge_file_threshold: u64,
    /// 流式读取块大小 (默认: 8MB)
    pub stream_chunk_size: usize,
    /// 是否启用流式处理 (默认: true)
    pub enable_streaming: bool,
}

impl Default for MmapConfig {
    fn default() -> Self {
        Self {
            threshold: 1024 * 1024, // 1MB
            max_maps: 8,
            temp_dir: None,
            huge_file_threshold: 100 * 1024 * 1024, // 100MB
            stream_chunk_size: 8 * 1024 * 1024,     // 8MB
            enable_streaming: true,
        }
    }
}

/// 内存映射条目
struct MmapEntry {
    _temp_file: NamedTempFile,
    mmap: Mmap,
}

/// 文件大小类别
#[derive(Debug, Clone, PartialEq)]
pub enum FileSizeCategory {
    /// 小文件（< threshold）
    Small,
    /// 大文件（>= threshold, < huge_file_threshold）
    Large,
    /// 超大文件（>= huge_file_threshold）
    Huge,
}

/// 推荐的处理策略
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingStrategy {
    /// 标准读取（直接内存加载）
    Standard,
    /// 内存映射（mmap）
    MemoryMap,
    /// 流式处理（分块处理）
    Streaming,
}

/// 文件详细信息
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// 文件名
    pub name: String,
    /// 原始大小（字节）
    pub size: u64,
    /// 压缩后大小（字节）
    pub compressed_size: u64,
    /// 压缩比率（压缩后大小/原始大小）
    pub compression_ratio: f64,
    /// 文件大小类别
    pub category: FileSizeCategory,
    /// 推荐的处理策略
    pub recommended_strategy: ProcessingStrategy,
}

// 基于 ZIP 的文档读取器，集成 memmap2 优化
pub struct ZipDocumentReader<R: Read + Seek> {
    pub(crate) zip: ZipArchive<R>,
    mmap_config: MmapConfig,
    mmap_cache: HashMap<String, MmapEntry>,
    access_count: HashMap<String, u64>,
}

impl<R: Read + Seek> ZipDocumentReader<R> {
    // 打开读取器
    pub fn new(r: R) -> io::Result<Self> {
        Self::with_mmap_config(r, MmapConfig::default())
    }

    // 使用指定配置打开读取器
    pub fn with_mmap_config(
        r: R,
        config: MmapConfig,
    ) -> io::Result<Self> {
        Ok(Self {
            zip: ZipArchive::new(r)?,
            mmap_config: config,
            mmap_cache: HashMap::new(),
            access_count: HashMap::new(),
        })
    }
    // 读取指定文件完整内容，自动选择最优策略
    pub fn read_all(
        &mut self,
        name: &str,
    ) -> io::Result<Vec<u8>> {
        // 使用智能读取策略
        self.read_smart(name)
    }

    // 智能读取：基于文件信息自动选择最优策略
    pub fn read_smart(
        &mut self,
        name: &str,
    ) -> io::Result<Vec<u8>> {
        let file_info = self.get_file_info(name)?;

        match file_info.recommended_strategy {
            ProcessingStrategy::Standard => {
                // 小文件：标准读取
                *self.access_count.entry(name.to_string()).or_insert(0) += 1;
                self.read_standard(name)
            },
            ProcessingStrategy::MemoryMap => {
                // 中等文件：优先使用 mmap，失败时回退
                match self.read_mmap(name) {
                    Ok(data) => Ok(data.to_vec()),
                    Err(_) => {
                        // mmap 失败，回退到标准读取
                        *self
                            .access_count
                            .entry(name.to_string())
                            .or_insert(0) += 1;
                        self.read_standard(name)
                    },
                }
            },
            ProcessingStrategy::Streaming => {
                // 超大文件：流式读取（如果启用）
                if self.mmap_config.enable_streaming {
                    *self.access_count.entry(name.to_string()).or_insert(0) +=
                        1;
                    self.read_huge_file_streaming(name)
                } else {
                    // 流式处理未启用，尝试 mmap 或标准读取
                    match self.read_mmap(name) {
                        Ok(data) => Ok(data.to_vec()),
                        Err(_) => {
                            *self
                                .access_count
                                .entry(name.to_string())
                                .or_insert(0) += 1;
                            self.read_standard(name)
                        },
                    }
                }
            },
        }
    }

    // 强制使用 mmap 读取（返回引用，零拷贝）
    pub fn read_mmap(
        &mut self,
        name: &str,
    ) -> io::Result<&[u8]> {
        // 检查缓存
        if self.mmap_cache.contains_key(name) {
            // 更新访问计数
            *self.access_count.entry(name.to_string()).or_insert(0) += 1;
            return Ok(&self.mmap_cache[name].mmap[..]);
        }

        // 检查缓存容量
        if self.mmap_cache.len() >= self.mmap_config.max_maps {
            self.evict_least_used();
        }

        // 创建新的 mmap 条目
        self.create_mmap_entry(name)?;

        // 初始化访问计数
        self.access_count.insert(name.to_string(), 1);

        Ok(&self.mmap_cache[name].mmap[..])
    }

    // 标准内存读取
    pub fn read_standard(
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
    pub fn read_all_plugin_states(
        &mut self
    ) -> io::Result<std::collections::HashMap<String, Vec<u8>>> {
        let mut plugin_states = std::collections::HashMap::new();

        // 先收集所有插件文件名
        let mut plugin_files = Vec::new();
        for i in 0..self.zip.len() {
            let file = self.zip.by_index(i)?;
            let file_name = file.name().to_string();

            if file_name.starts_with("plugins/") && !file_name.ends_with('/') {
                let plugin_name =
                    file_name.strip_prefix("plugins/").unwrap().to_string();
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
                let plugin_name =
                    file_name.strip_prefix("plugins/").unwrap().to_string();
                plugins.push(plugin_name);
            }
        }

        Ok(plugins)
    }

    // 检查是否存在插件状态
    pub fn has_plugin_state(
        &mut self,
        plugin_name: &str,
    ) -> bool {
        let plugin_file_path = format!("plugins/{}", plugin_name);
        self.zip.by_name(&plugin_file_path).is_ok()
    }

    // 创建内存映射条目
    fn create_mmap_entry(
        &mut self,
        name: &str,
    ) -> io::Result<()> {
        // 创建临时文件
        let mut temp_file =
            if let Some(ref temp_dir) = self.mmap_config.temp_dir {
                NamedTempFile::new_in(temp_dir)?
            } else {
                NamedTempFile::new()?
            };

        // 解压文件到临时文件
        {
            let mut zip_file = self.zip.by_name(name)?;
            let mut writer = BufWriter::new(&mut temp_file);
            std::io::copy(&mut zip_file, &mut writer)?;
            writer.flush()?;
        }

        // 确保数据写入磁盘
        temp_file.as_file().sync_all()?;

        // 创建内存映射
        let mmap = unsafe { MmapOptions::new().map(temp_file.as_file())? };

        // 添加到缓存
        self.mmap_cache.insert(
            name.to_string(),
            MmapEntry { _temp_file: temp_file, mmap },
        );

        Ok(())
    }

    // 清理最少使用的条目
    fn evict_least_used(&mut self) {
        if let Some((lru_name, _)) = self
            .access_count
            .iter()
            .min_by_key(|(_, count)| **count)
            .map(|(name, count)| (name.clone(), *count))
        {
            self.mmap_cache.remove(&lru_name);
            self.access_count.remove(&lru_name);
        }
    }

    // 获取 mmap 配置
    pub fn mmap_config(&self) -> &MmapConfig {
        &self.mmap_config
    }

    // 获取缓存统计信息
    pub fn mmap_stats(&self) -> MmapStats {
        let total_size: u64 =
            self.mmap_cache.values().map(|entry| entry.mmap.len() as u64).sum();

        MmapStats {
            cached_entries: self.mmap_cache.len(),
            total_cached_size: total_size,
            max_entries: self.mmap_config.max_maps,
            threshold_bytes: self.mmap_config.threshold,
        }
    }

    // 清理所有 mmap 缓存
    pub fn clear_mmap_cache(&mut self) {
        self.mmap_cache.clear();
        self.access_count.clear();
    }

    // 获取指定文件的大小（字节）
    pub fn get_file_size(
        &mut self,
        name: &str,
    ) -> io::Result<u64> {
        let f = self.zip.by_name(name)?;
        Ok(f.size())
    }

    // 获取指定文件的压缩大小（字节）
    pub fn get_compressed_size(
        &mut self,
        name: &str,
    ) -> io::Result<u64> {
        let f = self.zip.by_name(name)?;
        Ok(f.compressed_size())
    }

    // 检查文件大小类别
    pub fn classify_file_size(
        &mut self,
        name: &str,
    ) -> io::Result<FileSizeCategory> {
        let file_size = self.get_file_size(name)?;

        if file_size >= self.mmap_config.huge_file_threshold {
            Ok(FileSizeCategory::Huge)
        } else if file_size >= self.mmap_config.threshold {
            Ok(FileSizeCategory::Large)
        } else {
            Ok(FileSizeCategory::Small)
        }
    }

    // 获取文件的详细信息
    pub fn get_file_info(
        &mut self,
        name: &str,
    ) -> io::Result<FileInfo> {
        let (size, compressed_size) = {
            let f = self.zip.by_name(name)?;
            (f.size(), f.compressed_size())
        }; // f 在这里被销毁，释放了可变借用

        let category = if size >= self.mmap_config.huge_file_threshold {
            FileSizeCategory::Huge
        } else if size >= self.mmap_config.threshold {
            FileSizeCategory::Large
        } else {
            FileSizeCategory::Small
        };

        let recommended_strategy = self.recommend_processing_strategy(size);

        Ok(FileInfo {
            name: name.to_string(),
            size,
            compressed_size,
            compression_ratio: if size > 0 {
                compressed_size as f64 / size as f64
            } else {
                1.0
            },
            category,
            recommended_strategy,
        })
    }

    // 推荐处理策略
    pub fn recommend_processing_strategy(
        &self,
        file_size: u64,
    ) -> ProcessingStrategy {
        if file_size >= self.mmap_config.huge_file_threshold
            && self.mmap_config.enable_streaming
        {
            ProcessingStrategy::Streaming
        } else if file_size >= self.mmap_config.threshold {
            ProcessingStrategy::MemoryMap
        } else {
            ProcessingStrategy::Standard
        }
    }

    // 预热缓存：为指定文件创建 mmap
    pub fn preheat_mmap(
        &mut self,
        names: &[&str],
    ) -> io::Result<()> {
        for &name in names {
            if !self.mmap_cache.contains_key(name) {
                let file_size = self.get_file_size(name)?;

                if file_size >= self.mmap_config.threshold {
                    self.create_mmap_entry(name)?;
                }
            }
        }
        Ok(())
    }

    // 流式读取超大文件
    fn read_huge_file_streaming(
        &mut self,
        name: &str,
    ) -> io::Result<Vec<u8>> {
        let mut file = self.zip.by_name(name)?;
        let total_size = file.size() as usize;
        let mut result = Vec::with_capacity(total_size);

        let chunk_size = self.mmap_config.stream_chunk_size;
        let mut buffer = vec![0u8; chunk_size];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            result.extend_from_slice(&buffer[..bytes_read]);
        }

        Ok(result)
    }

    // 创建流式读取器（用于逐块处理）
    pub fn create_stream_reader(
        &mut self,
        name: &str,
    ) -> io::Result<ZipStreamReader> {
        // 一次性读取并分块存储，避免重复跳跃的性能问题
        let mut file = self.zip.by_name(name)?;
        let total_size = file.size();
        let chunk_size = self.mmap_config.stream_chunk_size;

        let mut chunks = Vec::new();
        let mut buffer = vec![0u8; chunk_size];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            chunks.push(buffer[..bytes_read].to_vec());
        }

        Ok(ZipStreamReader {
            chunks: chunks.into_iter(),
            total_size,
            current_pos: 0,
        })
    }

    // 智能处理：根据文件大小自动选择回调或直接返回策略
    pub fn process_smart<F>(
        &mut self,
        name: &str,
        mut processor: F,
    ) -> io::Result<()>
    where
        F: FnMut(&[u8]) -> io::Result<()>,
    {
        let file_info = self.get_file_info(name)?;

        match file_info.recommended_strategy {
            ProcessingStrategy::Standard => {
                // 小文件：直接读取后一次性回调
                *self.access_count.entry(name.to_string()).or_insert(0) += 1;
                let data = self.read_standard(name)?;
                processor(&data)
            },
            ProcessingStrategy::MemoryMap => {
                // 中等文件：尝试 mmap 零拷贝，失败时回退
                match self.read_mmap(name) {
                    Ok(data) => {
                        // 零拷贝：直接传递 mmap 的引用
                        processor(data)
                    },
                    Err(_) => {
                        // mmap 失败，回退到标准读取
                        *self
                            .access_count
                            .entry(name.to_string())
                            .or_insert(0) += 1;
                        let data = self.read_standard(name)?;
                        processor(&data)
                    },
                }
            },
            ProcessingStrategy::Streaming => {
                // 超大文件：强制使用流式回调处理
                if self.mmap_config.enable_streaming {
                    *self.access_count.entry(name.to_string()).or_insert(0) +=
                        1;
                    self.process_huge_file(name, processor)
                } else {
                    // 流式处理未启用，尝试 mmap 或回退
                    match self.read_mmap(name) {
                        Ok(data) => processor(data),
                        Err(_) => {
                            *self
                                .access_count
                                .entry(name.to_string())
                                .or_insert(0) += 1;
                            let data = self.read_standard(name)?;
                            processor(&data)
                        },
                    }
                }
            },
        }
    }

    // 智能批量处理：自动判断是否需要流式处理多个文件
    pub fn process_files_smart<F>(
        &mut self,
        file_names: &[&str],
        mut processor: F,
    ) -> io::Result<()>
    where
        F: FnMut(&str, &[u8]) -> io::Result<()>,
    {
        for &name in file_names {
            let file_info = self.get_file_info(name)?;

            // 根据策略决定处理方式
            match file_info.recommended_strategy {
                ProcessingStrategy::Standard
                | ProcessingStrategy::MemoryMap => {
                    // 小文件和中等文件：一次性处理
                    let data = self.read_smart(name)?;
                    processor(name, &data)?;
                },
                ProcessingStrategy::Streaming => {
                    // 超大文件：流式处理，累积数据
                    let mut accumulated_data = Vec::new();
                    self.process_smart(name, |chunk| {
                        accumulated_data.extend_from_slice(chunk);
                        Ok(())
                    })?;
                    processor(name, &accumulated_data)?;
                },
            }
        }

        Ok(())
    }

    // 处理超大文件的回调方式（避免内存占用）
    pub fn process_huge_file<F>(
        &mut self,
        name: &str,
        mut processor: F,
    ) -> io::Result<()>
    where
        F: FnMut(&[u8]) -> io::Result<()>,
    {
        let mut file = self.zip.by_name(name)?;
        let chunk_size = self.mmap_config.stream_chunk_size;
        let mut buffer = vec![0u8; chunk_size];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            processor(&buffer[..bytes_read])?;
        }

        Ok(())
    }
}

/// mmap 缓存统计信息
#[derive(Debug, Clone)]
pub struct MmapStats {
    /// 缓存的条目数量
    pub cached_entries: usize,
    /// 缓存的总大小（字节）
    pub total_cached_size: u64,
    /// 最大条目数量
    pub max_entries: usize,
    /// 使用 mmap 的阈值
    pub threshold_bytes: u64,
}

impl std::fmt::Display for MmapStats {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(
            f,
            "mmap 缓存: {}/{} 条目, {:.2} MB 总大小, 阈值 {:.2} MB",
            self.cached_entries,
            self.max_entries,
            self.total_cached_size as f64 / (1024.0 * 1024.0),
            self.threshold_bytes as f64 / (1024.0 * 1024.0)
        )
    }
}

/// 流式读取器，用于处理超大文件
/// 这是一个基于预读取的高效实现，避免了重复跳跃的性能问题
pub struct ZipStreamReader {
    chunks: std::vec::IntoIter<Vec<u8>>,
    total_size: u64,
    current_pos: u64,
}

impl ZipStreamReader {
    /// 读取下一个数据块
    pub fn read_chunk(&mut self) -> io::Result<Option<Vec<u8>>> {
        if let Some(chunk) = self.chunks.next() {
            self.current_pos += chunk.len() as u64;
            Ok(Some(chunk))
        } else {
            Ok(None)
        }
    }

    /// 获取总大小
    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    /// 获取当前位置
    pub fn position(&self) -> u64 {
        self.current_pos
    }

    /// 是否已完成读取
    pub fn is_finished(&self) -> bool {
        self.current_pos >= self.total_size
    }

    /// 重置到开头
    pub fn reset(&mut self) {
        self.current_pos = 0;
    }

    /// 逐块处理数据
    pub fn process_chunks<F>(
        &mut self,
        mut processor: F,
    ) -> io::Result<()>
    where
        F: FnMut(&[u8]) -> io::Result<()>,
    {
        while let Some(chunk) = self.read_chunk()? {
            processor(&chunk)?;
        }
        Ok(())
    }

    /// 流式读取所有数据到 Vec（适用于需要完整数据的场景）
    pub fn read_all_streaming(&mut self) -> io::Result<Vec<u8>> {
        let mut result = Vec::with_capacity(self.total_size as usize);

        while let Some(chunk) = self.read_chunk()? {
            result.extend_from_slice(&chunk);
        }

        Ok(result)
    }

    /// 计算数据的哈希值（无需加载到内存）
    pub fn compute_hash<H>(
        &mut self,
        mut hasher: H,
    ) -> io::Result<()>
    where
        H: FnMut(&[u8]),
    {
        self.reset();
        while let Some(chunk) = self.read_chunk()? {
            hasher(&chunk);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use crate::zipdoc::ZipDocumentWriter;

    #[test]
    fn test_mmap_integration_basic() -> io::Result<()> {
        // 创建测试 ZIP
        let mut zip_data = Vec::new();
        {
            let cursor = Cursor::new(&mut zip_data);
            let mut writer = ZipDocumentWriter::new(cursor)?;

            // 添加小文件（不会使用 mmap）
            writer.add_stored("small.txt", b"small content")?;

            // 添加大文件（会使用 mmap）
            let large_content = vec![42u8; 2 * 1024 * 1024]; // 2MB
            writer.add_stored("large.bin", &large_content)?;

            writer.finalize()?;
        }

        // 测试 memmap2 集成
        let cursor = Cursor::new(zip_data);
        let mut reader = ZipDocumentReader::new(cursor)?;

        // 读取小文件（应该走标准路径）
        let small_data = reader.read_all("small.txt")?;
        assert_eq!(small_data, b"small content");

        // 检查没有 mmap 缓存
        let stats = reader.mmap_stats();
        assert_eq!(stats.cached_entries, 0);

        // 读取大文件（应该使用 mmap）
        let large_data = reader.read_all("large.bin")?;
        assert_eq!(large_data.len(), 2 * 1024 * 1024);
        assert!(large_data.iter().all(|&b| b == 42));

        // 检查 mmap 缓存
        let stats = reader.mmap_stats();
        assert_eq!(stats.cached_entries, 1);
        assert_eq!(stats.total_cached_size, 2 * 1024 * 1024);

        // 再次读取大文件（应该命中缓存）
        let large_data2 = reader.read_all("large.bin")?;
        assert_eq!(large_data2, large_data);

        // 缓存应该保持不变
        let stats = reader.mmap_stats();
        assert_eq!(stats.cached_entries, 1);

        Ok(())
    }

    #[test]
    fn test_mmap_zero_copy_read() -> io::Result<()> {
        let mut zip_data = Vec::new();
        {
            let cursor = Cursor::new(&mut zip_data);
            let mut writer = ZipDocumentWriter::new(cursor)?;

            let test_data = vec![123u8; 3 * 1024 * 1024]; // 3MB
            writer.add_stored("test.bin", &test_data)?;
            writer.finalize()?;
        }

        let cursor = Cursor::new(zip_data);
        let mut reader = ZipDocumentReader::new(cursor)?;

        // 使用零拷贝读取
        let mmap_data = reader.read_mmap("test.bin")?;
        assert_eq!(mmap_data.len(), 3 * 1024 * 1024);
        assert!(mmap_data.iter().all(|&b| b == 123));

        // 验证数据是内存映射的，不是拷贝的
        let stats = reader.mmap_stats();
        assert_eq!(stats.cached_entries, 1);
        assert_eq!(stats.total_cached_size, 3 * 1024 * 1024);

        Ok(())
    }

    #[test]
    fn test_mmap_cache_eviction() -> io::Result<()> {
        let config = MmapConfig {
            threshold: 1024, // 1KB
            max_maps: 2,     // 最多2个
            temp_dir: None,
            huge_file_threshold: 100 * 1024 * 1024,
            stream_chunk_size: 8 * 1024 * 1024,
            enable_streaming: true,
        };

        let mut zip_data = Vec::new();
        {
            let cursor = Cursor::new(&mut zip_data);
            let mut writer = ZipDocumentWriter::new(cursor)?;

            // 添加3个大文件
            for i in 1..=3 {
                let content = vec![i as u8; 2048]; // 2KB each
                writer.add_stored(&format!("file{}.bin", i), &content)?;
            }

            writer.finalize()?;
        }

        let cursor = Cursor::new(zip_data);
        let mut reader = ZipDocumentReader::with_mmap_config(cursor, config)?;

        // 读取前两个文件
        let _data1 = reader.read_all("file1.bin")?;
        let _data2 = reader.read_all("file2.bin")?;

        assert_eq!(reader.mmap_stats().cached_entries, 2);

        // 读取第三个文件，应该触发缓存清理
        let _data3 = reader.read_all("file3.bin")?;

        // 仍然应该只有2个缓存条目
        assert_eq!(reader.mmap_stats().cached_entries, 2);

        Ok(())
    }

    #[test]
    fn test_mmap_config_threshold() -> io::Result<()> {
        let config = MmapConfig {
            threshold: 5 * 1024 * 1024, // 5MB 阈值
            max_maps: 8,
            temp_dir: None,
            huge_file_threshold: 100 * 1024 * 1024,
            stream_chunk_size: 8 * 1024 * 1024,
            enable_streaming: true,
        };

        let mut zip_data = Vec::new();
        {
            let cursor = Cursor::new(&mut zip_data);
            let mut writer = ZipDocumentWriter::new(cursor)?;

            // 添加一个小于阈值的文件
            let small_content = vec![1u8; 1024 * 1024]; // 1MB
            writer.add_stored("small.bin", &small_content)?;

            // 添加一个大于阈值的文件
            let large_content = vec![2u8; 6 * 1024 * 1024]; // 6MB
            writer.add_stored("large.bin", &large_content)?;

            writer.finalize()?;
        }

        let cursor = Cursor::new(zip_data);
        let mut reader = ZipDocumentReader::with_mmap_config(cursor, config)?;

        // 读取小文件，不应该使用 mmap
        let _small_data = reader.read_all("small.bin")?;
        assert_eq!(reader.mmap_stats().cached_entries, 0);

        // 读取大文件，应该使用 mmap
        let _large_data = reader.read_all("large.bin")?;
        assert_eq!(reader.mmap_stats().cached_entries, 1);

        Ok(())
    }

    #[test]
    fn test_mmap_preheat() -> io::Result<()> {
        let mut zip_data = Vec::new();
        {
            let cursor = Cursor::new(&mut zip_data);
            let mut writer = ZipDocumentWriter::new(cursor)?;

            for i in 1..=3 {
                let content = vec![i as u8; 2 * 1024 * 1024]; // 2MB each
                writer.add_stored(&format!("data{}.bin", i), &content)?;
            }

            writer.finalize()?;
        }

        let cursor = Cursor::new(zip_data);
        let mut reader = ZipDocumentReader::new(cursor)?;

        // 预热缓存
        reader.preheat_mmap(&["data1.bin", "data2.bin"])?;

        // 应该有2个缓存条目
        let stats = reader.mmap_stats();
        assert_eq!(stats.cached_entries, 2);

        // 后续读取应该直接命中缓存
        let _data1 = reader.read_mmap("data1.bin")?;
        let _data2 = reader.read_mmap("data2.bin")?;

        // 缓存条目数量应该保持不变
        let stats = reader.mmap_stats();
        assert_eq!(stats.cached_entries, 2);

        Ok(())
    }

    #[test]
    fn test_mmap_stats_display() {
        let stats = MmapStats {
            cached_entries: 3,
            total_cached_size: 5 * 1024 * 1024, // 5MB
            max_entries: 8,
            threshold_bytes: 1024 * 1024, // 1MB
        };

        let display = format!("{}", stats);
        assert!(display.contains("3/8 条目"));
        assert!(display.contains("5.00 MB"));
        assert!(display.contains("1.00 MB"));
    }
}
