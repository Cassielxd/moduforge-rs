//! 已废弃函数模块
//! 
//! 包含为了向后兼容而保留的已废弃函数。
//! 这些函数在新版本中已被更好的替代方案取代，不建议在新代码中使用。

use crate::functions::arguments::Arguments;
use crate::functions::defs::{FunctionDefinition, FunctionSignature, StaticFunction};
use crate::vm::helpers::{date_time, date_time_end_of, date_time_start_of, time};
use crate::Variable as V;
use anyhow::{anyhow, Context};
use chrono::{Datelike, NaiveDateTime, Timelike};
use rust_decimal::prelude::ToPrimitive;
use std::rc::Rc;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

/// 已废弃函数枚举
/// 
/// 定义了所有已废弃的函数，这些函数保留用于向后兼容
#[derive(Debug, PartialEq, Eq, Hash, Display, EnumString, EnumIter, IntoStaticStr, Clone, Copy)]
#[strum(serialize_all = "camelCase")]
pub enum DeprecatedFunction {
    /// 日期解析函数（已废弃，请使用d()函数）
    Date,
    /// 时间解析函数（已废弃，请使用d()函数的时间组件方法）
    Time,
    /// 持续时间解析函数（已废弃，请使用字符串字面量）
    Duration,
    /// 年份获取函数（已废弃，请使用date.year()方法）
    Year,
    /// 星期几获取函数（已废弃，请使用date.weekday()方法）
    DayOfWeek,
    /// 月中的天获取函数（已废弃，请使用date.day()方法）
    DayOfMonth,
    /// 年中的天获取函数（已废弃，请使用date.dayOfYear()方法）
    DayOfYear,
    /// 年中的周获取函数（已废弃，请使用date.week()方法）
    WeekOfYear,
    /// 月份获取函数（已废弃，请使用date.month()方法）
    MonthOfYear,
    /// 月份字符串获取函数（已废弃，请使用date.format()方法）
    MonthString,
    /// 日期字符串获取函数（已废弃，请使用date.format()方法）
    DateString,
    /// 星期字符串获取函数（已废弃，请使用date.format()方法）
    WeekdayString,
    /// 开始时间函数（已废弃，请使用date.startOf()方法）
    StartOf,
    /// 结束时间函数（已废弃，请使用date.endOf()方法）
    EndOf,
}

impl From<&DeprecatedFunction> for Rc<dyn FunctionDefinition> {
    /// 将已废弃函数枚举转换为函数定义
    /// 
    /// 为每个已废弃函数创建相应的函数定义，保持向后兼容性
    fn from(value: &DeprecatedFunction) -> Self {
        use crate::variable::VariableType as VT;
        use DeprecatedFunction as DF;

        let s: Rc<dyn FunctionDefinition> = match value {
            // 日期解析：接受任意类型输入，返回数字时间戳
            DF::Date => Rc::new(StaticFunction {
                implementation: Rc::new(imp::parse_date),
                signature: FunctionSignature::single(VT::Any, VT::Number),
            }),

            // 时间解析：接受任意类型输入，返回数字秒数
            DF::Time => Rc::new(StaticFunction {
                implementation: Rc::new(imp::parse_time),
                signature: FunctionSignature::single(VT::Any, VT::Number),
            }),

            // 持续时间解析：接受任意类型输入，返回数字秒数
            DF::Duration => Rc::new(StaticFunction {
                implementation: Rc::new(imp::parse_duration),
                signature: FunctionSignature::single(VT::Any, VT::Number),
            }),

            // 年份获取：接受任意类型输入，返回数字年份
            DF::Year => Rc::new(StaticFunction {
                implementation: Rc::new(imp::year),
                signature: FunctionSignature::single(VT::Any, VT::Number),
            }),

            // 星期几获取：接受任意类型输入，返回数字（1-7）
            DF::DayOfWeek => Rc::new(StaticFunction {
                implementation: Rc::new(imp::day_of_week),
                signature: FunctionSignature::single(VT::Any, VT::Number),
            }),

            // 月中的天获取：接受任意类型输入，返回数字（1-31）
            DF::DayOfMonth => Rc::new(StaticFunction {
                implementation: Rc::new(imp::day_of_month),
                signature: FunctionSignature::single(VT::Any, VT::Number),
            }),

            // 年中的天获取：接受任意类型输入，返回数字（1-366）
            DF::DayOfYear => Rc::new(StaticFunction {
                implementation: Rc::new(imp::day_of_year),
                signature: FunctionSignature::single(VT::Any, VT::Number),
            }),

            // 年中的周获取：接受任意类型输入，返回数字（1-53）
            DF::WeekOfYear => Rc::new(StaticFunction {
                implementation: Rc::new(imp::week_of_year),
                signature: FunctionSignature::single(VT::Any, VT::Number),
            }),

            // 月份获取：接受任意类型输入，返回数字（1-12）
            DF::MonthOfYear => Rc::new(StaticFunction {
                implementation: Rc::new(imp::month_of_year),
                signature: FunctionSignature::single(VT::Any, VT::Number),
            }),

            // 月份字符串获取：接受任意类型输入，返回字符串
            DF::MonthString => Rc::new(StaticFunction {
                implementation: Rc::new(imp::month_string),
                signature: FunctionSignature::single(VT::Any, VT::String),
            }),

            // 日期字符串获取：接受任意类型输入，返回字符串
            DF::DateString => Rc::new(StaticFunction {
                implementation: Rc::new(imp::date_string),
                signature: FunctionSignature::single(VT::Any, VT::String),
            }),

            // 星期字符串获取：接受任意类型输入，返回字符串
            DF::WeekdayString => Rc::new(StaticFunction {
                implementation: Rc::new(imp::weekday_string),
                signature: FunctionSignature::single(VT::Any, VT::String),
            }),

            // 开始时间：接受日期和时间单位字符串，返回数字时间戳
            DF::StartOf => Rc::new(StaticFunction {
                implementation: Rc::new(imp::start_of),
                signature: FunctionSignature {
                    parameters: vec![VT::Any, VT::String],
                    return_type: VT::Number,
                },
            }),

            // 结束时间：接受日期和时间单位字符串，返回数字时间戳
            DF::EndOf => Rc::new(StaticFunction {
                implementation: Rc::new(imp::end_of),
                signature: FunctionSignature {
                    parameters: vec![VT::Any, VT::String],
                    return_type: VT::Number,
                },
            }),
        };

        s
    }
}

