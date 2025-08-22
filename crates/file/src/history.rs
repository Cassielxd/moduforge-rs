use std::io;
use serde::{Deserialize, Serialize};

// 步骤帧：type_id 表示类型，data 为该类型的序列化字节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeWrapper {
    pub type_id: String,
    pub data: Vec<u8>,
}

// 编码步骤帧；可选 zstd 压缩
pub fn encode_history_frames(
    frames: &[TypeWrapper],
    compress: bool,
) -> io::Result<Vec<u8>> {
    let bytes =
        bincode::serde::encode_to_vec(frames, bincode::config::standard())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    if compress {
        Ok(zstd::stream::encode_all(&bytes[..], 1)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?)
    } else {
        Ok(bytes)
    }
}

// 解码步骤帧；如 compressed 为真先解压
pub fn decode_history_frames(
    bytes: &[u8],
    compressed: bool,
) -> io::Result<Vec<TypeWrapper>> {
    let raw = if compressed {
        zstd::stream::decode_all(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    } else {
        bytes.to_vec()
    };
    let (frames, _) = bincode::serde::decode_from_slice::<Vec<TypeWrapper>, _>(
        &raw,
        bincode::config::standard(),
    )
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(frames)
}
