use crate::functions::{FunctionKind, MethodKind};
use crate::lexer::Bracket;
use rust_decimal::Decimal;
use std::sync::Arc;
use strum_macros::Display;

/// 快速获取目标枚举
/// 用于优化成员访问的路径表示
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchFastTarget {
    Root,                // 根对象引用
    String(Arc<str>),    // 字符串属性名
    Number(u32),         // 数字索引
}

/// 虚拟机操作码枚举
/// 定义了虚拟机可执行的所有指令类型，采用栈式架构
#[derive(Debug, PartialEq, Eq, Clone, Display)]
pub enum Opcode {
    // 基本值入栈操作
    PushNull,                               // 将 null 值压入栈
    PushBool(bool),                         // 将布尔值压入栈
    PushString(Arc<str>),                   // 将字符串压入栈
    PushNumber(Decimal),                    // 将数字压入栈
    
    // 栈操作
    Pop,                                    // 弹出栈顶元素
    Flatten,                                // 展平数组（将嵌套数组合并）
    Join,                                   // 连接数组元素为字符串
    
    // 数据获取操作
    Fetch,                                  // 获取对象属性（从栈中获取对象和属性名）
    FetchRootEnv,                          // 获取根环境
    FetchEnv(Arc<str>),                    // 从环境中获取指定名称的值
    FetchFast(Vec<FetchFastTarget>),       // 快速获取（优化的属性访问路径）
    
    // 一元操作
    Negate,                                // 数值取负
    Not,                                   // 逻辑非
    
    // 比较操作
    Equal,                                 // 相等比较
    Jump(Jump, u32),                       // 条件或无条件跳转（跳转类型，跳转距离）
    In,                                    // 包含检查（检查元素是否在集合中）
    Compare(Compare),                      // 数值大小比较
    
    // 算术操作
    Add,                                   // 加法
    Subtract,                              // 减法
    Multiply,                              // 乘法
    Divide,                                // 除法
    Modulo,                                // 取模
    Exponent,                              // 幂运算
    
    // 数据结构操作
    Slice,                                 // 切片操作
    Array,                                 // 创建数组
    Object,                                // 创建对象
    
    // 循环和迭代相关
    Len,                                   // 获取长度
    IncrementIt,                           // 增加迭代器
    IncrementCount,                        // 增加计数器
    GetCount,                              // 获取计数值
    GetLen,                                // 获取长度值
    Pointer,                               // 获取当前指针值
    Begin,                                 // 开始循环/作用域
    End,                                   // 结束循环/作用域
    
    // 函数和方法调用
    CallFunction {
        kind: FunctionKind,                // 函数类型
        arg_count: u32,                    // 参数数量
    },
    CallMethod {
        kind: MethodKind,                  // 方法类型
        arg_count: u32,                    // 参数数量
    },
    
    // 区间操作
    Interval {
        left_bracket: Bracket,             // 左括号类型（开区间或闭区间）
        right_bracket: Bracket,            // 右括号类型（开区间或闭区间）
    },
}

/// 跳转类型枚举
/// 定义了不同的条件跳转和无条件跳转类型
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display)]
pub enum Jump {
    Forward,        // 向前跳转
    Backward,       // 向后跳转
    IfTrue,         // 条件为真时跳转
    IfFalse,        // 条件为假时跳转
    IfNotNull,      // 值不为null时跳转
    IfEnd,          // 到达结束条件时跳转
}

/// 比较操作类型枚举
/// 定义了数值比较的不同操作
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display)]
pub enum Compare {
    More,           // 大于
    Less,           // 小于
    MoreOrEqual,    // 大于等于
    LessOrEqual,    // 小于等于
}
