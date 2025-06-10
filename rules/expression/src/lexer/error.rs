use thiserror::Error;

/// 词法分析错误类型
/// 定义了在词法分析过程中可能遇到的各种错误情况
#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum LexerError {
    /// 意外的符号错误
    /// 当遇到无法识别的字符时触发
    #[error("Unexpected symbol: {symbol} at ({}, {})", span.0, span.1)]
    UnexpectedSymbol { symbol: String, span: (u32, u32) },

    /// 不匹配的符号错误
    /// 当遇到语法不正确的字符时触发（如单独的括号等）
    #[error("Unmatched symbol: {symbol} at {position}")]
    UnmatchedSymbol { symbol: char, position: u32 },

    /// 意外的文件结束错误
    /// 当在需要更多字符时文件已结束时触发（如未关闭的字符串等）
    #[error("Unexpected EOF: {symbol} at {position}")]
    UnexpectedEof { symbol: char, position: u32 },
}

/// 词法分析结果类型别名
/// 简化错误处理的类型声明
pub(crate) type LexerResult<T> = Result<T, LexerError>;
