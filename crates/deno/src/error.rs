use thiserror::Error;

/// Deno 集成相关错误类型
#[derive(Error, Debug)]
pub enum DenoError {
    #[error("Runtime error: {0}")]
    Runtime(#[from] anyhow::Error),

    #[error("JavaScript execution error: {0}")]
    JsExecution(String),

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("State error: {0}")]
    State(#[from] mf_state::error::StateError),

    #[error("Transform error: {0}")]
    Transform(#[from] mf_transform::TransformError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Deno 集成结果类型
pub type DenoResult<T> = Result<T, DenoError>;