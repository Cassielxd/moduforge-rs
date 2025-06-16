//! 编译器模块
//!
//! 该模块负责将抽象语法树(AST)转换为虚拟机可执行的操作码序列
//! 编译器将高级的表达式结构转换为低级的栈式操作指令

// 编译器核心实现模块
mod compiler;
// 编译错误定义模块
mod error;
// 操作码定义模块
mod opcode;

// 公开导出的类型和结构
pub use compiler::Compiler; // 编译器主结构
pub use error::CompilerError; // 编译错误类型
pub use opcode::{Compare, FetchFastTarget, Jump, Opcode}; // 操作码相关类型
