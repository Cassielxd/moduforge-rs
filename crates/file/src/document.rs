use std::io;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use blake3::Hasher as Blake3;
use serde::{Deserialize, Serialize};

use crate::common::{
    encode_segment, decode_segment,
    create_tail_pointer, parse_tail_pointer, validate_tail_offset,
    validate_payload,
    DIR_FLAG_ZSTD_SEGMENTS, TAIL_MAGIC, TAIL_POINTER_SIZE,
};
use crate::error::{FileError, Result};
use crate::record::{crc32, read_u32_le, Reader, Writer, HEADER_LEN, REC_HDR};

/// 段类型：用于描述容器中存储的数据类别
/// Segment type: describes the category of data stored in the container
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SegmentType(pub String);

/// 段目录项：记录段的类型、偏移、长度与CRC
/// Segment entry: records the type, offset, length and CRC of a segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentEntry {
    pub kind: SegmentType,      // 段类型
    pub offset: u64,             // 文件中的偏移位置
    pub length: u64,             // 段长度（包含头部）
    pub crc32: u32,              // CRC32校验和
}

/// 总目录：包含所有段的索引及文件级哈希
/// Directory: contains index of all segments and file-level hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    pub entries: Vec<SegmentEntry>,  // 所有段的条目列表
    pub flags: u32,                  // 目录标志（压缩等）
    pub file_hash: [u8; 32],         // 文件内容的Blake3哈希
}

/// 文档写入器：基于append-only模式写入段，并在末尾写入目录
/// Document writer: writes segments in append-only mode and writes directory at the end
pub struct DocumentWriter {
    w: Writer,                    // 底层记录写入器
    segments: Vec<SegmentEntry>,  // 已写入段的列表
    path: PathBuf,                // 文件路径
}
impl DocumentWriter {
    /// 开始写入新文档
    /// Begin writing a new document
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(path), fields(
        crate_name = "file",
        file_path = %path.as_ref().display()
    )))]
    pub fn begin<P: AsRef<Path>>(path: P) -> Result<Self> {
        let p = path.as_ref().to_path_buf();
        Ok(Self { w: Writer::create(&p, 0)?, segments: Vec::new(), path: p })
    }

    /// 追加一个段到文档
    /// Add a segment to the document
    pub fn add_segment(
        &mut self,
        kind: SegmentType,
        payload: &[u8],
    ) -> Result<()> {
        // 验证负载不为空
        validate_payload(payload)?;

        // 压缩数据
        let stored = encode_segment(payload)?;

        // 写入压缩数据并记录偏移
        let off = self.w.append(&stored)?;
        let crc = crc32(&stored);

        // 记录段信息
        self.segments.push(SegmentEntry {
            kind,
            offset: off,
            length: (REC_HDR as u64) + stored.len() as u64,
            crc32: crc,
        });
        Ok(())
    }

    /// 完成写入：生成并写入目录，计算全文件哈希
    /// Finalize writing: generate and write directory, calculate file hash
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self), fields(
        crate_name = "file",
        segment_count = self.segments.len(),
        file_path = %self.path.display()
    )))]
    pub fn finalize(mut self) -> Result<()> {
        // 刷新缓冲区并计算数据哈希
        // Flush buffer and calculate data hash
        self.w.flush()?;
        let mut hasher = Blake3::new();
        let r = Reader::open(&self.path)?;
        for bytes in r.iter() {
            hasher.update(bytes);
        }
        let hash = *hasher.finalize().as_bytes();

        // 创建并序列化目录
        // Create and serialize directory
        let flags = DIR_FLAG_ZSTD_SEGMENTS;
        let dir = Directory { entries: self.segments, flags, file_hash: hash };
        let bytes =
            bincode::serde::encode_to_vec(&dir, bincode::config::standard())
                .map_err(io::Error::other)
                .map_err(FileError::Io)?;
        let dir_off = self.w.append(&bytes)?;
        self.w.flush()?;

        // 写入尾指针，不计入逻辑长度
        // Write tail pointer without updating logical length
        // 这样Reader扫描逻辑结尾仍停在目录记录处，但可通过物理文件尾部快速读取目录偏移
        // This way Reader's logical end stops at directory record, but can quickly read directory offset from physical file end
        {
            let tail_pointer = create_tail_pointer(dir_off);
            let file = &mut self.w.file;
            file.seek(SeekFrom::Start(self.w.logical_end))?;
            file.write_all(&tail_pointer)?;
            file.sync_data()?;
        }
        Ok(())
    }
}

