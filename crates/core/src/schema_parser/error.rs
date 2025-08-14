use thiserror::Error;

/// XML Schema 解析错误类型
#[derive(Error, Debug)]
pub enum XmlSchemaError {
    #[error("XML 解析错误: {0}")]
    XmlParseError(#[from] quick_xml::Error),

    #[error("XML 反序列化错误: {0}")]
    DeserializeError(#[from] quick_xml::DeError),

    #[error("JSON 值解析错误: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("缺少必需的属性: {0}")]
    MissingAttribute(String),

    #[error("无效的节点定义: {0}")]
    InvalidNodeDefinition(String),

    #[error("无效的标记定义: {0}")]
    InvalidMarkDefinition(String),

    #[error("重复的节点名称: {0}")]
    DuplicateNodeName(String),

    #[error("重复的标记名称: {0}")]
    DuplicateMarkName(String),

    #[error("文件引用错误: {0}")]
    FileReferenceError(String),

    #[error("循环引用检测到: {0}")]
    CircularReference(String),

    #[error("文件不存在: {0}")]
    FileNotFound(String),

    #[error("相对路径解析错误: {0}")]
    PathResolutionError(String),
}

/// XML Schema 解析结果类型
pub type XmlSchemaResult<T> = Result<T, XmlSchemaError>;


