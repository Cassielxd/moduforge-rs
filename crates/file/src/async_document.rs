use crate::common::{
    encode_segment, decode_segment,
    create_tail_pointer, parse_tail_pointer, validate_tail_offset,
    validate_payload, is_zstd_compressed, has_parallel_compression,
    DIR_FLAG_ZSTD_SEGMENTS, DIR_FLAG_PARALLEL_COMPRESSION,
    DEFAULT_ZSTD_LEVEL, TAIL_MAGIC, TAIL_POINTER_SIZE, ZSTD_MAGIC_PREFIX,
};
use crate::error::{FileError, Result};
use crate::document::{Directory, SegmentEntry, SegmentType};
use crate::parallel_compression::{AsyncParallelCompressor, ParallelCompressionConfig};
use blake3::Hasher as Blake3;
use futures::stream::{Stream, StreamExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

/// 异步文档写入器，支持并行压缩
/// Async document writer with parallel compression support
pub struct AsyncDocumentWriter {
    writer: Arc<crate::async_record::AsyncWriter>,  // 底层异步记录写入器 / Underlying async record writer
    segments: Arc<Mutex<Vec<SegmentEntry>>>,        // 段条目列表 / List of segment entries
    compressor: Arc<AsyncParallelCompressor>,       // 异步并行压缩器 / Async parallel compressor
    enable_parallel: bool,                          // 是否启用并行压缩 / Whether parallel compression is enabled
    path: PathBuf,                                  // 文件路径（用于哈希计算和尾指针写入）/ File path (for hash calculation and tail pointer)
}

impl AsyncDocumentWriter {
    /// 开始写入新文档（默认启用并行压缩）
    /// Begin writing a new document (parallel compression enabled by default)
    pub async fn begin<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::begin_with_config(path, Default::default(), true).await
    }

    /// 开始写入新文档（仅使用标准压缩，与同步版本行为一致）
    /// Begin writing with standard compression (compatible with sync version)
    pub async fn begin_standard<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::begin_with_config(path, Default::default(), false).await
    }

    /// 使用自定义压缩配置开始写入
    /// Begin writing with custom compression configuration
    pub async fn begin_with_config<P: AsRef<Path>>(
        path: P,
        compression_config: ParallelCompressionConfig,
        enable_parallel: bool,
    ) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let writer = crate::async_record::AsyncWriter::create(&path_buf, 0).await?;
        let compressor = AsyncParallelCompressor::new(compression_config);

        Ok(Self {
            writer: Arc::new(writer),
            segments: Arc::new(Mutex::new(Vec::new())),
            compressor: Arc::new(compressor),
            enable_parallel,
            path: path_buf,
        })
    }

    /// 添加一个段（带异步压缩）
    /// Add a segment with async compression
    pub async fn add_segment(
        &self,
        kind: SegmentType,
        payload: Vec<u8>,
    ) -> Result<()> {
        // 验证负载不为空
        validate_payload(&payload)?;

        // 压缩数据
        // Compress data
        let compressed = if self.enable_parallel {
            // 使用并行压缩 / Use parallel compression
            self.compressor.compress(payload).await?
        } else {
            // 回退到标准压缩（使用与同步版本一致的压缩级别）
            // Fall back to standard compression (using same level as sync version)
            tokio::task::spawn_blocking(move || {
                zstd::stream::encode_all(&payload[..], DEFAULT_ZSTD_LEVEL)
            })
            .await
            .map_err(|e| FileError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e
            )))??
        };

        // 追加到文件
        // Append to file
        let offset = self.writer.append(&compressed).await?;
        let crc = crc32fast::hash(&compressed);

        // 添加到段列表
        // Add to segment list
        let mut segments = self.segments.lock().await;
        segments.push(SegmentEntry {
            kind,
            offset,
            length: (crate::record::REC_HDR as u64) + compressed.len() as u64,
            crc32: crc,
        });

        Ok(())
    }

    /// 批量并行添加多个段
    /// Add multiple segments in parallel batch
    pub async fn add_segments_batch(
        &self,
        items: Vec<(SegmentType, Vec<u8>)>,
    ) -> Result<()> {
        use futures::future::try_join_all;

        let futures = items.into_iter().map(|(kind, payload)| {
            self.add_segment(kind, payload)
        });

        try_join_all(futures).await?;
        Ok(())
    }

    /// 完成文档写入
    pub async fn finalize(self) -> Result<()> {
        // 刷新写入器
        self.writer.flush().await?;

        // 计算所有数据的哈希
        let hash = self.calculate_hash().await?;

        // 创建目录
        let segments = self.segments.lock().await.clone();
        let flags = if self.enable_parallel {
            DIR_FLAG_ZSTD_SEGMENTS | DIR_FLAG_PARALLEL_COMPRESSION
        } else {
            DIR_FLAG_ZSTD_SEGMENTS
        };

        let dir = Directory {
            entries: segments,
            flags,
            file_hash: hash,
        };

        // 序列化并追加目录
        let bytes = bincode::serde::encode_to_vec(&dir, bincode::config::standard())
            .map_err(|e| FileError::Io(std::io::Error::other(e)))?;

        let dir_off = self.writer.append(&bytes).await?;
        self.writer.flush().await?;

        // 写入尾指针，不计入逻辑长度
        // 这样 Reader 扫描逻辑结尾仍停在目录记录处，但可通过物理文件尾部快速读取目录偏移
        {
            use tokio::fs::OpenOptions;

            let mut file = OpenOptions::new()
                .write(true)
                .open(&self.path)
                .await?;

            let logical_end = self.writer.len().await;
            let tail_pointer = create_tail_pointer(dir_off);

            file.seek(tokio::io::SeekFrom::Start(logical_end)).await?;
            file.write_all(&tail_pointer).await?;
            file.sync_data().await?;
        }

        Ok(())
    }

    // 计算文件哈希（内部方法）
    async fn calculate_hash(&self) -> Result<[u8; 32]> {
        // 通过读取所有记录来计算哈希（与同步版本保持一致）
        let reader = crate::async_record::AsyncReader::open(&self.path).await?;
        let mut hasher = Blake3::new();

        // 流式读取所有记录并计算哈希
        use futures::StreamExt;
        let stream = reader.stream();
        futures::pin_mut!(stream);

        while let Some(result) = stream.next().await {
            if let Ok(data) = result {
                hasher.update(&data);
            } else {
                break;
            }
        }

        let hash = hasher.finalize();
        Ok(*hash.as_bytes())
    }
}

