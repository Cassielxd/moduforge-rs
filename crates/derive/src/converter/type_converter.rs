//! 类型转换器核心接口模块
//!
//! 提供类型转换的核心抽象和基础实现。
//! 严格遵循开闭原则，支持通过实现 TypeConverter trait 来扩展新的类型转换功能。

use proc_macro2::TokenStream as TokenStream2;
use syn::{Field, Type};
use crate::common::{MacroError, MacroResult, utils};

/// 类型转换器接口
///
/// 定义了类型转换的核心接口，遵循接口隔离原则。
/// 任何实现此接口的类型都能提供字段到 JSON 值的转换功能。
///
/// # 线程安全
///
/// 此 trait 要求实现 `Send + Sync`，确保转换器可以在多线程环境中安全使用。
/// 这对于全局注册表的线程安全是必要的。
///
/// # 设计原则体现
///
/// - **接口隔离原则**: 只定义转换相关的必要方法
/// - **开闭原则**: 通过实现此接口可以扩展新的转换器而不修改现有代码
/// - **里氏替换原则**: 所有实现都能够无缝替换使用
///
/// # 示例实现
///
/// ```rust
/// use crate::converter::type_converter::TypeConverter;
/// use proc_macro2::TokenStream as TokenStream2;
/// use syn::{Field, Type};
/// use crate::common::MacroResult;
///
/// struct CustomConverter;
///
/// impl TypeConverter for CustomConverter {
///     fn convert_field_to_json_value(&self, field: &Field) -> MacroResult<TokenStream2> {
///         // 实现自定义转换逻辑
///         unimplemented!()
///     }
///
///     fn supports_type(&self, field_type: &Type) -> bool {
///         // 检查是否支持此类型
///         false
///     }
/// }
/// ```
pub trait TypeConverter: Send + Sync {
    /// 将字段转换为 JSON 值的代码
    ///
    /// 生成将结构体字段转换为 `serde_json::Value` 的代码。
    /// 此方法是转换器的核心功能，必须生成正确的 Rust 代码。
    ///
    /// # 参数
    ///
    /// * `field` - 要转换的字段信息
    ///
    /// # 返回值
    ///
    /// 成功时返回生成的代码 `TokenStream`，失败时返回转换错误
    ///
    /// # 生成代码要求
    ///
    /// - 代码必须能正确编译
    /// - 必须返回 `serde_json::Value` 类型
    /// - 必须正确处理 Option 类型
    /// - 错误处理要求友好的错误消息
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责代码生成，不处理验证或其他逻辑
    /// - **里氏替换**: 任何实现都必须能生成有效的转换代码
    fn convert_field_to_json_value(
        &self,
        field: &Field,
    ) -> MacroResult<TokenStream2>;

    /// 检查是否支持指定类型
    ///
    /// 判断当前转换器是否能够处理给定的字段类型。
    /// 此方法用于转换器选择和验证逻辑。
    ///
    /// # 参数
    ///
    /// * `field_type` - 要检查的字段类型
    ///
    /// # 返回值
    ///
    /// 如果支持该类型则返回 true，否则返回 false
    ///
    /// # 实现要求
    ///
    /// - 必须能准确判断类型支持性
    /// - 判断逻辑要与 `convert_field_to_json_value` 一致
    /// - 对于不确定的类型应该返回 false
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责类型支持性检查
    /// - **接口隔离**: 提供独立的支持性检查接口
    fn supports_type(
        &self,
        field_type: &Type,
    ) -> bool;

    /// 获取转换器的优先级
    ///
    /// 返回转换器的优先级，用于注册表中的排序。
    /// 数值越大优先级越高，默认优先级为 0。
    ///
    /// # 返回值
    ///
    /// 返回转换器的优先级数值
    ///
    /// # 设计原则体现
    ///
    /// - **开闭原则**: 允许通过优先级控制转换器选择而不修改核心逻辑
    /// - **单一职责**: 只负责提供优先级信息
    fn priority(&self) -> i32 {
        0
    }

