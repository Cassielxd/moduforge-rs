//! 错误处理模块
//!
//! 提供统一的宏处理错误类型和友好的编译时错误消息。
//! 严格遵循单一职责原则，专门负责错误类型定义和错误处理逻辑。

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::spanned::Spanned;
use thiserror::Error;

/// 宏处理过程中的错误类型
///
/// 此枚举定义了宏处理过程中可能发生的所有错误类型，
/// 遵循接口隔离原则，每种错误类型都有明确的语义和用途。
#[derive(Error, Debug)]
pub enum MacroError {
    /// 缺少必需的宏属性错误
    ///
    /// 当结构体缺少必需的宏属性（如 `node_type` 或 `mark_type`）时触发此错误
    #[error("缺少必需的宏属性: {attribute}")]
    MissingAttribute {
        /// 缺少的属性名称
        attribute: String,
        /// 错误发生的代码位置
        span: Option<Span>,
    },

    /// 无效的属性值错误
    ///
    /// 当宏属性的值不符合预期格式或约束时触发此错误
    #[error("无效的属性值 '{value}' 用于属性 '{attribute}': {reason}")]
    InvalidAttributeValue {
        /// 属性名称
        attribute: String,
        /// 无效的属性值
        value: String,
        /// 无效的具体原因
        reason: String,
        /// 错误发生的代码位置
        span: Option<Span>,
    },

    /// 不支持的字段类型错误
    ///
    /// 当字段类型不支持转换为 JSON 值时触发此错误
    #[error("不支持的字段类型 '{field_type}' 在字段 '{field_name}' 中")]
    UnsupportedFieldType {
        /// 字段名称
        field_name: String,
        /// 不支持的类型名称
        field_type: String,
        /// 错误发生的代码位置
        span: Option<Span>,
    },

    /// 属性解析错误
    ///
    /// 当解析宏属性的语法结构时发生错误
    #[error("属性解析错误: {message}")]
    ParseError {
        /// 错误消息
        message: String,
        /// 错误发生的代码位置
        span: Option<Span>,
    },

    /// 代码生成错误
    ///
    /// 当生成 TokenStream 代码时发生错误
    #[error("代码生成错误: {message}")]
    GenerationError {
        /// 错误消息
        message: String,
        /// 错误发生的代码位置
        span: Option<Span>,
    },

    /// 验证错误
    ///
    /// 当验证配置的正确性时发生错误
    #[error("验证错误: {message}")]
    ValidationError {
        /// 错误消息
        message: String,
        /// 错误发生的代码位置
        span: Option<Span>,
    },

