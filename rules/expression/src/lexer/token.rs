use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use nohash_hasher::IsEnabled;
use strum_macros::{Display, EnumIter, EnumString, FromRepr, IntoStaticStr};

/// 令牌结构体
/// 包含词法分析过程中识别出的令牌信息
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token<'a> {
    pub span: (u32, u32),    // 令牌在源代码中的位置范围（开始位置，结束位置）
    pub kind: TokenKind,     // 令牌的类型
    pub value: &'a str,      // 令牌的原始字符串值
}

/// 令牌类型枚举
/// 定义了表达式中可能出现的所有令牌类型
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display)]
pub enum TokenKind {
    Identifier(Identifier),                 // 标识符（变量名、关键字等）
    Boolean(bool),                          // 布尔值（true/false）
    Number,                                 // 数字
    QuotationMark(QuotationMark),          // 引号（单引号、双引号、反引号）
    Literal,                               // 字面量（字符串内容等）
    Operator(Operator),                    // 操作符
    Bracket(Bracket),                      // 括号
    TemplateString(TemplateString),        // 模板字符串相关标记
}

/// 特殊标识符枚举
/// 定义了表达式中的特殊标识符
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display, EnumString, IntoStaticStr)]
pub enum Identifier {
    #[strum(serialize = "$")]
    ContextReference,     // 上下文引用 $
    #[strum(serialize = "$root")]
    RootReference,        // 根引用 $root
    #[strum(serialize = "#")]
    CallbackReference,    // 回调引用 #
    #[strum(serialize = "null")]
    Null,                 // 空值 null
}

/// 引号类型枚举
/// 定义了不同类型的引号
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display, EnumString, IntoStaticStr)]
pub enum QuotationMark {
    #[strum(serialize = "'")]
    SingleQuote,          // 单引号 '
    #[strum(serialize = "\"")]
    DoubleQuote,          // 双引号 "
    #[strum(serialize = "`")]
    Backtick,             // 反引号 `（用于模板字符串）
}

/// 模板字符串标记枚举
/// 用于标识模板字符串中的表达式开始和结束
#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumString, IntoStaticStr)]
pub enum TemplateString {
    #[strum(serialize = "${")]
    ExpressionStart,      // 表达式开始标记 ${
    #[strum(serialize = "}")]
    ExpressionEnd,        // 表达式结束标记 }
}

impl Display for TemplateString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            TemplateString::ExpressionStart => ::core::fmt::Display::fmt("${", f),
            TemplateString::ExpressionEnd => ::core::fmt::Display::fmt("}}", f),
        }
    }
}

/// 操作符枚举
/// 定义了表达式中的各种操作符
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operator {
    Arithmetic(ArithmeticOperator),    // 算术操作符
    Logical(LogicalOperator),          // 逻辑操作符
    Comparison(ComparisonOperator),    // 比较操作符
    Range,                             // 范围操作符 ..
    Comma,                             // 逗号 ,
    Slice,                             // 切片操作符 :
    Dot,                               // 点操作符 .
    QuestionMark,                      // 问号 ?
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Arithmetic(a) => write!(f, "{a}"),
            Operator::Logical(l) => write!(f, "{l}"),
            Operator::Comparison(c) => write!(f, "{c}"),
            Operator::Range => write!(f, ".."),
            Operator::Comma => write!(f, ","),
            Operator::Slice => write!(f, ":"),
            Operator::Dot => write!(f, "."),
            Operator::QuestionMark => write!(f, "?"),
        }
    }
}

impl FromStr for Operator {
    type Err = strum::ParseError;

    /// 从字符串解析操作符
    /// 按优先级尝试解析不同类型的操作符
    fn from_str(operator: &str) -> Result<Self, Self::Err> {
        match operator {
            ".." => Ok(Operator::Range),
            "," => Ok(Operator::Comma),
            ":" => Ok(Operator::Slice),
            "." => Ok(Operator::Dot),
            "?" => Ok(Operator::QuestionMark),
            _ => ArithmeticOperator::try_from(operator)
                .map(Operator::Arithmetic)
                .or_else(|_| LogicalOperator::try_from(operator).map(Operator::Logical))
                .or_else(|_| ComparisonOperator::try_from(operator).map(Operator::Comparison)),
        }
    }
}

