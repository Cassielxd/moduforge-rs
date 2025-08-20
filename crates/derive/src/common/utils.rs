//! 工具函数模块
//!
//! 提供宏处理过程中常用的工具函数和类型检查功能。
//! 遵循单一职责原则，每个函数都有明确的单一功能。

use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{GenericArgument, PathArguments, Type, TypePath};

/// 生成必要的导入语句
///
/// 为生成的代码添加必要的类型导入，确保生成的代码能够正确编译。
/// 遵循开闭原则，可以通过修改此函数来扩展导入的类型而不影响其他代码。
///
/// # 返回值
///
/// 返回包含所有必要导入语句的 TokenStream
///
/// # 设计原则体现
///
/// - **单一职责**: 只负责生成导入语句
/// - **开闭原则**: 可扩展新的导入而不修改调用方
/// - **接口隔离**: 提供简单明确的导入生成接口
pub fn generate_imports() -> TokenStream2 {
    quote! {
        // 模型类型导入 - Node 和 Mark 生成器所需的导入
        use mf_model::node_type::NodeSpec;
        use mf_model::schema::AttributeSpec;
        use std::collections::HashMap;
        use serde_json::Value as JsonValue;
    }
}

/// 检查类型是否为 Option<T>
///
/// 分析给定的类型是否为 Option<T> 的形式。
/// 这个函数遵循里氏替换原则，对于任何 Type 都能正确判断。
///
/// # 参数
///
/// * `ty` - 要检查的类型
///
/// # 返回值
///
/// 如果类型是 Option<T> 形式则返回 true，否则返回 false
///
/// # 示例
///
/// ```rust
/// use syn::parse_quote;
/// use crate::common::utils::is_option_type;
///
/// let option_type: syn::Type = parse_quote! { Option<String> };
/// assert!(is_option_type(&option_type));
///
/// let string_type: syn::Type = parse_quote! { String };
/// assert!(!is_option_type(&string_type));
/// ```
///
/// # 设计原则体现
///
/// - **单一职责**: 只负责判断是否为 Option 类型
/// - **里氏替换**: 任何 Type 实现都能正确处理
pub fn is_option_type(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            // 检查路径的最后一个段是否为 "Option"
            if let Some(segment) = path.segments.last() {
                segment.ident == "Option"
            } else {
                false
            }
        }
        _ => false,
    }
}

