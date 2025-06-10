//! 时间单位枚举模块
//! 
//! 定义日期时间计算中使用的各种时间单位，支持解析和转换功能。

use crate::variable::VariableType;
use std::rc::Rc;

/// 时间间隔单位枚举
/// 
/// 定义日期时间操作中支持的各种时间单位
#[derive(Debug, Clone, Copy)]
pub(crate) enum DurationUnit {
    /// 秒
    Second,
    /// 分钟
    Minute,
    /// 小时
    Hour,
    /// 天
    Day,
    /// 周
    Week,
    /// 月
    Month,
    /// 季度
    Quarter,
    /// 年
    Year,
}

impl DurationUnit {
    /// 获取时间单位的变量类型定义
    /// 
    /// 返回包含所有支持的时间单位字符串表示的枚举类型
    /// 
    /// # 返回值
    /// * `VariableType` - 包含所有时间单位别名的枚举变量类型
    pub fn variable_type() -> VariableType {
        VariableType::Enum(
            Some(Rc::from("DurationUnit")),
            vec![
                "seconds", "second", "secs", "sec", "s", "minutes", "minute", "min", "mins", "m",
                "hours", "hour", "hr", "hrs", "h", "days", "day", "d", "weeks", "week", "w",
                "months", "month", "mo", "M", "quarters", "quarter", "qtr", "q", "years", "year",
                "y",
            ]
            .into_iter()
            .map(Into::into)
            .collect(),
        )
    }

    /// 从字符串解析时间单位
    /// 
    /// 支持多种别名形式，包括单复数形式和缩写
    /// 
    /// # 参数
    /// * `unit` - 时间单位字符串
    /// 
    /// # 返回值
    /// * `Some(DurationUnit)` - 成功解析的时间单位
    /// * `None` - 无法识别的时间单位字符串
    pub fn parse(unit: &str) -> Option<Self> {
        match unit {
            "seconds" | "second" | "secs" | "sec" | "s" => Some(Self::Second),
            "minutes" | "minute" | "min" | "mins" | "m" => Some(Self::Minute),
            "hours" | "hour" | "hr" | "hrs" | "h" => Some(Self::Hour),
            "days" | "day" | "d" => Some(Self::Day),
            "weeks" | "week" | "w" => Some(Self::Week),
            "months" | "month" | "mo" | "M" => Some(Self::Month),
            "quarters" | "quarter" | "qtr" | "q" => Some(Self::Quarter),
            "years" | "year" | "y" => Some(Self::Year),
            _ => None,
        }
    }

    /// 获取时间单位对应的秒数
    /// 
    /// 固定时间单位（秒、分、时、天、周）可以精确转换为秒数，
    /// 而日历单位（月、季度、年）由于长度可变，返回None。
    /// 
    /// # 返回值
    /// * `Some(u64)` - 该时间单位对应的秒数
    /// * `None` - 日历单位无法精确转换为秒数
    pub fn as_secs(&self) -> Option<u64> {
        match self {
            DurationUnit::Second => Some(1),
            DurationUnit::Minute => Some(60),
            DurationUnit::Hour => Some(3600),
            DurationUnit::Day => Some(86_400),
            DurationUnit::Week => Some(86_400 * 7),
            // 日历单位长度可变，无法精确转换
            DurationUnit::Quarter => None,
            DurationUnit::Month => None,
            DurationUnit::Year => None,
        }
    }

    /// 获取时间单位对应的毫秒数
    /// 
    /// 基于秒数计算毫秒数，日历单位同样返回None。
    /// 
    /// # 返回值
    /// * `Some(f64)` - 该时间单位对应的毫秒数
    /// * `None` - 日历单位无法精确转换为毫秒数
    pub fn as_millis(&self) -> Option<f64> {
        self.as_secs().map(|s| s as f64 * 1000_f64)
    }
}