mod imp {
    use super::*;
    use crate::vm::helpers::DateUnit;

    fn __internal_convert_datetime(timestamp: &V) -> anyhow::Result<NaiveDateTime> {
        timestamp
            .try_into()
            .context("Failed to convert value to date time")
    }

    pub fn parse_date(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;

        let ts = match a {
            V::String(a) => {
                let dt = date_time(a.as_ref())?;
                #[allow(deprecated)]
                dt.timestamp()
            }
            V::Number(a) => a.to_i64().context("Number overflow")?,
            _ => return Err(anyhow!("Unsupported type for date function")),
        };

        Ok(V::Number(ts.into()))
    }

    pub fn parse_time(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;

        let ts = match a {
            V::String(a) => time(a.as_ref())?.num_seconds_from_midnight(),
            V::Number(a) => a.to_u32().context("Number overflow")?,
            _ => return Err(anyhow!("Unsupported type for time function")),
        };

        Ok(V::Number(ts.into()))
    }

    pub fn parse_duration(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;

        let dur = match a {
            V::String(a) => humantime::parse_duration(a.as_ref())?.as_secs(),
            V::Number(n) => n.to_u64().context("Number overflow")?,
            _ => return Err(anyhow!("Unsupported type for duration function")),
        };

        Ok(V::Number(dur.into()))
    }

    pub fn year(args: Arguments) -> anyhow::Result<V> {
        let timestamp = args.var(0)?;
        let time = __internal_convert_datetime(&timestamp)?;
        Ok(V::Number(time.year().into()))
    }

    pub fn day_of_week(args: Arguments) -> anyhow::Result<V> {
        let timestamp = args.var(0)?;
        let time = __internal_convert_datetime(&timestamp)?;
        Ok(V::Number(time.weekday().number_from_monday().into()))
    }

    pub fn day_of_month(args: Arguments) -> anyhow::Result<V> {
        let timestamp = args.var(0)?;
        let time = __internal_convert_datetime(&timestamp)?;
        Ok(V::Number(time.day().into()))
    }

    pub fn day_of_year(args: Arguments) -> anyhow::Result<V> {
        let timestamp = args.var(0)?;
        let time = __internal_convert_datetime(&timestamp)?;
        Ok(V::Number(time.ordinal().into()))
    }

    pub fn week_of_year(args: Arguments) -> anyhow::Result<V> {
        let timestamp = args.var(0)?;
        let time = __internal_convert_datetime(&timestamp)?;
        Ok(V::Number(time.iso_week().week().into()))
    }

    pub fn month_of_year(args: Arguments) -> anyhow::Result<V> {
        let timestamp = args.var(0)?;
        let time = __internal_convert_datetime(&timestamp)?;
        Ok(V::Number(time.month().into()))
    }

    pub fn month_string(args: Arguments) -> anyhow::Result<V> {
        let timestamp = args.var(0)?;
        let time = __internal_convert_datetime(&timestamp)?;
        Ok(V::String(Rc::from(time.format("%b").to_string())))
    }

    pub fn weekday_string(args: Arguments) -> anyhow::Result<V> {
        let timestamp = args.var(0)?;
        let time = __internal_convert_datetime(&timestamp)?;
        Ok(V::String(Rc::from(time.weekday().to_string())))
    }

    pub fn date_string(args: Arguments) -> anyhow::Result<V> {
        let timestamp = args.var(0)?;
        let time = __internal_convert_datetime(&timestamp)?;
        Ok(V::String(Rc::from(time.to_string())))
    }

    pub fn start_of(args: Arguments) -> anyhow::Result<V> {
        let timestamp = args.var(0)?;
        let unit_name = args.str(1)?;

        let datetime = __internal_convert_datetime(&timestamp)?;
        let unit = DateUnit::try_from(unit_name).context("Invalid date unit")?;

        let result =
            date_time_start_of(datetime, unit).context("Failed to calculate start of period")?;

        #[allow(deprecated)]
        Ok(V::Number(result.timestamp().into()))
    }

    pub fn end_of(args: Arguments) -> anyhow::Result<V> {
        let timestamp = args.var(0)?;
        let unit_name = args.str(1)?;

        let datetime = __internal_convert_datetime(&timestamp)?;
        let unit = DateUnit::try_from(unit_name).context("Invalid date unit")?;

        let result =
            date_time_end_of(datetime, unit).context("Failed to calculate end of period")?;

        #[allow(deprecated)]
        Ok(V::Number(result.timestamp().into()))
    }
}
