//! 同步和异步实现之间共享的常量、类型和工具函数
//! Common constants, types, and utilities shared between sync and async implementations

use crate::error::{FileError, Result};
use std::borrow::Cow;

// ============================================================================
// 文件格式常量
// File Format Constants
// ============================================================================

/// 尾指针魔数 - 用于快速定位目录
/// Magic bytes for tail pointer - used for fast directory lookup
pub const TAIL_MAGIC: &[u8; 8] = b"MFFTAIL1";

/// 尾指针结构大小（魔数 + 偏移量）
/// Size of tail pointer structure (magic + offset)
pub const TAIL_POINTER_SIZE: usize = 16; // 8字节魔数 + 8字节偏移量

/// 目录标志位
/// Directory flags
pub const DIR_FLAG_ZSTD_SEGMENTS: u32 = 0x0001;         // 段使用zstd压缩
pub const DIR_FLAG_PARALLEL_COMPRESSION: u32 = 0x0002;  // 启用并行压缩

/// 默认zstd压缩级别
/// Default zstd compression level
pub const DEFAULT_ZSTD_LEVEL: i32 = 1;

/// Zstd魔数前缀，用于检测压缩数据
/// Zstd magic bytes prefix for detection
pub const ZSTD_MAGIC_PREFIX: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];

// ============================================================================
// 压缩/解压缩工具
// Compression/Decompression Utilities
// ============================================================================

/// 使用默认级别的zstd压缩对段进行编码
/// Encode a segment using zstd compression with default level
pub fn encode_segment(payload: &[u8]) -> Result<Vec<u8>> {
    zstd::stream::encode_all(payload, DEFAULT_ZSTD_LEVEL)
        .map_err(FileError::Io)
}

/// 自动检测压缩格式并解码段
/// Decode a segment with automatic compression detection
///
/// 该函数处理：
/// This function handles:
/// - Zstd压缩数据（通过魔数或标志位检测）/ Zstd compressed data (detected by magic bytes or flags)
/// - 未压缩数据（向后兼容）/ Uncompressed data (for backward compatibility)
pub fn decode_segment<'a>(
    bytes: &'a [u8],
    dir_flags: u32,
) -> Result<Cow<'a, [u8]>> {
    // 检查zstd魔数
    // Check for zstd magic bytes
    let has_magic = bytes.len() >= ZSTD_MAGIC_PREFIX.len()
        && &bytes[..ZSTD_MAGIC_PREFIX.len()] == ZSTD_MAGIC_PREFIX;

    // 确定是否需要尝试解压
    // Determine if we should try decompression
    let should_try_decompress = (dir_flags & DIR_FLAG_ZSTD_SEGMENTS != 0) || has_magic;

    if !should_try_decompress {
        return Ok(Cow::Borrowed(bytes));
    }

    // 尝试解压
    // Try to decompress
    match zstd::stream::decode_all(bytes) {
        Ok(raw) => Ok(Cow::Owned(raw)),
        Err(_err) if dir_flags & DIR_FLAG_ZSTD_SEGMENTS == 0 => {
            // 对于没有压缩标志的旧文件，允许回退到原始数据
            // For old files without compression flag, allow fallback to raw data
            Ok(Cow::Borrowed(bytes))
        },
        Err(err) => Err(FileError::Io(err)),
    }
}

/// 检查数据是否为zstd压缩格式
/// Check if data appears to be zstd compressed
#[inline]
pub fn is_zstd_compressed(data: &[u8]) -> bool {
    data.len() >= ZSTD_MAGIC_PREFIX.len()
        && &data[..ZSTD_MAGIC_PREFIX.len()] == ZSTD_MAGIC_PREFIX
}

// ============================================================================
// 尾指针工具
// Tail Pointer Utilities
// ============================================================================

/// 创建尾指针缓冲区
/// Write tail pointer to a buffer
///
/// 返回包含尾指针的16字节缓冲区
/// Returns a 16-byte buffer containing the tail pointer
#[inline]
pub fn create_tail_pointer(directory_offset: u64) -> [u8; TAIL_POINTER_SIZE] {
    let mut tail = [0u8; TAIL_POINTER_SIZE];
    tail[..8].copy_from_slice(TAIL_MAGIC);
    tail[8..16].copy_from_slice(&directory_offset.to_le_bytes());
    tail
}

