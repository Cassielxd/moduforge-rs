use std::io;

#[derive(thiserror::Error, Debug)]
pub enum FileError {
    #[error("IO 错误: {0}")]
    Io(#[from] io::Error),
    #[error("文件头无效或格式错误")]
    BadHeader,
    #[error("记录过大: {0}")]
    RecordTooLarge(usize),
    #[error("记录为空")]
    EmptyRecord,
    #[error("CRC 校验失败，偏移量 {0}")]
    CrcMismatch(u64),
}

pub type Result<T> = std::result::Result<T, FileError>;