    /// 获取转换器的名称
    ///
    /// 返回转换器的名称，用于调试和错误消息。
    /// 默认实现返回类型名称。
    ///
    /// # 返回值
    ///
    /// 返回转换器的名称字符串
    ///
    /// # 设计原则体现
    ///
    /// - **接口隔离**: 提供可选的名称接口
    /// - **单一职责**: 只负责提供转换器标识信息
    fn name(&self) -> &'static str {
        "UnknownConverter"
    }
}

/// 内置类型转换器
///
/// 提供对 Rust 基本类型的转换支持。
/// 遵循单一职责原则，专门处理基本类型的转换逻辑。
///
/// # 支持的类型
///
/// - 字符串类型: String, &str, str
/// - 整数类型: i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
/// - 浮点类型: f32, f64
/// - 布尔类型: bool
/// - 可选类型: Option<T> (其中 T 是上述支持的类型)
///
/// # 设计原则体现
///
/// - **单一职责**: 只负责基本类型转换
/// - **里氏替换**: 实现了 TypeConverter，可以替换其他转换器使用
/// - **开闭原则**: 通过实现接口扩展功能而不修改现有代码
#[derive(Debug, Clone)]
pub struct BuiltinTypeConverter;

impl BuiltinTypeConverter {
    /// 创建新的内置类型转换器实例
    ///
    /// # 返回值
    ///
    /// 返回配置好的内置类型转换器
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::converter::type_converter::BuiltinTypeConverter;
    ///
    /// let converter = BuiltinTypeConverter::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// 为基本类型生成转换代码
    ///
    /// 根据字段类型生成相应的转换代码。
    /// 遵循单一职责原则，专门负责基本类型的代码生成。
    ///
    /// # 参数
    ///
    /// * `field` - 字段信息
    /// * `field_type` - 字段类型
    ///
    /// # 返回值
    ///
    /// 成功时返回转换代码，失败时返回错误
    ///
    /// # 生成的代码模式
    ///
    /// - 普通类型: `serde_json::to_value(&self.field_name).unwrap_or(JsonValue::Null)`
    /// - Option 类型: `self.field_name.as_ref().map(|v| serde_json::to_value(v).unwrap_or(JsonValue::Null)).unwrap_or(JsonValue::Null)`
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责基本类型的代码生成
    /// - **开闭原则**: 可扩展支持新的基本类型
    fn generate_basic_type_conversion(
        &self,
        field: &Field,
        field_type: &Type,
    ) -> MacroResult<TokenStream2> {
        let field_name = field.ident.as_ref().ok_or_else(|| {
            MacroError::parse_error("字段缺少名称（不支持匿名字段）", field)
        })?;

        // 使用通用工具函数生成转换代码
        let conversion_code =
            utils::generate_field_conversion(field_name, field_type);

        Ok(conversion_code)
    }

    /// 检查是否为支持的基本类型
    ///
    /// 检查字段类型是否为内置转换器支持的基本类型。
    /// 遵循单一职责原则，专门负责基本类型的支持性检查。
    ///
    /// # 参数
    ///
    /// * `field_type` - 要检查的字段类型
    ///
    /// # 返回值
    ///
    /// 如果是支持的基本类型则返回 true，否则返回 false
    ///
    /// # 支持的类型判断
    ///
    /// - 使用 `utils::is_supported_type` 进行类型检查
    /// - 支持基本类型和它们的 Option 包装版本
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责基本类型支持性检查
    /// - **里氏替换**: 与接口中的 supports_type 方法一致
    fn is_supported_basic_type(
        &self,
        field_type: &Type,
    ) -> bool {
        utils::is_supported_type(field_type)
    }
}

