//! Node 派生宏具体实现
//!
//! 提供 #[derive(Node)] 派生宏的核心处理逻辑。
//! 严格遵循单一职责原则，专门负责 Node 派生宏的端到端处理流程。

use proc_macro2::TokenStream as TokenStream2;
use syn::DeriveInput;
use crate::common::{MacroResult, MacroError};
use crate::parser::{AttributeParser, Validator};
use crate::generator::{GeneratorFactory, CodeGenerator};

/// 处理 Node 派生宏
///
/// 这是 Node 派生宏的主入口函数，负责完整的处理流程。
/// 遵循单一职责原则，专门处理 Node 派生宏的所有逻辑。
///
/// # 处理流程
///
/// 1. **属性解析**: 解析宏属性，提取 Node 配置信息
/// 2. **配置验证**: 验证配置的有效性和完整性
/// 3. **代码生成**: 根据配置生成 to_node() 方法实现
/// 4. **错误处理**: 统一处理各阶段可能出现的错误
///
/// # 参数
///
/// * `input` - 派生宏的输入，包含结构体定义和宏属性
///
/// # 返回值
///
/// 成功时返回生成的代码 TokenStream，失败时返回 MacroError
///
/// # 设计原则体现
///
/// - **单一职责原则**: 专门负责 Node 派生宏处理
/// - **开闭原则**: 通过模块化设计支持功能扩展
/// - **里氏替换原则**: 可以替换任何其他宏处理函数
/// - **接口隔离原则**: 提供清晰的宏处理接口
/// - **依赖倒置原则**: 依赖于抽象的解析器、验证器和生成器接口
///
/// # 错误处理
///
/// 此函数统一处理所有阶段的错误：
/// - 属性解析错误：无效的宏属性、缺少必需属性等
/// - 验证错误：配置不一致、字段类型不支持等
/// - 代码生成错误：无法生成有效的 Rust 代码
///
/// # 示例
///
/// ```rust
/// use syn::parse_quote;
/// use crate::node::derive_impl::process_derive_node;
///
/// let input = parse_quote! {
///     #[derive(Node)]
///     #[node_type = "paragraph"]
///     struct MyNode {
///         #[attr]
///         content: String,
///     }
/// };
///
/// let result = process_derive_node(input);
/// assert!(result.is_ok());
/// ```
pub fn process_derive_node(input: DeriveInput) -> MacroResult<TokenStream2> {
    // 第一阶段：属性解析
    // 从 DeriveInput 中提取和解析所有宏属性，构建 NodeConfig
    let config =
        AttributeParser::parse_node_attributes(&input).map_err(|e| {
            // 为属性解析错误添加上下文信息
            MacroError::parse_error(
                &format!("Node 属性解析失败: {}", e),
                &input,
            )
        })?;

    // 第二阶段：配置验证
    // 验证解析后的配置是否完整、有效和一致
    Validator::validate_node_config(&config).map_err(|e| {
        // 为验证错误添加上下文信息
        MacroError::validation_error(
            &format!("Node 配置验证失败: {}", e),
            &input,
        )
    })?;

    // 第三阶段：代码生成
    // 根据验证通过的配置生成 to_node() 方法实现
    let generator = GeneratorFactory::create_node_generator(&input, &config);
    let generated_code = generator.generate().map_err(|e| {
        // 为代码生成错误添加上下文信息
        MacroError::generation_error(
            &format!("Node 代码生成失败: {}", e),
            &input,
        )
    })?;

    Ok(generated_code)
}

