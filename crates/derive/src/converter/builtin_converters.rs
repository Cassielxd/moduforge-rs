//! 内置转换器模块
//!
//! 提供各种 Rust 基本类型到 `serde_json::Value` 的转换器实现。
//! 遵循开闭原则，每个转换器都是独立的实现，可以单独使用或组合使用。

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Field, Type};
use crate::common::{MacroError, MacroResult, utils};
use super::type_converter::TypeConverter;

/// 字符串类型转换器
///
/// 专门处理字符串相关类型的转换，包括 String、&str、str 和它们的 Option 版本。
/// 遵循单一职责原则，只负责字符串类型的转换逻辑。
///
/// # 支持的类型
///
/// - String
/// - &str (引用字符串)
/// - str (字符串切片)
/// - Option<String>
/// - Option<&str>
/// - Option<str>
///
/// # 设计原则体现
///
/// - **单一职责**: 只负责字符串类型转换
/// - **里氏替换**: 实现了 TypeConverter，可替换使用
/// - **接口隔离**: 提供专门的字符串转换接口
#[derive(Debug, Clone)]
pub struct StringConverter;

impl StringConverter {
    /// 创建新的字符串转换器实例
    ///
    /// # 返回值
    ///
    /// 返回配置好的字符串转换器
    pub fn new() -> Self {
        Self
    }

    /// 检查是否为字符串类型
    ///
    /// 判断给定类型是否为字符串相关类型。
    /// 遵循单一职责原则，专门负责字符串类型识别。
    ///
    /// # 参数
    ///
    /// * `field_type` - 要检查的字段类型
    ///
    /// # 返回值
    ///
    /// 如果是字符串类型则返回 true，否则返回 false
    ///
    /// # 识别逻辑
    ///
    /// - 对于普通类型，检查是否为 String、&str、str
    /// - 对于 Option<T>，检查内部类型 T 是否为字符串类型
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责字符串类型识别
    /// - **里氏替换**: 任何字符串类型都能正确识别
    fn is_string_type(
        &self,
        field_type: &Type,
    ) -> bool {
        if utils::is_option_type(field_type) {
            // Option<T> 类型，检查内部类型
            if let Some(inner_type) =
                utils::extract_option_inner_type(field_type)
            {
                self.is_basic_string_type(inner_type)
            } else {
                false
            }
        } else {
            // 普通类型
            self.is_basic_string_type(field_type)
        }
    }

    /// 检查是否为基本字符串类型
    ///
    /// 判断类型是否为基本的字符串类型（不包括 Option 包装）。
    /// 遵循单一职责原则，专门负责基本字符串类型检查。
    ///
    /// # 参数
    ///
    /// * `field_type` - 要检查的字段类型
    ///
    /// # 返回值
    ///
    /// 如果是基本字符串类型则返回 true，否则返回 false
    fn is_basic_string_type(
        &self,
        field_type: &Type,
    ) -> bool {
        let type_name = utils::extract_type_name(field_type);
        matches!(type_name.as_str(), "String" | "str" | "&str")
    }
}

impl Default for StringConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeConverter for StringConverter {
    fn convert_field_to_json_value(
        &self,
        field: &Field,
    ) -> MacroResult<TokenStream2> {
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

        let field_name = field.ident.as_ref().ok_or_else(|| {
            MacroError::parse_error("字段缺少名称（不支持匿名字段）", field)
        })?;

        // 生成字符串转换代码
        let conversion_code =
            utils::generate_field_conversion(field_name, &field.ty);
        Ok(conversion_code)
    }

    fn supports_type(
        &self,
        field_type: &Type,
    ) -> bool {
        self.is_string_type(field_type)
    }

    fn priority(&self) -> i32 {
        90 // 比通用转换器高，但比数值转换器低
    }

    fn name(&self) -> &'static str {
        "StringConverter"
    }
}

