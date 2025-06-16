//! 虚拟机辅助函数模块
//!
//! 提供日期时间解析、格式化和计算等辅助功能。
//! 支持多种日期时间格式的解析和时间单位的操作。

use crate::vm::error::{VMError, VMResult};
use chrono::{
    DateTime, Datelike, Days, NaiveDate, NaiveDateTime, NaiveTime, Timelike,
    Utc, Weekday,
};
use once_cell::sync::Lazy;

/// 零点时间常量：00:00:00
///
/// 用于将日期转换为日期时间时的默认时间部分
#[allow(clippy::unwrap_used)]
static ZERO_TIME: Lazy<NaiveTime> =
    Lazy::new(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());

// 日期时间格式常量
/// 完整日期时间格式：YYYY-MM-DD HH:MM:SS
static DATE_TIME: &str = "%Y-%m-%d %H:%M:%S";
/// 日期格式：YYYY-MM-DD
static DATE: &str = "%Y-%m-%d";
/// 时分秒格式：HH:MM:SS
static TIME_HMS: &str = "%H:%M:%S";
/// 时分格式：HH:MM
static TIME_HM: &str = "%H:%M";
/// 小时格式：HH
static TIME_H: &str = "%H";

/// 解析日期时间字符串
///
/// 支持多种格式的日期时间解析，包括：
/// - "now" 特殊值表示当前时间
/// - YYYY-MM-DD HH:MM:SS 格式
/// - YYYY-MM-DD 格式（自动添加00:00:00时间）
/// - RFC3339格式
///
/// # 参数
/// * `str` - 待解析的日期时间字符串
///
/// # 返回值
/// * `Ok(NaiveDateTime)` - 成功解析的日期时间
/// * `Err(VMError)` - 解析失败的错误信息
pub(crate) fn date_time(str: &str) -> VMResult<NaiveDateTime> {
    if str == "now" {
        return Ok(Utc::now().naive_utc());
    }

    NaiveDateTime::parse_from_str(str, DATE_TIME)
        .or(NaiveDate::parse_from_str(str, DATE)
            .map(|c| c.and_time(*ZERO_TIME)))
        .or(DateTime::parse_from_rfc3339(str).map(|dt| dt.naive_utc()))
        .map_err(|_| VMError::ParseDateTimeErr { timestamp: str.to_string() })
}

/// 解析时间字符串
///
/// 支持多种格式的时间解析，包括：
/// - "now" 特殊值表示当前时间的时间部分
/// - HH:MM:SS 格式
/// - HH:MM 格式
/// - HH 格式
/// - 从完整日期时间或RFC3339格式中提取时间部分
///
/// # 参数
/// * `str` - 待解析的时间字符串
///
/// # 返回值
/// * `Ok(NaiveTime)` - 成功解析的时间
/// * `Err(VMError)` - 解析失败的错误信息
pub(crate) fn time(str: &str) -> VMResult<NaiveTime> {
    let now = Utc::now();

    if str == "now" {
        return Ok(now.naive_utc().time());
    }

    return NaiveTime::parse_from_str(str, DATE_TIME)
        .or(NaiveTime::parse_from_str(str, TIME_HMS))
        .or(NaiveTime::parse_from_str(str, TIME_HM))
        .or(NaiveTime::parse_from_str(str, TIME_H))
        .or(DateTime::parse_from_rfc3339(str).map(|dt| dt.naive_utc().time()))
        .map_err(|_| VMError::ParseDateTimeErr { timestamp: str.to_string() });
}

/// 日期时间单位枚举
///
/// 定义了日期时间计算中使用的各种时间单位
pub(crate) enum DateUnit {
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
    /// 年
    Year,
}

impl TryFrom<&str> for DateUnit {
    type Error = VMError;

