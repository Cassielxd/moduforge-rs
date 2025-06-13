//! 日期方法模块
//!
//! 提供各种日期和时间的操作方法，包括算术运算、比较、格式化和属性获取

use crate::functions::defs::{
    CompositeFunction, FunctionDefinition, FunctionSignature, StaticFunction,
};
use crate::vm::date::DurationUnit;
use std::rc::Rc;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

/// 日期方法枚举
///
/// 定义了所有可用的日期操作方法
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
pub enum DateMethod {
    // 算术运算
    /// 日期加法：向日期添加时间间隔
    Add,
    /// 日期减法：从日期减去时间间隔
    Sub,
    /// 设置日期组件：设置年、月、日等特定组件
    Set,
    /// 格式化：将日期格式化为字符串
    Format,
    /// 开始时间：获取某个时间单位的开始时间（如月初、年初）
    StartOf,
    /// 结束时间：获取某个时间单位的结束时间（如月末、年末）
    EndOf,
    /// 时间差：计算两个日期之间的差值
    Diff,
    /// 时区转换：将日期转换到指定时区
    Tz,

    // 比较方法
    /// 相同判断：检查两个日期是否相同
    IsSame,
    /// 早于判断：检查第一个日期是否早于第二个日期
    IsBefore,
    /// 晚于判断：检查第一个日期是否晚于第二个日期
    IsAfter,
    /// 相同或早于：检查第一个日期是否相同或早于第二个日期
    IsSameOrBefore,
    /// 相同或晚于：检查第一个日期是否相同或晚于第二个日期
    IsSameOrAfter,

    // 属性获取方法
    /// 秒：获取日期的秒数
    Second,
    /// 分钟：获取日期的分钟数
    Minute,
    /// 小时：获取日期的小时数
    Hour,
    /// 日：获取日期的天数
    Day,
    /// 年中的天：获取日期在一年中的第几天
    DayOfYear,
    /// 周：获取日期在一年中的第几周
    Week,
    /// 星期几：获取日期是星期几
    Weekday,
    /// 月：获取日期的月份
    Month,
    /// 季度：获取日期所在的季度
    Quarter,
    /// 年：获取日期的年份
    Year,
    /// 时间戳：获取日期的Unix时间戳
    Timestamp,
    /// 时区名称：获取日期的时区名称
    OffsetName,

    // 状态检查方法
    /// 有效性检查：检查日期是否有效
    IsValid,
    /// 昨天判断：检查日期是否为昨天
    IsYesterday,
    /// 今天判断：检查日期是否为今天
    IsToday,
    /// 明天判断：检查日期是否为明天
    IsTomorrow,
    /// 闰年判断：检查日期所在年份是否为闰年
    IsLeapYear,
}

/// 比较操作类型
///
/// 用于统一处理各种日期比较操作
enum CompareOperation {
    IsSame,
    IsBefore,
    IsAfter,
    IsSameOrBefore,
    IsSameOrAfter,
}

/// 属性获取操作类型
///
/// 用于统一处理各种日期属性获取操作
enum GetterOperation {
    Second,
    Minute,
    Hour,
    Day,
    Weekday,
    DayOfYear,
    Week,
    Month,
    Quarter,
    Year,
    Timestamp,
    OffsetName,

    IsValid,
    IsYesterday,
    IsToday,
    IsTomorrow,
    IsLeapYear,
}

