//! 类型转换器模块
//!
//! 提供完整的类型转换系统，支持 Rust 基本类型到 `serde_json::Value` 的转换。
//! 严格遵循 SOLID 设计原则，确保系统的可扩展性和可维护性。
//!
//! # 模块组成
//!
//! - `type_converter`: 类型转换器核心接口和基础实现
//! - `builtin_converters`: 内置转换器实现（字符串、数值、布尔等类型）
//! - `converter_registry`: 转换器注册表，支持全局转换器管理
//!
//! # 设计原则体现
//!
//! - **单一职责原则 (SRP)**: 每个转换器都专注于特定类型的转换
//! - **开闭原则 (OCP)**: 通过实现 TypeConverter trait 扩展新的转换器
//! - **里氏替换原则 (LSP)**: 所有转换器都可以无缝替换使用
//! - **接口隔离原则 (ISP)**: 提供专门的转换接口，不强制依赖不需要的功能
//! - **依赖倒置原则 (DIP)**: 依赖于抽象接口而非具体实现

// Library code may have unused items that are part of the public API
#![allow(dead_code)]
//!
//! # 使用方式
//!
//! ## 基本使用
//!
//! ```rust
//! use syn::parse_quote;
//! use crate::converter::converter_registry::GlobalConverterRegistry;
//!
//! // 转换字段
//! let field: syn::Field = parse_quote! {
//!     name: String
//! };
//!
//! let conversion_code = GlobalConverterRegistry::convert_field(&field)?;
//! ```
//!
//! ## 注册自定义转换器
//!
//! ```rust
//! use crate::converter::{
//!     type_converter::TypeConverter,
//!     converter_registry::GlobalConverterRegistry
//! };
//!
//! // 实现自定义转换器
//! struct CustomConverter;
//!
//! impl TypeConverter for CustomConverter {
//!     fn convert_field_to_json_value(&self, field: &syn::Field) -> MacroResult<TokenStream2> {
//!         // 自定义转换逻辑
//!         todo!()
//!     }
//!
//!     fn supports_type(&self, field_type: &syn::Type) -> bool {
//!         // 支持性检查
//!         false
//!     }
//!
//!     fn priority(&self) -> i32 {
//!         200 // 高优先级
//!     }
//! }
//!
//! // 注册到全局注册表
//! GlobalConverterRegistry::register(Box::new(CustomConverter))?;
//! ```
//!
//! # 支持的类型
//!
//! ## 基本类型
//!
//! - **字符串**: String, &str, str
//! - **整数**: i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
//! - **浮点数**: f32, f64
//! - **布尔**: bool
//!
//! ## 可选类型
//!
//! - **Option<T>**: 其中 T 是上述任意支持的基本类型
//!
//! # 转换器优先级
//!
//! 转换器按优先级排序，数值越大优先级越高：
//!
//! - **BuiltinTypeConverter**: 100 - 通用基本类型转换器
//! - **NumericConverter**: 95 - 专门的数值类型转换器
//! - **StringConverter**: 90 - 专门的字符串类型转换器
//! - **BooleanConverter**: 85 - 专门的布尔类型转换器
//! - **SpecialTypeConverter**: 50 - 特殊类型转换器（预留）
//!
//! # 错误处理
//!
//! 转换过程中可能出现的错误：
//!
//! - **UnsupportedFieldType**: 字段类型不受支持
//! - **ParseError**: 字段解析错误（如缺少名称）
//! - **GenerationError**: 代码生成过程中的错误
//!
//! # 线程安全
//!
//! 全局转换器注册表是线程安全的：
//! - 读操作（转换）可以并发执行
//! - 写操作（注册）会独占锁
//! - 所有操作都有适当的错误处理
//!
//! # 性能考虑
//!
//! - 转换器按优先级预排序，查找效率高
//! - 使用 `once_cell` 实现全局单例，初始化开销低
//! - 代码生成采用零开销抽象，运行时无额外成本

/// 类型转换器核心接口模块
///
/// 定义了 TypeConverter trait 和 BuiltinTypeConverter 基础实现。
/// 遵循接口隔离原则，只包含转换相关的核心功能。
pub mod type_converter;

/// 内置转换器实现模块
///
/// 提供各种专门化的转换器实现，每个转换器专注于特定类型。
/// 遵循单一职责原则，确保转换逻辑的清晰和可维护性。
pub mod builtin_converters;

