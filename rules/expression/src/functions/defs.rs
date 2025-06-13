//! 函数定义接口模块
//!
//! 定义了函数的基本接口和具体实现类型，包括静态函数和复合函数

use crate::functions::arguments::Arguments;
use crate::variable::VariableType;
use crate::Variable;
use std::any::Any;
use std::collections::HashSet;
use std::rc::Rc;

/// 函数定义特征
///
/// 所有函数（内置、自定义、已废弃）都必须实现此特征
pub trait FunctionDefinition: Any {
    /// 返回必需参数的数量
    fn required_parameters(&self) -> usize;
    /// 返回可选参数的数量
    fn optional_parameters(&self) -> usize;
    /// 检查参数类型是否匹配
    fn check_types(
        &self,
        args: &[Rc<VariableType>],
    ) -> FunctionTypecheck;
    /// 执行函数调用
    fn call(
        &self,
        args: Arguments,
    ) -> anyhow::Result<Variable>;
    /// 获取指定位置参数的类型
    fn param_type(
        &self,
        index: usize,
    ) -> Option<VariableType>;
    /// 获取指定位置参数类型的字符串表示
    fn param_type_str(
        &self,
        index: usize,
    ) -> String;
    /// 获取函数返回值类型
    fn return_type(&self) -> VariableType;
    /// 获取函数返回值类型的字符串表示
    fn return_type_str(&self) -> String;
}

/// 函数类型检查结果
///
/// 包含类型检查过程中发现的错误信息和推断的返回类型
#[derive(Debug, Default)]
pub struct FunctionTypecheck {
    /// 通用错误信息（如参数数量不匹配）
    pub general: Option<String>,
    /// 参数错误列表：(参数索引, 错误信息)
    pub arguments: Vec<(usize, String)>,
    /// 推断的返回类型
    pub return_type: VariableType,
}

/// 函数签名
///
/// 描述函数的参数类型和返回类型
#[derive(Clone)]
pub struct FunctionSignature {
    /// 参数类型列表
    pub parameters: Vec<VariableType>,
    /// 返回类型
    pub return_type: VariableType,
}

impl FunctionSignature {
    /// 创建单参数函数签名
    ///
    /// # 参数
    /// * `parameter` - 参数类型
    /// * `return_type` - 返回类型
    pub fn single(
        parameter: VariableType,
        return_type: VariableType,
    ) -> Self {
        Self { parameters: vec![parameter], return_type }
    }
}

/// 静态函数
///
/// 具有固定签名的函数实现
#[derive(Clone)]
pub struct StaticFunction {
    /// 函数签名
    pub signature: FunctionSignature,
    /// 函数实现
    pub implementation: Rc<dyn Fn(Arguments) -> anyhow::Result<Variable>>,
}

impl FunctionDefinition for StaticFunction {
    /// 返回必需参数数量（等于签名中的参数数量）
    fn required_parameters(&self) -> usize {
        self.signature.parameters.len()
    }

    /// 静态函数没有可选参数
    fn optional_parameters(&self) -> usize {
        0
    }

    /// 检查参数类型是否与签名匹配
    fn check_types(
        &self,
        args: &[Rc<VariableType>],
    ) -> FunctionTypecheck {
        let mut typecheck = FunctionTypecheck::default();
        typecheck.return_type = self.signature.return_type.clone();

        // 检查参数数量
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

    /// 执行函数调用
    fn call(
        &self,
        args: Arguments,
    ) -> anyhow::Result<Variable> {
        (&self.implementation)(args)
    }

    /// 获取指定位置的参数类型
    fn param_type(
        &self,
        index: usize,
    ) -> Option<VariableType> {
        self.signature.parameters.get(index).cloned()
    }

    /// 获取指定位置参数类型的字符串表示
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

    /// 获取返回类型
    fn return_type(&self) -> VariableType {
        self.signature.return_type.clone()
    }

    /// 获取返回类型的字符串表示
    fn return_type_str(&self) -> String {
        self.signature.return_type.to_string()
    }
}

/// 复合函数
///
/// 支持多个函数重载的函数实现
#[derive(Clone)]
pub struct CompositeFunction {
    /// 函数重载签名列表
    pub signatures: Vec<FunctionSignature>,
    /// 函数实现（需要根据参数类型选择合适的重载）
    pub implementation: Rc<dyn Fn(Arguments) -> anyhow::Result<Variable>>,
}

impl FunctionDefinition for CompositeFunction {
    /// 返回最少参数数量（所有重载中参数最少的）
    fn required_parameters(&self) -> usize {
        self.signatures.iter().map(|x| x.parameters.len()).min().unwrap_or(0)
    }

