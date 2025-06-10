use thiserror::Error;

/// 语法分析错误类型
/// 定义了在语法分析过程中可能遇到的各种错误情况
#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum ParserError {
    /// 节点错误
    /// 当AST节点包含错误信息时触发，错误消息来自节点内部
    #[error("{0}")]
    NodeError(String),

    /// 解析未完成错误
    /// 当解析器无法完全处理所有令牌时触发
    #[error("Incomplete parser output")]
    Incomplete,
}
