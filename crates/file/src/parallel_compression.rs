use crate::error::{FileError, Result};
use rayon::prelude::*;
use std::sync::Arc;
use zstd::stream::{encode_all, decode_all};

/// 并行压缩配置
#[derive(Debug, Clone)]
pub struct ParallelCompressionConfig {
    /// 压缩级别 (1-22, 默认: 3)
    pub level: i32,
    /// 并行处理的块大小 (默认: 4MB)
    pub chunk_size: usize,
    /// 线程数 (0 = 使用所有可用线程)
    pub num_threads: usize,
    /// 触发并行压缩的最小大小 (默认: 1MB)
    pub parallel_threshold: usize,
}

impl Default for ParallelCompressionConfig {
    fn default() -> Self {
        Self {
            level: 1,                       // 默认级别 1（与文档模块一致）
            chunk_size: 4 * 1024 * 1024,    // 4MB
            num_threads: 0,                 // 使用所有可用线程
            parallel_threshold: 1024 * 1024, // 1MB
        }
    }
}

/// 使用 rayon 的并行压缩引擎
pub struct ParallelCompressor {
    config: ParallelCompressionConfig,
}

impl ParallelCompressor {
    /// 创建新的并行压缩器
    pub fn new(config: ParallelCompressionConfig) -> Self {
        if config.num_threads > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(config.num_threads)
                .build_global()
                .ok();
        }
        Self { config }
    }

    /// 压缩数据，对大输入使用并行处理
    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < self.config.parallel_threshold {
            // 小数据：使用单线程压缩
            encode_all(data, self.config.level)
                .map_err(FileError::Io)
        } else {
            // 大数据：并行块压缩
            self.compress_parallel(data)
        }
    }

    /// 解压数据，对大输入使用并行处理
    pub fn decompress(&self, compressed: &[u8]) -> Result<Vec<u8>> {
        // 首先检查是否是我们的并行格式
        if compressed.len() >= 4 && &compressed[..4] == b"PCMP" {
            // 是我们的并行格式，使用并行逻辑解压
            self.decompress_parallel(compressed)
        } else {
            // 标准 zstd 格式，使用标准解压
            decode_all(compressed)
                .map_err(FileError::Io)
        }
    }

    /// 大数据的并行压缩（内部方法）
    fn compress_parallel(&self, data: &[u8]) -> Result<Vec<u8>> {
        let chunk_size = self.config.chunk_size;

        // Split data into chunks
        let chunks: Vec<_> = data
            .chunks(chunk_size)
            .collect();

        // Compress chunks in parallel
        let compressed_chunks: Vec<Vec<u8>> = chunks
            .par_iter()
            .map(|chunk| {
                encode_all(*chunk, self.config.level)
                    .map_err(FileError::Io)
            })
            .collect::<Result<Vec<_>>>()?;

        // Create output with metadata
        let mut output = Vec::new();

        // Write header: magic + chunk count + chunk sizes
        output.extend_from_slice(b"PCMP"); // Magic
        output.extend_from_slice(&(compressed_chunks.len() as u32).to_le_bytes());

        // Write chunk sizes
        for chunk in &compressed_chunks {
            output.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
        }

        // Write compressed chunks
        for chunk in compressed_chunks {
            output.extend(chunk);
        }

        Ok(output)
    }

    /// Parallel decompression of large data
    fn decompress_parallel(&self, compressed: &[u8]) -> Result<Vec<u8>> {
        // Check if it's our parallel format
        if compressed.len() >= 4 && &compressed[..4] == b"PCMP" {
            // It's our parallel format, process it
        } else {
            // Not parallel-compressed, try standard decompression
            return decode_all(compressed)
                .map_err(FileError::Io);
        }

        let mut pos = 4;

        // Read chunk count
        if compressed.len() < pos + 4 {
            return Err(FileError::BadHeader);
        }
        let chunk_count = u32::from_le_bytes(
            compressed[pos..pos + 4].try_into().unwrap()
        ) as usize;
        pos += 4;

        // Read chunk sizes
        let mut chunk_sizes = Vec::with_capacity(chunk_count);
        for _ in 0..chunk_count {
            if compressed.len() < pos + 4 {
                return Err(FileError::BadHeader);
            }
            let size = u32::from_le_bytes(
                compressed[pos..pos + 4].try_into().unwrap()
            ) as usize;
            chunk_sizes.push(size);
            pos += 4;
        }

        // Extract compressed chunks
        let mut compressed_chunks = Vec::with_capacity(chunk_count);
        for size in chunk_sizes {
            if compressed.len() < pos + size {
                return Err(FileError::BadHeader);
            }
            compressed_chunks.push(&compressed[pos..pos + size]);
            pos += size;
        }

        // Decompress chunks in parallel
        let decompressed_chunks: Vec<Vec<u8>> = compressed_chunks
            .par_iter()
            .map(|chunk| decode_all(*chunk).map_err(FileError::Io))
            .collect::<Result<Vec<_>>>()?;

        // Concatenate results
        let total_size: usize = decompressed_chunks.iter().map(|c| c.len()).sum();
        let mut output = Vec::with_capacity(total_size);
        for chunk in decompressed_chunks {
            output.extend(chunk);
        }

        Ok(output)
    }

    /// Compress multiple independent items in parallel
    pub fn compress_batch(&self, items: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
        items
            .par_iter()
            .map(|item| self.compress(item))
            .collect()
    }

    /// Decompress multiple independent items in parallel
    pub fn decompress_batch(&self, items: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
        items
            .par_iter()
            .map(|item| self.decompress(item))
            .collect()
    }

    /// Get compression ratio estimate for data
    pub fn estimate_compression_ratio(&self, data: &[u8]) -> f64 {
        let sample_size = data.len().min(1024 * 1024); // Sample up to 1MB
        let sample = &data[..sample_size];

        if let Ok(compressed) = encode_all(sample, 1) {
            compressed.len() as f64 / sample.len() as f64
        } else {
            1.0
        }
    }
}

