//! Mark 代码生成器实现
//!
//! 专门负责为 #[derive(Mark)] 宏生成 to_mark() 方法的代码。
//! 严格遵循单一职责原则，只负责 Mark 相关的代码生成逻辑。

use proc_macro2::{TokenStream as TokenStream2, Ident};
use quote::quote;
use syn::DeriveInput;
use crate::common::{MacroResult, MacroError};
use crate::parser::{MarkConfig, FieldConfig};
use crate::converter::converter_registry::GlobalConverterRegistry;
use super::CodeGenerator;

/// Mark 代码生成器
///
/// 负责为结构体生成 to_mark() 方法，将结构体实例转换为 mf_core::mark::Mark。
/// 遵循单一职责原则，专门处理 Mark 相关的代码生成。
///
/// # 设计原则体现
///
/// - **单一职责原则**: 只负责 Mark 代码生成，不处理其他类型
/// - **开闭原则**: 通过配置和转换器扩展功能而不修改核心逻辑
/// - **里氏替换原则**: 实现了 CodeGenerator trait，可以替换其他生成器使用
#[derive(Debug)]
pub struct MarkGenerator<'a> {
    /// 派生宏的输入，包含结构体定义
    input: &'a DeriveInput,
    
    /// Mark 配置信息，包含所有解析后的属性
    config: &'a MarkConfig,
}

impl<'a> MarkGenerator<'a> {
    /// 创建新的 Mark 代码生成器
    ///
    /// # 参数
    ///
    /// * `input` - 派生宏的输入，包含结构体定义
    /// * `config` - Mark 配置信息，包含解析后的所有属性
    ///
    /// # 返回值
    ///
    /// 返回配置好的 Mark 代码生成器实例
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责实例初始化
    /// - **依赖注入**: 通过参数接收依赖的配置信息
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::generator::mark_generator::MarkGenerator;
    /// use crate::parser::{AttributeParser, MarkConfig};
    /// use syn::parse_quote;
    ///
    /// let input = parse_quote! {
    ///     #[derive(Mark)]
    ///     #[mark_type = "bold"]
    ///     struct MyMark {
    ///         #[attr]
    ///         style: String,
    ///     }
    /// };
    ///
    /// let config = AttributeParser::parse_mark_attributes(&input).unwrap();
    /// let generator = MarkGenerator::new(&input, &config);
    /// ```
    pub fn new(input: &'a DeriveInput, config: &'a MarkConfig) -> Self {
        Self {
            input,
            config,
        }
    }

    /// 生成 to_mark() 方法的实现代码
    ///
    /// 根据配置信息生成完整的 to_mark() 方法实现。
    /// 此方法是代码生成器的核心功能。
    ///
    /// # 返回值
    ///
    /// 成功时返回生成的代码 TokenStream，失败时返回生成错误
    ///
    /// # 生成的方法结构
    ///
    /// ```rust
    /// impl StructName {
    ///     pub fn to_mark(&self) -> mf_core::mark::Mark {
    ///         // 导入必要的类型
    ///         use mf_model::mark_type::MarkSpec;
    ///         use std::collections::HashMap;
    ///         use serde_json::Value as JsonValue;
    ///         
    ///         // 构建属性映射
    ///         let mut attrs = HashMap::with_capacity(field_count);
    ///         // ... 属性设置代码 ...
    ///         
    ///         // 构建 MarkSpec
    ///         let spec = MarkSpec { ... };
    ///         
    ///         // 创建并返回 Mark
    ///         mf_core::mark::Mark::create("mark_type", spec)
    ///     }
    /// }
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责生成 to_mark() 方法代码
    /// - **开闭原则**: 通过配置和转换器支持扩展
    pub fn generate_to_mark_method(&self) -> MacroResult<TokenStream2> {
        let struct_name = &self.input.ident;
        let mark_type = self.config.mark_type.as_ref()
            .ok_or_else(|| MacroError::validation_error(
                "Mark 配置缺少必需的 mark_type 属性",
                self.input,
            ))?;

        // 生成必要的导入语句
        let imports = self.generate_imports();

        // 生成属性映射构建代码
        let attrs_code = self.generate_attrs_map_code()?;

        // 只返回 to_mark 方法的实现，不包含 impl 块
        let method_impl = quote! {
            /// 将结构体转换为 mf_model::mark::Mark 实例
            ///
            /// 此方法由 #[derive(Mark)] 宏自动生成，根据结构体的字段
            /// 和宏属性配置创建相应的 Mark 实例。
            ///
            /// # 返回值
            /// 
            /// 返回配置好的 `mf_model::mark::Mark` 实例
            ///
            /// # 生成说明
            ///
            /// 这个方法是由 ModuForge-RS 宏系统自动生成的，
            /// 它遵循以下设计原则：
            /// - **单一职责**: 只负责 Mark 实例的创建
            /// - **里氏替换**: 生成的 Mark 可以替换手动创建的实例
            pub fn to_mark(&self) -> mf_model::mark::Mark {
                #imports
                
                #attrs_code
                
                // 创建并返回 Mark 实例
                mf_model::mark::Mark {
                    r#type: #mark_type.to_string(),
                    attrs: mf_model::attrs::Attrs::from(attrs_map),
                }
            }
        };

        Ok(method_impl)
    }

