//! 内置函数模块
//!
//! 提供表达式系统中的所有内置函数，包括通用、字符串、数学、类型和映射相关函数

use crate::functions::defs::{
    CompositeFunction, FunctionDefinition, FunctionSignature, StaticFunction,
};
use std::rc::Rc;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

/// 内置函数枚举
///
/// 定义了表达式系统中所有可用的内置函数
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
pub enum InternalFunction {
    // 通用函数
    /// 长度函数：获取字符串长度或数组元素个数
    Len,
    /// 包含函数：检查字符串包含子串或数组包含元素
    Contains,
    /// 扁平化函数：将嵌套数组展平为一维数组
    Flatten,

    // 字符串函数
    /// 转大写：将字符串转换为大写
    Upper,
    /// 转小写：将字符串转换为小写
    Lower,
    /// 去空格：移除字符串两端的空白字符
    Trim,
    /// 开始匹配：检查字符串是否以指定前缀开始
    StartsWith,
    /// 结束匹配：检查字符串是否以指定后缀结束
    EndsWith,
    /// 正则匹配：检查字符串是否匹配正则表达式
    Matches,
    /// 提取匹配：使用正则表达式提取字符串中的匹配内容
    Extract,
    /// 模糊匹配：计算两个字符串或字符串数组的相似度
    FuzzyMatch,
    /// 分割：使用分隔符将字符串分割为数组
    Split,

    // 数学函数
    /// 绝对值：返回数字的绝对值
    Abs,
    /// 求和：计算数组中所有数字的和
    Sum,
    /// 平均值：计算数组中所有数字的平均值
    Avg,
    /// 最小值：返回数组中的最小值
    Min,
    /// 最大值：返回数组中的最大值
    Max,
    /// 随机数：生成0到指定数字之间的随机数
    Rand,
    /// 中位数：计算数组的中位数
    Median,
    /// 众数：计算数组的众数
    Mode,
    /// 向下取整：返回不大于给定数字的最大整数
    Floor,
    /// 向上取整：返回不小于给定数字的最小整数
    Ceil,
    /// 四舍五入：对数字进行四舍五入
    Round,
    /// 截断：截断数字的小数部分
    Trunc,

    // 类型函数
    /// 数字检查：检查值是否为数字类型
    IsNumeric,
    /// 字符串转换：将值转换为字符串
    String,
    /// 数字转换：将值转换为数字
    Number,
    /// 布尔转换：将值转换为布尔值
    Bool,
    /// 类型获取：返回值的类型名称
    Type,

    // 映射函数
    /// 键列表：获取对象的所有键
    Keys,
    /// 值列表：获取对象的所有值
    Values,

    /// 日期函数：创建或解析日期（使用简写'd'）
    #[strum(serialize = "d")]
    Date,
}