/// 转换器注册表模块
///
/// 提供全局转换器管理和查找功能。
/// 遵循单例模式和门面模式，简化转换器的使用。
pub mod converter_registry;

// 重新导出核心类型和函数，遵循接口隔离原则

/// 转换器模块的便利函数
///
/// 提供一些常用的转换器操作的快捷方式。
/// 遵循门面模式，隐藏底层复杂性。
pub mod utils {
    use proc_macro2::TokenStream as TokenStream2;
    use syn::{Field, Type};
    use crate::common::MacroResult;
    use super::converter_registry::GlobalConverterRegistry;

    /// 便利函数：转换字段
    ///
    /// 使用全局注册表转换字段的快捷方式。
    /// 等同于 `GlobalConverterRegistry::convert_field(field)`。
    ///
    /// # 参数
    ///
    /// * `field` - 要转换的字段
    ///
    /// # 返回值
    ///
    /// 成功时返回转换代码，失败时返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use syn::parse_quote;
    /// use crate::converter::utils::convert_field;
    ///
    /// let field: syn::Field = parse_quote! {
    ///     name: String
    /// };
    ///
    /// let code = convert_field(&field)?;
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **门面模式**: 提供简化的接口
    /// - **单一职责**: 只负责字段转换
    pub fn convert_field(field: &Field) -> MacroResult<TokenStream2> {
        GlobalConverterRegistry::convert_field(field)
    }

    /// 便利函数：检查类型支持性
    ///
    /// 检查全局注册表是否支持指定类型的快捷方式。
    ///
    /// # 参数
    ///
    /// * `field_type` - 要检查的字段类型
    ///
    /// # 返回值
    ///
    /// 如果类型受支持则返回 true，否则返回 false
    /// 如果访问全局注册表失败则返回 false
    ///
    /// # 示例
    ///
    /// ```rust
    /// use syn::parse_quote;
    /// use crate::converter::utils::is_type_supported;
    ///
    /// let string_type: syn::Type = parse_quote! { String };
    /// assert!(is_type_supported(&string_type));
    ///
    /// let unsupported_type: syn::Type = parse_quote! { Vec<String> };
    /// assert!(!is_type_supported(&unsupported_type));
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **门面模式**: 提供简化的检查接口
    /// - **单一职责**: 只负责支持性检查
    pub fn is_type_supported(field_type: &Type) -> bool {
        GlobalConverterRegistry::supports_type(field_type).unwrap_or(false)
    }

    /// 便利函数：获取支持类型列表
    ///
    /// 返回当前支持的所有基本类型的列表。
    /// 主要用于文档生成和错误提示。
    ///
    /// # 返回值
    ///
    /// 返回支持的类型名称列表
    ///
    /// # 设计原则体现
    ///
    /// - **信息隐藏**: 提供类型信息而不暴露内部实现
    /// - **单一职责**: 只负责提供类型信息
    pub fn get_supported_types() -> Vec<&'static str> {
        vec![
            // 字符串类型
            "String", "&str", "str", // 有符号整数类型
            "i8", "i16", "i32", "i64", "i128", "isize",
            // 无符号整数类型
            "u8", "u16", "u32", "u64", "u128", "usize",
            // 浮点数类型
            "f32", "f64", // 布尔类型
            "bool",
        ]
    }

    /// 便利函数：生成支持类型的错误提示
    ///
    /// 生成友好的错误提示信息，列出所有支持的类型。
    /// 用于改善用户体验的错误消息。
    ///
    /// # 返回值
    ///
    /// 返回包含支持类型列表的错误提示字符串
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::converter::utils::generate_supported_types_hint;
    ///
    /// let hint = generate_supported_types_hint();
    /// assert!(hint.contains("String"));
    /// assert!(hint.contains("i32"));
    /// assert!(hint.contains("bool"));
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **用户体验**: 提供友好的错误提示
    /// - **信息封装**: 统一错误消息格式
    pub fn generate_supported_types_hint() -> String {
        let basic_types = get_supported_types();
        let basic_list = basic_types.join(", ");

        format!(
            "支持的基本类型: {basic_list}\n\
            支持的可选类型: Option<T> (其中 T 是上述任意基本类型)\n\
            \n\
            示例：\n\
            - String, Option<String>\n\
            - i32, Option<i32>\n\
            - bool, Option<bool>\n\
            \n\
            如需支持其他类型，请实现自定义 TypeConverter 并注册到全局注册表。"
        )
    }
}
