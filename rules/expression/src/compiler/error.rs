use thiserror::Error;

/// 编译器错误类型
/// 定义了在编译AST到操作码过程中可能遇到的各种错误情况
#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum CompilerError {
    /// 未知一元操作符错误
    /// 当遇到编译器不支持的一元操作符时触发
    #[error("未知的一元操作符: {operator}")]
    UnknownUnaryOperator { operator: String },

    /// 未知二元操作符错误
    /// 当遇到编译器不支持的二元操作符时触发
    #[error("未知二元操作符: {operator}")]
    UnknownBinaryOperator { operator: String },

    /// 函数参数未找到错误
    /// 当函数调用时指定索引的参数不存在时触发
    #[error("函数参数未找到: {function} 索引 {index}")]
    ArgumentNotFound { function: String, index: usize },

    /// 意外错误节点错误
    /// 当遇到包含错误信息的AST节点时触发
    #[error("意外错误节点")]
    UnexpectedErrorNode,

    /// 未知函数错误
    /// 当调用未定义的函数时触发
    #[error("未知函数: {name}")]
    UnknownFunction { name: String },

    /// 无效函数调用错误
    /// 当函数调用的参数数量或类型不正确时触发
    #[error("无效函数调用: {name}: {message}")]
    InvalidFunctionCall { name: String, message: String },

    /// 无效方法调用错误
    /// 当方法调用的参数数量或类型不正确时触发
    #[error("无效方法调用: {name}: {message}")]
    InvalidMethodCall { name: String, message: String },
}

/// 编译器结果类型别名
/// 简化编译过程中的错误处理
pub(crate) type CompilerResult<T> = Result<T, CompilerError>;
