use crc32fast::Hasher as Crc32;
use memmap2::{Mmap, MmapOptions};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::error::{FileError, Result};

pub const MAGIC: &[u8; 8] = b"MFFILE01";
pub const HEADER_LEN: usize = 16; // 8 字节魔数 + 8 字节预留区
pub const REC_HDR: usize = 8; // 记录头: u32 负载长度 + u32 CRC32

#[inline]
pub fn crc32(data: &[u8]) -> u32 {
    let mut h = Crc32::new();
    h.update(data);
    h.finalize()
}
#[inline]
pub fn read_u32_le(buf: &[u8]) -> u32 {
    u32::from_le_bytes(buf.try_into().unwrap())
}
#[inline]
pub fn write_u32_le(
    out: &mut [u8],
    v: u32,
) {
    out.copy_from_slice(&v.to_le_bytes());
}

// 写入文件头（包含魔数）
fn write_header(file: &mut File) -> Result<()> {
    file.seek(SeekFrom::Start(0))?;
    let mut buf = [0u8; HEADER_LEN];
    buf[..8].copy_from_slice(MAGIC);
    file.write_all(&buf)?;
    Ok(())
}

// 校验文件头（校验魔数）
fn check_header(file: &mut File) -> Result<()> {
    file.seek(SeekFrom::Start(0))?;
    let mut hdr = [0u8; HEADER_LEN];
    file.read_exact(&mut hdr)?;
    if &hdr[..8] != MAGIC {
        return Err(FileError::BadHeader);
    }
    Ok(())
}

#[derive(Debug)]
pub struct Writer {
    pub(crate) file: File,
    buf: BufWriter<File>,
    pub(crate) logical_end: u64,
    prealloc_until: u64,
    prealloc_chunk: u64,
}

impl Writer {
    // 创建写入器; prealloc_chunk 为预分配块大小（0 表示不预分配）
    pub fn create<P: AsRef<Path>>(
        path: P,
        prealloc_chunk: u64,
    ) -> Result<Self> {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)?;

        let meta_len = file.metadata()?.len();
        if meta_len == 0 {
            write_header(&mut file)?;
        } else {
            check_header(&mut file)?;
        }

        // 通过 mmap 扫描逻辑结尾（容忍尾部不完整记录）
        let (logical_end, file_len) = {
            let mmap = unsafe { MmapOptions::new().map(&file)? };
            let l = scan_logical_end(&mmap)?;
            (l, mmap.len() as u64)
        };

        let mut prealloc_until = file_len.max(logical_end);
        if prealloc_chunk > 0 && prealloc_until < logical_end + prealloc_chunk {
            prealloc_until =
                (logical_end + prealloc_chunk).max(HEADER_LEN as u64);
            file.set_len(prealloc_until)?;
        }

        file.seek(SeekFrom::Start(logical_end))?;
        let buf = BufWriter::with_capacity(8 * 1024 * 1024, file.try_clone()?);

