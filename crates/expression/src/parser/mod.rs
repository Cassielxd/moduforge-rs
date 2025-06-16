//! 语法分析器模块
//!
//! 该模块负责将词法分析器产生的令牌序列转换为抽象语法树(AST)
//!
//! 提供两种专门的解析器变体：
//! - Standard：标准解析器，用于全面的表达式求值，可产生任何类型的结果
//! - Unary：一元解析器，专门用于真值测试，只产生布尔类型的结果

// 抽象语法树节点定义模块
mod ast;
// 操作符常量和优先级定义模块
mod constants;
// 错误类型定义模块
mod error;
// 解析器核心实现模块
mod parser;
// 解析结果类型定义模块
mod result;
// 标准解析器实现模块
mod standard;
// 一元解析器实现模块
mod unary;

// 公开导出的类型和结构
pub use ast::Node; // AST节点类型
pub use error::ParserError; // 解析错误类型
pub use parser::Parser; // 解析器结构
pub use result::{NodeMetadata, ParserResult}; // 解析结果和元数据类型
pub use standard::Standard; // 标准解析器
pub use unary::Unary; // 一元解析器
