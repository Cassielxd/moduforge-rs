//! 虚拟机错误类型定义
//! 
//! 定义了虚拟机执行过程中可能遇到的各种错误类型

use thiserror::Error;

/// 虚拟机错误枚举
/// 
/// 包含虚拟机运行时可能出现的所有错误类型
#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum VMError {
    /// 操作码执行错误
    /// 
    /// 当特定操作码执行失败时产生，包含操作码名称和错误信息
    #[error("操作码 {opcode}: {message}")]
    OpcodeErr { 
        /// 操作码名称
        opcode: String, 
        /// 错误信息
        message: String 
    },

    /// 操作码索引越界错误
    /// 
    /// 当程序计数器指向无效的操作码位置时产生
    #[error("操作码越界")]
    OpcodeOutOfBounds { 
        /// 无效的索引位置
        index: usize, 
        /// 当前字节码的调试信息
        bytecode: String 
    },

    /// 栈操作越界错误
    /// 
    /// 当尝试从空栈弹出元素或栈索引无效时产生
    #[error("栈越界")]
    StackOutOfBounds { 
        /// 当前栈状态的调试信息
        stack: String 
    },

    /// 日期时间解析错误
    /// 
    /// 当无法解析日期时间字符串时产生
    #[error("解析日期时间失败")]
    ParseDateTimeErr { 
        /// 导致解析失败的时间戳字符串
        timestamp: String 
    },

    /// 数字转换错误
    /// 
    /// 当数字类型转换失败时产生
    #[error("数字转换错误")]
    NumberConversionError,
}

/// 虚拟机操作结果类型
/// 
/// 所有虚拟机操作的标准返回类型，成功时返回T，失败时返回VMError
pub(crate) type VMResult<T> = Result<T, VMError>;
