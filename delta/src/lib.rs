use serde::{Deserialize, Serialize};
use zstd::encode_all;

pub mod delta;
pub mod snapshot;

pub fn to_binary<T: Serialize>(data: T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let json = serde_json::to_vec(&data)?;
    Ok(encode_all(json.as_slice(), 3)?)
}

pub fn from_binary<T: for<'de> Deserialize<'de>>(
    data: Vec<u8>,
) -> Result<T, Box<dyn std::error::Error>> {
    let json = zstd::decode_all(data.as_slice())?;
    Ok(serde_json::from_slice(&json)?)
}