impl From<&InternalFunction> for Rc<dyn FunctionDefinition> {
    /// 将内置函数枚举转换为函数定义
    ///
    /// 为每个内置函数创建相应的函数定义，包括函数签名和实现
    fn from(value: &InternalFunction) -> Self {
        use crate::variable::VariableType as VT;
        use InternalFunction as IF;

        let s: Rc<dyn FunctionDefinition> = match value {
            // 长度函数：支持字符串和数组
            IF::Len => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::len),
                signatures: vec![
                    FunctionSignature::single(VT::String, VT::Number),
                    FunctionSignature::single(VT::Any.array(), VT::Number),
                ],
            }),

            // 包含函数：支持字符串包含和数组包含
            IF::Contains => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::contains),
                signatures: vec![
                    FunctionSignature {
                        parameters: vec![VT::String, VT::String],
                        return_type: VT::Bool,
                    },
                    FunctionSignature {
                        parameters: vec![VT::Any.array(), VT::Any],
                        return_type: VT::Bool,
                    },
                ],
            }),

            // 扁平化函数：将嵌套数组展平
            IF::Flatten => Rc::new(StaticFunction {
                implementation: Rc::new(imp::flatten),
                signature: FunctionSignature::single(
                    VT::Any.array(),
                    VT::Any.array(),
                ),
            }),

            // 字符串转大写
            IF::Upper => Rc::new(StaticFunction {
                implementation: Rc::new(imp::upper),
                signature: FunctionSignature::single(VT::String, VT::String),
            }),

            // 字符串转小写
            IF::Lower => Rc::new(StaticFunction {
                implementation: Rc::new(imp::lower),
                signature: FunctionSignature::single(VT::String, VT::String),
            }),

            // 字符串去空格
            IF::Trim => Rc::new(StaticFunction {
                implementation: Rc::new(imp::trim),
                signature: FunctionSignature::single(VT::String, VT::String),
            }),

            // 字符串开始匹配
            IF::StartsWith => Rc::new(StaticFunction {
                implementation: Rc::new(imp::starts_with),
                signature: FunctionSignature {
                    parameters: vec![VT::String, VT::String],
                    return_type: VT::Bool,
                },
            }),

            // 字符串结束匹配
            IF::EndsWith => Rc::new(StaticFunction {
                implementation: Rc::new(imp::ends_with),
                signature: FunctionSignature {
                    parameters: vec![VT::String, VT::String],
                    return_type: VT::Bool,
                },
            }),

            // 正则表达式匹配
            IF::Matches => Rc::new(StaticFunction {
                implementation: Rc::new(imp::matches),
                signature: FunctionSignature {
                    parameters: vec![VT::String, VT::String],
                    return_type: VT::Bool,
                },
            }),

            // 正则表达式提取
            IF::Extract => Rc::new(StaticFunction {
                implementation: Rc::new(imp::extract),
                signature: FunctionSignature {
                    parameters: vec![VT::String, VT::String],
                    return_type: VT::String.array(),
                },
            }),

            // 字符串分割
            IF::Split => Rc::new(StaticFunction {
                implementation: Rc::new(imp::split),
                signature: FunctionSignature {
                    parameters: vec![VT::String, VT::String],
                    return_type: VT::String.array(),
                },
            }),

            // 模糊匹配：支持单个字符串和字符串数组
            IF::FuzzyMatch => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::fuzzy_match),
                signatures: vec![
                    FunctionSignature {
                        parameters: vec![VT::String, VT::String],
                        return_type: VT::Number,
                    },
                    FunctionSignature {
                        parameters: vec![VT::String.array(), VT::String],
                        return_type: VT::Number.array(),
                    },
                ],
            }),

            // 绝对值
            IF::Abs => Rc::new(StaticFunction {
                implementation: Rc::new(imp::abs),
                signature: FunctionSignature::single(VT::Number, VT::Number),
            }),

            // 随机数生成
            IF::Rand => Rc::new(StaticFunction {
                implementation: Rc::new(imp::rand),
                signature: FunctionSignature::single(VT::Number, VT::Number),
            }),

            // 向下取整
            IF::Floor => Rc::new(StaticFunction {
                implementation: Rc::new(imp::floor),
                signature: FunctionSignature::single(VT::Number, VT::Number),
            }),

            // 向上取整
            IF::Ceil => Rc::new(StaticFunction {
                implementation: Rc::new(imp::ceil),
                signature: FunctionSignature::single(VT::Number, VT::Number),
            }),

            // 四舍五入：支持指定小数位数
            IF::Round => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::round),
                signatures: vec![
                    FunctionSignature {
                        parameters: vec![VT::Number],
                        return_type: VT::Number,
                    },
                    FunctionSignature {
                        parameters: vec![VT::Number, VT::Number],
                        return_type: VT::Number,
                    },
                ],
            }),

            // 截断：支持指定小数位数
            IF::Trunc => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::trunc),
                signatures: vec![
                    FunctionSignature {
                        parameters: vec![VT::Number],
                        return_type: VT::Number,
                    },
                    FunctionSignature {
                        parameters: vec![VT::Number, VT::Number],
                        return_type: VT::Number,
                    },
                ],
            }),

            // 求和
            IF::Sum => Rc::new(StaticFunction {
                implementation: Rc::new(imp::sum),
                signature: FunctionSignature::single(
                    VT::Number.array(),
                    VT::Number,
                ),
            }),

            // 平均值
            IF::Avg => Rc::new(StaticFunction {
                implementation: Rc::new(imp::avg),
                signature: FunctionSignature::single(
                    VT::Number.array(),
                    VT::Number,
                ),
            }),

            // 最小值
            IF::Min => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::min),
                signatures: vec![
                    FunctionSignature::single(VT::Number.array(), VT::Number),
                    FunctionSignature::single(VT::Date.array(), VT::Date),
                ],
            }),

            // 最大值
            IF::Max => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::max),
                signatures: vec![
                    FunctionSignature::single(VT::Number.array(), VT::Number),
                    FunctionSignature::single(VT::Date.array(), VT::Date),
                ],
            }),

            // 中位数
            IF::Median => Rc::new(StaticFunction {
                implementation: Rc::new(imp::median),
                signature: FunctionSignature::single(
                    VT::Number.array(),
                    VT::Number,
                ),
            }),

            // 众数
            IF::Mode => Rc::new(StaticFunction {
                implementation: Rc::new(imp::mode),
                signature: FunctionSignature::single(
                    VT::Number.array(),
                    VT::Number,
                ),
            }),

            // 类型获取
            IF::Type => Rc::new(StaticFunction {
                implementation: Rc::new(imp::to_type),
                signature: FunctionSignature::single(VT::Any, VT::String),
            }),

            // 字符串转换
            IF::String => Rc::new(StaticFunction {
                implementation: Rc::new(imp::to_string),
                signature: FunctionSignature::single(VT::Any, VT::String),
            }),

            // 布尔转换
            IF::Bool => Rc::new(StaticFunction {
                implementation: Rc::new(imp::to_bool),
                signature: FunctionSignature::single(VT::Any, VT::Bool),
            }),

            // 数字检查
            IF::IsNumeric => Rc::new(StaticFunction {
                implementation: Rc::new(imp::is_numeric),
                signature: FunctionSignature::single(VT::Any, VT::Bool),
            }),

            // 数字转换
            IF::Number => Rc::new(StaticFunction {
                implementation: Rc::new(imp::to_number),
                signature: FunctionSignature::single(VT::Any, VT::Number),
            }),

            // 键列表
            IF::Keys => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::keys),
                signatures: vec![
                    FunctionSignature::single(
                        VT::Object(Default::default()),
                        VT::String.array(),
                    ),
                    FunctionSignature::single(
                        VT::Any.array(),
                        VT::Number.array(),
                    ),
                ],
            }),

            // 值列表
            IF::Values => Rc::new(StaticFunction {
                implementation: Rc::new(imp::values),
                signature: FunctionSignature::single(
                    VT::Object(Default::default()),
                    VT::Any.array(),
                ),
            }),

            // 日期函数
            IF::Date => Rc::new(CompositeFunction {
                implementation: Rc::new(imp::date),
                signatures: vec![
                    FunctionSignature {
                        parameters: vec![],
                        return_type: VT::Date,
                    },
                    FunctionSignature {
                        parameters: vec![VT::Any],
                        return_type: VT::Date,
                    },
                    FunctionSignature {
                        parameters: vec![VT::Any, VT::String],
                        return_type: VT::Date,
                    },
                ],
            }),
        };

        s
    }
}