/// Async parallel compressor using tokio and rayon
pub struct AsyncParallelCompressor {
    compressor: Arc<ParallelCompressor>,
}

impl AsyncParallelCompressor {
    /// Create a new async parallel compressor
    pub fn new(config: ParallelCompressionConfig) -> Self {
        Self {
            compressor: Arc::new(ParallelCompressor::new(config)),
        }
    }

    /// Compress data asynchronously
    pub async fn compress(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        let compressor = self.compressor.clone();
        tokio::task::spawn_blocking(move || compressor.compress(&data))
            .await
            .map_err(|e| FileError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e
            )))?
    }

    /// Decompress data asynchronously
    pub async fn decompress(&self, compressed: Vec<u8>) -> Result<Vec<u8>> {
        let compressor = self.compressor.clone();
        tokio::task::spawn_blocking(move || compressor.decompress(&compressed))
            .await
            .map_err(|e| FileError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e
            )))?
    }

    /// Compress multiple items asynchronously
    pub async fn compress_batch(&self, items: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
        let compressor = self.compressor.clone();
        tokio::task::spawn_blocking(move || compressor.compress_batch(items))
            .await
            .map_err(|e| FileError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e
            )))?
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_compression() {
        let config = ParallelCompressionConfig {
            chunk_size: 1024,
            parallel_threshold: 2048,
            ..Default::default()
        };

        let compressor = ParallelCompressor::new(config);

        // Test small data (single-threaded)
        let small_data = b"Hello, World!";
        let compressed = compressor.compress(small_data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();
        assert_eq!(decompressed, small_data);

        // Test large data (parallel)
        let large_data = vec![42u8; 10 * 1024 * 1024]; // 10MB
        let compressed = compressor.compress(&large_data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();
        assert_eq!(decompressed, large_data);

        // Test that we can decompress standard zstd format
        let standard_compressed = encode_all(&large_data[..], 3).unwrap();
        let standard_decompressed = compressor.decompress(&standard_compressed).unwrap();
        assert_eq!(standard_decompressed, large_data);

        println!("Compression ratio: {:.2}%",
                 compressed.len() as f64 / large_data.len() as f64 * 100.0);
    }

    #[test]
    fn test_batch_compression() {
        let compressor = ParallelCompressor::new(Default::default());

        let items = vec![
            vec![1u8; 1024],
            vec![2u8; 2048],
            vec![3u8; 4096],
        ];

        let compressed = compressor.compress_batch(items.clone()).unwrap();
        let decompressed = compressor.decompress_batch(compressed).unwrap();

        assert_eq!(decompressed, items);
    }

    #[tokio::test]
    async fn test_async_compression() {
        let compressor = AsyncParallelCompressor::new(Default::default());

        let data = vec![42u8; 1024 * 1024];
        let compressed = compressor.compress(data.clone()).await.unwrap();
        let decompressed = compressor.decompress(compressed).await.unwrap();

        assert_eq!(decompressed, data);
    }
}