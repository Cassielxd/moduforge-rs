//! 方法注册表模块
//!
//! 管理各种类型的方法，包括日期方法等

use crate::functions::date_method::DateMethod;
use crate::functions::defs::FunctionDefinition;
use nohash_hasher::{BuildNoHashHasher, IsEnabled};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;
use strum::IntoEnumIterator;

impl IsEnabled for DateMethod {}

/// 方法类型枚举
///
/// 定义了所有可用的方法类型
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MethodKind {
    /// 日期方法
    DateMethod(DateMethod),
}

impl TryFrom<&str> for MethodKind {
    type Error = strum::ParseError;

    /// 从字符串解析方法类型
    ///
    /// # 参数
    /// * `value` - 方法名称字符串
    ///
    /// # 返回值
    /// * `Ok(MethodKind)` - 成功解析的方法类型
    /// * `Err` - 未知的方法名称
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        DateMethod::try_from(value).map(MethodKind::DateMethod)
    }
}

impl Display for MethodKind {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            MethodKind::DateMethod(d) => write!(f, "{d}"),
        }
    }
}

/// 方法注册表
///
/// 负责管理和查找各种类型的方法定义
pub struct MethodRegistry {
    /// 日期方法映射表：方法枚举 -> 方法定义
    date_methods: HashMap<
        DateMethod,
        Rc<dyn FunctionDefinition>,
        BuildNoHashHasher<DateMethod>,
    >,
}

impl MethodRegistry {
    // 线程本地存储的方法注册表实例
    thread_local!(
        static INSTANCE: RefCell<MethodRegistry> = RefCell::new(MethodRegistry::new_internal())
    );

    /// 根据方法类型获取方法定义
    ///
    /// # 参数
    /// * `kind` - 方法类型
    ///
    /// # 返回值
    /// * `Some(Rc<dyn FunctionDefinition>)` - 找到的方法定义
    /// * `None` - 未找到对应的方法定义
    pub fn get_definition(
        kind: &MethodKind
    ) -> Option<Rc<dyn FunctionDefinition>> {
        match kind {
            MethodKind::DateMethod(dm) => {
                Self::INSTANCE.with_borrow(|i| i.date_methods.get(&dm).cloned())
            },
        }
    }

    /// 创建内部方法注册表实例
    ///
    /// 初始化时自动注册所有日期方法
    fn new_internal() -> Self {
        // 遍历所有日期方法并创建映射
        let date_methods =
            DateMethod::iter().map(|i| (i.clone(), (&i).into())).collect();

        Self { date_methods }
    }
}