/// 内置函数实现模块
///
/// 包含所有内置函数的具体实现代码
pub(crate) mod imp {
    use crate::functions::arguments::Arguments;
    use crate::vm::VmDate;
    use crate::{Variable as V, Variable};
    use anyhow::{anyhow, Context};
    use chrono_tz::Tz;
    #[cfg(not(feature = "regex-lite"))]
    use regex::Regex;
    #[cfg(feature = "regex-lite")]
    use regex_lite::Regex;
    use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
    use rust_decimal::{Decimal, RoundingStrategy};
    use rust_decimal_macros::dec;
    use std::collections::BTreeMap;
    use std::rc::Rc;
    use std::str::FromStr;

    /// 辅助函数：从参数中提取数字数组
    ///
    /// # 参数
    /// * `args` - 函数参数
    /// * `pos` - 参数位置
    ///
    /// # 返回值
    /// * `Ok(Vec<Decimal>)` - 成功提取的数字数组
    /// * `Err` - 参数不是数组或包含非数字元素
    fn __internal_number_array(
        args: &Arguments,
        pos: usize,
    ) -> anyhow::Result<Vec<Decimal>> {
        let a = args.array(pos)?;
        let arr = a.borrow();

        arr.iter()
            .map(|v| v.as_number())
            .collect::<Option<Vec<_>>>()
            .context("Expected a number array")
    }