/// 提取 Option<T> 中的内部类型 T
///
/// 从 Option<T> 类型中提取内部的类型 T。
/// 如果输入不是 Option<T> 类型，则返回 None。
///
/// # 参数
///
/// * `ty` - Option<T> 类型
///
/// # 返回值
///
/// 如果成功提取则返回 Some(&T)，否则返回 None
///
/// # 示例
///
/// ```rust
/// use syn::parse_quote;
/// use crate::common::utils::extract_option_inner_type;
///
/// let option_type: syn::Type = parse_quote! { Option<String> };
/// let inner = extract_option_inner_type(&option_type);
/// assert!(inner.is_some());
/// ```
///
/// # 设计原则体现
///
/// - **单一职责**: 只负责提取 Option 的内部类型
/// - **接口隔离**: 提供明确的类型提取接口
pub fn extract_option_inner_type(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            // 获取路径的最后一个段
            let last_segment = path.segments.last()?;
            
            // 确认是 Option 类型
            if last_segment.ident != "Option" {
                return None;
            }
            
            // 提取泛型参数
            match &last_segment.arguments {
                PathArguments::AngleBracketed(args) => {
                    // 获取第一个泛型参数
                    args.args.first().and_then(|arg| match arg {
                        GenericArgument::Type(ty) => Some(ty),
                        _ => None,
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// 生成字段到 JsonValue 的转换代码
///
/// 根据字段类型生成相应的转换代码，将字段值转换为 serde_json::Value。
/// 此函数体现了开闭原则，可以通过添加新的类型处理分支来扩展功能。
///
/// # 参数
///
/// * `field_name` - 字段名称
/// * `field_type` - 字段类型
///
/// # 返回值
///
/// 返回转换代码的 TokenStream
///
/// # 转换规则
///
/// - Option<T> 类型: 如果是 Some(value) 则转换 value，如果是 None 则返回 JsonValue::Null
/// - 普通类型: 直接使用 serde_json::to_value 转换
///
/// # 示例生成的代码
///
/// ```rust
/// // 对于 Option<String> 类型的字段 name
/// self.name.as_ref().map(|v| serde_json::to_value(v).unwrap_or(JsonValue::Null))
///     .unwrap_or(JsonValue::Null)
///
/// // 对于 String 类型的字段 title  
/// serde_json::to_value(&self.title).unwrap_or(JsonValue::Null)
/// ```
///
/// # 设计原则体现
///
/// - **开闭原则**: 可扩展新的类型转换而不修改现有逻辑
/// - **单一职责**: 只负责生成字段转换代码
pub fn generate_field_conversion(field_name: &Ident, field_type: &Type) -> TokenStream2 {
    if is_option_type(field_type) {
        // Option<T> 类型的转换逻辑
        quote! {
            self.#field_name.as_ref()
                .map(|v| serde_json::to_value(v).unwrap_or(JsonValue::Null))
                .unwrap_or(JsonValue::Null)
        }
    } else {
        // 普通类型的转换逻辑
        quote! {
            serde_json::to_value(&self.#field_name).unwrap_or(JsonValue::Null)
        }
    }
}

/// 检查类型是否为支持的基本类型
///
/// 验证字段类型是否为宏系统支持的基本类型。
/// 遵循开闭原则，通过修改支持类型列表来扩展功能。
///
/// # 参数
///
/// * `ty` - 要检查的类型
///
/// # 返回值
///
/// 如果类型受支持则返回 true，否则返回 false
///
/// # 支持的类型
///
/// - String, str (字符串类型)
/// - i32, i64, u32, u64 (整数类型) 
/// - f32, f64 (浮点数类型)
/// - bool (布尔类型)
/// - usize, isize (指针大小整数类型)
///
/// # 设计原则体现
///
/// - **开闭原则**: 通过修改类型列表扩展而不修改核心逻辑
/// - **单一职责**: 只负责类型支持性检查
pub fn is_supported_basic_type(ty: &Type) -> bool {
    // 支持的基本类型列表
    const SUPPORTED_TYPES: &[&str] = &[
        "String", "str", "&str",
        "i32", "i64", "u32", "u64", "i8", "i16", "u8", "u16", "i128", "u128",
        "f32", "f64",
        "bool",
        "usize", "isize",
        "serde_json::Value", "Value", "uuid::Uuid", "Uuid", "Vec<u8>", "Vec<String>"
    ];
    
    // 获取类型的字符串表示并去除空格
    let type_str = quote! { #ty }.to_string().replace(" ", "");
    
    // 检查是否精确匹配支持的类型（而不是简单的包含）
    SUPPORTED_TYPES.iter().any(|&supported| {
        type_str == supported
    })
}

/// 检查类型是否为支持的类型（包括 Option 包装）
///
/// 检查类型是否为直接支持的基本类型或其 Option 包装版本。
/// 此函数遵循里氏替换原则，可以处理所有合法的类型输入。
///
/// # 参数
///
/// * `ty` - 要检查的类型
///
/// # 返回值
///
/// 如果类型受支持则返回 true，否则返回 false
///
/// # 示例
///
/// ```rust
/// use syn::parse_quote;
/// use crate::common::utils::is_supported_type;
///
/// let string_type: syn::Type = parse_quote! { String };
/// assert!(is_supported_type(&string_type));
///
/// let option_string: syn::Type = parse_quote! { Option<String> };
/// assert!(is_supported_type(&option_string));
///
/// let unsupported: syn::Type = parse_quote! { Vec<String> };
/// assert!(!is_supported_type(&unsupported));
/// ```
///
/// # 设计原则体现
///
/// - **里氏替换**: 任何 Type 都能正确处理
/// - **单一职责**: 专门负责检查类型支持性
pub fn is_supported_type(ty: &Type) -> bool {
    if is_option_type(ty) {
        // 对于 Option<T>，检查内部类型 T 是否支持
        if let Some(inner_type) = extract_option_inner_type(ty) {
            is_supported_basic_type(inner_type)
        } else {
            false
        }
    } else {
        // 对于普通类型，直接检查是否支持
        is_supported_basic_type(ty)
    }
}

/// 提取类型的简单名称
///
/// 从复杂的类型路径中提取简单的类型名称，用于错误消息和调试。
/// 遵循接口隔离原则，提供简洁明确的类型名称提取功能。
///
/// # 参数
///
/// * `ty` - 要提取名称的类型
///
/// # 返回值
///
/// 返回类型的简单名称字符串
///
/// # 示例
///
/// ```rust
/// use syn::parse_quote;
/// use crate::common::utils::extract_type_name;
///
/// let ty: syn::Type = parse_quote! { std::option::Option<String> };
/// assert_eq!(extract_type_name(&ty), "Option<String>");
/// ```
///
/// # 设计原则体现
///
/// - **接口隔离**: 只提供类型名称提取功能
/// - **单一职责**: 专门负责类型名称的提取和格式化
pub fn extract_type_name(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            // 提取路径的各个段
            let segments: Vec<String> = type_path.path.segments
                .iter()
                .map(|segment| {
                    let ident = &segment.ident;
                    match &segment.arguments {
                        PathArguments::AngleBracketed(args) => {
                            // 处理泛型参数
                            let args_str: Vec<String> = args.args
                                .iter()
                                .map(|arg| match arg {
                                    GenericArgument::Type(ty) => extract_type_name(ty),
                                    _ => "?".to_string(),
                                })
                                .collect();
                            format!("{}<{}>", ident, args_str.join(", "))
                        }
                        _ => ident.to_string(),
                    }
                })
                .collect();
            
            // 返回最后一个段作为类型名称
            segments.last().cloned().unwrap_or_else(|| "Unknown".to_string())
        }
        _ => {
            // 对于其他类型，返回其 TokenStream 表示
            quote! { #ty }.to_string()
        }
    }
}

/// 生成属性设置代码
///
/// 为给定的字段生成设置属性的代码，调用相应的 set_attr 方法。
/// 此函数遵循单一职责原则，专门负责属性设置代码的生成。
///
/// # 参数
///
/// * `field_name` - 字段名称标识符
/// * `field_type` - 字段类型
/// * `target` - 目标对象名称 (如 "node" 或 "mark")
///
/// # 返回值
///
/// 返回设置属性的代码 TokenStream
///
/// # 生成的代码示例
///
/// ```rust
/// // 对于字段名 "name"，类型 String
/// node.set_attr("name", Some(serde_json::to_value(&self.name).unwrap_or(JsonValue::Null)));
/// ```
///
/// # 设计原则体现
///
/// - **单一职责**: 只负责生成属性设置代码
/// - **开闭原则**: 可扩展不同的目标类型而不修改核心逻辑
pub fn generate_attr_setter_code(
    field_name: &Ident,
    field_type: &Type,
    target: &str,
) -> TokenStream2 {
    let conversion = generate_field_conversion(field_name, field_type);
    let field_name_str = field_name.to_string();
    let target_ident = syn::parse_str::<Ident>(target).unwrap_or_else(|_| {
        syn::parse_str("target").unwrap()
    });
    
    quote! {
        #target_ident.set_attr(#field_name_str, Some(#conversion));
    }
}

/// 验证标识符格式
///
/// 验证字符串是否为有效的 Rust 标识符格式。
/// 遵循单一职责原则，专门负责标识符格式验证。
///
/// # 参数
///
/// * `identifier` - 要验证的标识符字符串
///
/// # 返回值
///
/// 如果标识符格式有效则返回 true，否则返回 false
///
/// # 验证规则
///
/// - 不能为空字符串
/// - 只能包含字母、数字和下划线
/// - 必须以字母或下划线开头
///
/// # 设计原则体现
///
/// - **单一职责**: 只负责标识符格式验证
/// - **接口隔离**: 提供简单的验证接口
pub fn is_valid_identifier(identifier: &str) -> bool {
    if identifier.is_empty() {
        return false;
    }
    
    // 检查第一个字符是否为字母或下划线
    let first_char = identifier.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }
    
    // 检查其余字符是否为字母、数字或下划线
    identifier.chars().all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    /// 测试 Option 类型检查功能
    #[test]
    fn test_is_option_type() {
        let option_string: Type = parse_quote! { Option<String> };
        assert!(is_option_type(&option_string));
        
        let string: Type = parse_quote! { String };
        assert!(!is_option_type(&string));
        
        let option_int: Type = parse_quote! { Option<i32> };
        assert!(is_option_type(&option_int));
    }

    /// 测试提取 Option 内部类型功能
    #[test]
    fn test_extract_option_inner_type() {
        let option_string: Type = parse_quote! { Option<String> };
        let inner = extract_option_inner_type(&option_string);
        assert!(inner.is_some());
        
        let string: Type = parse_quote! { String };
        let inner = extract_option_inner_type(&string);
        assert!(inner.is_none());
    }

    /// 测试支持类型检查功能
    #[test]
    fn test_is_supported_type() {
        let string: Type = parse_quote! { String };
        assert!(is_supported_type(&string));
        
        let option_string: Type = parse_quote! { Option<String> };
        assert!(is_supported_type(&option_string));
        
        let vec_string: Type = parse_quote! { Vec<String> };
        assert!(!is_supported_type(&vec_string));
        
        let i32_type: Type = parse_quote! { i32 };
        assert!(is_supported_type(&i32_type));
        
        let option_i32: Type = parse_quote! { Option<i32> };
        assert!(is_supported_type(&option_i32));
    }

    /// 测试类型名称提取功能
    #[test]
    fn test_extract_type_name() {
        let string: Type = parse_quote! { String };
        assert_eq!(extract_type_name(&string), "String");
        
        let option_string: Type = parse_quote! { Option<String> };
        assert_eq!(extract_type_name(&option_string), "Option<String>");
        
        let option_i32: Type = parse_quote! { Option<i32> };
        assert_eq!(extract_type_name(&option_i32), "Option<i32>");
    }

    /// 测试标识符格式验证功能
    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("valid_name"));
        assert!(is_valid_identifier("ValidName"));
        assert!(is_valid_identifier("valid123"));
        assert!(is_valid_identifier("_private"));
        
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123invalid"));
        assert!(!is_valid_identifier("invalid-name"));
        assert!(!is_valid_identifier("invalid name"));
    }

    /// 测试导入语句生成功能
    #[test]
    fn test_generate_imports() {
        let imports = generate_imports();
        let imports_str = imports.to_string();
        
        assert!(imports_str.contains("mf_model :: node_type :: NodeSpec"));
        assert!(imports_str.contains("mf_model :: schema :: AttributeSpec"));
        assert!(imports_str.contains("serde_json :: Value"));
        assert!(imports_str.contains("std :: collections :: HashMap"));
    }

    /// 测试字段转换代码生成功能
    #[test]
    fn test_generate_field_conversion() {
        let field_name = syn::parse_str::<Ident>("test_field").unwrap();
        
        // 测试普通类型转换
        let string_type: Type = parse_quote! { String };
        let conversion = generate_field_conversion(&field_name, &string_type);
        let conversion_str = conversion.to_string();
        assert!(conversion_str.contains("serde_json :: to_value"));
        assert!(conversion_str.contains("test_field"));
        
        // 测试 Option 类型转换  
        let option_type: Type = parse_quote! { Option<String> };
        let conversion = generate_field_conversion(&field_name, &option_type);
        let conversion_str = conversion.to_string();
        assert!(conversion_str.contains("as_ref"));
        assert!(conversion_str.contains("unwrap_or"));
    }
}