use std::io;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use blake3::Hasher as Blake3;
use serde::{Deserialize, Serialize};

use crate::error::{FileError, Result};
use crate::record::{crc32, read_u32_le, Reader, Writer, HEADER_LEN, REC_HDR};

// 固定尾指针：用于在 finalize 后快速定位目录起始偏移，避免全量扫描
const TAIL_MAGIC: &[u8; 8] = b"MFFTAIL1"; // 8B 魔数 + 8B 目录偏移 (LE)

// 段类型：用于描述容器中存储的数据类别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SegmentType(String);
// 段目录项：记录段的类型、偏移、长度与 CRC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentEntry {
    pub kind: SegmentType,
    pub offset: u64,
    pub length: u64,
    pub crc32: u32,
}

// 总目录：包含所有段的索引及文件级哈希
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    pub entries: Vec<SegmentEntry>,
    pub flags: u32,
    pub file_hash: [u8; 32],
}

// 文档写入器：基于 append-only 文件写入段，并在末尾写目录
pub struct DocumentWriter {
    w: Writer,
    segments: Vec<SegmentEntry>,
    path: PathBuf,
}
impl DocumentWriter {
    // 开始写入
    pub fn begin<P: AsRef<Path>>(path: P) -> Result<Self> {
        let p = path.as_ref().to_path_buf();
        Ok(Self { w: Writer::create(&p, 0)?, segments: Vec::new(), path: p })
    }
    // 追加一个段
    pub fn add_segment(
        &mut self,
        kind: SegmentType,
        payload: &[u8],
    ) -> Result<()> {
        let off = self.w.len();
        let _ = self.w.append(payload)?;
        let crc = crc32(payload);
        self.segments.push(SegmentEntry {
            kind,
            offset: off,
            length: (REC_HDR as u64) + payload.len() as u64,
            crc32: crc,
        });
        Ok(())
    }
    // 完成写入：生成并写入目录，计算全文件哈希
    pub fn finalize(mut self) -> Result<()> {
        // 计算数据哈希
        self.w.flush()?;
        let mut hasher = Blake3::new();
        let r = Reader::open(&self.path)?;
        for bytes in r.iter() {
            hasher.update(bytes);
        }
        let hash = *hasher.finalize().as_bytes();
        // 写入目录记录
        let dir =
            Directory { entries: self.segments, flags: 0, file_hash: hash };
        let bytes =
            bincode::serde::encode_to_vec(&dir, bincode::config::standard())
                .map_err(io::Error::other)
                .map_err(FileError::Io)?;
        let dir_off = self.w.append(&bytes)?;
        self.w.flush()?;

        // 写入尾指针，不计入逻辑长度：MAGIC(8) + dir_off(8)
        // 这样 Reader 扫描逻辑结尾仍停在目录记录处，但可通过物理文件尾部快速读取目录偏移
        {
            // 直接使用底层文件写入尾部，不更新 logical_end
            let file = &mut self.w.file;
            file.seek(SeekFrom::Start(self.w.logical_end))?;
            file.write_all(TAIL_MAGIC)?;
            file.write_all(&dir_off.to_le_bytes())?;
            file.sync_data()?;
        }
        Ok(())
    }
}

// 文档读取器：读取末尾目录并提供段访问
pub struct DocumentReader {
    r: Reader,
    dir: Directory,
}
impl DocumentReader {
    // 打开并读取目录
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let r = Reader::open(path)?;
        // 优先通过尾指针快速定位目录偏移
        let mut last_off = HEADER_LEN as u64;
        let phys_len = r.mmap.len();
        if phys_len >= 16 {
            let tail = &r.mmap[phys_len - 16..phys_len];
            if &tail[..8] == TAIL_MAGIC {
                let mut off_bytes = [0u8; 8];
                off_bytes.copy_from_slice(&tail[8..16]);
                let off = u64::from_le_bytes(off_bytes);
                // 基本校验：offset 落在逻辑区间内且指向一条有效记录
                if (off as usize) + REC_HDR <= r.logical_end as usize {
                    let len =
                        read_u32_le(&r.mmap[off as usize..off as usize + 4])
                            as usize;
                    let s = off as usize + REC_HDR;
                    let e = s + len;
                    if e <= r.logical_end as usize {
                        let stored_crc = read_u32_le(
                            &r.mmap[off as usize + 4..off as usize + 8],
                        );
                        if crc32(&r.mmap[s..e]) == stored_crc {
                            last_off = off;
                        }
                    }
                }
            }
        }
        // 如尾指针缺失/非法，回退到顺序扫描
        if last_off == (HEADER_LEN as u64) {
            let mut p = HEADER_LEN;
            let end = r.logical_end as usize;
            let mut fallback_last = HEADER_LEN as u64;
            while p + REC_HDR <= end {
                let len = read_u32_le(&r.mmap[p..p + 4]) as usize;
                if len == 0 {
                    break;
                }
                let s = p + REC_HDR;
                let e = s + len;
                if e > end {
                    break;
                }
                let stored_crc = read_u32_le(&r.mmap[p + 4..p + 8]);
                if crc32(&r.mmap[s..e]) != stored_crc {
                    break;
                }
                fallback_last = p as u64;
                p = e;
            }
            last_off = fallback_last;
        }
        let dir_bytes = r.get_at(last_off)?;
        let (dir, _) = bincode::serde::decode_from_slice::<Directory, _>(
            dir_bytes,
            bincode::config::standard(),
        )
        .map_err(io::Error::other)
        .map_err(FileError::Io)?;
        // 校验除目录外的数据哈希
        let mut hasher = Blake3::new();
        let mut q = HEADER_LEN;
        let end2 = last_off as usize;
        while q + REC_HDR <= end2 {
            let len = read_u32_le(&r.mmap[q..q + 4]) as usize;
            if len == 0 {
                break;
            }
            let s = q + REC_HDR;
            let e = s + len;
            if e > end2 {
                break;
            }
            let stored_crc = read_u32_le(&r.mmap[q + 4..q + 8]);
            if crc32(&r.mmap[s..e]) != stored_crc {
                break;
            }
            hasher.update(&r.mmap[s..e]);
            q = e;
        }
        let calc = *hasher.finalize().as_bytes();
        if calc != dir.file_hash {
            return Err(FileError::BadHeader);
        }
        Ok(Self { r, dir })
    }

    // 读取所有指定类型的段
    pub fn read_segments(
        &self,
        kind: SegmentType,
    ) -> Result<Vec<&[u8]>> {
        let mut segments = Vec::new();
        for entry in self.dir.entries.iter().rev() {
            if entry.kind == kind {
                let bytes = self.r.get_at(entry.offset)?;
                if crc32(bytes) != entry.crc32 {
                    return Err(FileError::CrcMismatch(entry.offset));
                }
                segments.push(bytes);
            }
        }
        Ok(segments)
    }
}
