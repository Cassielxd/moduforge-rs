//! Node 代码生成器实现
//!
//! 专门负责为 #[derive(Node)] 宏生成 to_node() 方法的代码。
//! 严格遵循单一职责原则，只负责 Node 相关的代码生成逻辑。

use proc_macro2::{TokenStream as TokenStream2, Ident};
use quote::quote;
use syn::DeriveInput;
use crate::common::{MacroResult, MacroError, utils};
use crate::parser::{NodeConfig, FieldConfig};
use crate::converter::converter_registry::GlobalConverterRegistry;
use super::CodeGenerator;

/// Node 代码生成器
///
/// 负责为结构体生成 to_node() 方法，将结构体实例转换为 mf_core::node::Node。
/// 遵循单一职责原则，专门处理 Node 相关的代码生成。
///
/// # 设计原则体现
///
/// - **单一职责原则**: 只负责 Node 代码生成，不处理其他类型
/// - **开闭原则**: 通过配置和转换器扩展功能而不修改核心逻辑
/// - **里氏替换原则**: 实现了 CodeGenerator trait，可以替换其他生成器使用
#[derive(Debug)]
pub struct NodeGenerator<'a> {
    /// 派生宏的输入，包含结构体定义
    input: &'a DeriveInput,
    
    /// Node 配置信息，包含所有解析后的属性
    config: &'a NodeConfig,
    
}

impl<'a> NodeGenerator<'a> {
    /// 创建新的 Node 代码生成器
    ///
    /// # 参数
    ///
    /// * `input` - 派生宏的输入，包含结构体定义
    /// * `config` - Node 配置信息，包含解析后的所有属性
    ///
    /// # 返回值
    ///
    /// 返回配置好的 Node 代码生成器实例
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责实例初始化
    /// - **依赖注入**: 通过参数接收依赖的配置信息
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::generator::node_generator::NodeGenerator;
    /// use crate::parser::{AttributeParser, NodeConfig};
    /// use syn::parse_quote;
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
    /// let config = AttributeParser::parse_node_attributes(&input).unwrap();
    /// let generator = NodeGenerator::new(&input, &config);
    /// ```
    pub fn new(input: &'a DeriveInput, config: &'a NodeConfig) -> Self {
        Self {
            input,
            config,
        }
    }

    /// 生成 to_node() 方法的实现代码
    ///
    /// 根据配置信息生成完整的 to_node() 方法实现。
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
    ///     pub fn to_node(&self) -> mf_core::node::Node {
    ///         // 导入必要的类型
    ///         use mf_model::node_type::NodeSpec;
    ///         use std::collections::HashMap;
    ///         use serde_json::Value as JsonValue;
    ///         
    ///         // 构建属性映射
    ///         let mut attrs = HashMap::with_capacity(field_count);
    ///         // ... 属性设置代码 ...
    ///         
    ///         // 构建 NodeSpec
    ///         let spec = NodeSpec { ... };
    ///         
    ///         // 创建并返回 Node
    ///         mf_core::node::Node::create("node_type", spec)
    ///     }
    /// }
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责生成 to_node() 方法代码
    /// - **开闭原则**: 通过配置和转换器支持扩展
    pub fn generate_to_node_method(&self) -> MacroResult<TokenStream2> {
        let struct_name = &self.input.ident;
        let node_type = self.config.node_type.as_ref()
            .ok_or_else(|| MacroError::validation_error(
                "Node 配置缺少必需的 node_type 属性",
                self.input,
            ))?;

        // 生成必要的导入语句
        let imports = self.generate_imports();

        // 生成 NodeSpec 构建代码
        let spec_code = self.generate_node_spec_code()?;

        // 只返回 to_node 方法的实现，不包含 impl 块
        let method_impl = quote! {
            /// 将结构体转换为 mf_core::node::Node 实例
            ///
            /// 此方法由 #[derive(Node)] 宏自动生成，根据结构体的字段
            /// 和宏属性配置创建相应的 Node 实例。
            ///
            /// # 返回值
            /// 
            /// 返回配置好的 `mf_core::node::Node` 实例
            ///
            /// # 生成说明
            ///
            /// 这个方法是由 ModuForge-RS 宏系统自动生成的，
            /// 它遵循以下设计原则：
            /// - **单一职责**: 只负责 Node 实例的创建
            /// - **里氏替换**: 生成的 Node 可以替换手动创建的实例
            pub fn to_node(&self) -> mf_core::node::Node {
                #imports
                
                #spec_code
                
                // 创建并返回 Node 实例
                mf_core::node::Node::create(#node_type, spec)
            }
        };

        Ok(method_impl)
    }

    /// 生成必要的导入语句
    ///
    /// 生成 to_node() 方法中需要的所有类型导入。
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
        utils::generate_imports()
    }


