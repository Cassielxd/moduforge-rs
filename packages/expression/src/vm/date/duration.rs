//! 时间间隔结构体模块
//!
//! 定义时间间隔的表示和操作，支持复合时间间隔（年+月+秒）。

use crate::vm::date::duration_parser::{DurationParseError, DurationParser};
use crate::vm::date::duration_unit::DurationUnit;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use std::ops::Neg;

/// 时间间隔结构体
///
/// 表示一个复合时间间隔，可以同时包含年、月和秒的组合。
/// 这种设计允许精确处理不同长度的日历单位。
#[derive(Debug, Clone, Default)]
pub(crate) struct Duration {
    /// 秒数部分（包含秒、分、时、天、周）
    pub seconds: i64,
    /// 月数部分
    pub months: i32,
    /// 年数部分
    pub years: i32,
}

impl Duration {
    /// 从字符串解析时间间隔
    ///
    /// 支持类似"1year 2months 3days 4hours 5minutes 6seconds"的格式
    ///
    /// # 参数
    /// * `s` - 时间间隔字符串
    ///
    /// # 返回值
    /// * `Ok(Duration)` - 成功解析的时间间隔
    /// * `Err(DurationParseError)` - 解析失败的错误
    pub fn parse(s: &str) -> Result<Self, DurationParseError> {
        DurationParser {
            iter: s.chars(),
            src: s,
            duration: Duration::default(),
        }
        .parse()
    }

    /// 从数值和时间单位创建时间间隔
    ///
    /// 根据时间单位类型，将数值转换为相应的时间间隔。
    /// 固定单位转换为秒数，日历单位保持原有精度。
    ///
    /// # 参数
    /// * `n` - 数值
    /// * `unit` - 时间单位
    ///
    /// # 返回值
    /// * `Some(Duration)` - 成功创建的时间间隔
    /// * `None` - 转换失败（通常是数值溢出）
    pub fn from_unit(
        n: Decimal,
        unit: DurationUnit,
    ) -> Option<Self> {
        // 固定时间单位：转换为秒数
        if let Some(secs) = unit.as_secs() {
            return Some(Self {
                seconds: n.checked_mul(Decimal::from_u64(secs)?)?.to_i64()?,
                ..Default::default()
            });
        };

        // 日历时间单位：保持原有精度
        match unit {
            DurationUnit::Month => {
                Some(Duration { months: n.to_i32()?, ..Default::default() })
            },
            DurationUnit::Quarter => Some(Duration {
                months: n.to_i32()? * 3, // 1季度 = 3个月
                ..Default::default()
            }),
            DurationUnit::Year => {
                Some(Duration { years: n.to_i32()?, ..Default::default() })
            },
            _ => None,
        }
    }

    /// 取时间间隔的相反值
    ///
    /// 将所有分量都取负值，用于时间减法操作
    ///
    /// # 返回值
    /// * `Duration` - 相反的时间间隔
    pub fn negate(self) -> Self {
        Self {
            years: self.years.neg(),
            months: self.months.neg(),
            seconds: self.seconds.neg(),
        }
    }

    /// 创建一天的时间间隔
    ///
    /// 常用的便捷方法，创建24小时的时间间隔
    ///
    /// # 返回值
    /// * `Duration` - 一天的时间间隔
    pub fn day() -> Self {
        Self {
            seconds: DurationUnit::Day.as_secs().unwrap_or_default() as i64,
            ..Default::default()
        }
    }
}