/// 数值类型转换器
///
/// 专门处理数值类型的转换，包括整数和浮点数类型及它们的 Option 版本。
/// 遵循单一职责原则，只负责数值类型的转换逻辑。
///
/// # 支持的类型
///
/// - 有符号整数: i8, i16, i32, i64, i128, isize
/// - 无符号整数: u8, u16, u32, u64, u128, usize
/// - 浮点数: f32, f64
/// - 对应的 Option 包装版本
///
/// # 设计原则体现
///
/// - **单一职责**: 只负责数值类型转换
/// - **里氏替换**: 实现了 TypeConverter，可替换使用
/// - **接口隔离**: 提供专门的数值转换接口
#[derive(Debug, Clone)]
pub struct NumericConverter;

impl NumericConverter {
    /// 创建新的数值转换器实例
    pub fn new() -> Self {
        Self
    }

    /// 检查是否为数值类型
    ///
    /// 判断给定类型是否为数值相关类型。
    /// 遵循单一职责原则，专门负责数值类型识别。
    ///
    /// # 参数
    ///
    /// * `field_type` - 要检查的字段类型
    ///
    /// # 返回值
    ///
    /// 如果是数值类型则返回 true，否则返回 false
    fn is_numeric_type(
        &self,
        field_type: &Type,
    ) -> bool {
        if utils::is_option_type(field_type) {
            // Option<T> 类型，检查内部类型
            if let Some(inner_type) =
                utils::extract_option_inner_type(field_type)
            {
                self.is_basic_numeric_type(inner_type)
            } else {
                false
            }
        } else {
            // 普通类型
            self.is_basic_numeric_type(field_type)
        }
    }

    /// 检查是否为基本数值类型
    ///
    /// 判断类型是否为基本的数值类型（不包括 Option 包装）。
    /// 遵循单一职责原则，专门负责基本数值类型检查。
    fn is_basic_numeric_type(
        &self,
        field_type: &Type,
    ) -> bool {
        let type_name = utils::extract_type_name(field_type);
        matches!(
            type_name.as_str(),
            "i8" | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "f32"
                | "f64"
        )
    }
}

impl Default for NumericConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeConverter for NumericConverter {
    fn convert_field_to_json_value(
        &self,
        field: &Field,
    ) -> MacroResult<TokenStream2> {
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

        let field_name = field.ident.as_ref().ok_or_else(|| {
            MacroError::parse_error("字段缺少名称（不支持匿名字段）", field)
        })?;

        // 生成数值转换代码
        let conversion_code =
            utils::generate_field_conversion(field_name, &field.ty);
        Ok(conversion_code)
    }

    fn supports_type(
        &self,
        field_type: &Type,
    ) -> bool {
        self.is_numeric_type(field_type)
    }

    fn priority(&self) -> i32 {
        95 // 比字符串转换器高，数值转换通常更严格
    }

    fn name(&self) -> &'static str {
        "NumericConverter"
    }
}

/// 布尔类型转换器
///
/// 专门处理布尔类型的转换。
/// 遵循单一职责原则，只负责布尔类型的转换逻辑。
///
/// # 支持的类型
///
/// - bool
/// - Option<bool>
///
/// # 设计原则体现
///
/// - **单一职责**: 只负责布尔类型转换
/// - **里氏替换**: 实现了 TypeConverter，可替换使用
/// - **接口隔离**: 提供专门的布尔转换接口
#[derive(Debug, Clone)]
pub struct BooleanConverter;

impl BooleanConverter {
    /// 创建新的布尔转换器实例
    pub fn new() -> Self {
        Self
    }

    /// 检查是否为布尔类型
    ///
    /// 判断给定类型是否为布尔类型。
    /// 遵循单一职责原则，专门负责布尔类型识别。
    fn is_boolean_type(
        &self,
        field_type: &Type,
    ) -> bool {
        if utils::is_option_type(field_type) {
            // Option<T> 类型，检查内部类型
            if let Some(inner_type) =
                utils::extract_option_inner_type(field_type)
            {
                self.is_basic_boolean_type(inner_type)
            } else {
                false
            }
        } else {
            // 普通类型
            self.is_basic_boolean_type(field_type)
        }
    }

