use crate::error::{FileError, Result};
use crate::record::{crc32, read_u32_le, write_u32_le, HEADER_LEN, MAGIC, REC_HDR};
use bytes::Bytes;
use futures::stream::Stream;
use memmap2::{Mmap, MmapOptions};
use std::path::Path;
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufWriter};
use tokio::sync::Mutex;

/// 异步写入器，支持追加式记录文件和并行压缩
/// Async writer supporting append-only record files and parallel compression
pub struct AsyncWriter {
    file: Arc<Mutex<File>>,                // 文件句柄 / File handle
    buf: Arc<Mutex<BufWriter<File>>>,      // 缓冲写入器 / Buffered writer
    logical_end: Arc<Mutex<u64>>,          // 逻辑结束位置 / Logical end position
    prealloc_until: Arc<Mutex<u64>>,       // 预分配到的位置 / Pre-allocated until
    prealloc_chunk: u64,                   // 预分配块大小 / Pre-allocation chunk size
}

impl AsyncWriter {
    /// 创建新的异步写入器，支持预分配
    /// Create new async writer with pre-allocation support
    pub async fn create<P: AsRef<Path>>(
        path: P,
        prealloc_chunk: u64,
    ) -> Result<Self> {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .await?;

        let meta_len = file.metadata().await?.len();
        if meta_len == 0 {
            Self::write_header(&mut file).await?;
        } else {
            Self::check_header(&mut file).await?;
        }

        // Scan for logical end using sync mmap (for compatibility)
        let logical_end = {
            let std_file = file.try_clone().await?.into_std().await;
            let mmap = unsafe { MmapOptions::new().map(&std_file)? };
            crate::record::scan_logical_end(&mmap)?
        };

        let file_len = file.metadata().await?.len();
        let mut prealloc_until = file_len.max(logical_end);

        if prealloc_chunk > 0 && prealloc_until < logical_end + prealloc_chunk {
            prealloc_until =
                (logical_end + prealloc_chunk).max(HEADER_LEN as u64);
            file.set_len(prealloc_until).await?;
        }

        file.seek(tokio::io::SeekFrom::Start(logical_end)).await?;
        let buf =
            BufWriter::with_capacity(8 * 1024 * 1024, file.try_clone().await?);

        Ok(Self {
            file: Arc::new(Mutex::new(file)),
            buf: Arc::new(Mutex::new(buf)),
            logical_end: Arc::new(Mutex::new(logical_end)),
            prealloc_until: Arc::new(Mutex::new(prealloc_until)),
            prealloc_chunk,
        })
    }

    /// 异步追加一条记录
    pub async fn append(
        &self,
        payload: &[u8],
    ) -> Result<u64> {
        if payload.is_empty() {
            return Err(FileError::EmptyRecord);
        }
        if payload.len() > (u32::MAX as usize) {
            return Err(FileError::RecordTooLarge(payload.len()));
        }

        let need = REC_HDR as u64 + payload.len() as u64;
        self.ensure_capacity(need).await?;

        let offset = {
            let mut logical_end = self.logical_end.lock().await;
            let current = *logical_end;
            *logical_end += need;
            current
        };

        let mut hdr = [0u8; REC_HDR];
        write_u32_le(&mut hdr[0..4], payload.len() as u32);
        write_u32_le(&mut hdr[4..8], crc32(payload));

        let mut buf = self.buf.lock().await;
        buf.write_all(&hdr).await?;
        buf.write_all(payload).await?;

        Ok(offset)
    }

    /// 并行批量追加多条记录
    pub async fn append_batch(
        &self,
        payloads: Vec<Vec<u8>>,
    ) -> Result<Vec<u64>> {
        use futures::future::join_all;

        let futures = payloads
            .into_iter()
            .map(|payload| async move { self.append(&payload).await });

        join_all(futures).await.into_iter().collect()
    }

    /// 异步刷新缓冲区
    pub async fn flush(&self) -> Result<()> {
        let mut buf = self.buf.lock().await;
        buf.flush().await?;

        let file = self.file.lock().await;
        file.sync_data().await?;
        Ok(())
    }

    /// 获取当前逻辑长度
    pub async fn len(&self) -> u64 {
        *self.logical_end.lock().await
    }

    /// 检查是否为空
    pub async fn is_empty(&self) -> bool {
        *self.logical_end.lock().await == HEADER_LEN as u64
    }

    async fn ensure_capacity(
        &self,
        need: u64,
    ) -> Result<()> {
        if self.prealloc_chunk == 0 {
            return Ok(());
        }

        let logical_end = *self.logical_end.lock().await;
        let want = logical_end + need;

        let mut prealloc_until = self.prealloc_until.lock().await;
        if want <= *prealloc_until {
            return Ok(());
        }

        let mut new_size = *prealloc_until;
        while new_size < want {
            new_size += self.prealloc_chunk;
        }

        let mut buf = self.buf.lock().await;
        buf.flush().await?;
        drop(buf);

        let file = self.file.lock().await;
        file.set_len(new_size).await?;
        *prealloc_until = new_size;
        Ok(())
    }