        Ok(Self { file, buf, logical_end, prealloc_until, prealloc_chunk })
    }

    // 追加一条记录，返回该记录的起始偏移
    pub fn append(
        &mut self,
        payload: &[u8],
    ) -> Result<u64> {
        if payload.is_empty() {
            return Err(FileError::EmptyRecord);
        }
        if payload.len() > (u32::MAX as usize) {
            return Err(FileError::RecordTooLarge(payload.len()));
        }
        let need = REC_HDR as u64 + payload.len() as u64;
        self.ensure_capacity(need)?;

        let offset = self.logical_end;
        let mut hdr = [0u8; REC_HDR];
        write_u32_le(&mut hdr[0..4], payload.len() as u32);
        write_u32_le(&mut hdr[4..8], crc32(payload));
        self.buf.write_all(&hdr)?;
        self.buf.write_all(payload)?;
        self.logical_end += need;
        Ok(offset)
    }

    // 刷新缓冲区并同步到磁盘
    pub fn flush(&mut self) -> Result<()> {
        self.buf.flush()?;
        self.file.sync_data()?;
        Ok(())
    }
    // 当前逻辑长度
    pub fn len(&self) -> u64 {
        self.logical_end
    }

    // 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.logical_end == HEADER_LEN as u64
    }

    // 确保物理空间足够; 按块扩容
    fn ensure_capacity(
        &mut self,
        need: u64,
    ) -> Result<()> {
        if self.prealloc_chunk == 0 {
            return Ok(());
        }
        let want = self.logical_end + need;
        if want <= self.prealloc_until {
            return Ok(());
        }
        let mut new_size = self.prealloc_until;
        while new_size < want {
            new_size += self.prealloc_chunk;
        }
        self.buf.flush()?;
        self.file.set_len(new_size)?;
        self.prealloc_until = new_size;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Reader {
    pub(crate) _file: File, // 保持文件句柄存活以维持 mmap 有效性
    pub(crate) mmap: Mmap,
    pub(crate) logical_end: u64,
}

impl Reader {
    // 打开只读映射
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = OpenOptions::new().read(true).open(path)?;
        check_header(&mut file)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let logical_end = scan_logical_end(&mmap)?;
        Ok(Self { _file: file, mmap, logical_end })
    }
    // 逻辑结尾
    pub fn logical_len(&self) -> u64 {
        self.logical_end
    }
    // 读取指定偏移的记录负载
    pub fn get_at(
        &self,
        offset: u64,
    ) -> Result<&[u8]> {
        let end = usize::try_from(self.logical_end)
            .map_err(|_| FileError::BadHeader)?;
        let p = usize::try_from(offset).map_err(|_| FileError::BadHeader)?;
        if p + REC_HDR > end {
            return Err(FileError::BadHeader);
        }
        let len: usize = read_u32_le(&self.mmap[p..p + 4]) as usize;
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
        Ok(payload)
    }
    // 迭代所有记录（校验 CRC，遇到损坏或不完整即停止）
    pub fn iter(&self) -> Iter<'_> {
        Iter { mmap: &self.mmap, p: HEADER_LEN, end: self.logical_end as usize }
    }
}

pub struct Iter<'a> {
    mmap: &'a Mmap,
    p: usize,
    end: usize,
}
impl<'a> Iterator for Iter<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        if self.p + REC_HDR > self.end {
            return None;
        }
        let len = read_u32_le(&self.mmap[self.p..self.p + 4]) as usize;
        let stored_crc = read_u32_le(&self.mmap[self.p + 4..self.p + 8]);
        if len == 0 {
            return None;
        }
        let s = self.p + REC_HDR;
        let e = s + len;
        if e > self.end {
            return None;
        }
        let payload = &self.mmap[s..e];
        if crc32(payload) != stored_crc {
            return None;
        }
        self.p = e;
        Some(payload)
    }
}

// 扫描逻辑结尾：从文件头开始按记录推进，直到遇到越界/校验失败/零长度
pub fn scan_logical_end(mmap: &Mmap) -> Result<u64> {
    if mmap.len() < HEADER_LEN {
        return Err(FileError::BadHeader);
    }
    if &mmap[..8] != MAGIC {
        return Err(FileError::BadHeader);
    }
    let mut p = HEADER_LEN;
    let n = mmap.len();
    while p + REC_HDR <= n {
        let len = read_u32_le(&mmap[p..p + 4]) as usize;
        if len == 0 {
            break;
        }
        let s = p + REC_HDR;
        let e = s + len;
        if e > n {
            break;
        }
        let stored_crc = read_u32_le(&mmap[p + 4..p + 8]);
        if crc32(&mmap[s..e]) != stored_crc {
            break;
        }
        p = e;
    }
    Ok(p as u64)
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn reject_zero_length_records() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("zero.mff");

        let mut writer = Writer::create(&path, 0).unwrap();
        let err = writer.append(&[]).unwrap_err();
        assert!(matches!(err, FileError::EmptyRecord));
        writer.flush().unwrap();
        drop(writer);

        let reader = Reader::open(&path).unwrap();
        assert_eq!(reader.logical_len(), HEADER_LEN as u64);
        assert_eq!(reader.iter().count(), 0);
    }
}