    /// 生成必要的导入语句
    ///
    /// 生成 to_mark() 方法中需要的所有类型导入。
    /// 遵循单一职责原则，专门负责导入语句的生成。
    ///
    /// # 返回值
    ///
    /// 返回导入语句的 TokenStream
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责生成导入语句
    /// - **开闭原则**: 可扩展新的导入而不修改其他代码
    fn generate_imports(&self) -> TokenStream2 {
        quote! {
            use std::collections::HashMap;
            use serde_json::Value as JsonValue;
        }
    }

    /// 生成属性映射构建代码
    ///
    /// 为所有标记为 #[attr] 的字段生成属性映射构建代码。
    /// 此方法遵循单一职责原则，专门负责属性映射的代码生成。
    ///
    /// # 返回值
    ///
    /// 成功时返回属性映射构建代码，失败时返回生成错误
    ///
    /// # 生成的代码结构
    ///
    /// ```rust
    /// let mut attrs = HashMap::with_capacity(field_count);
    /// attrs.insert("field1", AttributeSpec {
    ///     default: Some(serde_json::to_value(&self.field1).unwrap_or(JsonValue::Null))
    /// });
    /// // ... 更多字段 ...
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责属性映射代码生成
    /// - **开闭原则**: 通过转换器系统支持新的字段类型
    fn generate_attrs_map_code(&self) -> MacroResult<TokenStream2> {
        let attr_fields = &self.config.attr_fields;
        
        if attr_fields.is_empty() {
            // 没有属性字段时，创建空的 HashMap
            return Ok(quote! {
                let attrs_map = imbl::HashMap::new();
            });
        }

        let mut field_setters = Vec::new();

        // 为每个属性字段生成设置代码
        for field_config in attr_fields {
            let field_setter = self.generate_field_attr_code(field_config)?;
            field_setters.push(field_setter);
        }

        // 生成完整的属性映射构建代码
        let attrs_code = quote! {
            let mut attrs_map = imbl::HashMap::new();
            #(#field_setters)*
        };

        Ok(attrs_code)
    }

    /// 生成单个字段的属性设置代码
    ///
    /// 为单个属性字段生成相应的属性设置代码。
    /// 遵循单一职责原则，专门处理单个字段的转换逻辑。
    ///
    /// # 参数
    ///
    /// * `field_config` - 字段配置信息
    ///
    /// # 返回值
    ///
    /// 成功时返回字段属性设置代码，失败时返回转换错误
    ///
    /// # 生成的代码示例
    ///
    /// ```rust
    /// attrs_map.insert("field_name".to_string(), AttributeSpec {
    ///     default: Some(serde_json::to_value(&self.field_name).unwrap_or(JsonValue::Null))
    /// });
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责单个字段的属性设置代码生成
    /// - **里氏替换**: 对任何字段配置都能正确处理
    fn generate_field_attr_code(&self, field_config: &FieldConfig) -> MacroResult<TokenStream2> {
        let field_name = &field_config.name;
        let field_ident = syn::parse_str::<Ident>(field_name)
            .map_err(|_| MacroError::parse_error(
                &format!("无效的字段名称: {}", field_name),
                &field_config.field,
            ))?;

        // 生成属性设置代码
        let attr_code = quote! {
            attrs_map.insert(#field_name.to_string(), serde_json::to_value(&self.#field_ident).unwrap_or(serde_json::Value::Null));
        };

        Ok(attr_code)
    }

    /// 生成 MarkSpec 构建代码
    ///
    /// 根据配置信息生成 MarkSpec 的构建代码。
    /// 遵循单一职责原则，专门负责 MarkSpec 的代码生成。
    ///
    /// # 返回值
    ///
    /// 成功时返回 MarkSpec 构建代码，失败时返回生成错误
    ///
    /// # 生成的代码结构
    ///
    /// ```rust
    /// let spec = MarkSpec {
    ///     attrs: attrs,
    ///     group: None,
    ///     desc: None,
    /// };
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责 MarkSpec 构建代码生成
    /// - **开闭原则**: 通过配置支持扩展而不修改代码
    fn generate_mark_spec_code(&self) -> MacroResult<TokenStream2> {
        let spec_code = quote! {
            let spec = MarkSpec {
                attrs: attrs,
                excludes: None,
                group: None,
                spanning: None,
                desc: None,
            };
        };

        Ok(spec_code)
    }
}

impl<'a> CodeGenerator for MarkGenerator<'a> {
    /// 生成完整的 Mark 代码
    ///
    /// 实现 CodeGenerator trait 的核心方法，生成完整的 Mark 转换代码。
    /// 
    /// # 返回值
    ///
    /// 成功时返回生成的代码 TokenStream，失败时返回生成错误
    ///
    /// # 设计原则体现
    ///
    /// - **里氏替换原则**: 完全符合 CodeGenerator 接口契约
    /// - **单一职责**: 委托给专门的方法处理具体生成逻辑
    fn generate(&self) -> MacroResult<TokenStream2> {
        let struct_name = &self.input.ident;
        let to_mark_method = self.generate_to_mark_method()?;
        
        Ok(quote! {
            impl #struct_name {
                #to_mark_method
            }
        })
    }