/// 算术操作符枚举
/// 定义了基本的算术运算操作符
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display, EnumString)]
pub enum ArithmeticOperator {
    #[strum(serialize = "+")]
    Add,          // 加法 +
    #[strum(serialize = "-")]
    Subtract,     // 减法 -
    #[strum(serialize = "*")]
    Multiply,     // 乘法 *
    #[strum(serialize = "/")]
    Divide,       // 除法 /
    #[strum(serialize = "%")]
    Modulus,      // 取模 %
    #[strum(serialize = "^")]
    Power,        // 幂运算 ^
}

/// 逻辑操作符枚举
/// 定义了逻辑运算操作符
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display, EnumString)]
pub enum LogicalOperator {
    #[strum(serialize = "and")]
    And,                    // 逻辑与 and
    #[strum(serialize = "or")]
    Or,                     // 逻辑或 or
    #[strum(serialize = "not", serialize = "!")]
    Not,                    // 逻辑非 not 或 !
    #[strum(serialize = "??")]
    NullishCoalescing,      // 空值合并操作符 ??
}

/// 比较操作符枚举
/// 定义了比较运算操作符
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display, EnumString)]
pub enum ComparisonOperator {
    #[strum(serialize = "==")]
    Equal,                  // 等于 ==
    #[strum(serialize = "!=")]
    NotEqual,               // 不等于 !=
    #[strum(serialize = "<")]
    LessThan,               // 小于 <
    #[strum(serialize = ">")]
    GreaterThan,            // 大于 >
    #[strum(serialize = "<=")]
    LessThanOrEqual,        // 小于等于 <=
    #[strum(serialize = ">=")]
    GreaterThanOrEqual,     // 大于等于 >=
    #[strum(serialize = "in")]
    In,                     // 包含 in
    #[strum(serialize = "not in")]
    NotIn,                  // 不包含 not in
}

/// 括号枚举
/// 定义了各种类型的括号
#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumString, IntoStaticStr, EnumIter, FromRepr)]
pub enum Bracket {
    #[strum(serialize = "(")]
    LeftParenthesis,        // 左圆括号 (
    #[strum(serialize = ")")]
    RightParenthesis,       // 右圆括号 )
    #[strum(serialize = "[")]
    LeftSquareBracket,      // 左方括号 [
    #[strum(serialize = "]")]
    RightSquareBracket,     // 右方括号 ]
    #[strum(serialize = "{")]
    LeftCurlyBracket,       // 左花括号 {
    #[strum(serialize = "}")]
    RightCurlyBracket,      // 右花括号 }
}

impl Display for Bracket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Bracket::LeftParenthesis => ::core::fmt::Display::fmt("(", f),
            Bracket::RightParenthesis => ::core::fmt::Display::fmt(")", f),
            Bracket::LeftSquareBracket => ::core::fmt::Display::fmt("[", f),
            Bracket::RightSquareBracket => ::core::fmt::Display::fmt("]", f),
            Bracket::LeftCurlyBracket => ::core::fmt::Display::fmt("{", f),
            Bracket::RightCurlyBracket => ::core::fmt::Display::fmt("}}", f),
        }
    }
}

impl Operator {
    /// 获取操作符的变体编号
    /// 用于哈希和比较操作，为每个操作符分配唯一的数字标识
    pub fn variant(&self) -> u8 {
        match &self {
            Operator::Arithmetic(a) => match a {
                ArithmeticOperator::Add => 1,
                ArithmeticOperator::Subtract => 2,
                ArithmeticOperator::Multiply => 3,
                ArithmeticOperator::Divide => 4,
                ArithmeticOperator::Modulus => 5,
                ArithmeticOperator::Power => 6,
            },
            Operator::Logical(l) => match l {
                LogicalOperator::And => 7,
                LogicalOperator::Or => 8,
                LogicalOperator::Not => 9,
                LogicalOperator::NullishCoalescing => 10,
            },
            Operator::Comparison(c) => match c {
                ComparisonOperator::Equal => 11,
                ComparisonOperator::NotEqual => 12,
                ComparisonOperator::LessThan => 13,
                ComparisonOperator::GreaterThan => 14,
                ComparisonOperator::LessThanOrEqual => 15,
                ComparisonOperator::GreaterThanOrEqual => 16,
                ComparisonOperator::In => 17,
                ComparisonOperator::NotIn => 18,
            },
            Operator::Range => 19,
            Operator::Comma => 20,
            Operator::Slice => 21,
            Operator::Dot => 22,
            Operator::QuestionMark => 23,
        }
    }
}

impl Hash for Operator {
    /// 为操作符实现哈希
    /// 使用变体编号作为哈希值
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(self.variant());
    }
}

// 启用nohash_hasher优化
impl IsEnabled for Operator {}
