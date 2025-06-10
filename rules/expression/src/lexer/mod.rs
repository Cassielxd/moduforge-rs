//! 词法分析器模块
//!
//! 该模块负责对字符串输入进行词法分析，将字符串转换为令牌(tokens)
//! 使用Strum库进行枚举类型的字符串转换和匹配

// 错误处理模块
mod error;
// 令牌定义模块
mod token;

// 字符类型宏定义模块
mod codes;
// 游标实现模块，用于字符串遍历
mod cursor;
// 词法分析器主要实现模块
mod lexer;

// 公开导出的类型和结构
pub use error::LexerError;
pub use lexer::Lexer;
pub use token::*;