    async fn write_header(file: &mut File) -> Result<()> {
        file.seek(tokio::io::SeekFrom::Start(0)).await?;
        let mut buf = [0u8; HEADER_LEN];
        buf[..8].copy_from_slice(MAGIC);
        file.write_all(&buf).await?;
        Ok(())
    }

    async fn check_header(file: &mut File) -> Result<()> {
        file.seek(tokio::io::SeekFrom::Start(0)).await?;
        let mut hdr = [0u8; HEADER_LEN];
        file.read_exact(&mut hdr).await?;
        if &hdr[..8] != MAGIC {
            return Err(FileError::BadHeader);
        }
        Ok(())
    }
}

/// 异步读取器，支持记录文件流式读取
/// Async reader supporting streaming read of record files
pub struct AsyncReader {
    _file: Arc<File>,     // 文件句柄（保持文件打开）/ File handle (keep file open)
    mmap: Arc<Mmap>,      // 内存映射 / Memory map
    logical_end: u64,     // 逻辑结束位置 / Logical end position
}

impl AsyncReader {
    /// 打开文件进行异步读取
    /// Open file for async reading
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = OpenOptions::new().read(true).open(&path).await?;

        // Check header
        file.seek(tokio::io::SeekFrom::Start(0)).await?;
        let mut hdr = [0u8; HEADER_LEN];
        file.read_exact(&mut hdr).await?;
        if &hdr[..8] != MAGIC {
            return Err(FileError::BadHeader);
        }

        // Create mmap from std file
        let std_file = file.try_clone().await?.into_std().await;
        let mmap = unsafe { MmapOptions::new().map(&std_file)? };
        let logical_end = crate::record::scan_logical_end(&mmap)?;

        Ok(Self { _file: Arc::new(file), mmap: Arc::new(mmap), logical_end })
    }

    /// 获取指定偏移量处的记录
    pub async fn get_at(
        &self,
        offset: u64,
    ) -> Result<Bytes> {
        let end = usize::try_from(self.logical_end)
            .map_err(|_| FileError::BadHeader)?;
        let p = usize::try_from(offset).map_err(|_| FileError::BadHeader)?;

        if p + REC_HDR > end {
            return Err(FileError::BadHeader);
        }

        let len = read_u32_le(&self.mmap[p..p + 4]) as usize;
        let stored_crc = read_u32_le(&self.mmap[p + 4..p + 8]);

        if len == 0 {
            return Err(FileError::BadHeader);
        }

        let s = p + REC_HDR;
        let e = s + len;
        if e > end {
            return Err(FileError::BadHeader);
        }

        let payload = &self.mmap[s..e];
        if crc32(payload) != stored_crc {
            return Err(FileError::CrcMismatch(offset));
        }

        Ok(Bytes::copy_from_slice(payload))
    }

    /// Stream all records
    pub fn stream(&self) -> impl Stream<Item = Result<Bytes>> + '_ {
        futures::stream::unfold(HEADER_LEN as u64, move |pos| async move {
            if pos >= self.logical_end {
                return None;
            }

            match self.get_at(pos).await {
                Ok(data) => {
                    let next_pos = pos + REC_HDR as u64 + data.len() as u64;
                    Some((Ok(data), next_pos))
                },
                Err(e) => Some((Err(e), self.logical_end)),
            }
        })
    }

    /// Get logical length
    pub fn logical_len(&self) -> u64 {
        self.logical_end
    }

    /// Process records in parallel batches
    pub async fn process_parallel<F, T>(
        &self,
        batch_size: usize,
        processor: F,
    ) -> Result<Vec<T>>
    where
        F: Fn(Bytes) -> T + Send + Sync + Clone + 'static,
        T: Send + 'static,
    {
        use futures::stream::TryStreamExt;
        use rayon::prelude::*;

        let records: Vec<Bytes> = self.stream().try_collect().await?;

        // Process in parallel using rayon
        let results = tokio::task::spawn_blocking(move || {
            records
                .par_chunks(batch_size)
                .flat_map(|batch| {
                    batch.par_iter().map(|bytes| processor(bytes.clone()))
                })
                .collect()
        })
        .await
        .map_err(|e| {
            FileError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        })?;

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn async_roundtrip() -> Result<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("async_data.mff");

        // Write
        let writer = AsyncWriter::create(&path, 64 * 1024 * 1024).await?;
        let off1 = writer.append(b"hello").await?;
        let off2 = writer.append(b"world").await?;
        let big = vec![42u8; 128 * 1024];
        let off3 = writer.append(&big).await?;
        writer.flush().await?;

        assert!(off2 > off1 && off3 > off2);

        // Read
        let reader = AsyncReader::open(&path).await?;
        assert_eq!(reader.get_at(off3).await?, &big[..]);

        // Stream
        use futures::StreamExt;
        let records: Vec<Bytes> = reader
            .stream()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;
        assert_eq!(records.len(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn batch_append() -> Result<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("batch_data.mff");

        let writer = AsyncWriter::create(&path, 0).await?;

        let payloads =
            vec![b"first".to_vec(), b"second".to_vec(), b"third".to_vec()];

        let offsets = writer.append_batch(payloads).await?;
        writer.flush().await?;

        assert_eq!(offsets.len(), 3);
        assert!(offsets[1] > offsets[0]);
        assert!(offsets[2] > offsets[1]);

        Ok(())
    }
}