    /// 联合类型：表示数字数组或日期数组
    enum Either<A, B> {
        Left(A),
        Right(B),
    }

    /// 辅助函数：从参数中提取数字数组或日期数组
    ///
    /// 根据数组的第一个元素类型自动判断是数字数组还是日期数组
    ///
    /// # 参数
    /// * `args` - 函数参数
    /// * `pos` - 参数位置
    ///
    /// # 返回值
    /// * `Ok(Either::Left(Vec<Decimal>))` - 数字数组
    /// * `Ok(Either::Right(Vec<VmDate>))` - 日期数组
    /// * `Err` - 参数无效或类型不匹配
    fn __internal_number_or_date_array(
        args: &Arguments,
        pos: usize,
    ) -> anyhow::Result<Either<Vec<Decimal>, Vec<VmDate>>> {
        let a = args.array(pos)?;
        let arr = a.borrow();

        let is_number = arr.first().map(|v| v.as_number()).flatten().is_some();
        if is_number {
            Ok(Either::Left(
                arr.iter()
                    .map(|v| v.as_number())
                    .collect::<Option<Vec<_>>>()
                    .context("Expected a number array")?,
            ))
        } else {
            Ok(Either::Right(
                arr.iter()
                    .map(|v| match v {
                        Variable::Dynamic(d) => d.as_date().cloned(),
                        _ => None,
                    })
                    .collect::<Option<Vec<_>>>()
                    .context("Expected a number array")?,
            ))
        }
    }

    /// 字符串开始匹配函数实现
    ///
    /// 检查第一个字符串是否以第二个字符串开始
    pub fn starts_with(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        let b = args.str(1)?;

        Ok(V::Bool(a.starts_with(b)))
    }

    /// 字符串结束匹配函数实现
    ///
    /// 检查第一个字符串是否以第二个字符串结束
    pub fn ends_with(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        let b = args.str(1)?;

        Ok(V::Bool(a.ends_with(b)))
    }

    /// 正则表达式匹配函数实现
    ///
    /// 使用正则表达式检查字符串是否匹配模式
    pub fn matches(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        let b = args.str(1)?;

        let regex =
            Regex::new(b.as_ref()).context("Invalid regular expression")?;

        Ok(V::Bool(regex.is_match(a.as_ref())))
    }

    /// 字符串转大写函数实现
    ///
    /// 将字符串转换为大写形式
    pub fn upper(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        Ok(V::String(a.to_uppercase().into()))
    }

    /// 字符串转小写函数实现
    ///
    /// 将字符串转换为小写形式
    pub fn lower(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        Ok(V::String(a.to_lowercase().into()))
    }

    /// 字符串去空格函数实现
    ///
    /// 移除字符串两端的空白字符
    pub fn trim(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        Ok(V::String(a.trim().into()))
    }

