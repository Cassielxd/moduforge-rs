//! 函数模块
//!
//! 提供表达式中可用的各种函数类型，包括内置函数、自定义函数、方法和已废弃函数

pub use crate::functions::date_method::DateMethod;
pub use crate::functions::defs::FunctionTypecheck;
pub use crate::functions::deprecated::DeprecatedFunction;
pub use crate::functions::internal::InternalFunction;
pub use crate::functions::method::{MethodKind, MethodRegistry};
pub use crate::functions::registry::FunctionRegistry;
pub use crate::functions::custom::CustomFunction;
pub use crate::functions::state_guard::{StateGuard, with_state_async};

use std::fmt::Display;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

pub mod arguments; // 函数参数处理
pub mod custom;
mod date_method; // 日期方法
pub mod defs; // 函数定义接口
mod deprecated; // 已废弃函数
pub mod internal; // 内置函数
mod method; // 方法注册表
pub(crate) mod registry; // 函数注册表
pub mod state_guard; // State 守卫模块

/// 函数类型枚举
///
/// 定义了表达式系统中所有可用的函数类型
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FunctionKind {
    /// 内置函数：系统预定义的标准函数
    Internal(InternalFunction),
    /// 已废弃函数：为向后兼容保留的旧函数
    Deprecated(DeprecatedFunction),
    /// 闭包函数：用于数组操作的特殊函数
    Closure(ClosureFunction),
    /// 自定义函数：用户定义的扩展函数
    Custom(CustomFunction),
}

impl TryFrom<&str> for FunctionKind {
    type Error = strum::ParseError;

    /// 从字符串解析函数类型
    ///
    /// 按优先级顺序尝试匹配：内置函数 > 已废弃函数 > 闭包函数 > 自定义函数
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        InternalFunction::try_from(value)
            .map(FunctionKind::Internal)
            .or_else(|_| {
                DeprecatedFunction::try_from(value)
                    .map(FunctionKind::Deprecated)
            })
            .or_else(|_| {
                ClosureFunction::try_from(value).map(FunctionKind::Closure)
            })
            .or_else(|_| {
                CustomFunction::try_from(value).map(FunctionKind::Custom)
            })
    }
}

impl Display for FunctionKind {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            FunctionKind::Internal(i) => write!(f, "{i}"),
            FunctionKind::Deprecated(d) => write!(f, "{d}"),
            FunctionKind::Closure(c) => write!(f, "{c}"),
            FunctionKind::Custom(c) => write!(f, "{c}"),
        }
    }
}

/// 闭包函数枚举
///
/// 定义了用于数组操作的特殊闭包函数
#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    Display,
    EnumString,
    EnumIter,
    IntoStaticStr,
    Clone,
    Copy,
)]
#[strum(serialize_all = "camelCase")]
pub enum ClosureFunction {
    /// 全部匹配：检查数组中所有元素是否都满足条件
    All,
    /// 无匹配：检查数组中是否没有元素满足条件
    None,
    /// 存在匹配：检查数组中是否有元素满足条件
    Some,
    /// 唯一匹配：检查数组中是否只有一个元素满足条件
    One,
    /// 过滤：返回满足条件的元素组成的新数组
    Filter,
    /// 映射：对每个元素应用函数，返回结果数组
    Map,
    /// 扁平映射：对每个元素应用函数并展平结果
    FlatMap,
    /// 计数：统计满足条件的元素数量
    Count,
}