/// 异步文档读取器，支持并行解压
/// Async document reader with parallel decompression support
pub struct AsyncDocumentReader {
    reader: Arc<crate::async_record::AsyncReader>,  // 底层异步记录读取器 / Underlying async record reader
    dir: Directory,                                  // 文档目录 / Document directory
    compressor: Arc<AsyncParallelCompressor>,       // 异步并行压缩器 / Async parallel compressor
}

impl AsyncDocumentReader {
    /// 打开文档进行读取
    /// Open document for reading
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        // 首先尝试通过尾指针快速定位目录
        // First try to quickly locate directory through tail pointer
        let reader = crate::async_record::AsyncReader::open(&path).await?;

        // 优先通过尾指针快速定位目录偏移
        // Try to quickly locate directory offset through tail pointer
        let dir_offset = {
            use tokio::fs::File;
            use tokio::io::AsyncReadExt;
            use tokio::io::AsyncSeekExt;

            let mut file = File::open(path.as_ref()).await?;
            let file_len = file.metadata().await?.len();

            if file_len >= TAIL_POINTER_SIZE as u64 {
                // 读取尾部
                file.seek(tokio::io::SeekFrom::End(-(TAIL_POINTER_SIZE as i64))).await?;
                let mut tail = [0u8; TAIL_POINTER_SIZE];
                file.read_exact(&mut tail).await?;

                // 解析尾指针
                if let Some(off) = parse_tail_pointer(&tail) {
                    // 验证偏移量有效性
                    if validate_tail_offset(off, crate::record::HEADER_LEN as u64, file_len) {
                        Some(off)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        // 读取目录
        let dir_bytes = if let Some(offset) = dir_offset {
            // 使用尾指针快速定位
            reader.get_at(offset).await?
        } else {
            // 回退到顺序扫描（查找最后一条记录）
            use futures::StreamExt;
            let stream = reader.stream();
            futures::pin_mut!(stream);

            let mut last_record = None;
            while let Some(result) = stream.next().await {
                if let Ok(data) = result {
                    last_record = Some(data);
                }
            }

            last_record
                .ok_or_else(|| FileError::Io(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "未找到目录"
                )))?
        };

        // 反序列化目录
        let dir: Directory = bincode::serde::decode_from_slice(
            &dir_bytes,
            bincode::config::standard()
        )
        .map_err(|e| FileError::Io(std::io::Error::other(e)))?
        .0;

        // 根据标志位配置压缩器
        let compression_config = if has_parallel_compression(dir.flags) {
            Default::default()  // 启用并行压缩
        } else {
            ParallelCompressionConfig {
                parallel_threshold: usize::MAX, // 禁用并行
                ..Default::default()
            }
        };

        let compressor = AsyncParallelCompressor::new(compression_config);

        // 第二遍：创建新的读取器供实际使用
        let reader = crate::async_record::AsyncReader::open(&path).await?;

        Ok(Self {
            reader: Arc::new(reader),
            dir,
            compressor: Arc::new(compressor),
        })
    }

    /// 根据类型获取段
    pub async fn get_segment(&self, kind: &SegmentType) -> Result<Option<Vec<u8>>> {
        let entry = self.dir.entries.iter().find(|e| &e.kind == kind);

        match entry {
            Some(entry) => {
                let compressed = self.reader.get_at(entry.offset).await?;

                // 验证 CRC
                if crc32fast::hash(&compressed) != entry.crc32 {
                    return Err(FileError::CrcMismatch(entry.offset));
                }

                // 解压数据（兼容同步和异步生成的文件）
                let decompressed = if has_parallel_compression(self.dir.flags) {
                    // 使用并行解压（异步生成的文件）
                    self.compressor.decompress(compressed.to_vec()).await?
                } else {
                    // 标准解压（同步生成的文件，或异步标准模式）
                    if (self.dir.flags & DIR_FLAG_ZSTD_SEGMENTS != 0) || is_zstd_compressed(&compressed) {
                        // zstd压缩的数据
                        let compressed = compressed.to_vec();
                        tokio::task::spawn_blocking(move || {
                            zstd::stream::decode_all(&compressed[..])
                        })
                        .await
                        .map_err(|e| FileError::Io(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            e
                        )))??
                    } else {
                        // 未压缩的数据（向后兼容）
                        compressed.to_vec()
                    }
                };

                Ok(Some(decompressed))
            }
            None => Ok(None),
        }
    }

    /// 获取特定类型的所有段
    pub async fn get_segments_by_type(&self, kind: &SegmentType) -> Result<Vec<Vec<u8>>> {
        use futures::future::try_join_all;

        let entries: Vec<_> = self.dir.entries
            .iter()
            .filter(|e| &e.kind == kind)
            .cloned()
            .collect();

        let futures = entries.into_iter().map(|entry| async move {
            let compressed = self.reader.get_at(entry.offset).await?;

            if crc32fast::hash(&compressed) != entry.crc32 {
                return Err(FileError::CrcMismatch(entry.offset));
            }

            if has_parallel_compression(self.dir.flags) {
                self.compressor.decompress(compressed.to_vec()).await
            } else if (self.dir.flags & DIR_FLAG_ZSTD_SEGMENTS != 0) || is_zstd_compressed(&compressed) {
                let compressed = compressed.to_vec();
                tokio::task::spawn_blocking(move || {
                    zstd::stream::decode_all(&compressed[..])
                        .map_err(FileError::Io)
                })
                .await
                .map_err(|e| FileError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e
                )))?
            } else {
                Ok(compressed.to_vec())
            }
        });