    /// 正则表达式提取函数实现
    ///
    /// 使用正则表达式从字符串中提取匹配的捕获组
    /// 返回包含所有捕获组的字符串数组
    pub fn extract(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        let b = args.str(1)?;

        let regex =
            Regex::new(b.as_ref()).context("Invalid regular expression")?;

        let captures = regex
            .captures(a.as_ref())
            .map(|capture| {
                capture
                    .iter()
                    .map(|c| c.map(|c| c.as_str()))
                    .filter_map(|c| c)
                    .map(|s| V::String(Rc::from(s)))
                    .collect()
            })
            .unwrap_or_default();

        Ok(V::from_array(captures))
    }

    /// 字符串分割函数实现
    ///
    /// 使用指定分隔符将字符串分割为字符串数组
    pub fn split(args: Arguments) -> anyhow::Result<V> {
        let a = args.str(0)?;
        let b = args.str(1)?;

        let arr = Vec::from_iter(
            a.split(b).into_iter().map(|s| V::String(s.to_string().into())),
        );

        Ok(V::from_array(arr))
    }

    /// 数组扁平化函数实现
    ///
    /// 将嵌套数组展平为一维数组，非数组元素保持不变
    pub fn flatten(args: Arguments) -> anyhow::Result<V> {
        let a = args.array(0)?;

        let arr = a.borrow();
        let mut flat_arr = Vec::with_capacity(arr.len());
        arr.iter().for_each(|v| match v {
            V::Array(b) => {
                let arr = b.borrow();
                arr.iter().for_each(|v| flat_arr.push(v.clone()))
            },
            _ => flat_arr.push(v.clone()),
        });

        Ok(V::from_array(flat_arr))
    }

    /// 绝对值函数实现
    ///
    /// 返回数字的绝对值
    pub fn abs(args: Arguments) -> anyhow::Result<V> {
        let a = args.number(0)?;
        Ok(V::Number(a.abs()))
    }

    /// 向上取整函数实现
    ///
    /// 返回不小于给定数字的最小整数
    pub fn ceil(args: Arguments) -> anyhow::Result<V> {
        let a = args.number(0)?;
        Ok(V::Number(a.ceil()))
    }

    /// 向下取整函数实现
    ///
    /// 返回不大于给定数字的最大整数
    pub fn floor(args: Arguments) -> anyhow::Result<V> {
        let a = args.number(0)?;
        Ok(V::Number(a.floor()))
    }

    /// 四舍五入函数实现
    ///
    /// 对数字进行四舍五入，可选择指定小数位数
    pub fn round(args: Arguments) -> anyhow::Result<V> {
        let a = args.number(0)?;
        let dp = args
            .onumber(1)?
            .map(|v| v.to_u32().context("Invalid number of decimal places"))
            .transpose()?
            .unwrap_or(0);

        Ok(V::Number(a.round_dp_with_strategy(
            dp,
            RoundingStrategy::MidpointAwayFromZero,
        )))
    }

    /// 截断函数实现
    ///
    /// 截断数字的小数部分，可选择指定保留的小数位数
    pub fn trunc(args: Arguments) -> anyhow::Result<V> {
        let a = args.number(0)?;
        let dp = args
            .onumber(1)?
            .map(|v| v.to_u32().context("Invalid number of decimal places"))
            .transpose()?
            .unwrap_or(0);

        Ok(V::Number(a.trunc_with_scale(dp)))
    }

    /// 随机数生成函数实现
    ///
    /// 生成0到指定数字之间的随机整数
    pub fn rand(args: Arguments) -> anyhow::Result<V> {
        let a = args.number(0)?;
        let upper_range = a.round().to_i64().context("Invalid upper range")?;

        let random_number = fastrand::i64(0..=upper_range);
        Ok(V::Number(Decimal::from(random_number)))
    }

    /// 最小值函数实现
    ///
    /// 返回数字数组或日期数组中的最小值
    pub fn min(args: Arguments) -> anyhow::Result<V> {
        let a = __internal_number_or_date_array(&args, 0)?;

        match a {
            Either::Left(arr) => {
                let min = arr.into_iter().min().context("Empty array")?;
                Ok(V::Number(Decimal::from(min)))
            },
            Either::Right(arr) => {
                let min = arr.into_iter().min().context("Empty array")?;
                Ok(V::Dynamic(Rc::new(min)))
            },
        }
    }