    /// 检查是否为基本布尔类型
    fn is_basic_boolean_type(
        &self,
        field_type: &Type,
    ) -> bool {
        let type_name = utils::extract_type_name(field_type);
        type_name == "bool"
    }
}

impl Default for BooleanConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeConverter for BooleanConverter {
    fn convert_field_to_json_value(
        &self,
        field: &Field,
    ) -> MacroResult<TokenStream2> {
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

        let field_name = field.ident.as_ref().ok_or_else(|| {
            MacroError::parse_error("字段缺少名称（不支持匿名字段）", field)
        })?;

        // 生成布尔转换代码
        let conversion_code =
            utils::generate_field_conversion(field_name, &field.ty);
        Ok(conversion_code)
    }

    fn supports_type(
        &self,
        field_type: &Type,
    ) -> bool {
        self.is_boolean_type(field_type)
    }

    fn priority(&self) -> i32 {
        85 // 较低优先级，布尔类型相对简单
    }

    fn name(&self) -> &'static str {
        "BooleanConverter"
    }
}

/// 特殊类型转换器
///
/// 处理一些特殊的类型转换，如集合类型、自定义序列化类型等。
/// 遵循开闭原则，可以通过扩展此转换器来支持更多特殊类型。
///
/// # 当前支持的类型
///
/// 暂时为空，预留给未来的扩展使用。
/// 可以通过实现此转换器来支持 Vec、HashMap 等集合类型。
///
/// # 设计原则体现
///
/// - **开闭原则**: 可扩展支持新的特殊类型而不修改现有代码
/// - **单一职责**: 专门负责特殊类型转换
/// - **里氏替换**: 实现了 TypeConverter，可替换使用
#[derive(Debug, Clone)]
pub struct SpecialTypeConverter;

impl SpecialTypeConverter {
    /// 创建新的特殊类型转换器实例
    pub fn new() -> Self {
        Self
    }

    /// 检查是否为支持的特殊类型
    ///
    /// 判断给定类型是否为支持的特殊类型。
    /// 当前返回 false，预留给未来扩展。
    ///
    /// # 参数
    ///
    /// * `_field_type` - 要检查的字段类型（当前未使用）
    ///
    /// # 返回值
    ///
    /// 当前总是返回 false，未来可以扩展支持特殊类型
    ///
    /// # 设计原则体现
    ///
    /// - **开闭原则**: 为未来扩展预留接口
    /// - **单一职责**: 只负责特殊类型识别
    #[allow(unused_variables)]
    fn is_special_type(
        &self,
        _field_type: &Type,
    ) -> bool {
        // 暂时不支持任何特殊类型，为未来扩展预留
        // 未来可以在这里添加对 Vec、HashMap、自定义类型等的支持
        false
    }
}

impl Default for SpecialTypeConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeConverter for SpecialTypeConverter {
    fn convert_field_to_json_value(
        &self,
        field: &Field,
    ) -> MacroResult<TokenStream2> {
        // 当前不支持任何特殊类型，直接返回错误
        let type_name = utils::extract_type_name(&field.ty);
        let field_name = field
            .ident
            .as_ref()
            .map(|i| i.to_string())
            .unwrap_or_else(|| "匿名字段".to_string());

        Err(MacroError::unsupported_field_type(&field_name, &type_name, field))
    }

    fn supports_type(
        &self,
        field_type: &Type,
    ) -> bool {
        self.is_special_type(field_type)
    }

    fn priority(&self) -> i32 {
        50 // 中等优先级，特殊类型可能需要优先处理
    }

    fn name(&self) -> &'static str {
        "SpecialTypeConverter"
    }
}