        try_join_all(futures).await
    }

    /// Stream all segments
    pub fn stream_segments(&self) -> impl Stream<Item = Result<(SegmentType, Vec<u8>)>> + '_ {
        futures::stream::iter(self.dir.entries.clone())
            .then(move |entry| {
                let reader = self.reader.clone();
                let compressor = self.compressor.clone();
                let flags = self.dir.flags;

                async move {
                    let compressed = reader.get_at(entry.offset).await?;

                    if crc32fast::hash(&compressed) != entry.crc32 {
                        return Err(FileError::CrcMismatch(entry.offset));
                    }

                    let decompressed = if has_parallel_compression(flags) {
                        compressor.decompress(compressed.to_vec()).await?
                    } else if (flags & DIR_FLAG_ZSTD_SEGMENTS != 0) || is_zstd_compressed(&compressed) {
                        let compressed = compressed.to_vec();
                        tokio::task::spawn_blocking(move || {
                            zstd::stream::decode_all(&compressed[..])
                                .map_err(FileError::Io)
                        })
                        .await
                        .map_err(|e| FileError::Io(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            e
                        )))??
                    } else {
                        compressed.to_vec()
                    };

                    Ok((entry.kind, decompressed))
                }
            })
    }

    /// Process all segments in parallel
    pub async fn process_all_parallel<F, T>(&self, processor: F) -> Result<Vec<T>>
    where
        F: Fn(SegmentType, Vec<u8>) -> T + Send + Sync + Clone + 'static,
        T: Send + 'static,
    {
        use futures::stream::TryStreamExt;

        let segments: Vec<(SegmentType, Vec<u8>)> = self
            .stream_segments()
            .try_collect()
            .await?;

        // Process in parallel using rayon
        let results = tokio::task::spawn_blocking(move || {
            use rayon::prelude::*;

            segments
                .into_par_iter()
                .map(|(kind, data)| processor(kind, data))
                .collect()
        })
        .await
        .map_err(|e| FileError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e
        )))?;

        Ok(results)
    }

    /// Get directory info
    pub fn directory(&self) -> &Directory {
        &self.dir
    }

    /// Get all segment entries
    pub fn segments(&self) -> &[SegmentEntry] {
        &self.dir.entries
    }

    /// Get logical length of the document
    pub async fn logical_len(&self) -> u64 {
        self.reader.logical_len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_async_document() -> Result<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("async_doc.mfd");

        // Write document
        {
            let writer = AsyncDocumentWriter::begin(&path).await?;

            writer.add_segment(
                SegmentType("metadata".to_string()),
                b"test metadata".to_vec()
            ).await?;

            writer.add_segment(
                SegmentType("data".to_string()),
                vec![42u8; 10000]
            ).await?;

            writer.finalize().await?;
        }

        // Read document
        {
            let reader = AsyncDocumentReader::open(&path).await?;

            let metadata = reader.get_segment(&SegmentType("metadata".to_string()))
                .await?
                .unwrap();
            assert_eq!(metadata, b"test metadata");

            let data = reader.get_segment(&SegmentType("data".to_string()))
                .await?
                .unwrap();
            assert_eq!(data.len(), 10000);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_parallel_segments() -> Result<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("parallel_doc.mfd");

        // Write multiple segments
        {
            let writer = AsyncDocumentWriter::begin(&path).await?;

            let segments = vec![
                (SegmentType("seg1".to_string()), vec![1u8; 1000]),
                (SegmentType("seg2".to_string()), vec![2u8; 2000]),
                (SegmentType("seg3".to_string()), vec![3u8; 3000]),
            ];

            writer.add_segments_batch(segments).await?;
            writer.finalize().await?;
        }

        // Read and process in parallel
        {
            let reader = AsyncDocumentReader::open(&path).await?;

            let results = reader.process_all_parallel(|kind, data| {
                (kind, data.len())
            }).await?;

            assert_eq!(results.len(), 3);
        }

        Ok(())
    }
}