/// 处理 Node 派生宏（带错误恢复）
///
/// 这是一个增强版的处理函数，包含更丰富的错误处理和恢复机制。
/// 当主要处理流程失败时，会尝试提供有用的错误信息和建议。
///
/// # 参数
///
/// * `input` - 派生宏的输入
///
/// # 返回值
///
/// 总是返回 TokenStream2，在出错时返回编译时错误
///
/// # 设计原则体现
///
/// - **用户体验优先**: 提供友好的错误信息
/// - **单一职责**: 专门负责带错误恢复的处理
/// - **开闭原则**: 可扩展新的错误恢复策略
///
/// # 错误恢复策略
///
/// 1. **友好错误消息**: 提供清晰的错误描述和修复建议
/// 2. **上下文信息**: 包含错误发生的具体位置信息
/// 3. **示例代码**: 在适当时候提供正确使用的示例
pub fn process_derive_node_with_recovery(input: DeriveInput) -> TokenStream2 {
    match process_derive_node(input) {
        Ok(tokens) => tokens,
        Err(error) => {
            // 生成友好的编译时错误消息
            let error_message = create_friendly_error_message(&error);
            quote::quote! {
                compile_error!(#error_message);
            }
        },
    }
}

/// 创建友好的错误消息
///
/// 将 MacroError 转换为用户友好的错误消息，包含修复建议。
/// 遵循单一职责原则，专门负责错误消息的格式化。
///
/// # 参数
///
/// * `error` - 宏错误实例
///
/// # 返回值
///
/// 返回格式化后的错误消息字符串
///
/// # 设计原则体现
///
/// - **单一职责**: 只负责错误消息格式化
/// - **用户体验**: 提供清晰、可操作的错误信息
fn create_friendly_error_message(error: &MacroError) -> String {
    match error {
        MacroError::ParseError { message, .. } => {
            format!(
                "ModuForge Node 派生宏解析错误:\n\n{}\n\n帮助信息:\n• 检查宏属性的语法是否正确\n• 确保所有必需的属性都已设置\n• 参考文档中的示例用法",
                message
            )
        },
        MacroError::ValidationError { message, .. } => {
            format!(
                "ModuForge Node 派生宏验证错误:\n\n{}\n\n帮助信息:\n• 检查字段类型是否受支持\n• 确保属性值符合要求\n• 验证配置的一致性",
                message
            )
        },
        MacroError::UnsupportedFieldType { field_name, field_type, .. } => {
            format!(
                "ModuForge Node 派生宏类型错误:\n\n字段 '{}' 的类型 '{}' 不受支持\n\n支持的类型包括:\n• 基本类型: String, i32, f64, bool 等\n• 可选类型: Option<T> (T 为任意支持的基本类型)\n\n如需支持其他类型，请参考自定义转换器文档",
                field_name, field_type
            )
        },
        MacroError::GenerationError { message, .. } => {
            format!(
                "ModuForge Node 派生宏代码生成错误:\n\n{}\n\n这通常是内部错误，请报告此问题:\n• 包含完整的错误信息\n• 提供导致错误的代码示例\n• 说明您的使用场景",
                message
            )
        },
        MacroError::MissingAttribute { attribute, .. } => {
            format!(
                "ModuForge Node 派生宏缺少属性错误:\n\n缺少必需的属性: {}\n\n帮助信息:\n• 确保在结构体上添加了所有必需的宏属性\n• 检查属性名称的拼写是否正确\n• 参考文档中的完整示例",
                attribute
            )
        },
        MacroError::InvalidAttributeValue {
            attribute, value, reason, ..
        } => {
            format!(
                "ModuForge Node 派生宏无效属性值错误:\n\n属性 '{}' 的值 '{}' 无效: {}\n\n帮助信息:\n• 检查属性值的格式是否符合要求\n• 确认属性值不为空且符合语法规则\n• 参考文档中的有效属性值示例",
                attribute, value, reason
            )
        },
        MacroError::SyntaxError(syn_error) => {
            format!(
                "ModuForge Node 派生宏语法错误:\n\n{}\n\n帮助信息:\n• 检查代码的语法是否正确\n• 确认所有括号和引号都已正确闭合\n• 验证结构体定义的完整性",
                syn_error
            )
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    /// 测试基本的 Node 派生宏处理
    #[test]
    fn test_basic_node_derive_processing() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "paragraph"]
            struct TestNode {
                #[attr]
                content: String,
            }
        };

        let result = process_derive_node(input);
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();

        // 验证生成的代码包含关键元素
        assert!(code_str.contains("impl TestNode"));
        assert!(code_str.contains("pub fn to_node"));
        assert!(code_str.contains("mf_model::node::Node"));
        assert!(code_str.contains("paragraph"));
    }

    /// 测试完整配置的 Node 派生宏处理
    #[test]
    fn test_full_node_derive_processing() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "paragraph"]
            #[marks = "bold italic"]
            #[content = "text*"]
            struct TestNode {
                #[attr]
                content: String,

                #[attr]
                alignment: Option<String>,
            }
        };

        let result = process_derive_node(input);
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();

        // 验证生成的代码包含基本信息
        assert!(code_str.contains("paragraph"));
        assert!(code_str.contains("content"));
        assert!(code_str.contains("alignment"));
        assert!(code_str.contains("to_node"));
        assert!(code_str.contains("from"));
    }

    /// 测试缺少必需属性的错误处理
    #[test]
    fn test_missing_required_attribute_error() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            // 缺少 node_type 属性
            struct TestNode {
                #[attr]
                content: String,
            }
        };

        let result = process_derive_node(input);
        assert!(result.is_err());

        if let Err(MacroError::ParseError { message, .. }) = result {
            assert!(message.contains("node_type") || message.contains("必需"));
        } else {
            panic!("期望 ParseError 或 ValidationError");
        }
    }

    /// 测试不支持的字段类型错误处理
    #[test]
    fn test_unsupported_field_type_error() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "test"]
            struct TestNode {
                #[attr]
                data: Vec<String>, // 不支持的类型
            }
        };

        let result = process_derive_node(input);
        assert!(result.is_err());

        // 验证错误类型和消息
        match result.unwrap_err() {
            MacroError::UnsupportedFieldType {
                field_name, field_type, ..
            } => {
                assert_eq!(field_name, "data");
                assert!(field_type.contains("Vec"));
            },
            MacroError::ValidationError { .. } => {
                // 也可能在验证阶段被捕获
            },
            _ => panic!("期望 UnsupportedFieldType 或 ValidationError"),
        }
    }

    /// 测试错误恢复功能
    #[test]
    fn test_error_recovery() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            // 无效配置
            struct TestNode {
                invalid_field: UnknownType,
            }
        };

        let result = process_derive_node_with_recovery(input);
        let result_str = result.to_string();

        // 验证返回了编译错误而不是 panic
        assert!(result_str.contains("compile_error"));
    }

    /// 测试友好错误消息生成
    #[test]
    fn test_friendly_error_message_creation() {
        let parse_error = MacroError::ParseError {
            message: "测试解析错误".to_string(),
            span: None,
        };

        let friendly_message = create_friendly_error_message(&parse_error);
        assert!(friendly_message.contains("ModuForge Node 派生宏解析错误"));
        assert!(friendly_message.contains("帮助信息"));
        assert!(friendly_message.contains("测试解析错误"));

        let unsupported_error = MacroError::UnsupportedFieldType {
            field_name: "test_field".to_string(),
            field_type: "Vec<String>".to_string(),
            span: None,
        };

        let friendly_message =
            create_friendly_error_message(&unsupported_error);
        assert!(friendly_message.contains("类型错误"));
        assert!(friendly_message.contains("test_field"));
        assert!(friendly_message.contains("Vec<String>"));
        assert!(friendly_message.contains("支持的类型"));
    }

    /// 测试无属性字段的 Node 处理
    #[test]
    fn test_node_without_attr_fields() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "divider"]
            struct DividerNode;
        };

        let result = process_derive_node(input);
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();

        // 验证生成的代码正确处理无属性情况
        assert!(code_str.contains("impl DividerNode"));
        assert!(code_str.contains("divider"));
        assert!(
            code_str.contains("default") || code_str.contains("Attrs::default")
        );
    }

    /// 测试复杂场景的端到端处理
    #[test]
    fn test_complex_scenario_end_to_end() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "complex_paragraph"]
            #[marks = "bold italic underline"]
            #[content = "text* | block*"]
            struct ComplexParagraphNode {
                #[attr]
                content: String,

                #[attr]
                alignment: Option<String>,

                #[attr]
                line_height: Option<f64>,

                #[attr]
                font_size: Option<i32>,

                #[attr]
                is_highlighted: Option<bool>,

                // 非属性字段
                internal_id: uuid::Uuid,
                cached_data: Vec<u8>,
            }
        };

        let result = process_derive_node(input);
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();

        // 验证基本配置被正确处理
        assert!(code_str.contains("complex_paragraph"));
        assert!(code_str.contains("to_node"));
        assert!(code_str.contains("from"));

        // 验证所有属性字段都被处理
        assert!(code_str.contains("content"));
        assert!(code_str.contains("alignment"));
        assert!(code_str.contains("line_height"));
        assert!(code_str.contains("font_size"));
        assert!(code_str.contains("is_highlighted"));

        // 验证非属性字段不出现在生成的代码中
        assert!(!code_str.contains("internal_id"));
        assert!(!code_str.contains("cached_data"));
    }
}
