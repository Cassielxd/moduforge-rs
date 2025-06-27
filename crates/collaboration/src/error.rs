use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransmissionError {
    #[error("Yrs 操作 错误: {0}")]
    YrsError(String),

    #[error("Yrs 编码/解码 错误: {0}")]
    YrsCodecError(#[from] yrs::encoding::read::Error),

    #[error("WebSocket 错误: {0}")]
    WebSocketError(String),

    #[error("序列化 错误: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("房间 不存在: {0}")]
    RoomNotFound(String),

    #[error("客户端 不存在: {0}")]
    ClientNotFound(String),

    #[error("同步 错误: {0}")]
    SyncError(String),

    #[error("其他 错误: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, TransmissionError>;