impl From<&DateMethod> for Rc<dyn FunctionDefinition> {
    /// 将日期方法枚举转换为函数定义
    ///
    /// 为每个日期方法创建相应的函数定义，包括函数签名和实现
    fn from(value: &DateMethod) -> Self {
        use crate::variable::VariableType as VT;
        use DateMethod as DM;

        let unit_vt = DurationUnit::variable_type();

        // 操作签名：支持字符串和数字+单位两种形式
        let op_signature = vec![
            FunctionSignature {
                parameters: vec![VT::Date, VT::String],
                return_type: VT::Date,
            },
            FunctionSignature {
                parameters: vec![VT::Date, VT::Number, unit_vt.clone()],
                return_type: VT::Date,
            },
        ];

        match value {
            // 日期加法：支持字符串和数字+单位
            DM::Add => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::add),
                signatures: op_signature.clone(),
            }),
            // 日期减法：支持字符串和数字+单位
            DM::Sub => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::sub),
                signatures: op_signature.clone(),
            }),
            // 设置日期组件：单位 + 数值
            DM::Set => Rc::new(StaticFunction {
                implementation: Rc::new(imp::set),
                signature: FunctionSignature {
                    parameters: vec![VT::Date, unit_vt.clone(), VT::Number],
                    return_type: VT::Date,
                },
            }),
            // 时区转换
            DM::Tz => Rc::new(StaticFunction {
                implementation: Rc::new(imp::tz),
                signature: FunctionSignature {
                    parameters: vec![VT::Date, VT::String],
                    return_type: VT::Date,
                },
            }),
            // 格式化：支持默认格式和自定义格式
            DM::Format => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::format),
                signatures: vec![
                    FunctionSignature {
                        parameters: vec![VT::Date],
                        return_type: VT::String,
                    },
                    FunctionSignature {
                        parameters: vec![VT::Date, VT::String],
                        return_type: VT::String,
                    },
                ],
            }),
            // 获取时间单位的开始时间
            DM::StartOf => Rc::new(StaticFunction {
                implementation: Rc::new(imp::start_of),
                signature: FunctionSignature {
                    parameters: vec![VT::Date, unit_vt.clone()],
                    return_type: VT::Date,
                },
            }),
            // 获取时间单位的结束时间
            DM::EndOf => Rc::new(StaticFunction {
                implementation: Rc::new(imp::end_of),
                signature: FunctionSignature {
                    parameters: vec![VT::Date, unit_vt.clone()],
                    return_type: VT::Date,
                },
            }),
            // 计算时间差：支持默认单位和指定单位
            DM::Diff => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::diff),
                signatures: vec![
                    FunctionSignature {
                        parameters: vec![VT::Date, VT::Date],
                        return_type: VT::Number,
                    },
                    FunctionSignature {
                        parameters: vec![VT::Date, VT::Date, unit_vt.clone()],
                        return_type: VT::Number,
                    },
                ],
            }),
            // 日期比较方法
            DateMethod::IsSame => imp::compare_using(CompareOperation::IsSame),
            DateMethod::IsBefore => {
                imp::compare_using(CompareOperation::IsBefore)
            },
            DateMethod::IsAfter => {
                imp::compare_using(CompareOperation::IsAfter)
            },
            DateMethod::IsSameOrBefore => {
                imp::compare_using(CompareOperation::IsSameOrBefore)
            },
            DateMethod::IsSameOrAfter => {
                imp::compare_using(CompareOperation::IsSameOrAfter)
            },

            // 日期属性获取方法
            DateMethod::Second => imp::getter(GetterOperation::Second),
            DateMethod::Minute => imp::getter(GetterOperation::Minute),
            DateMethod::Hour => imp::getter(GetterOperation::Hour),
            DateMethod::Day => imp::getter(GetterOperation::Day),
            DateMethod::Weekday => imp::getter(GetterOperation::Weekday),
            DateMethod::DayOfYear => imp::getter(GetterOperation::DayOfYear),
            DateMethod::Week => imp::getter(GetterOperation::Week),
            DateMethod::Month => imp::getter(GetterOperation::Month),
            DateMethod::Quarter => imp::getter(GetterOperation::Quarter),
            DateMethod::Year => imp::getter(GetterOperation::Year),
            DateMethod::Timestamp => imp::getter(GetterOperation::Timestamp),
            DateMethod::OffsetName => imp::getter(GetterOperation::OffsetName),

            // 日期状态检查方法
            DateMethod::IsValid => imp::getter(GetterOperation::IsValid),
            DateMethod::IsYesterday => {
                imp::getter(GetterOperation::IsYesterday)
            },
            DateMethod::IsToday => imp::getter(GetterOperation::IsToday),
            DateMethod::IsTomorrow => imp::getter(GetterOperation::IsTomorrow),
            DateMethod::IsLeapYear => imp::getter(GetterOperation::IsLeapYear),
        }
    }
}

mod imp {
    use crate::functions::arguments::Arguments;
    use crate::functions::date_method::{CompareOperation, GetterOperation};
    use crate::functions::defs::{
        CompositeFunction, FunctionDefinition, FunctionSignature,
        StaticFunction,
    };
    use crate::variable::VariableType as VT;
    use crate::vm::date::{Duration, DurationUnit};
    use crate::vm::VmDate;
    use crate::Variable as V;
    use anyhow::{anyhow, Context};
    use chrono::{Datelike, Timelike};
    use chrono_tz::Tz;
    use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
    use rust_decimal::Decimal;
    use std::rc::Rc;
    use std::str::FromStr;