    /// 返回可选参数数量（最多参数数量 - 最少参数数量）
    fn optional_parameters(&self) -> usize {
        let required_params = self.required_parameters();
        let max = self
            .signatures
            .iter()
            .map(|x| x.parameters.len())
            .max()
            .unwrap_or(0);

        max - required_params
    }

    /// 检查参数类型是否匹配任一重载
    fn check_types(
        &self,
        args: &[Rc<VariableType>],
    ) -> FunctionTypecheck {
        let mut typecheck = FunctionTypecheck::default();
        if self.signatures.is_empty() {
            typecheck.general = Some("No implementation".to_string());
            return typecheck;
        }

        let required_params = self.required_parameters();
        let optional_params = self.optional_parameters();
        let total_params = required_params + optional_params;

        // 检查参数数量是否在允许范围内
        if args.len() < required_params || args.len() > total_params {
            typecheck.general = Some(format!(
                "Expected `{required_params} - {total_params}` arguments, got `{}`.",
                args.len()
            ))
        }

        // 查找完全匹配的重载
        for signature in &self.signatures {
            let all_match = args
                .iter()
                .zip(signature.parameters.iter())
                .all(|(arg, param)| arg.satisfies(param));
            if all_match {
                typecheck.return_type = signature.return_type.clone();
                return typecheck;
            }
        }

        // 检查每个参数位置的类型错误
        for (i, arg) in args.iter().enumerate() {
            let possible_types: Vec<&VariableType> = self
                .signatures
                .iter()
                .filter_map(|sig| sig.parameters.get(i))
                .collect();

            if !possible_types.iter().any(|param| arg.satisfies(param)) {
                let type_union = self.param_type_str(i);
                typecheck.arguments.push((
                    i,
                    format!(
                        "Argument of type `{arg}` is not assignable to parameter of type `{type_union}`.",
                    ),
                ))
            }
        }

        // 生成可用重载的错误信息
        let available_signatures = self
            .signatures
            .iter()
            .map(|sig| {
                let param_list = sig
                    .parameters
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("`({param_list}) -> {}`", sig.return_type)
            })
            .collect::<Vec<_>>()
            .join("\n");
        typecheck.general = Some(format!(
            "No function overload matches provided arguments. Available overloads:\n{available_signatures}"
        ));

        typecheck
    }

    /// 执行函数调用
    fn call(
        &self,
        args: Arguments,
    ) -> anyhow::Result<Variable> {
        (&self.implementation)(args)
    }

    /// 获取指定位置的参数类型（所有重载中该位置类型的并集）
    fn param_type(
        &self,
        index: usize,
    ) -> Option<VariableType> {
        self.signatures
            .iter()
            .filter_map(|sig| sig.parameters.get(index))
            .cloned()
            .reduce(|a, b| a.merge(&b))
    }

    /// 获取指定位置参数类型的字符串表示（包含所有可能的类型）
    fn param_type_str(
        &self,
        index: usize,
    ) -> String {
        let possible_types: Vec<String> = self
            .signatures
            .iter()
            .filter_map(|sig| sig.parameters.get(index))
            .map(|x| x.to_string())
            .collect();
        if possible_types.is_empty() {
            return String::from("never");
        }

        let is_optional = possible_types.len() != self.signatures.len();
        let possible_types: Vec<String> = possible_types
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let type_union = possible_types.join(" | ");
        if is_optional {
            return format!("Optional<{type_union}>");
        }

        type_union
    }

    /// 获取返回类型（所有重载返回类型的并集）
    fn return_type(&self) -> VariableType {
        self.signatures
            .iter()
            .map(|sig| &sig.return_type)
            .cloned()
            .reduce(|a, b| a.merge(&b))
            .unwrap_or(VariableType::Null)
    }

    /// 获取返回类型的字符串表示（包含所有可能的返回类型）
    fn return_type_str(&self) -> String {
        let possible_types: Vec<String> = self
            .signatures
            .iter()
            .map(|sig| sig.return_type.clone())
            .map(|x| x.to_string())
            .collect();
        if possible_types.is_empty() {
            return String::from("never");
        }

        possible_types
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(" | ")
    }
}