impl Default for BuiltinTypeConverter {
    /// 创建默认的内置类型转换器
    ///
    /// # 返回值
    ///
    /// 返回默认配置的内置类型转换器
    fn default() -> Self {
        Self::new()
    }
}

impl TypeConverter for BuiltinTypeConverter {
    /// 将字段转换为 JSON 值的代码
    ///
    /// 为基本类型字段生成转换为 `serde_json::Value` 的代码。
    /// 支持基本类型和它们的 Option 包装版本。
    ///
    /// # 参数
    ///
    /// * `field` - 要转换的字段信息
    ///
    /// # 返回值
    ///
    /// 成功时返回生成的转换代码，失败时返回转换错误
    ///
    /// # 错误情况
    ///
    /// - 字段缺少名称（匿名字段）
    /// - 字段类型不受支持
    /// - 代码生成过程中的其他错误
    ///
    /// # 设计原则体现
    ///
    /// - **里氏替换**: 完全符合 TypeConverter 接口契约
    /// - **单一职责**: 只负责基本类型的转换代码生成
    fn convert_field_to_json_value(
        &self,
        field: &Field,
    ) -> MacroResult<TokenStream2> {
        // 首先检查是否支持该类型
        if !self.supports_type(&field.ty) {
            let type_name = utils::extract_type_name(&field.ty);
            let field_name = field
                .ident
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or_else(|| "匿名字段".to_string());

            return Err(MacroError::unsupported_field_type(
                &field_name,
                &type_name,
                field,
            ));
        }

        // 生成转换代码
        self.generate_basic_type_conversion(field, &field.ty)
    }

    /// 检查是否支持指定类型
    ///
    /// 判断内置转换器是否能处理给定的字段类型。
    /// 支持所有基本类型和它们的 Option 包装版本。
    ///
    /// # 参数
    ///
    /// * `field_type` - 要检查的字段类型
    ///
    /// # 返回值
    ///
    /// 如果支持该类型则返回 true，否则返回 false
    ///
    /// # 支持性检查逻辑
    ///
    /// - 委托给 `utils::is_supported_type` 进行具体检查
    /// - 确保与 `convert_field_to_json_value` 的逻辑一致
    ///
    /// # 设计原则体现
    ///
    /// - **里氏替换**: 与接口定义完全一致
    /// - **单一职责**: 只负责类型支持性判断
    fn supports_type(
        &self,
        field_type: &Type,
    ) -> bool {
        self.is_supported_basic_type(field_type)
    }

    /// 获取转换器的优先级
    ///
    /// 内置转换器的优先级为 100，确保它在自定义转换器之前被选择。
    ///
    /// # 返回值
    ///
    /// 返回优先级数值 100
    ///
    /// # 设计原则体现
    ///
    /// - **开闭原则**: 通过优先级机制支持转换器覆盖
    fn priority(&self) -> i32 {
        100
    }

    /// 获取转换器的名称
    ///
    /// 返回内置转换器的名称，用于调试和错误消息。
    ///
    /// # 返回值
    ///
    /// 返回转换器名称 "BuiltinTypeConverter"
    ///
    /// # 设计原则体现
    ///
    /// - **接口隔离**: 提供清晰的转换器标识
    fn name(&self) -> &'static str {
        "BuiltinTypeConverter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    /// 测试内置类型转换器的创建
    #[test]
    fn test_builtin_converter_creation() {
        let converter = BuiltinTypeConverter::new();
        assert_eq!(converter.name(), "BuiltinTypeConverter");
        assert_eq!(converter.priority(), 100);
    }

    /// 测试默认构造器
    #[test]
    fn test_builtin_converter_default() {
        let converter = BuiltinTypeConverter;
        assert_eq!(converter.name(), "BuiltinTypeConverter");
    }