    fn __internal_extract_duration(
        args: &Arguments,
        from: usize,
    ) -> anyhow::Result<Duration> {
        match args.var(from)? {
            V::String(s) => Ok(Duration::parse(s.as_ref())?),
            V::Number(n) => {
                let unit = __internal_extract_duration_unit(args, from + 1)?;
                Ok(Duration::from_unit(*n, unit)
                    .context("Invalid duration unit")?)
            },
            _ => Err(anyhow!("无效的时间参数")),
        }
    }

    fn __internal_extract_duration_unit(
        args: &Arguments,
        pos: usize,
    ) -> anyhow::Result<DurationUnit> {
        let unit_str = args.str(pos)?;
        DurationUnit::parse(unit_str).context("无效的持续时间单位")
    }

    fn __internal_extract_duration_unit_opt(
        args: &Arguments,
        pos: usize,
    ) -> anyhow::Result<Option<DurationUnit>> {
        let unit_ostr = args.ostr(pos)?;
        let Some(unit_str) = unit_ostr else {
            return Ok(None);
        };

        Ok(Some(DurationUnit::parse(unit_str).context("无效的持续时间单位")?))
    }

    pub fn add(args: Arguments) -> anyhow::Result<V> {
        let this = args.dynamic::<VmDate>(0)?;
        let duration = __internal_extract_duration(&args, 1)?;

        let date_time = this.add(duration);
        Ok(V::Dynamic(Rc::new(date_time)))
    }

    pub fn sub(args: Arguments) -> anyhow::Result<V> {
        let this = args.dynamic::<VmDate>(0)?;
        let duration = __internal_extract_duration(&args, 1)?;

        let date_time = this.sub(duration);
        Ok(V::Dynamic(Rc::new(date_time)))
    }

    pub fn set(args: Arguments) -> anyhow::Result<V> {
        let this = args.dynamic::<VmDate>(0)?;
        let unit = __internal_extract_duration_unit(&args, 1)?;
        let value = args.number(2)?;

        let value_u32 = value.to_u32().context("无效的持续时间值")?;

        let date_time = this.set(value_u32, unit);
        Ok(V::Dynamic(Rc::new(date_time)))
    }

    pub fn format(args: Arguments) -> anyhow::Result<V> {
        let this = args.dynamic::<VmDate>(0)?;
        let format = args.ostr(1)?;

        let formatted = this.format(format);
        Ok(V::String(Rc::from(formatted)))
    }

    pub fn start_of(args: Arguments) -> anyhow::Result<V> {
        let this = args.dynamic::<VmDate>(0)?;
        let unit = __internal_extract_duration_unit(&args, 1)?;

        let date_time = this.start_of(unit);
        Ok(V::Dynamic(Rc::new(date_time)))
    }

    pub fn end_of(args: Arguments) -> anyhow::Result<V> {
        let this = args.dynamic::<VmDate>(0)?;
        let unit = __internal_extract_duration_unit(&args, 1)?;

        let date_time = this.end_of(unit);
        Ok(V::Dynamic(Rc::new(date_time)))
    }

    pub fn diff(args: Arguments) -> anyhow::Result<V> {
        let this = args.dynamic::<VmDate>(0)?;
        let date_time = VmDate::new(args.var(1)?.clone(), None);
        let maybe_unit = __internal_extract_duration_unit_opt(&args, 2)?;

        let var =
            match this.diff(&date_time, maybe_unit).and_then(Decimal::from_i64)
            {
                Some(n) => V::Number(n),
                None => V::Null,
            };

        Ok(var)
    }

    pub fn tz(args: Arguments) -> anyhow::Result<V> {
        let this = args.dynamic::<VmDate>(0)?;
        let tz_str = args.str(1)?;

        let timezone = Tz::from_str(tz_str).context("无效的时区")?;
        Ok(V::Dynamic(Rc::new(this.tz(timezone))))
    }