    /// 最大值函数实现
    ///
    /// 返回数字数组或日期数组中的最大值
    pub fn max(args: Arguments) -> anyhow::Result<V> {
        let a = __internal_number_or_date_array(&args, 0)?;

        match a {
            Either::Left(arr) => {
                let max = arr.into_iter().max().context("Empty array")?;
                Ok(V::Number(Decimal::from(max)))
            },
            Either::Right(arr) => {
                let max = arr.into_iter().max().context("Empty array")?;
                Ok(V::Dynamic(Rc::new(max)))
            },
        }
    }

    /// 平均值函数实现
    ///
    /// 计算数字数组的算术平均值
    pub fn avg(args: Arguments) -> anyhow::Result<V> {
        let a = __internal_number_array(&args, 0)?;
        let sum = a.iter().fold(Decimal::ZERO, |acc, x| acc + x);

        Ok(V::Number(Decimal::from(
            sum.checked_div(Decimal::from(a.len())).context("Empty array")?,
        )))
    }

    /// 求和函数实现
    ///
    /// 计算数字数组中所有元素的和
    pub fn sum(args: Arguments) -> anyhow::Result<V> {
        let a = __internal_number_array(&args, 0)?;
        let sum = a.iter().fold(Decimal::ZERO, |acc, v| acc + v);

        Ok(V::Number(Decimal::from(sum)))
    }

    /// 中位数函数实现
    ///
    /// 计算数字数组的中位数（中间值）
    pub fn median(args: Arguments) -> anyhow::Result<V> {
        let mut a = __internal_number_array(&args, 0)?;
        a.sort();

        let center = a.len() / 2;
        if a.len() % 2 == 1 {
            // 奇数个元素，取中间的元素
            let center_num = a.get(center).context("Index out of bounds")?;
            Ok(V::Number(*center_num))
        } else {
            // 偶数个元素，取中间两个元素的平均值
            let center_left =
                a.get(center - 1).context("Index out of bounds")?;
            let center_right = a.get(center).context("Index out of bounds")?;

            let median = ((*center_left) + (*center_right)) / dec!(2);
            Ok(V::Number(median))
        }
    }

    /// 众数函数实现
    ///
    /// 计算数字数组的众数（出现次数最多的值）
    pub fn mode(args: Arguments) -> anyhow::Result<V> {
        let a = __internal_number_array(&args, 0)?;
        let mut counts = BTreeMap::new();
        for num in a {
            *counts.entry(num).or_insert(0) += 1;
        }

        let most_common = counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(num, _)| num)
            .context("Empty array")?;

