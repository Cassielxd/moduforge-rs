use thiserror::Error;

/// 容器错误类型
#[derive(Error, Debug)]
pub enum ContainerError {
    #[error("组件未找到: {name}")]
    ComponentNotFound { name: String },
    
    #[error("循环依赖检测到: {components:?}")]
    CircularDependency { components: Vec<String> },
    
    #[error("组件创建失败: {name}, 原因: {source}")]
    ComponentCreationFailed {
        name: String,
        #[source]
        source: anyhow::Error,
    },
    
    #[error("依赖解析失败: {name}, 缺少依赖: {dependency}")]
    DependencyResolutionFailed {
        name: String,
        dependency: String,
    },
    
    #[error("生命周期管理错误: {message}")]
    LifecycleError { message: String },
    
    #[error("容器初始化失败: {source}")]
    InitializationFailed {
        #[source]
        source: anyhow::Error,
    },
    
    #[error("组件已存在: {name}")]
    ComponentAlreadyExists { name: String },
    
    #[error("类型不匹配: 期望 {expected}, 实际 {actual}")]
    TypeMismatch { expected: String, actual: String },
}

pub type ContainerResult<T> = Result<T, ContainerError>;