    pub fn compare_using(op: CompareOperation) -> Rc<dyn FunctionDefinition> {
        Rc::new(CompositeFunction {
            signatures: vec![
                FunctionSignature {
                    parameters: vec![VT::Date, VT::Date],
                    return_type: VT::Date,
                },
                FunctionSignature {
                    parameters: vec![
                        VT::Date,
                        VT::Date,
                        DurationUnit::variable_type(),
                    ],
                    return_type: VT::Date,
                },
            ],
            implementation: Rc::new(
                move |args: Arguments| -> anyhow::Result<V> {
                    let this = args.dynamic::<VmDate>(0)?;
                    let date_time = VmDate::new(args.var(1)?.clone(), None);
                    let maybe_unit =
                        __internal_extract_duration_unit_opt(&args, 2)?;

                    let check = match op {
                        CompareOperation::IsSame => {
                            this.is_same(&date_time, maybe_unit)
                        },
                        CompareOperation::IsBefore => {
                            this.is_before(&date_time, maybe_unit)
                        },
                        CompareOperation::IsAfter => {
                            this.is_after(&date_time, maybe_unit)
                        },
                        CompareOperation::IsSameOrBefore => {
                            this.is_same_or_before(&date_time, maybe_unit)
                        },
                        CompareOperation::IsSameOrAfter => {
                            this.is_same_or_after(&date_time, maybe_unit)
                        },
                    };

                    Ok(V::Bool(check))
                },
            ),
        })
    }

    pub fn getter(op: GetterOperation) -> Rc<dyn FunctionDefinition> {
        Rc::new(StaticFunction {
            signature: FunctionSignature {
                parameters: vec![VT::Date],
                return_type: match op {
                    GetterOperation::Second
                    | GetterOperation::Minute
                    | GetterOperation::Hour
                    | GetterOperation::Day
                    | GetterOperation::Weekday
                    | GetterOperation::DayOfYear
                    | GetterOperation::Week
                    | GetterOperation::Month
                    | GetterOperation::Quarter
                    | GetterOperation::Year
                    | GetterOperation::Timestamp => VT::Number,
                    GetterOperation::IsValid
                    | GetterOperation::IsYesterday
                    | GetterOperation::IsToday
                    | GetterOperation::IsTomorrow
                    | GetterOperation::IsLeapYear => VT::Bool,
                    GetterOperation::OffsetName => VT::String,
                },
            },
            implementation: Rc::new(
                move |args: Arguments| -> anyhow::Result<V> {
                    let this = args.dynamic::<VmDate>(0)?;
                    if let GetterOperation::IsValid = op {
                        return Ok(V::Bool(this.is_valid()));
                    }

                    let Some(dt) = this.0 else {
                        return Ok(V::Null);
                    };

                    Ok(match op {
                        GetterOperation::Second => {
                            V::Number(dt.second().into())
                        },
                        GetterOperation::Minute => {
                            V::Number(dt.minute().into())
                        },
                        GetterOperation::Hour => V::Number(dt.hour().into()),
                        GetterOperation::Day => V::Number(dt.day().into()),
                        GetterOperation::Weekday => {
                            V::Number(dt.weekday().number_from_monday().into())
                        },
                        GetterOperation::DayOfYear => {
                            V::Number(dt.ordinal().into())
                        },
                        GetterOperation::Week => {
                            V::Number(dt.iso_week().week().into())
                        },
                        GetterOperation::Month => V::Number(dt.month().into()),
                        GetterOperation::Quarter => {
                            V::Number(dt.quarter().into())
                        },
                        GetterOperation::Year => V::Number(dt.year().into()),
                        GetterOperation::Timestamp => {
                            V::Number(dt.timestamp_millis().into())
                        },
                        // Boolean
                        GetterOperation::IsValid => V::Bool(true),
                        GetterOperation::IsYesterday => V::Bool(this.is_same(
                            &VmDate::yesterday(),
                            Some(DurationUnit::Day),
                        )),
                        GetterOperation::IsToday => {
                            V::Bool(this.is_same(
                                &VmDate::now(),
                                Some(DurationUnit::Day),
                            ))
                        },
                        GetterOperation::IsTomorrow => V::Bool(this.is_same(
                            &VmDate::tomorrow(),
                            Some(DurationUnit::Day),
                        )),
                        GetterOperation::IsLeapYear => {
                            V::Bool(dt.date_naive().leap_year())
                        },
                        // String
                        GetterOperation::OffsetName => {
                            V::String(Rc::from(dt.timezone().name()))
                        },
                    })
                },
            ),
        })
    }
}