        Ok(V::Number(most_common))
    }

    pub fn to_type(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        Ok(V::String(a.type_name().into()))
    }

    pub fn to_bool(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let val = match a {
            V::Null => false,
            V::Bool(v) => *v,
            V::Number(n) => !n.is_zero(),
            V::Array(_) | V::Object(_) | V::Dynamic(_) => true,
            V::String(s) => match (*s).trim() {
                "true" => true,
                "false" => false,
                _ => s.is_empty(),
            },
        };

        Ok(V::Bool(val))
    }

    pub fn to_string(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let val = match a {
            V::Null => Rc::from("null"),
            V::Bool(v) => Rc::from(v.to_string().as_str()),
            V::Number(n) => Rc::from(n.to_string().as_str()),
            V::String(s) => s.clone(),
            _ => {
                return Err(anyhow!(
                    "Cannot convert type {} to string",
                    a.type_name()
                ));
            },
        };

        Ok(V::String(val))
    }

    pub fn to_number(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let val = match a {
            V::Number(n) => *n,
            V::String(str) => {
                let s = str.trim();
                Decimal::from_str_exact(s)
                    .or_else(|_| Decimal::from_scientific(s))
                    .context("Invalid number")?
            },
            V::Bool(b) => match *b {
                true => Decimal::ONE,
                false => Decimal::ZERO,
            },
            _ => {
                return Err(anyhow!(
                    "Cannot convert type {} to number",
                    a.type_name()
                ));
            },
        };

        Ok(V::Number(val))
    }

    pub fn is_numeric(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let is_ok = match a {
            V::Number(_) => true,
            V::String(str) => {
                let s = str.trim();
                Decimal::from_str_exact(s)
                    .or_else(|_| Decimal::from_scientific(s))
                    .is_ok()
            },
            _ => false,
        };

        Ok(V::Bool(is_ok))
    }

    pub fn len(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let len = match a {
            V::String(s) => s.len(),
            V::Array(s) => {
                let arr = s.borrow();
                arr.len()
            },
            _ => {
                return Err(anyhow!(
                    "Cannot determine len of type {}",
                    a.type_name()
                ));
            },
        };

        Ok(V::Number(len.into()))
    }

    pub fn contains(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let b = args.var(1)?;

        let val = match (a, b) {
            (V::String(a), V::String(b)) => a.contains(b.as_ref()),
            (V::Array(a), _) => {
                let arr = a.borrow();

                arr.iter().any(|a| match (a, b) {
                    (V::Number(a), V::Number(b)) => a == b,
                    (V::String(a), V::String(b)) => a == b,
                    (V::Bool(a), V::Bool(b)) => a == b,
                    (V::Null, V::Null) => true,
                    _ => false,
                })
            },
            _ => {
                return Err(anyhow!(
                    "无法确定类型 {} 和 {} 的包含关系",
                    a.type_name(),
                    b.type_name()
                ));
            },
        };

        Ok(V::Bool(val))
    }

    pub fn fuzzy_match(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let b = args.str(1)?;

        let val = match a {
            V::String(a) => {
                let sim = strsim::normalized_damerau_levenshtein(
                    a.as_ref(),
                    b.as_ref(),
                );
                // This is okay, as NDL will return [0, 1]
                V::Number(Decimal::from_f64(sim).unwrap_or(dec!(0)))
            },
            V::Array(_a) => {
                let a = _a.borrow();
                let mut sims = Vec::with_capacity(a.len());
                for v in a.iter() {
                    let s = v.as_str().context("期望字符串数组")?;

                    let sim = Decimal::from_f64(
                        strsim::normalized_damerau_levenshtein(
                            s.as_ref(),
                            b.as_ref(),
                        ),
                    )
                    .unwrap_or(dec!(0));
                    sims.push(V::Number(sim));
                }

                V::from_array(sims)
            },
            _ => {
                return Err(anyhow!("模糊匹配不适用于类型 {} ", a.type_name()));
            },
        };

        Ok(val)
    }

    pub fn keys(args: Arguments) -> anyhow::Result<V> {
        let a = args.var(0)?;
        let var = match a {
            V::Array(a) => {
                let arr = a.borrow();
                let indices = arr
                    .iter()
                    .enumerate()
                    .map(|(index, _)| V::Number(index.into()))
                    .collect();

                V::from_array(indices)
            },
            V::Object(a) => {
                let obj = a.borrow();
                let keys =
                    obj.iter().map(|(key, _)| V::String(key.clone())).collect();

                V::from_array(keys)
            },
            _ => {
                return Err(anyhow!("无法确定类型 {} 的键", a.type_name()));
            },
        };

        Ok(var)
    }

    pub fn values(args: Arguments) -> anyhow::Result<V> {
        let a = args.object(0)?;
        let obj = a.borrow();
        let values: Vec<_> = obj.values().cloned().collect();

        Ok(V::from_array(values))
    }

    pub fn date(args: Arguments) -> anyhow::Result<V> {
        let provided = args.ovar(0);
        let tz = args
            .ostr(1)?
            .map(|v| Tz::from_str(v).context("无效的时区"))
            .transpose()?;

        let date_time = match provided {
            Some(v) => VmDate::new(v.clone(), tz),
            None => VmDate::now(),
        };

        Ok(V::Dynamic(Rc::new(date_time)))
    }
}
