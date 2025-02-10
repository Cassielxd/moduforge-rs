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
    Ok(json)
}

/// 二进制反序列化
pub fn from_binary<T: Decode>(data: &Vec<u8>) -> Result<T, DecodeError> {
    let d: T = bincode::decode_from_slice(&data, config::standard().with_no_limit())?.0;
    Ok(d)
}