    /// 从字符串解析日期单位
    ///
    /// 支持多种表示形式：
    /// - 秒：s, second, seconds
    /// - 分钟：m, minute, minutes
    /// - 小时：h, hour, hours
    /// - 天：d, day, days
    /// - 周：w, week, weeks
    /// - 月：M, month, months
    /// - 年：y, year, years
    ///
    /// # 参数
    /// * `value` - 时间单位字符串
    ///
    /// # 返回值
    /// * `Ok(DateUnit)` - 成功解析的时间单位
    /// * `Err(VMError)` - 未知的时间单位
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "s" | "second" | "seconds" => Ok(Self::Second),
            "m" | "minute" | "minutes" => Ok(Self::Minute),
            "h" | "hour" | "hours" => Ok(Self::Hour),
            "d" | "day" | "days" => Ok(Self::Day),
            "w" | "week" | "weeks" => Ok(Self::Week),
            "M" | "month" | "months" => Ok(Self::Month),
            "y" | "year" | "years" => Ok(Self::Year),
            _ => Err(VMError::OpcodeErr {
                opcode: "DateUnit".into(),
                message: "Unknown date unit".into(),
            }),
        }
    }
}

/// 获取指定时间单位的开始时间
///
/// 根据指定的时间单位，将日期时间调整到该单位的开始时刻。
/// 例如：日期的开始是00:00:00，月份的开始是1号00:00:00等。
///
/// # 参数
/// * `date` - 输入的日期时间
/// * `unit` - 时间单位
///
/// # 返回值
/// * `Some(NaiveDateTime)` - 调整后的开始时间
/// * `None` - 调整失败（通常是日期无效）
pub(crate) fn date_time_start_of(
    date: NaiveDateTime,
    unit: DateUnit,
) -> Option<NaiveDateTime> {
    match unit {
        DateUnit::Second => Some(date),
        DateUnit::Minute => date.with_second(0),
        DateUnit::Hour => date.with_second(0)?.with_minute(0),
        DateUnit::Day => date.with_second(0)?.with_minute(0)?.with_hour(0),
        DateUnit::Week => {
            date.with_second(0)?.with_minute(0)?.with_hour(0)?.checked_sub_days(
                Days::new(date.weekday().num_days_from_monday() as u64),
            )
        },
        DateUnit::Month => {
            date.with_second(0)?.with_minute(0)?.with_hour(0)?.with_day0(0)
        },
        DateUnit::Year => date
            .with_second(0)?
            .with_minute(0)?
            .with_hour(0)?
            .with_day0(0)?
            .with_month0(0),
    }
}

/// 获取指定时间单位的结束时间
///
/// 根据指定的时间单位，将日期时间调整到该单位的结束时刻。
/// 例如：日期的结束是23:59:59，月份的结束是月末23:59:59等。
///
/// # 参数
/// * `date` - 输入的日期时间
/// * `unit` - 时间单位
///
/// # 返回值
/// * `Some(NaiveDateTime)` - 调整后的结束时间
/// * `None` - 调整失败（通常是日期无效）
pub(crate) fn date_time_end_of(
    date: NaiveDateTime,
    unit: DateUnit,
) -> Option<NaiveDateTime> {
    match unit {
        DateUnit::Second => Some(date),
        DateUnit::Minute => date.with_second(59),
        DateUnit::Hour => date.with_second(59)?.with_minute(59),
        DateUnit::Day => date.with_second(59)?.with_minute(59)?.with_hour(23),
        DateUnit::Week => date
            .with_second(59)?
            .with_minute(59)?
            .with_hour(23)?
            .checked_add_days(Days::new(
                Weekday::Sun as u64 - date.weekday() as u64,
            )),
        DateUnit::Month => date
            .with_second(59)?
            .with_minute(59)?
            .with_hour(23)?
            .with_day(get_month_days(&date)? as u32),
        DateUnit::Year => date
            .with_second(59)?
            .with_minute(59)?
            .with_hour(23)?
            .with_day(get_month_days(&date)? as u32)?
            .with_month0(11),
    }
}

/// 获取指定月份的天数
///
/// 计算给定日期所在月份的总天数，考虑闰年等因素。
///
/// # 参数
/// * `date` - 输入的日期时间
///
/// # 返回值
/// * `Some(i64)` - 该月的天数
/// * `None` - 计算失败
fn get_month_days(date: &NaiveDateTime) -> Option<i64> {
    Some(
        NaiveDate::from_ymd_opt(
            match date.month() {
                12 => date.year() + 1,
                _ => date.year(),
            },
            match date.month() {
                12 => 1,
                _ => date.month() + 1,
            },
            1,
        )?
        .signed_duration_since(NaiveDate::from_ymd_opt(
            date.year(),
            date.month(),
            1,
        )?)
        .num_days(),
    )
}