    /// 生成 NodeSpec 构建代码
    ///
    /// 根据配置信息生成 NodeSpec 的构建代码。
    /// 遵循单一职责原则，专门负责 NodeSpec 的代码生成。
    ///
    /// # 返回值
    ///
    /// 成功时返回 NodeSpec 构建代码，失败时返回生成错误
    ///
    /// # 生成的代码结构
    ///
    /// ```rust
    /// let spec = NodeSpec {
    ///     content: Some("content_expression".to_string()),
    ///     marks: Some("mark1,mark2".to_string()),
    ///     attrs: attrs,
    ///     group: None,
    ///     desc: None,
    /// };
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责 NodeSpec 构建代码生成
    /// - **开闭原则**: 通过配置支持扩展而不修改代码
    fn generate_node_spec_code(&self) -> MacroResult<TokenStream2> {
        let content = self.config.content.as_ref().map(|c| quote! { Some(#c.to_string()) })
            .unwrap_or_else(|| quote! { None });
            
        let marks = self.config.marks_string().map(|m| quote! { Some(#m.to_string()) })
            .unwrap_or_else(|| quote! { None });

        // 生成属性映射构建代码
        let attrs_code = self.generate_attrs_spec_code()?;

        let spec_code = quote! {
            #attrs_code
            
            let spec = mf_model::node_type::NodeSpec {
                content: #content,
                marks: #marks,
                attrs,
                group: None,
                desc: None,
            };
        };

        Ok(spec_code)
    }

    /// 生成属性映射构建代码 (for NodeSpec)
    ///
    /// 为所有标记为 #[attr] 的字段生成 NodeSpec 的属性映射构建代码。
    /// 此方法遵循单一职责原则，专门负责属性映射的代码生成。
    ///
    /// # 返回值
    ///
    /// 成功时返回属性映射构建代码，失败时返回生成错误
    ///
    /// # 生成的代码结构
    ///
    /// ```rust
    /// let mut attrs = std::collections::HashMap::new();
    /// attrs.insert("field1".to_string(), AttributeSpec { default: Some(serde_json::to_value(&self.field1).unwrap_or(JsonValue::Null)) });
    /// // ... 更多字段 ...
    /// let attrs = Some(attrs);
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责属性映射代码生成
    /// - **开闭原则**: 通过转换器系统支持新的字段类型
    fn generate_attrs_spec_code(&self) -> MacroResult<TokenStream2> {
        let attr_fields = &self.config.attr_fields;
        
        if attr_fields.is_empty() {
            // 没有属性字段时，创建空的 attrs
            return Ok(quote! {
                let attrs = None;
            });
        }

        let mut field_setters = Vec::new();

        // 为每个属性字段生成设置代码
        for field_config in attr_fields {
            let field_setter = self.generate_field_spec_code(field_config)?;
            field_setters.push(field_setter);
        }

        // 生成完整的属性映射构建代码
        let attrs_code = quote! {
            let mut attrs_map = std::collections::HashMap::new();
            #(#field_setters)*
            let attrs = Some(attrs_map);
        };

        Ok(attrs_code)
    }

    /// 生成单个字段的属性设置代码 (for NodeSpec)
    ///
    /// 为单个属性字段生成相应的 NodeSpec 属性设置代码。
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
    /// attrs_map.insert("field_name".to_string(), mf_model::schema::AttributeSpec {
    ///     default: Some(serde_json::to_value(&self.field_name).unwrap_or(serde_json::Value::Null))
    /// });
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责单个字段的属性设置代码生成
    /// - **里氏替换**: 对任何字段配置都能正确处理
    fn generate_field_spec_code(&self, field_config: &FieldConfig) -> MacroResult<TokenStream2> {
        let field_name = &field_config.name;
        let field_ident = syn::parse_str::<Ident>(field_name)
            .map_err(|_| MacroError::parse_error(
                &format!("无效的字段名称: {}", field_name),
                &field_config.field,
            ))?;

        // 生成属性设置代码，创建 AttributeSpec
        let attr_code = quote! {
            attrs_map.insert(#field_name.to_string(), mf_model::schema::AttributeSpec {
                default: Some(serde_json::to_value(&self.#field_ident).unwrap_or(serde_json::Value::Null))
            });
        };

        Ok(attr_code)
    }

    /// 生成 from 方法的实现代码
    ///
    /// 根据配置信息生成 from 方法，该方法接受 mf_model::node::Node 参数
    /// 并返回当前结构体的实例。
    ///
    /// # 返回值
    ///
    /// 成功时返回生成的代码 TokenStream，失败时返回生成错误
    ///
    /// # 生成的方法结构
    ///
    /// ```rust
    /// impl StructName {
    ///     pub fn from(node: &mf_model::node::Node) -> Self {
    ///         // 从 node.attrs 中提取字段值
    ///         Self {
    ///             field1: extract_field_value(&node.attrs, "field1").unwrap_or_default(),
    ///             field2: extract_field_value(&node.attrs, "field2").unwrap_or_default(),
    ///             ...
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责生成 from 方法代码
    /// - **开闭原则**: 通过配置和转换器支持扩展
    pub fn generate_from_method(&self) -> MacroResult<TokenStream2> {
        let struct_name = &self.input.ident;
        let node_type = self.config.node_type.as_ref()
            .ok_or_else(|| MacroError::validation_error(
                "Node 配置缺少必需的 node_type 属性",
                self.input,
            ))?;

        // 生成字段初始化代码
        let field_inits = self.generate_field_initializers()?;

        // 只返回 from 方法的实现，不包含 impl 块
        let method_impl = quote! {
            /// 从 mf_model::node::Node 创建结构体实例
            ///
            /// 此方法由 #[derive(Node)] 宏自动生成，根据 Node 的属性
            /// 创建相应的结构体实例。
            ///
            /// # 参数
            ///
            /// * `node` - 要转换的 Node 实例
            ///
            /// # 返回值
            /// 
            /// 成功时返回结构体实例，失败时返回错误信息
            ///
            /// # 错误
            ///
            /// 当节点类型不匹配时，返回包含错误信息的 Result
            ///
            /// # 生成说明
            ///
            /// 这个方法是由 ModuForge-RS 宏系统自动生成的，
            /// 它遵循以下设计原则：
            /// - **单一职责**: 只负责从 Node 创建结构体实例
            /// - **错误安全**: 使用 Result 类型处理类型不匹配错误
            pub fn from(node: &mf_model::node::Node) -> Result<Self, String> {
                use serde_json::Value as JsonValue;
                
                // 验证节点类型匹配
                if node.r#type != #node_type {
                    return Err(format!("节点类型不匹配: 期望 '{}', 实际 '{}'", #node_type, node.r#type));
                }
                
                Ok(Self {
                    #field_inits
                })
            }
        };

        Ok(method_impl)
    }

    /// 生成字段初始化代码
    ///
    /// 为所有标记为 #[attr] 的字段生成初始化代码，从 Node 的属性中提取值。
    ///
    /// # 返回值
    ///
    /// 成功时返回字段初始化代码，失败时返回生成错误
    fn generate_field_initializers(&self) -> MacroResult<TokenStream2> {
        let attr_fields = &self.config.attr_fields;
        let mut field_inits = Vec::new();

        for field_config in attr_fields {
            let field_init = self.generate_field_initialization(field_config)?;
            field_inits.push(field_init);
        }

        Ok(quote! {
            #(#field_inits),*
        })
    }

    /// 生成单个字段的初始化代码
    ///
    /// 为单个属性字段生成从 Node 属性中提取值的初始化代码。
    ///
    /// # 参数
    ///
    /// * `field_config` - 字段配置信息
    ///
    /// # 返回值
    ///
    /// 成功时返回字段初始化代码，失败时返回转换错误
    fn generate_field_initialization(&self, field_config: &FieldConfig) -> MacroResult<TokenStream2> {
        let field_name = &field_config.name;
        let field_ident = syn::parse_str::<Ident>(field_name)
            .map_err(|_| MacroError::parse_error(
                &format!("无效的字段名称: {}", field_name),
                &field_config.field,
            ))?;

        // 根据字段类型生成不同的提取逻辑
        let extraction_code = self.generate_field_extraction_code(field_config)?;

        Ok(quote! {
            #field_ident: #extraction_code
        })
    }

    /// 生成字段值提取代码
    ///
    /// 根据字段类型生成相应的值提取和转换代码。
    ///
    /// # 参数
    ///
    /// * `field_config` - 字段配置信息
    ///
    /// # 返回值
    ///
    /// 成功时返回字段值提取代码，失败时返回转换错误
    fn generate_field_extraction_code(&self, field_config: &FieldConfig) -> MacroResult<TokenStream2> {
        let field_name = &field_config.name;
        let type_name = &field_config.type_name;

        // 为不同类型生成不同的提取逻辑
        let extraction = match type_name.as_str() {
            "String" => quote! {
                node.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default()
            },
            "i32" => quote! {
                node.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_i64())
                    .map(|i| i as i32)
                    .unwrap_or_default()
            },
            "f64" => quote! {
                node.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_f64())
                    .unwrap_or_default()
            },
            "bool" => quote! {
                node.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_bool())
                    .unwrap_or_default()
            },
            "serde_json::Value" | "Value" => quote! {
                node.attrs.attrs.get(#field_name)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null)
            },
            "uuid::Uuid" | "Uuid" => quote! {
                node.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_str())
                    .and_then(|s| uuid::Uuid::parse_str(s).ok())
                    .unwrap_or_else(uuid::Uuid::new_v4)
            },
            "Vec<u8>" => quote! {
                node.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_u64().map(|u| u as u8)).collect())
                    .unwrap_or_default()
            },
            "Vec<String>" => quote! {
                node.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default()
            },
            _ if type_name.starts_with("Option<") => {
                // 处理 Option 类型
                let inner_type = self.extract_option_inner_type(type_name);
                match inner_type.as_str() {
                    "String" => quote! {
                        node.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    },
                    "i32" => quote! {
                        node.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_i64())
                            .map(|i| i as i32)
                    },
                    "f64" => quote! {
                        node.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_f64())
                    },
                    "bool" => quote! {
                        node.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_bool())
                    },
                    "serde_json::Value" | "Value" => quote! {
                        node.attrs.attrs.get(#field_name).cloned()
                    },
                    "uuid::Uuid" | "Uuid" => quote! {
                        node.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_str())
                            .and_then(|s| uuid::Uuid::parse_str(s).ok())
                    },
                    _ => quote! {
                        None
                    }
                }
            },
            _ => {
                return Err(MacroError::validation_error(
                    &format!("不支持的字段类型: {}", type_name),
                    self.input,
                ));
            }
        };

        Ok(extraction)
    }