    /// 语法错误（从 syn::Error 转换而来）
    ///
    /// 当 syn 解析语法时发生的底层错误
    #[error("语法错误: {0}")]
    SyntaxError(#[from] syn::Error),
}

impl MacroError {
    /// 转换为编译时错误
    ///
    /// 将 MacroError 转换为 `proc_macro2::TokenStream`，
    /// 以便在编译时显示友好的错误消息。
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责错误到 TokenStream 的转换
    /// - **里氏替换**: 所有 MacroError 类型都能统一转换
    ///
    /// # 返回值
    ///
    /// 返回包含 `compile_error!` 宏的 TokenStream
    pub fn to_compile_error(&self) -> TokenStream2 {
        let message = self.to_string();
        let span = self.get_span().unwrap_or_else(Span::call_site);

        // 生成带有位置信息的编译错误
        quote::quote_spanned! { span =>
            compile_error!(#message);
        }
    }

    /// 创建带有具体位置的缺少属性错误
    ///
    /// 为缺少属性错误提供精确的代码位置信息。
    ///
    /// # 参数
    ///
    /// * `attribute` - 缺少的属性名称
    /// * `spanned` - 提供位置信息的语法节点
    ///
    /// # 返回值
    ///
    /// 返回带有位置信息的 MacroError
    pub fn missing_attribute<T: Spanned>(
        attribute: &str,
        spanned: &T,
    ) -> Self {
        Self::MissingAttribute {
            attribute: attribute.to_string(),
            span: Some(spanned.span()),
        }
    }

    /// 创建带有具体位置的无效属性值错误
    ///
    /// 为无效属性值错误提供精确的代码位置信息和详细说明。
    ///
    /// # 参数
    ///
    /// * `attribute` - 属性名称
    /// * `value` - 无效的属性值
    /// * `reason` - 无效的具体原因
    /// * `spanned` - 提供位置信息的语法节点
    ///
    /// # 返回值
    ///
    /// 返回带有位置信息的 MacroError
    pub fn invalid_attribute_value<T: Spanned>(
        attribute: &str,
        value: &str,
        reason: &str,
        spanned: &T,
    ) -> Self {
        Self::InvalidAttributeValue {
            attribute: attribute.to_string(),
            value: value.to_string(),
            reason: reason.to_string(),
            span: Some(spanned.span()),
        }
    }

    /// 创建带有具体位置的不支持类型错误
    ///
    /// 为不支持的字段类型错误提供精确的代码位置信息。
    ///
    /// # 参数
    ///
    /// * `field_name` - 字段名称
    /// * `field_type` - 字段类型
    /// * `spanned` - 提供位置信息的语法节点
    ///
    /// # 返回值
    ///
    /// 返回带有位置信息的 MacroError
    pub fn unsupported_field_type<T: Spanned>(
        field_name: &str,
        field_type: &str,
        spanned: &T,
    ) -> Self {
        Self::UnsupportedFieldType {
            field_name: field_name.to_string(),
            field_type: field_type.to_string(),
            span: Some(spanned.span()),
        }
    }

    /// 创建带有具体位置的解析错误
    ///
    /// 为解析错误提供精确的代码位置信息。
    ///
    /// # 参数
    ///
    /// * `message` - 错误消息
    /// * `spanned` - 提供位置信息的语法节点
    ///
    /// # 返回值
    ///
    /// 返回带有位置信息的 MacroError
    pub fn parse_error<T: Spanned>(
        message: &str,
        spanned: &T,
    ) -> Self {
        Self::ParseError {
            message: message.to_string(),
            span: Some(spanned.span()),
        }
    }

    /// 创建带有具体位置的验证错误
    ///
    /// 为验证错误提供精确的代码位置信息。
    ///
    /// # 参数
    ///
    /// * `message` - 错误消息
    /// * `spanned` - 提供位置信息的语法节点
    ///
    /// # 返回值
    ///
    /// 返回带有位置信息的 MacroError
    pub fn validation_error<T: Spanned>(
        message: &str,
        spanned: &T,
    ) -> Self {
        Self::ValidationError {
            message: message.to_string(),
            span: Some(spanned.span()),
        }
    }

    /// 创建带有具体位置的代码生成错误
    ///
    /// 为代码生成错误提供精确的代码位置信息。
    ///
    /// # 参数
    ///
    /// * `message` - 错误消息
    /// * `spanned` - 提供位置信息的语法节点
    ///
    /// # 返回值
    ///
    /// 返回带有位置信息的 MacroError
    pub fn generation_error<T: Spanned>(
        message: &str,
        spanned: &T,
    ) -> Self {
        Self::GenerationError {
            message: message.to_string(),
            span: Some(spanned.span()),
        }
    }

    /// 获取错误的位置信息
    ///
    /// 提取错误发生的代码位置，如果没有位置信息则返回 None。
    /// 这个方法遵循接口隔离原则，只提供位置信息获取功能。
    ///
    /// # 返回值
    ///
    /// 返回错误发生的 Span，如果没有位置信息则返回 None
    fn get_span(&self) -> Option<Span> {
        match self {
            Self::MissingAttribute { span, .. } => *span,
            Self::InvalidAttributeValue { span, .. } => *span,
            Self::UnsupportedFieldType { span, .. } => *span,
            Self::ParseError { span, .. } => *span,
            Self::GenerationError { span, .. } => *span,
            Self::ValidationError { span, .. } => *span,
            Self::SyntaxError(err) => Some(err.span()),
        }
    }

    /// 为错误添加修复建议
    ///
    /// 根据错误类型提供具体的修复建议，帮助开发者快速解决问题。
    /// 此方法体现了友好的用户体验设计。
    ///
    /// # 返回值
    ///
    /// 返回包含修复建议的字符串
    pub fn suggestion(&self) -> String {
        match self {
            Self::MissingAttribute { attribute, .. } => {
                match attribute.as_str() {
                    "node_type" => format!(
                        "请在结构体上添加 #[node_type = \"类型名\"] 属性，例如: #[node_type = \"paragraph\"]"
                    ),
                    "mark_type" => format!(
                        "请在结构体上添加 #[mark_type = \"类型名\"] 属性，例如: #[mark_type = \"bold\"]"
                    ),
                    _ => format!(
                        "请在结构体上添加 #[{} = \"值\"] 属性",
                        attribute
                    ),
                }
            },
            Self::InvalidAttributeValue { attribute, .. } => {
                format!("请检查 #{} 属性的值格式是否正确", attribute)
            },
            Self::UnsupportedFieldType { field_name, field_type, .. } => {
                format!(
                    "字段 '{}' 的类型 '{}' 不受支持，请使用支持的基本类型：String, i32, i64, f32, f64, bool 或其 Option 包装版本",
                    field_name, field_type
                )
            },
            _ => "请检查宏的使用方式是否符合文档要求".to_string(),
        }
    }
}

/// 宏处理结果类型
///
/// 为宏处理提供统一的结果类型，简化错误处理。
/// 遵循开闭原则，可以方便地扩展新的成功类型而无需修改现有代码。
pub type MacroResult<T> = Result<T, MacroError>;

/// 创建友好的编译错误消息
///
/// 辅助函数，用于快速创建编译错误消息。
/// 遵循单一职责原则，专门负责创建编译错误消息。
///
/// # 参数
///
/// * `message` - 错误消息内容
///
/// # 返回值
///
/// 返回包含编译错误的 TokenStream
pub fn create_compile_error(message: &str) -> TokenStream2 {
    quote! {
        compile_error!(#message);
    }
}

/// 创建带有修复建议的编译错误消息
///
/// 辅助函数，用于创建包含修复建议的编译错误消息。
/// 提供更好的开发者体验。
///
/// # 参数
///
/// * `error_msg` - 主要错误消息
/// * `suggestion` - 修复建议
///
/// # 返回值
///
/// 返回包含错误和建议的 TokenStream
pub fn create_compile_error_with_suggestion(
    error_msg: &str,
    suggestion: &str,
) -> TokenStream2 {
    let combined_message = format!("{}\n\n修复建议: {}", error_msg, suggestion);
    quote! {
        compile_error!(#combined_message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    /// 测试缺少属性错误的创建和转换
    #[test]
    fn test_missing_attribute_error() {
        let tokens = quote! { struct Test {} };
        let error = MacroError::missing_attribute("node_type", &tokens);

        // 验证错误消息格式
        let error_str = error.to_string();
        assert!(error_str.contains("缺少必需的宏属性: node_type"));

        // 验证修复建议
        let suggestion = error.suggestion();
        assert!(suggestion.contains("node_type"));
        assert!(suggestion.contains("paragraph"));
    }

    /// 测试无效属性值错误的创建和转换
    #[test]
    fn test_invalid_attribute_value_error() {
        let tokens = quote! { #[node_type = ""] };
        let error = MacroError::invalid_attribute_value(
            "node_type",
            "",
            "值不能为空",
            &tokens,
        );

        let error_str = error.to_string();
        assert!(error_str.contains("无效的属性值"));
        assert!(error_str.contains("node_type"));
        assert!(error_str.contains("值不能为空"));
    }

    /// 测试编译错误的生成
    #[test]
    fn test_to_compile_error() {
        let error = MacroError::ParseError {
            message: "测试错误".to_string(),
            span: None,
        };

        let compile_error = error.to_compile_error();
        let compile_error_str = compile_error.to_string();

        // 验证生成的编译错误包含 compile_error! 宏
        assert!(compile_error_str.contains("compile_error"));
        assert!(compile_error_str.contains("测试错误"));
    }

    /// 测试修复建议功能
    #[test]
    fn test_error_suggestions() {
        let missing_node_type = MacroError::MissingAttribute {
            attribute: "node_type".to_string(),
            span: None,
        };

        let suggestion = missing_node_type.suggestion();
        assert!(suggestion.contains("node_type"));
        assert!(suggestion.contains("paragraph"));

        let missing_mark_type = MacroError::MissingAttribute {
            attribute: "mark_type".to_string(),
            span: None,
        };

        let suggestion = missing_mark_type.suggestion();
        assert!(suggestion.contains("mark_type"));
        assert!(suggestion.contains("bold"));
    }
}