/// 文档读取器：读取末尾目录并提供段访问
/// Document reader: reads directory at the end and provides segment access
pub struct DocumentReader {
    r: Reader,      // 底层记录读取器
    dir: Directory, // 文档目录
}

impl DocumentReader {
    /// 打开文档并读取目录
    /// Open document and read directory
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(path), fields(
        crate_name = "file",
        file_path = %path.as_ref().display()
    )))]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let r = Reader::open(path)?;

        // 优先通过尾指针快速定位目录偏移
        // Try to quickly locate directory offset through tail pointer first
        let mut last_off = HEADER_LEN as u64;
        let phys_len = r.mmap.len();
        if phys_len >= TAIL_POINTER_SIZE {
            let tail = &r.mmap[phys_len - TAIL_POINTER_SIZE..phys_len];
            if let Some(off) = parse_tail_pointer(tail) {
                // 基本校验：offset 落在逻辑区间内且指向一条有效记录
                if validate_tail_offset(off, HEADER_LEN as u64, phys_len as u64) {
                    if (off as usize) + REC_HDR <= r.logical_end as usize {
                        let len = read_u32_le(&r.mmap[off as usize..off as usize + 4]) as usize;
                        let s = off as usize + REC_HDR;
                        let e = s + len;
                        if e <= r.logical_end as usize {
                            let stored_crc = read_u32_le(&r.mmap[off as usize + 4..off as usize + 8]);
                            if crc32(&r.mmap[s..e]) == stored_crc {
                                last_off = off;
                            }
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
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, callback), fields(
        crate_name = "file",
        segment_type = ?kind,
        total_segments = self.dir.entries.len()
    )))]
    pub fn read_segments<F>(
        &self,
        kind: SegmentType,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(usize, &[u8]) -> Result<()>,
    {
        for (index, entry) in self.dir.entries.iter().enumerate() {
            if entry.kind == kind {
                let bytes = self.r.get_at(entry.offset)?;
                if crc32(bytes) != entry.crc32 {
                    return Err(FileError::CrcMismatch(entry.offset));
                }
                let decoded = decode_segment(bytes, self.dir.flags)?;
                callback(index, decoded.as_ref())?;
            }
        }
        Ok(())
    }

    /// 返回完整的段目录元数据
    pub fn directory(&self) -> &Directory {
        &self.dir
    }

    /// 返回所有段记录，按写入顺序排列
    pub fn segments(&self) -> &[SegmentEntry] {
        &self.dir.entries
    }

    /// 返回文档的逻辑长度（不含尾指针）
    pub fn logical_len(&self) -> u64 {
        self.r.logical_len()
    }

    /// 读取指定索引的段负载（含 CRC 校验）
    pub fn segment_payload(
        &self,
        index: usize,
    ) -> Result<Vec<u8>> {
        let entry = self.dir.entries.get(index).ok_or(FileError::BadHeader)?;
        let bytes = self.r.get_at(entry.offset)?;
        if crc32(bytes) != entry.crc32 {
            return Err(FileError::CrcMismatch(entry.offset));
        }
        let decoded = decode_segment(bytes, self.dir.flags)?;
        Ok(decoded.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn write_and_read_zstd_segments() -> Result<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("zstd_roundtrip.mff");

        let mut writer = DocumentWriter::begin(&path)?;
        writer.add_segment(SegmentType("json".to_string()), br#"{"a":1}"#)?;
        writer.add_segment(SegmentType("bin".to_string()), &[1u8, 2, 3, 4])?;
        writer.finalize()?;

        let reader = DocumentReader::open(&path)?;
        assert_eq!(
            reader.directory().flags & DIR_FLAG_ZSTD_SEGMENTS,
            DIR_FLAG_ZSTD_SEGMENTS
        );
        assert_eq!(reader.segments().len(), 2);

        assert_eq!(reader.segment_payload(0)?, br#"{"a":1}"#);

        let mut seen = Vec::new();
        reader.read_segments(SegmentType("bin".to_string()), |_, bytes| {
            seen.push(bytes.to_vec());
            Ok(())
        })?;
        assert_eq!(seen, vec![vec![1, 2, 3, 4]]);
        Ok(())
    }
}
