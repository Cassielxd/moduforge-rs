//! 函数注册表模块
//!
//! 管理内置函数和已废弃函数的注册和查找

use crate::functions::defs::FunctionDefinition;
use crate::functions::{DeprecatedFunction, FunctionKind, InternalFunction};
use crate::functions::custom::CustomFunctionRegistry;
use nohash_hasher::{BuildNoHashHasher, IsEnabled};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use strum::IntoEnumIterator;

impl IsEnabled for InternalFunction {}
impl IsEnabled for DeprecatedFunction {}

/// 函数注册表
///
/// 负责管理和查找所有可用的函数定义，包括内置函数和已废弃函数
pub struct FunctionRegistry {
    /// 内置函数映射表：函数枚举 -> 函数定义
    internal_functions: HashMap<
        InternalFunction,
        Rc<dyn FunctionDefinition>,
        BuildNoHashHasher<InternalFunction>,
    >,
    /// 已废弃函数映射表：函数枚举 -> 函数定义
    deprecated_functions: HashMap<
        DeprecatedFunction,
        Rc<dyn FunctionDefinition>,
        BuildNoHashHasher<DeprecatedFunction>,
    >,
}

impl FunctionRegistry {
    // 线程本地存储的注册表实例
    thread_local!(
        static INSTANCE: RefCell<FunctionRegistry> = RefCell::new(FunctionRegistry::new_internal())
    );

    /// 根据函数类型获取函数定义
    ///
    /// # 参数
    /// * `kind` - 函数类型（内置、已废弃、闭包或自定义）
    ///
    /// # 返回值
    /// * `Some(Rc<dyn FunctionDefinition>)` - 找到的函数定义
    /// * `None` - 未找到对应的函数定义
    pub fn get_definition(
        kind: &FunctionKind
    ) -> Option<Rc<dyn FunctionDefinition>> {
        match kind {
            FunctionKind::Internal(internal) => Self::INSTANCE
                .with_borrow(|i| i.internal_functions.get(&internal).cloned()),
            FunctionKind::Deprecated(deprecated) => {
                Self::INSTANCE.with_borrow(|i| {
                    i.deprecated_functions.get(&deprecated).cloned()
                })
            },
            FunctionKind::Closure(_) => None, // 闭包函数不在注册表中，由编译器特殊处理
            FunctionKind::Custom(custom) => {
                CustomFunctionRegistry::get_definition(&custom.name)
            },
        }
    }

    /// 创建内部注册表实例
    ///
    /// 初始化时自动注册所有内置函数和已废弃函数
    fn new_internal() -> Self {
        // 遍历所有内置函数并创建映射
        let internal_functions = InternalFunction::iter()
            .map(|i| (i.clone(), (&i).into()))
            .collect();

        // 遍历所有已废弃函数并创建映射
        let deprecated_functions = DeprecatedFunction::iter()
            .map(|i| (i.clone(), (&i).into()))
            .collect();

        Self { internal_functions, deprecated_functions }
    }
}
