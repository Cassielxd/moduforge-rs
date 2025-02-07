use bincode::{config, Decode, Encode};


pub mod delta;
pub mod snapshot;

pub fn to_binary<T: Encode>(data: T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let json = bincode::encode_to_vec(&data,config::standard())?;
    Ok(json)
}

pub fn from_binary<T: Decode>(
    data: Vec<u8>,
) -> Result<T, Box<dyn std::error::Error>> {
    let d:T =bincode::decode_from_slice(&data,config::standard())?.0;
    Ok(d)
}