    /// 获取生成器名称
    ///
    /// 返回 Mark 代码生成器的名称，用于调试和错误消息。
    ///
    /// # 返回值
    ///
    /// 返回生成器名称 "MarkGenerator"
    ///
    /// # 设计原则体现
    ///
    /// - **接口隔离**: 提供清晰的生成器标识
    fn name(&self) -> &'static str {
        "MarkGenerator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::AttributeParser;
    use syn::parse_quote;

    /// 测试 Mark 代码生成器的创建
    #[test]
    fn test_mark_generator_creation() {
        let input: DeriveInput = parse_quote! {
            #[derive(Mark)]
            #[mark_type = "bold"]
            struct TestMark {
                #[attr]
                weight: String,
            }
        };

        let config = AttributeParser::parse_mark_attributes(&input).unwrap();
        let generator = MarkGenerator::new(&input, &config);
        
        assert_eq!(generator.name(), "MarkGenerator");
    }

    /// 测试基本的 Mark 代码生成
    #[test]
    fn test_basic_mark_code_generation() {
        let input: DeriveInput = parse_quote! {
            #[derive(Mark)]
            #[mark_type = "bold"]
            struct TestMark {
                #[attr]
                weight: String,
            }
        };

        let config = AttributeParser::parse_mark_attributes(&input).unwrap();
        let generator = MarkGenerator::new(&input, &config);
        
        let result = generator.generate();
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();
        
        // 验证生成的代码包含关键元素
        assert!(code_str.contains("impl TestMark"));
        assert!(code_str.contains("pub fn to_mark"));
        assert!(code_str.contains("mf_model::mark::Mark"));
        assert!(code_str.contains("bold"));
        assert!(code_str.contains("weight"));
    }

    /// 测试完整配置的 Mark 代码生成
    #[test]
    fn test_full_mark_code_generation() {
        let input: DeriveInput = parse_quote! {
            #[derive(Mark)]
            #[mark_type = "styled"]
            struct TestMark {
                #[attr]
                weight: String,
                
                #[attr]
                color: Option<String>,
            }
        };

        let config = AttributeParser::parse_mark_attributes(&input).unwrap();
        let generator = MarkGenerator::new(&input, &config);
        
        let result = generator.generate();
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();
        
        // 验证生成的代码包含所有配置信息
        assert!(code_str.contains("styled"));
        assert!(code_str.contains("weight"));
        assert!(code_str.contains("color"));
    }

    /// 测试没有属性字段的 Mark 代码生成
    #[test]
    fn test_mark_without_attr_fields() {
        let input: DeriveInput = parse_quote! {
            #[derive(Mark)]
            #[mark_type = "simple"]
            struct SimpleMark;
        };

        let config = AttributeParser::parse_mark_attributes(&input).unwrap();
        let generator = MarkGenerator::new(&input, &config);
        
        let result = generator.generate();
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();
        
        // 验证生成的代码正确处理空属性情况
        assert!(code_str.contains("impl SimpleMark"));
        assert!(code_str.contains("simple"));
        assert!(code_str.contains("imbl") && code_str.contains("HashMap") && code_str.contains("new"));
    }

    /// 测试导入语句生成
    #[test]
    fn test_imports_generation() {
        let input: DeriveInput = parse_quote! {
            #[derive(Mark)]
            #[mark_type = "test"]
            struct TestMark;
        };

        let config = AttributeParser::parse_mark_attributes(&input).unwrap();
        let generator = MarkGenerator::new(&input, &config);
        
        let imports = generator.generate_imports();
        let imports_str = imports.to_string();
        
        // 验证生成的导入语句包含必要的类型
        assert!(imports_str.contains("HashMap") || imports_str.contains("JsonValue"));
    }
}