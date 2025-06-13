//! 自定义函数模块
//!
//! 支持在运行时动态注册自定义函数，并可以访问State

use crate::functions::defs::{
    FunctionDefinition, FunctionSignature, StaticFunction,
};
use crate::functions::arguments::Arguments;
use crate::variable::{Variable, VariableType};
use moduforge_state::State;
use std::rc::Rc;
use std::sync::Arc;
use std::collections::HashMap;
use std::cell::RefCell;
use std::fmt::Display;
use anyhow::Result as AnyhowResult;

/// 自定义函数标识符
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct CustomFunction {
    /// 函数名称
    pub name: String,
}

impl CustomFunction {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Display for CustomFunction {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TryFrom<&str> for CustomFunction {
    type Error = strum::ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // 检查是否为已注册的自定义函数
        if CustomFunctionRegistry::is_registered(value) {
            Ok(CustomFunction::new(value.to_string()))
        } else {
            Err(strum::ParseError::VariantNotFound)
        }
    }
}

/// 自定义函数的执行器类型
pub type CustomFunctionExecutor = Box<
    dyn Fn(&Arguments, Option<&Arc<State>>) -> AnyhowResult<Variable> + 'static,
>;

/// 自定义函数定义
pub struct CustomFunctionDefinition {
    /// 函数名称
    pub name: String,
    /// 函数签名
    pub signature: FunctionSignature,
    /// 执行器
    pub executor: CustomFunctionExecutor,
}

impl CustomFunctionDefinition {
    pub fn new(
        name: String,
        signature: FunctionSignature,
        executor: CustomFunctionExecutor,
    ) -> Self {
        Self { name, signature, executor }
    }
}

impl FunctionDefinition for CustomFunctionDefinition {
    fn call(
        &self,
        args: Arguments,
    ) -> AnyhowResult<Variable> {
        // 尝试获取State上下文（如果可用）
        let state = CURRENT_STATE.with(|s| s.borrow().clone());
        (self.executor)(&args, state.as_ref())
    }

    fn required_parameters(&self) -> usize {
        self.signature.parameters.len()
    }

    fn optional_parameters(&self) -> usize {
        0 // 暂时不支持可选参数
    }

    fn check_types(
        &self,
        args: &[Rc<VariableType>],
    ) -> crate::functions::defs::FunctionTypecheck {
        let mut typecheck =
            crate::functions::defs::FunctionTypecheck::default();
        typecheck.return_type = self.signature.return_type.clone();

        if args.len() != self.required_parameters() {
            typecheck.general = Some(format!(
                "期望 `{}` 参数, 实际 `{}` 参数.",
                self.required_parameters(),
                args.len()
            ));
        }

        // 检查每个参数类型
        for (i, (arg, expected_type)) in
            args.iter().zip(self.signature.parameters.iter()).enumerate()
        {
            if !arg.satisfies(expected_type) {
                typecheck.arguments.push((
                    i,
                    format!(
                        "参数类型 `{arg}` 不能赋值给参数类型 `{expected_type}`.",
                    ),
                ));
            }
        }

        typecheck
    }

    fn param_type(
        &self,
        index: usize,
    ) -> Option<VariableType> {
        self.signature.parameters.get(index).cloned()
    }

    fn param_type_str(
        &self,
        index: usize,
    ) -> String {
        self.signature
            .parameters
            .get(index)
            .map(|x| x.to_string())
            .unwrap_or_else(|| "never".to_string())
    }

    fn return_type(&self) -> VariableType {
        self.signature.return_type.clone()
    }

    fn return_type_str(&self) -> String {
        self.signature.return_type.to_string()
    }
}

thread_local! {
    /// 当前State上下文（用于自定义函数访问）
    static CURRENT_STATE: RefCell<Option<Arc<State>>> = RefCell::new(None);
}

/// 自定义函数注册表
pub struct CustomFunctionRegistry {
    functions: HashMap<String, Rc<CustomFunctionDefinition>>,
}

impl CustomFunctionRegistry {
    thread_local!(
        static INSTANCE: RefCell<CustomFunctionRegistry> = RefCell::new(CustomFunctionRegistry::new())
    );

    fn new() -> Self {
        Self { functions: HashMap::new() }
    }

    /// 注册自定义函数
    pub fn register_function(
        name: String,
        signature: FunctionSignature,
        executor: CustomFunctionExecutor,
    ) -> Result<(), String> {
        Self::INSTANCE.with(|registry| {
            let mut reg = registry.borrow_mut();
            if reg.functions.contains_key(&name) {
                return Err(format!("函数 '{}' 已经存在", name));
            }

            let definition = CustomFunctionDefinition::new(
                name.clone(),
                signature,
                executor,
            );
            reg.functions.insert(name, Rc::new(definition));
            Ok(())
        })
    }

    /// 获取函数定义
    pub fn get_definition(name: &str) -> Option<Rc<dyn FunctionDefinition>> {
        Self::INSTANCE.with(|registry| {
            registry
                .borrow()
                .functions
                .get(name)
                .map(|def| def.clone() as Rc<dyn FunctionDefinition>)
        })
    }

    /// 检查函数是否已注册
    pub fn is_registered(name: &str) -> bool {
        Self::INSTANCE
            .with(|registry| registry.borrow().functions.contains_key(name))
    }

    /// 设置当前State上下文
    pub fn set_current_state(state: Option<Arc<State>>) {
        CURRENT_STATE.with(|s| {
            *s.borrow_mut() = state;
        });
    }

    /// 列出所有已注册的函数
    pub fn list_functions() -> Vec<String> {
        Self::INSTANCE.with(|registry| {
            registry.borrow().functions.keys().cloned().collect()
        })
    }

    /// 清空所有注册的函数
    pub fn clear() {
        Self::INSTANCE.with(|registry| {
            registry.borrow_mut().functions.clear();
        });
    }
}

impl From<&CustomFunction> for Rc<dyn FunctionDefinition> {
    fn from(custom: &CustomFunction) -> Self {
        CustomFunctionRegistry::get_definition(&custom.name).unwrap_or_else(
            || {
                // 如果函数不存在，返回一个错误函数
                Rc::new(StaticFunction {
                    signature: FunctionSignature {
                        parameters: vec![],
                        return_type: VariableType::Null,
                    },
                    implementation: Rc::new(|_| {
                        Err(anyhow::anyhow!("自定义函数未找到"))
                    }),
                })
            },
        )
    }
}
