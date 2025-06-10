use nohash_hasher::BuildNoHashHasher;
use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::lexer::{ArithmeticOperator, ComparisonOperator, LogicalOperator, Operator};
use Associativity::{Left, Right};

/// 高性能哈希映射类型别名
/// 使用无哈希构建器优化操作符查找性能
type NoHasher = BuildNoHashHasher<Operator>;

/// 操作符结合性枚举
/// 定义操作符的结合方向，影响相同优先级操作符的求值顺序
#[derive(Debug, PartialEq)]
pub(crate) enum Associativity {
    Left,   // 左结合：a + b + c = (a + b) + c
    Right,  // 右结合：a ^ b ^ c = a ^ (b ^ c)
}

/// 解析器操作符结构体
/// 包含操作符的优先级和结合性信息
#[derive(Debug, PartialEq)]
pub(crate) struct ParserOperator {
    pub precedence: u8,             // 优先级（数值越大优先级越高）
    pub associativity: Associativity, // 结合性
}

/// 创建HashMap的宏
/// 提供简洁的语法来初始化HashMap
macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = hashmap!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::default();
            _map.reserve(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}

/// 二元操作符优先级和结合性映射表
/// 定义了所有二元操作符的优先级（数值越大优先级越高）和结合性
/// 
/// 优先级顺序（从低到高）：
/// - 10: 逻辑或 (or)
/// - 15: 逻辑与 (and)  
/// - 20: 比较操作符 (==, !=, <, >, <=, >=, in, not in)
/// - 30: 加减法 (+, -)
/// - 60: 乘除取模 (*, /, %)
/// - 70: 幂运算 (^) - 右结合
/// - 80: 空值合并 (??)
pub(crate) static BINARY_OPERATORS: Lazy<HashMap<Operator, ParserOperator, NoHasher>> = Lazy::new(
    || {
        hashmap! {
            // 逻辑操作符（最低优先级）
            Operator::Logical(LogicalOperator::Or) => ParserOperator { precedence: 10, associativity: Left },
            Operator::Logical(LogicalOperator::And) => ParserOperator { precedence: 15, associativity: Left },
            
            // 比较操作符
            Operator::Comparison(ComparisonOperator::Equal) => ParserOperator { precedence: 20, associativity: Left },
            Operator::Comparison(ComparisonOperator::NotEqual) => ParserOperator { precedence: 20, associativity: Left },
            Operator::Comparison(ComparisonOperator::LessThan) => ParserOperator { precedence: 20, associativity: Left },
            Operator::Comparison(ComparisonOperator::GreaterThan) => ParserOperator { precedence: 20, associativity: Left },
            Operator::Comparison(ComparisonOperator::LessThanOrEqual) => ParserOperator { precedence: 20, associativity: Left },
            Operator::Comparison(ComparisonOperator::GreaterThanOrEqual) => ParserOperator { precedence: 20, associativity: Left },
            Operator::Comparison(ComparisonOperator::NotIn) => ParserOperator { precedence: 20, associativity: Left },
            Operator::Comparison(ComparisonOperator::In) => ParserOperator { precedence: 20, associativity: Left },
            
            // 算术操作符
            Operator::Arithmetic(ArithmeticOperator::Add) => ParserOperator { precedence: 30, associativity: Left },
            Operator::Arithmetic(ArithmeticOperator::Subtract) => ParserOperator { precedence: 30, associativity: Left },
            Operator::Arithmetic(ArithmeticOperator::Multiply) => ParserOperator { precedence: 60, associativity: Left },
            Operator::Arithmetic(ArithmeticOperator::Divide) => ParserOperator { precedence: 60, associativity: Left },
            Operator::Arithmetic(ArithmeticOperator::Modulus) => ParserOperator { precedence: 60, associativity: Left },
            Operator::Arithmetic(ArithmeticOperator::Power) => ParserOperator { precedence: 70, associativity: Right }, // 右结合
            
            // 特殊操作符
            Operator::Logical(LogicalOperator::NullishCoalescing) => ParserOperator { precedence: 80, associativity: Left },
        }
    },
);

/// 一元操作符优先级和结合性映射表
/// 定义了所有一元操作符的优先级和结合性
/// 
/// 优先级顺序：
/// - 50: 逻辑非 (not)
/// - 200: 一元加减 (+, -) - 最高优先级
pub(crate) static UNARY_OPERATORS: Lazy<HashMap<Operator, ParserOperator, NoHasher>> = Lazy::new(
    || {
        hashmap! {
            // 逻辑非操作符
            Operator::Logical(LogicalOperator::Not) => ParserOperator { precedence: 50, associativity: Left },
            
            // 一元算术操作符（最高优先级）
            Operator::Arithmetic(ArithmeticOperator::Add) => ParserOperator { precedence: 200, associativity: Left },
            Operator::Arithmetic(ArithmeticOperator::Subtract) => ParserOperator { precedence: 200, associativity: Left },
        }
    },
);
