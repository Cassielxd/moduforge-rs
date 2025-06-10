//! 区间类型模块
//! 
//! 提供区间（interval）类型的实现，支持数字区间和日期区间。
//! 支持开区间和闭区间的表示和操作。

use crate::lexer::Bracket;
use crate::variable::DynamicVariable;
use crate::vm::VmDate;
use crate::Variable;
use anyhow::anyhow;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde_json::Value;
use std::any::Any;
use std::fmt::{Display, Formatter};

/// 虚拟机区间类型
/// 
/// 表示一个区间范围，支持开区间和闭区间。
/// 可以包含数字或日期类型的边界。
#[derive(Debug, Clone)]
pub(crate) struct VmInterval {
    /// 左括号类型：决定左边界是否包含在区间内
    pub left_bracket: Bracket,
    /// 右括号类型：决定右边界是否包含在区间内
    pub right_bracket: Bracket,
    /// 左边界值
    pub left: VmIntervalData,
    /// 右边界值
    pub right: VmIntervalData,
}

impl DynamicVariable for VmInterval {
    fn type_name(&self) -> &'static str {
        "interval"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl VmInterval {
    /// 将数字区间转换为数组
    /// 
    /// 如果区间的两个边界都是数字，则将区间转换为包含所有整数的数组。
    /// 根据括号类型决定是否包含边界值。
    /// 
    /// # 返回值
    /// * `Some(Vec<Variable>)` - 包含区间内所有整数的数组
    /// * `None` - 如果区间包含非数字类型或转换失败
    pub fn to_array(&self) -> Option<Vec<Variable>> {
        let (left, right) = match (&self.left, &self.right) {
            (VmIntervalData::Number(l), VmIntervalData::Number(r)) => (*l, *r),
            _ => return None,
        };

        // 根据左括号类型确定起始值
        let start = match &self.left_bracket {
            Bracket::LeftParenthesis => left.to_i64()? + 1,  // 开区间：不包含左边界
            Bracket::LeftSquareBracket => left.to_i64()?,    // 闭区间：包含左边界
            _ => return None,
        };

        // 根据右括号类型确定结束值
        let end = match &self.right_bracket {
            Bracket::RightParenthesis => right.to_i64()? - 1, // 开区间：不包含右边界
            Bracket::RightSquareBracket => right.to_i64()?,   // 闭区间：包含右边界
            _ => return None,
        };

        // 生成区间内的所有整数
        let list = (start..=end)
            .map(|n| Variable::Number(Decimal::from(n)))
            .collect::<Vec<_>>();

        Some(list)
    }

    /// 检查值是否在区间内
    /// 
    /// 根据区间的括号类型和边界值，判断给定值是否包含在区间内。
    /// 支持开区间、闭区间以及混合区间的判断。
    /// 
    /// # 参数
    /// * `v` - 要检查的值
    /// 
    /// # 返回值
    /// * `Ok(true)` - 值在区间内
    /// * `Ok(false)` - 值不在区间内
    /// * `Err` - 不支持的括号类型或其他错误
    pub fn includes(&self, v: VmIntervalData) -> anyhow::Result<bool> {
        let mut is_open = false;
        let l = &self.left;
        let r = &self.right;

        // 检查左边界条件
        let first = match &self.left_bracket {
            Bracket::LeftParenthesis => l < &v,      // 开区间：左边界值必须小于v
            Bracket::LeftSquareBracket => l <= &v,   // 闭区间：左边界值可以等于v
            Bracket::RightParenthesis => {           // 开区间的另一种表示
                is_open = true;
                l > &v
            }
            Bracket::RightSquareBracket => {         // 闭区间的另一种表示
                is_open = true;
                l >= &v
            }
            _ => return Err(anyhow!("Unsupported bracket")),
        };

        // 检查右边界条件
        let second = match &self.right_bracket {
            Bracket::RightParenthesis => r > &v,     // 开区间：右边界值必须大于v
            Bracket::RightSquareBracket => r >= &v,  // 闭区间：右边界值可以等于v
            Bracket::LeftParenthesis => r < &v,      // 开区间的另一种表示
            Bracket::LeftSquareBracket => r <= &v,   // 闭区间的另一种表示
            _ => return Err(anyhow!("Unsupported bracket")),
        };

        // 根据区间类型计算最终结果
        let open_stmt = is_open && (first || second);    // 开区间逻辑
        let closed_stmt = !is_open && first && second;   // 闭区间逻辑

        Ok(open_stmt || closed_stmt)
    }
}

impl Display for VmInterval {
    /// 格式化区间为字符串表示
    /// 
    /// 格式：`左括号左值..右值右括号`
    /// 例如：`[1..10]`, `(0..5)`, `[2..8)`
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}..{}{}",
            self.left_bracket, self.left, self.right, self.right_bracket
        )
    }
}

/// 区间数据类型
/// 
/// 表示区间边界可以包含的数据类型
#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub(crate) enum VmIntervalData {
    /// 数字类型边界
    Number(Decimal),
    /// 日期类型边界
    Date(VmDate),
}

impl Display for VmIntervalData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VmIntervalData::Number(n) => write!(f, "{n}"),
            VmIntervalData::Date(d) => write!(f, "{d}"),
        }
    }
}