/// 从缓冲区解析尾指针
/// Parse tail pointer from a buffer
///
/// 如果有效则返回目录偏移量，否则返回None
/// Returns the directory offset if valid, None otherwise
#[inline]
pub fn parse_tail_pointer(tail_bytes: &[u8]) -> Option<u64> {
    if tail_bytes.len() < TAIL_POINTER_SIZE {
        return None;
    }

    // 检查魔数字节
    // Check magic bytes
    if &tail_bytes[..8] != TAIL_MAGIC {
        return None;
    }

    // 解析偏移量
    // Parse offset
    let mut off_bytes = [0u8; 8];
    off_bytes.copy_from_slice(&tail_bytes[8..16]);
    Some(u64::from_le_bytes(off_bytes))
}

/// 验证尾指针偏移量是否在合理范围内
/// Validate a tail pointer offset is within reasonable bounds
#[inline]
pub fn validate_tail_offset(
    offset: u64,
    min_offset: u64,
    file_size: u64,
) -> bool {
    offset >= min_offset && offset < file_size.saturating_sub(TAIL_POINTER_SIZE as u64)
}

// ============================================================================
// 验证工具
// Validation Utilities
// ============================================================================

/// 验证负载不为空
/// Validate that a payload is not empty
#[inline]
pub fn validate_payload(payload: &[u8]) -> Result<()> {
    if payload.is_empty() {
        Err(FileError::EmptyRecord)
    } else {
        Ok(())
    }
}

/// 检查目录标志是否表示启用了压缩
/// Check if a directory flag indicates compression is enabled
#[inline]
pub fn has_compression(flags: u32) -> bool {
    (flags & DIR_FLAG_ZSTD_SEGMENTS) != 0 || (flags & DIR_FLAG_PARALLEL_COMPRESSION) != 0
}

/// 检查是否启用了并行压缩
/// Check if parallel compression is enabled
#[inline]
pub fn has_parallel_compression(flags: u32) -> bool {
    (flags & DIR_FLAG_PARALLEL_COMPRESSION) != 0
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tail_pointer_roundtrip() {
        let offset = 0x123456789ABCDEF0u64;
        let tail = create_tail_pointer(offset);
        let parsed = parse_tail_pointer(&tail).unwrap();
        assert_eq!(parsed, offset);
    }

    #[test]
    fn test_invalid_tail_pointer() {
        let bad_tail = [0u8; 16];
        assert_eq!(parse_tail_pointer(&bad_tail), None);

        let short_tail = [0u8; 8];
        assert_eq!(parse_tail_pointer(&short_tail), None);
    }

    #[test]
    fn test_compression_detection() {
        let mut data = vec![0u8; 100];
        assert!(!is_zstd_compressed(&data));

        data[..4].copy_from_slice(&ZSTD_MAGIC_PREFIX);
        assert!(is_zstd_compressed(&data));
    }

    #[test]
    fn test_compression_flags() {
        assert!(has_compression(DIR_FLAG_ZSTD_SEGMENTS));
        assert!(has_compression(DIR_FLAG_PARALLEL_COMPRESSION));
        assert!(has_compression(DIR_FLAG_ZSTD_SEGMENTS | DIR_FLAG_PARALLEL_COMPRESSION));
        assert!(!has_compression(0));

        assert!(has_parallel_compression(DIR_FLAG_PARALLEL_COMPRESSION));
        assert!(!has_parallel_compression(DIR_FLAG_ZSTD_SEGMENTS));
    }

    #[test]
    fn test_validate_tail_offset() {
        assert!(validate_tail_offset(100, 16, 1000));
        assert!(!validate_tail_offset(10, 16, 1000));  // Too small
        assert!(!validate_tail_offset(990, 16, 1000)); // Too close to end
        assert!(!validate_tail_offset(1000, 16, 1000)); // At end
    }
}