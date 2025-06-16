use crate::functions::{FunctionKind, MethodKind};
use crate::lexer::{Bracket, Operator};
use rust_decimal::Decimal;
use std::cell::Cell;
use strum_macros::IntoStaticStr;
use thiserror::Error;

/// 抽象语法树节点枚举
/// 定义了表达式中所有可能的节点类型
#[derive(Debug, PartialEq, Clone, IntoStaticStr)]
pub enum Node<'a> {
    /// 空值节点
    Null,
    /// 布尔值节点
    Bool(bool),
    /// 数字节点（使用高精度十进制）
    Number(Decimal),
    /// 字符串节点
    String(&'a str),
    /// 模板字符串节点（包含多个子节点的数组）
    TemplateString(&'a [&'a Node<'a>]),
    /// 指针节点（回调引用 #）
    Pointer,
    /// 数组节点
    Array(&'a [&'a Node<'a>]),
    /// 对象节点（键值对数组）
    Object(&'a [(&'a Node<'a>, &'a Node<'a>)]),
    /// 标识符节点
    Identifier(&'a str),
    /// 闭包节点
    Closure(&'a Node<'a>),
    /// 括号表达式节点
    Parenthesized(&'a Node<'a>),
    /// 根节点（$ 引用）
    Root,
    /// 成员访问节点（对象.属性 或 对象["属性"]）
    Member {
        node: &'a Node<'a>,     // 被访问的对象
        property: &'a Node<'a>, // 属性名
    },
    /// 切片节点（数组[from:to]）
    Slice {
        node: &'a Node<'a>,         // 被切片的对象
        from: Option<&'a Node<'a>>, // 开始位置（可选）
        to: Option<&'a Node<'a>>,   // 结束位置（可选）
    },
    /// 区间节点（[a, b] 或 (a, b) 等）
    Interval {
        left: &'a Node<'a>,     // 左边界
        right: &'a Node<'a>,    // 右边界
        left_bracket: Bracket,  // 左括号类型
        right_bracket: Bracket, // 右括号类型
    },
    /// 条件表达式节点（三元操作符 condition ? true_expr : false_expr）
    Conditional {
        condition: &'a Node<'a>, // 条件表达式
        on_true: &'a Node<'a>,   // 条件为真时的表达式
        on_false: &'a Node<'a>,  // 条件为假时的表达式
    },
    /// 一元操作节点（如 -x, !x, +x）
    Unary {
        node: &'a Node<'a>, // 操作数
        operator: Operator, // 操作符
    },
    /// 二元操作节点（如 x + y, x == y）
    Binary {
        left: &'a Node<'a>,  // 左操作数
        operator: Operator,  // 操作符
        right: &'a Node<'a>, // 右操作数
    },
    /// 函数调用节点
    FunctionCall {
        kind: FunctionKind,            // 函数类型
        arguments: &'a [&'a Node<'a>], // 参数列表
    },
    /// 方法调用节点
    MethodCall {
        kind: MethodKind,              // 方法类型
        this: &'a Node<'a>,            // 调用对象（this）
        arguments: &'a [&'a Node<'a>], // 参数列表
    },
    /// 错误节点（包含解析错误信息）
    Error {
        node: Option<&'a Node<'a>>, // 可选的关联节点
        error: AstNodeError<'a>,    // 错误信息
    },
}

impl<'a> Node<'a> {
    /// 遍历AST节点
    /// 对每个节点（包括子节点）执行指定的函数
    pub fn walk<F>(
        &self,
        mut func: F,
    ) where
        F: FnMut(&Self) + Clone,
    {
        // 先对当前节点执行函数
        {
            func(self);
        };

        // 然后递归遍历子节点
        match self {
            // 叶子节点：无子节点
            Node::Null => {},
            Node::Bool(_) => {},
            Node::Number(_) => {},
            Node::String(_) => {},
            Node::Pointer => {},
            Node::Identifier(_) => {},
            Node::Root => {},

            // 错误节点：可能包含一个子节点
            Node::Error { node, .. } => {
                if let Some(n) = node {
                    n.walk(func.clone())
                }
            },

            // 包含多个子节点的节点
            Node::TemplateString(parts) => {
                parts.iter().for_each(|n| n.walk(func.clone()))
            },
            Node::Array(parts) => {
                parts.iter().for_each(|n| n.walk(func.clone()))
            },
            Node::Object(obj) => obj.iter().for_each(|(k, v)| {
                k.walk(func.clone());
                v.walk(func.clone());
            }),

            // 包含单个子节点的节点
            Node::Closure(closure) => closure.walk(func.clone()),
            Node::Parenthesized(c) => c.walk(func.clone()),

            // 包含两个子节点的节点
            Node::Member { node, property } => {
                node.walk(func.clone());
                property.walk(func.clone());
            },
            Node::Slice { node, to, from } => {
                node.walk(func.clone());
                if let Some(to) = to {
                    to.walk(func.clone());
                }
                if let Some(from) = from {
                    from.walk(func.clone());
                }
            },
            Node::Interval { left, right, .. } => {
                left.walk(func.clone());
                right.walk(func.clone());
            },

            // 一元操作节点
            Node::Unary { node, .. } => {
                node.walk(func);
            },

            // 二元操作节点
            Node::Binary { left, right, .. } => {
                left.walk(func.clone());
                right.walk(func.clone());
            },

            // 函数调用节点
            Node::FunctionCall { arguments, .. } => {
                arguments.iter().for_each(|n| n.walk(func.clone()));
            },

            // 方法调用节点
            Node::MethodCall { this, arguments, .. } => {
                this.walk(func.clone());
                arguments.iter().for_each(|n| n.walk(func.clone()));
            },

            // 条件表达式节点
            Node::Conditional { on_true, condition, on_false } => {
                condition.walk(func.clone());
                on_true.walk(func.clone());
                on_false.walk(func.clone());
            },
        };
    }

    /// 查找AST中的第一个错误
    /// 返回第一个遇到的错误节点中的错误信息
    pub fn first_error(&self) -> Option<AstNodeError> {
        let error_cell = Cell::new(None);
        self.walk(|n| {
            if let Node::Error { error, .. } = n {
                error_cell.set(Some(error.clone()))
            }
        });

        error_cell.into_inner()
    }

    /// 检查AST是否包含错误节点
    pub fn has_error(&self) -> bool {
        self.first_error().is_some()
    }

    /// 获取节点的位置范围
    /// 只有错误节点才有位置信息
    pub(crate) fn span(&self) -> Option<(u32, u32)> {
        match self {
            Node::Error { error, .. } => match error {
                AstNodeError::UnknownBuiltIn { span, .. } => Some(span.clone()),
                AstNodeError::UnknownMethod { span, .. } => Some(span.clone()),
                AstNodeError::UnexpectedIdentifier { span, .. } => {
                    Some(span.clone())
                },
                AstNodeError::UnexpectedToken { span, .. } => {
                    Some(span.clone())
                },
                AstNodeError::InvalidNumber { span, .. } => Some(span.clone()),
                AstNodeError::InvalidBoolean { span, .. } => Some(span.clone()),
                AstNodeError::InvalidProperty { span, .. } => {
                    Some(span.clone())
                },
                AstNodeError::MissingToken { position, .. } => {
                    Some((*position as u32, *position as u32))
                },
                AstNodeError::Custom { span, .. } => Some(span.clone()),
            },
            _ => None,
        }
    }
}

/// AST节点错误枚举
/// 定义了AST节点中可能出现的各种错误类型
#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum AstNodeError<'a> {
    /// 未知内置函数错误
    #[error("Unknown function `{name}` at ({}, {})", span.0, span.1)]
    UnknownBuiltIn { name: &'a str, span: (u32, u32) },

    /// 未知方法错误
    #[error("Unknown method `{name}` at ({}, {})", span.0, span.1)]
    UnknownMethod { name: &'a str, span: (u32, u32) },

    /// 意外标识符错误
    #[error("Unexpected identifier: {received} at ({}, {}); Expected {expected}.", span.0, span.1)]
    UnexpectedIdentifier {
        received: &'a str, // 实际收到的标识符
        expected: &'a str, // 期望的标识符
        span: (u32, u32),  // 位置范围
    },

    /// 意外令牌错误
    #[error("Unexpected token: {received} at ({}, {}); Expected {expected}.", span.0, span.1)]
    UnexpectedToken {
        received: &'a str, // 实际收到的令牌
        expected: &'a str, // 期望的令牌
        span: (u32, u32),  // 位置范围
    },

    /// 无效数字错误
    #[error("Invalid number: {number} at ({}, {})", span.0, span.1)]
    InvalidNumber { number: &'a str, span: (u32, u32) },

    /// 无效布尔值错误
    #[error("Invalid boolean: {boolean} at ({}, {})", span.0, span.1)]
    InvalidBoolean { boolean: &'a str, span: (u32, u32) },

    /// 无效属性错误
    #[error("Invalid property: {property} at ({}, {})", span.0, span.1)]
    InvalidProperty { property: &'a str, span: (u32, u32) },

    /// 缺少期望令牌错误
    #[error("Missing expected token: {expected} at {position}")]
    MissingToken { expected: &'a str, position: usize },

    /// 自定义错误
    #[error("{message} at ({}, {})", span.0, span.1)]
    Custom { message: &'a str, span: (u32, u32) },
}