/// 获取所有内置转换器
///
/// 返回一个包含所有内置转换器的向量。
/// 遵循工厂模式，统一创建和管理内置转换器实例。
///
/// # 返回值
///
/// 返回包含所有内置转换器的 Box<dyn TypeConverter> 向量
///
/// # 转换器顺序
///
/// 转换器按优先级排序（高到低）：
/// 1. NumericConverter (95) - 数值类型转换器
/// 2. StringConverter (90) - 字符串类型转换器  
/// 3. BooleanConverter (85) - 布尔类型转换器
/// 4. SpecialTypeConverter (50) - 特殊类型转换器
///
/// # 设计原则体现
///
/// - **工厂模式**: 统一创建转换器实例
/// - **开闭原则**: 可以扩展新的内置转换器而不修改调用方
/// - **依赖注入**: 返回接口类型而非具体类型
///
/// # 示例
///
/// ```rust
/// use crate::converter::builtin_converters::get_all_builtin_converters;
///
/// let converters = get_all_builtin_converters();
/// assert_eq!(converters.len(), 4);
///
/// // 验证转换器优先级顺序
/// assert!(converters[0].priority() >= converters[1].priority());
/// ```
pub fn get_all_builtin_converters() -> Vec<Box<dyn TypeConverter>> {
    let mut converters: Vec<Box<dyn TypeConverter>> = vec![
        Box::new(NumericConverter::new()),
        Box::new(StringConverter::new()),
        Box::new(BooleanConverter::new()),
        Box::new(SpecialTypeConverter::new()),
    ];

    // 按优先级降序排序
    converters.sort_by(|a, b| b.priority().cmp(&a.priority()));

    converters
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    /// 测试字符串转换器
    #[test]
    fn test_string_converter() {
        let converter = StringConverter::new();

        // 测试支持的类型
        let string_type: Type = parse_quote! { String };
        assert!(converter.supports_type(&string_type));

        let str_type: Type = parse_quote! { &str };
        assert!(converter.supports_type(&str_type));

        let option_string: Type = parse_quote! { Option<String> };
        assert!(converter.supports_type(&option_string));

        // 测试不支持的类型
        let i32_type: Type = parse_quote! { i32 };
        assert!(!converter.supports_type(&i32_type));

        // 测试名称和优先级
        assert_eq!(converter.name(), "StringConverter");
        assert_eq!(converter.priority(), 90);
    }

    /// 测试数值转换器
    #[test]
    fn test_numeric_converter() {
        let converter = NumericConverter::new();

        // 测试支持的类型
        let i32_type: Type = parse_quote! { i32 };
        assert!(converter.supports_type(&i32_type));

        let f64_type: Type = parse_quote! { f64 };
        assert!(converter.supports_type(&f64_type));

        let option_i32: Type = parse_quote! { Option<i32> };
        assert!(converter.supports_type(&option_i32));

        // 测试不支持的类型
        let string_type: Type = parse_quote! { String };
        assert!(!converter.supports_type(&string_type));

        // 测试名称和优先级
        assert_eq!(converter.name(), "NumericConverter");
        assert_eq!(converter.priority(), 95);
    }

    /// 测试布尔转换器
    #[test]
    fn test_boolean_converter() {
        let converter = BooleanConverter::new();

        // 测试支持的类型
        let bool_type: Type = parse_quote! { bool };
        assert!(converter.supports_type(&bool_type));

        let option_bool: Type = parse_quote! { Option<bool> };
        assert!(converter.supports_type(&option_bool));

        // 测试不支持的类型
        let i32_type: Type = parse_quote! { i32 };
        assert!(!converter.supports_type(&i32_type));

        // 测试名称和优先级
        assert_eq!(converter.name(), "BooleanConverter");
        assert_eq!(converter.priority(), 85);
    }

    /// 测试特殊类型转换器
    #[test]
    fn test_special_type_converter() {
        let converter = SpecialTypeConverter::new();

        // 当前不支持任何类型
        let vec_type: Type = parse_quote! { Vec<String> };
        assert!(!converter.supports_type(&vec_type));

        let hashmap_type: Type = parse_quote! { HashMap<String, i32> };
        assert!(!converter.supports_type(&hashmap_type));

        // 测试名称和优先级
        assert_eq!(converter.name(), "SpecialTypeConverter");
        assert_eq!(converter.priority(), 50);
    }

    /// 测试字符串转换器的代码生成
    #[test]
    fn test_string_converter_code_generation() {
        let converter = StringConverter::new();

        let field: syn::Field = parse_quote! {
            name: String
        };

        let result = converter.convert_field_to_json_value(&field);
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();
        assert!(code_str.contains("serde_json::to_value"));
        assert!(code_str.contains("name"));
    }

    /// 测试数值转换器的代码生成
    #[test]
    fn test_numeric_converter_code_generation() {
        let converter = NumericConverter::new();

        let field: syn::Field = parse_quote! {
            age: i32
        };

        let result = converter.convert_field_to_json_value(&field);
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();
        assert!(code_str.contains("serde_json::to_value"));
        assert!(code_str.contains("age"));
    }

    /// 测试布尔转换器的代码生成
    #[test]
    fn test_boolean_converter_code_generation() {
        let converter = BooleanConverter::new();

        let field: syn::Field = parse_quote! {
            active: bool
        };

        let result = converter.convert_field_to_json_value(&field);
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();
        assert!(code_str.contains("serde_json::to_value"));
        assert!(code_str.contains("active"));
    }

    /// 测试特殊转换器的错误处理
    #[test]
    fn test_special_converter_error_handling() {
        let converter = SpecialTypeConverter::new();

        let field: syn::Field = parse_quote! {
            data: Vec<String>
        };

        let result = converter.convert_field_to_json_value(&field);
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

    /// 测试获取所有内置转换器
    #[test]
    fn test_get_all_builtin_converters() {
        let converters = get_all_builtin_converters();

        assert_eq!(converters.len(), 4);

        // 验证包含所有转换器
        let names: Vec<_> = converters.iter().map(|c| c.name()).collect();
        assert!(names.contains(&"NumericConverter"));
        assert!(names.contains(&"StringConverter"));
        assert!(names.contains(&"BooleanConverter"));
        assert!(names.contains(&"SpecialTypeConverter"));

        // 验证优先级排序（降序）
        for i in 0..converters.len() - 1 {
            assert!(
                converters[i].priority() >= converters[i + 1].priority(),
                "转换器未按优先级正确排序"
            );
        }
    }

    /// 测试转换器默认构造器
    #[test]
    fn test_converter_defaults() {
        let string_converter = StringConverter::default();
        assert_eq!(string_converter.name(), "StringConverter");

        let numeric_converter = NumericConverter::default();
        assert_eq!(numeric_converter.name(), "NumericConverter");

        let boolean_converter = BooleanConverter::default();
        assert_eq!(boolean_converter.name(), "BooleanConverter");

        let special_converter = SpecialTypeConverter::default();
        assert_eq!(special_converter.name(), "SpecialTypeConverter");
    }

    /// 测试转换器类型覆盖
    #[test]
    fn test_converter_type_coverage() {
        let converters = get_all_builtin_converters();

        // 测试各种类型的支持情况
        let test_types = vec![
            (parse_quote! { String }, true),
            (parse_quote! { i32 }, true),
            (parse_quote! { f64 }, true),
            (parse_quote! { bool }, true),
            (parse_quote! { Option<String> }, true),
            (parse_quote! { Option<i32> }, true),
            (parse_quote! { Option<bool> }, true),
            (parse_quote! { Vec<String> }, false),
            (parse_quote! { HashMap<String, i32> }, false),
        ];

        for (ty, should_be_supported) in test_types {
            let supported = converters.iter().any(|c| c.supports_type(&ty));
            assert_eq!(
                supported, should_be_supported,
                "类型 {:?} 的支持状态不正确",
                ty
            );
        }
    }
}
