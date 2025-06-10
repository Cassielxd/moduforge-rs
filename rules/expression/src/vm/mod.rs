//! 虚拟机模块 - 操作码执行引擎
//!
//! VM (虚拟机) 模块负责执行编译器生成的机器可读操作码。
//! 提供完整的表达式运行时环境，包括栈管理、变量操作、函数调用等功能。
//! 
//! ## 主要组件
//! 
//! - **VM**: 主虚拟机执行器，管理栈和作用域
//! - **VMError**: 虚拟机执行过程中的错误类型  
//! - **VmDate**: 日期时间类型和相关操作
//! - **helpers**: 日期时间解析和操作辅助函数
//! - **interval**: 区间类型支持

pub use error::VMError;
pub use vm::VM;

pub(crate) mod date;    // 日期时间处理模块
mod error;              // 虚拟机错误定义
pub(crate) mod helpers; // 辅助函数集合
mod interval;           // 区间类型实现
mod vm;                 // 虚拟机核心实现

pub(crate) use date::VmDate;