    /// 测试支持的类型检查
    #[test]
    fn test_builtin_converter_supports_type() {
        let converter = BuiltinTypeConverter::new();

        // 测试支持的基本类型
        let string_type: Type = parse_quote! { String };
        assert!(converter.supports_type(&string_type));

        let i32_type: Type = parse_quote! { i32 };
        assert!(converter.supports_type(&i32_type));

        let bool_type: Type = parse_quote! { bool };
        assert!(converter.supports_type(&bool_type));

        // 测试支持的 Option 类型
        let option_string: Type = parse_quote! { Option<String> };
        assert!(converter.supports_type(&option_string));

        let option_i32: Type = parse_quote! { Option<i32> };
        assert!(converter.supports_type(&option_i32));

        // 测试不支持的类型
        let vec_type: Type = parse_quote! { Vec<String> };
        assert!(!converter.supports_type(&vec_type));

        let hashmap_type: Type = parse_quote! { HashMap<String, i32> };
        assert!(!converter.supports_type(&hashmap_type));
    }

    /// 测试字段转换代码生成
    #[test]
    fn test_builtin_converter_field_conversion() {
        let converter = BuiltinTypeConverter::new();

        // 测试基本类型字段转换
        let field: syn::Field = parse_quote! {
            name: String
        };

        let result = converter.convert_field_to_json_value(&field);
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();
        assert!(code_str.contains("serde_json::to_value"));
        assert!(code_str.contains("name"));

        // 测试 Option 类型字段转换
        let option_field: syn::Field = parse_quote! {
            age: Option<i32>
        };

        let result = converter.convert_field_to_json_value(&option_field);
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();
        assert!(code_str.contains("as_ref"));
        assert!(code_str.contains("unwrap_or"));
    }

    /// 测试不支持类型的错误处理
    #[test]
    fn test_builtin_converter_unsupported_type() {
        let converter = BuiltinTypeConverter::new();

        // 测试不支持的类型
        let unsupported_field: syn::Field = parse_quote! {
            data: Vec<String>
        };

        let result = converter.convert_field_to_json_value(&unsupported_field);
        assert!(result.is_err());

        if let Err(MacroError::UnsupportedFieldType {
            field_name,
            field_type,
            ..
        }) = result
        {
            assert_eq!(field_name, "data");
            assert!(field_type.contains("Vec"));
        } else {
            panic!("期望 UnsupportedFieldType 错误");
        }
    }

    /// 测试匿名字段的错误处理
    #[test]
    fn test_builtin_converter_anonymous_field() {
        let converter = BuiltinTypeConverter::new();

        // 测试匿名字段（元组结构体字段）
        let anonymous_field: syn::Field = parse_quote! {
            String
        };

        let result = converter.convert_field_to_json_value(&anonymous_field);
        assert!(result.is_err());

        if let Err(MacroError::UnsupportedFieldType { .. }) = result {
            // 正确的错误类型
        } else if let Err(MacroError::ParseError { .. }) = result {
            // 也可能是解析错误
        } else {
            panic!("期望 UnsupportedFieldType 或 ParseError");
        }
    }

    /// 测试类型支持性与转换的一致性
    #[test]
    fn test_builtin_converter_consistency() {
        let converter = BuiltinTypeConverter::new();

        let test_types = vec![
            parse_quote! { String },
            parse_quote! { i32 },
            parse_quote! { Option<String> },
            parse_quote! { Option<i32> },
            parse_quote! { Vec<String> },
            parse_quote! { HashMap<String, i32> },
        ];

        for ty in test_types {
            let supports = converter.supports_type(&ty);

            // 创建一个测试字段
            let field: syn::Field = parse_quote! {
                test_field: #ty
            };

            let conversion_result =
                converter.convert_field_to_json_value(&field);

            // 支持性检查与实际转换应该一致
            if supports {
                assert!(
                    conversion_result.is_ok(),
                    "类型 {ty:?} 声称支持但转换失败"
                );
            } else {
                assert!(
                    conversion_result.is_err(),
                    "类型 {ty:?} 声称不支持但转换成功"
                );
            }
        }
    }
}
