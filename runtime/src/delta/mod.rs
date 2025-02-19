use bincode::{
    config,
    error::{DecodeError, EncodeError},
    Decode, Encode,
};

pub mod delta;
pub mod snapshot;
/// 二进制序列化
pub fn to_binary<T: Encode>(data: T) -> Result<Vec<u8>, EncodeError> {
    let json = bincode::encode_to_vec(&data, config::standard().with_no_limit())?;
    let compressed = zstd::encode_all(json.as_slice(), 3).unwrap();
    Ok(compressed)
}

/// 二进制反序列化
pub fn from_binary<T: Decode>(data: &Vec<u8>) -> Result<T, DecodeError> {
    let decompressed = zstd::decode_all(data.as_slice()).unwrap();
    let d: T = bincode::decode_from_slice(&decompressed, config::standard().with_no_limit())?.0;
    Ok(d)
}
