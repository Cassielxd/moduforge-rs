use std::io;
use std::path::{Path, PathBuf};

use blake3::Hasher as Blake3;
use serde::{Deserialize, Serialize};

use crate::error::{FileError, Result};
use crate::record::{crc32, read_u32_le, Reader, Writer, HEADER_LEN, REC_HDR};

// 段类型：用于描述容器中存储的数据类别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SegmentType { Meta, Schema, Snapshot, Assets, History, Index, Directory }

// 段目录项：记录段的类型、偏移、长度与 CRC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentEntry { pub kind: SegmentType, pub offset: u64, pub length: u64, pub crc32: u32 }

// 总目录：包含所有段的索引及文件级哈希
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory { pub entries: Vec<SegmentEntry>, pub flags: u32, pub file_hash: [u8; 32] }

// 文档写入器：基于 append-only 文件写入段，并在末尾写目录
pub struct DocumentWriter { w: Writer, segments: Vec<SegmentEntry>, path: PathBuf }
impl DocumentWriter {
    // 开始写入
    pub fn begin<P: AsRef<Path>>(path: P) -> Result<Self> {
        let p = path.as_ref().to_path_buf();
        Ok(Self { w: Writer::create(&p, 0)?, segments: Vec::new(), path: p })
    }
    // 追加一个段
    pub fn add_segment(&mut self, kind: SegmentType, payload: &[u8]) -> Result<()> {
        let off = self.w.len();
        let _ = self.w.append(payload)?;
        let crc = crc32(payload);
        self.segments.push(SegmentEntry { kind, offset: off, length: (REC_HDR as u64) + payload.len() as u64, crc32: crc });
        Ok(())
    }
    // 完成写入：生成并写入目录，计算全文件哈希
    pub fn finalize(mut self) -> Result<()> {
        self.w.flush()?;
        let mut hasher = Blake3::new();
        let r = Reader::open(&self.path)?;
        for bytes in r.iter() { hasher.update(bytes); }
        let hash = *hasher.finalize().as_bytes();
        let dir = Directory { entries: self.segments, flags: 0, file_hash: hash };
        let bytes = bincode::serde::encode_to_vec(&dir, bincode::config::standard()).map_err(|e| io::Error::new(io::ErrorKind::Other, e)).map_err(FileError::Io)?;
        let _ = self.w.append(&bytes)?;
        self.w.flush()?; Ok(())
    }
}

// 文档读取器：读取末尾目录并提供段访问
pub struct DocumentReader { r: Reader, dir: Directory }
impl DocumentReader {
    // 打开并读取目录
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let r = Reader::open(path)?;
        // 查找最后一条记录（目录）
        let mut p = HEADER_LEN; let end = r.logical_end as usize; let mut last_off = HEADER_LEN as u64;
        while p + REC_HDR <= end {
            let len = read_u32_le(&r.mmap[p..p+4]) as usize; if len == 0 { break; }
            let s = p + REC_HDR; let e = s + len; if e > end { break; }
            let stored_crc = read_u32_le(&r.mmap[p+4..p+8]); if crc32(&r.mmap[s..e]) != stored_crc { break; }
            last_off = p as u64; p = e;
        }
        let dir_bytes = r.get_at(last_off)?;
        let (dir, _) = bincode::serde::decode_from_slice::<Directory, _>(dir_bytes, bincode::config::standard()).map_err(|e| io::Error::new(io::ErrorKind::Other, e)).map_err(FileError::Io)?;
        // 校验除目录外的数据哈希
        let mut hasher = Blake3::new(); let mut q = HEADER_LEN; let end2 = last_off as usize;
        while q + REC_HDR <= end2 {
            let len = read_u32_le(&r.mmap[q..q+4]) as usize; if len == 0 { break; }
            let s = q + REC_HDR; let e = s + len; if e > end2 { break; }
            let stored_crc = read_u32_le(&r.mmap[q+4..q+8]); if crc32(&r.mmap[s..e]) != stored_crc { break; }
            hasher.update(&r.mmap[s..e]); q = e;
        }
        let calc = *hasher.finalize().as_bytes(); if calc != dir.file_hash { return Err(FileError::BadHeader); }
        Ok(Self { r, dir })
    }
    // 按类型读取段负载
    pub fn read_segment(&self, kind: SegmentType) -> Result<Option<&[u8]>> {
        if let Some(entry) = self.dir.entries.iter().rev().find(|e| e.kind == kind) {
            let bytes = self.r.get_at(entry.offset)?; if crc32(bytes) != entry.crc32 { return Err(FileError::CrcMismatch(entry.offset)); }
            return Ok(Some(bytes));
        }
        Ok(None)
    }
}