    /// 提取 Option<T> 类型的内部类型
    ///
    /// 从 "Option<T>" 字符串中提取 "T" 部分。
    ///
    /// # 参数
    ///
    /// * `type_name` - 类型名称字符串
    ///
    /// # 返回值
    ///
    /// 返回内部类型名称
    fn extract_option_inner_type(&self, type_name: &str) -> String {
        if let Some(start) = type_name.find('<') {
            if let Some(end) = type_name.rfind('>') {
                if start < end {
                    return type_name[start + 1..end].to_string();
                }
            }
        }
        "String".to_string() // 默认返回 String
    }
}

impl<'a> CodeGenerator for NodeGenerator<'a> {
    /// 生成完整的 Node 代码
    ///
    /// 实现 CodeGenerator trait 的核心方法，生成完整的 Node 转换代码。
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
        let to_node_method = self.generate_to_node_method()?;
        let from_method = self.generate_from_method()?;
        
        Ok(quote! {
            impl #struct_name {
                #to_node_method
                
                #from_method
            }
        })
    }

    /// 获取生成器名称
    ///
    /// 返回 Node 代码生成器的名称，用于调试和错误消息。
    ///
    /// # 返回值
    ///
    /// 返回生成器名称 "NodeGenerator"
    ///
    /// # 设计原则体现
    ///
    /// - **接口隔离**: 提供清晰的生成器标识
    fn name(&self) -> &'static str {
        "NodeGenerator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::AttributeParser;
    use syn::parse_quote;

    /// 测试 Node 代码生成器的创建
    #[test]
    fn test_node_generator_creation() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "paragraph"]
            struct TestNode {
                #[attr]
                content: String,
            }
        };

        let config = AttributeParser::parse_node_attributes(&input).unwrap();
        let generator = NodeGenerator::new(&input, &config);
        
        assert_eq!(generator.name(), "NodeGenerator");
    }

    /// 测试基本的 Node 代码生成
    #[test]
    fn test_basic_node_code_generation() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "paragraph"]
            struct TestNode {
                #[attr]
                content: String,
            }
        };

        let config = AttributeParser::parse_node_attributes(&input).unwrap();
        let generator = NodeGenerator::new(&input, &config);
        
        let result = generator.generate();
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();
        
        // 验证生成的代码包含关键元素
        assert!(code_str.contains("impl TestNode"));
        assert!(code_str.contains("pub fn to_node"));
        assert!(code_str.contains("pub fn from"));
        assert!(code_str.contains("mf_model::node::Node"));
        assert!(code_str.contains("paragraph"));
        assert!(code_str.contains("content"));
    }

    /// 测试带有 marks 和 content 的 Node 代码生成
    #[test]
    fn test_full_node_code_generation() {
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

        let config = AttributeParser::parse_node_attributes(&input).unwrap();
        let generator = NodeGenerator::new(&input, &config);
        
        let result = generator.generate();
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

    /// 测试没有属性字段的 Node 代码生成
    #[test]
    fn test_node_without_attr_fields() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "divider"]
            struct DividerNode;
        };

        let config = AttributeParser::parse_node_attributes(&input).unwrap();
        let generator = NodeGenerator::new(&input, &config);
        
        let result = generator.generate();
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();
        
        // 验证生成的代码正确处理空属性情况
        assert!(code_str.contains("impl DividerNode"));
        assert!(code_str.contains("divider"));
        assert!(code_str.contains("default") || code_str.contains("Attrs::default"));
    }

    /// 测试 from 方法的 Result 返回类型
    #[test]
    fn test_from_method_result_type() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "test_node"]
            struct TestNode {
                #[attr]
                content: String,
            }
        };

        let config = AttributeParser::parse_node_attributes(&input).unwrap();
        let generator = NodeGenerator::new(&input, &config);
        
        let result = generator.generate();
        assert!(result.is_ok());

        let code = result.unwrap();
        let code_str = code.to_string();
        
        // 验证 from 方法返回 Result 类型
        assert!(code_str.contains("pub fn from"));
        assert!(code_str.contains("Result < Self , String >"));
        assert!(code_str.contains("节点类型不匹配"));
        assert!(code_str.contains("Ok (Self {"));
        assert!(code_str.contains("return Err"));
    }

    /// 测试导入语句生成
    #[test]
    fn test_imports_generation() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "test"]
            struct TestNode;
        };

        let config = AttributeParser::parse_node_attributes(&input).unwrap();
        let generator = NodeGenerator::new(&input, &config);
        
        let imports = generator.generate_imports();
        let imports_str = imports.to_string();
        
        // 验证生成的导入语句包含必要的类型
        assert!(imports_str.contains("HashMap") || imports_str.contains("JsonValue"));
    }
}