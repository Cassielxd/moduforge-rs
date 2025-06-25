use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransmissionError {
    #[error("Yrs error: {0}")]
    YrsError(String),
    
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Room not found: {0}")]
    RoomNotFound(String),
    
    #[error("Client not found: {0}")]
    ClientNotFound(String),
    
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, TransmissionError>; 