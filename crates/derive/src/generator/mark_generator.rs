//! Mark 代码生成器实现
//!
//! 专门负责为 #[derive(Mark)] 宏生成 to_mark() 方法的代码。
//! 严格遵循单一职责原则，只负责 Mark 相关的代码生成逻辑。

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DeriveInput;
use crate::common::{MacroResult, MacroError};
use crate::parser::{MarkConfig, FieldConfig};
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
    pub fn new(
        input: &'a DeriveInput,
        config: &'a MarkConfig,
    ) -> Self {
        Self { input, config }
    }

    /// 生成 mark_definition() 方法的实现代码
    ///
    /// 根据配置信息生成完整的 mark_definition() 方法实现。
    /// 此方法返回标记定义而不是具体实例。
    /// **重要**: 只为标记了 #[attr] 的字段生成属性定义。
    ///
    /// # 返回值
    ///
    /// 成功时返回生成的代码 TokenStream，失败时返回生成错误
    ///
    /// # 生成的方法结构
    ///
    /// ```rust
    /// impl StructName {
    ///     pub fn mark_definition() -> mf_core::mark::Mark {
    ///         // 导入必要的类型
    ///         use mf_model::mark_definition::MarkSpec;
    ///         use std::collections::HashMap;
    ///         use serde_json::Value as JsonValue;
    ///         
    ///         // 只为 #[attr] 字段构建属性映射
    ///         let mut attrs_map = std::collections::HashMap::new();
    ///         // 支持自定义类型表达式 (需要实现 Default + Serialize)
    ///         attrs_map.insert("field_name".to_string(), AttributeSpec {
    ///             default: Some(serde_json::to_value(CustomType::new()).unwrap_or(null))
    ///         });
    ///         
    ///         // 构建 MarkSpec
    ///         let spec = MarkSpec { attrs: Some(attrs_map), ... };
    ///         
    ///         // 创建并返回 Mark 定义
    ///         mf_core::mark::Mark::new("mark_type", spec)
    ///     }
    /// }
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责生成 mark_definition() 方法代码
    /// - **开闭原则**: 支持自定义类型表达式和转换器扩展
    /// - **语义清晰**: 方法名明确表示返回的是定义而非实例
    /// - **属性精确性**: 只包含 #[attr] 标记的字段，符合标记定义语义
    pub fn generate_mark_definition_method(&self) -> MacroResult<TokenStream2> {
        let _struct_name = &self.input.ident;
        let mark_type = self.config.mark_type.as_ref().ok_or_else(|| {
            MacroError::validation_error(
                "Mark 配置缺少必需的 mark_type 属性",
                self.input,
            )
        })?;

        // 生成必要的导入语句
        let imports = self.generate_imports();

        // 生成 MarkSpec 构建代码
        let spec_code = self.generate_mark_spec_code()?;

        // 返回 mark_definition 方法的实现，不包含 impl 块
        let method_impl = quote! {
            /// 获取标记定义
            ///
            /// 此方法由 #[derive(Mark)] 宏自动生成，根据结构体的字段
            /// 和宏属性配置创建标记定义（而非具体实例）。
            ///
            /// # 返回值
            ///
            /// 返回配置好的 `mf_core::mark::Mark` 定义
            ///
            /// # 生成说明
            ///
            /// 这个方法是由 ModuForge-RS 宏系统自动生成的，
            /// 它遵循以下设计原则：
            /// - **单一职责**: 只负责 Mark 定义的创建
            /// - **语义清晰**: 方法名明确表示返回的是定义而非实例
            /// - **里氏替换**: 生成的 Mark 定义可以替换手动创建的定义
            /// - **属性精确性**: 只包含 #[attr] 标记的字段，符合标记定义语义
            pub fn mark_definition() -> mf_core::mark::Mark {
                #imports

                #spec_code

                // 创建并返回 Mark 定义
                mf_core::mark::Mark::new(#mark_type, spec)
            }
        };

        Ok(method_impl)
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
    ///         use mf_model::mark_definition::MarkSpec;
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
    ///         mf_core::mark::Mark::new("mark_type", spec)
    ///     }
    /// }
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责生成 to_mark() 方法代码
    /// - **开闭原则**: 通过配置和转换器支持扩展
    pub fn generate_to_mark_method(&self) -> MacroResult<TokenStream2> {
        let _struct_name = &self.input.ident;
        let mark_type = self.config.mark_type.as_ref().ok_or_else(|| {
            MacroError::validation_error(
                "Mark 配置缺少必需的 mark_type 属性",
                self.input,
            )
        })?;

        // 生成必要的导入语句
        let imports = self.generate_imports();

        // 生成 MarkSpec 构建代码
        let spec_code = self.generate_mark_spec_code()?;

        // 只返回 to_mark 方法的实现，不包含 impl 块
        let method_impl = quote! {
            /// 将结构体转换为 mf_core::mark::Mark 实例
            ///
            /// 此方法由 #[derive(Mark)] 宏自动生成，根据结构体的字段
            /// 和宏属性配置创建相应的 Mark 实例。
            ///
            /// # 返回值
            ///
            /// 返回配置好的 `mf_core::mark::Mark` 实例
            ///
            /// # 生成说明
            ///
            /// 这个方法是由 ModuForge-RS 宏系统自动生成的，
            /// 它遵循以下设计原则：
            /// - **单一职责**: 只负责 Mark 实例的创建
            /// - **里氏替换**: 生成的 Mark 可以替换手动创建的实例
            pub fn to_mark(&self) -> mf_core::mark::Mark {
                #imports

                #spec_code

                // 创建并返回 Mark 实例
                mf_core::mark::Mark::new(#mark_type, spec)
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
        // 生成属性映射构建代码
        let attrs_code = self.generate_attrs_spec_code()?;

        let spec_code = quote! {
            #attrs_code
            let spec = mf_model::mark_definition::MarkSpec {
                attrs,
                excludes: None,
                group: None,
                spanning: None,
                desc: None,
            };
        };

        Ok(spec_code)
    }

    /// 生成属性映射构建代码 (for MarkSpec)
    ///
    /// 为所有标记为 #[attr] 的字段生成 MarkSpec 的属性映射构建代码。
    /// 此方法遵循单一职责原则，专门负责属性映射的代码生成。
    ///
    /// # 返回值
    ///
    /// 成功时返回属性映射构建代码，失败时返回生成错误
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

    /// 生成单个字段的属性设置代码 (for MarkSpec)
    ///
    /// 为单个属性字段生成相应的 MarkSpec 属性设置代码。
    /// 遵循单一职责原则，专门处理单个字段的转换逻辑。
    /// 支持自定义类型表达式和JSON默认值。
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
    /// // 如果有 default 属性，使用 default 值
    /// attrs_map.insert("field_name".to_string(), mf_model::schema::AttributeSpec {
    ///     default: Some(serde_json::json!("default_value"))
    /// });
    ///
    /// // 如果没有 default 属性，使用类型默认值
    /// attrs_map.insert("field_name".to_string(), mf_model::schema::AttributeSpec {
    ///     default: Some(serde_json::json!(String::default()))
    /// });
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责单个字段的属性设置代码生成
    /// - **里氏替换**: 对任何字段配置都能正确处理
    /// - **开闭原则**: 支持 default 属性扩展而不修改核心逻辑
    fn generate_field_spec_code(
        &self,
        field_config: &FieldConfig,
    ) -> MacroResult<TokenStream2> {
        let field_name = &field_config.name;

        // 生成默认值表达式
        let default_value_expr =
            self.generate_default_value_expression(field_config)?;

        // 生成属性设置代码，创建 AttributeSpec
        let attr_code = quote! {
            attrs_map.insert(#field_name.to_string(), mf_model::schema::AttributeSpec {
                default: Some(#default_value_expr)
            });
        };

        Ok(attr_code)
    }

    /// 生成字段的默认值表达式
    ///
    /// 根据字段配置生成相应的默认值表达式：
    /// 1. 如果字段有 default 属性，使用该默认值
    /// 2. 如果没有 default 属性，使用类型的默认值
    ///
    /// # 参数
    ///
    /// * `field_config` - 字段配置信息
    ///
    /// # 返回值
    ///
    /// 成功时返回默认值表达式代码，失败时返回生成错误
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 专门负责默认值表达式生成
    /// - **开闭原则**: 支持新的默认值类型扩展
    fn generate_default_value_expression(
        &self,
        field_config: &FieldConfig,
    ) -> MacroResult<TokenStream2> {
        // 检查是否有 default 属性
        if let Some(default_value) = &field_config.default_value {
            // 使用 attr 中的 default 值
            return self.generate_default_value_from_attr(default_value);
        }

        // 没有 default 属性时，使用类型的默认值
        self.generate_type_default_value(&field_config.type_name)
    }

    /// 从 attr 的 default 属性生成默认值表达式
    ///
    /// 根据默认值的类型生成相应的 JSON 表达式。
    ///
    /// # 参数
    ///
    /// * `default_value` - 默认值配置
    ///
    /// # 返回值
    ///
    /// 返回默认值的 JSON 表达式代码
    fn generate_default_value_from_attr(
        &self,
        default_value: &crate::parser::default_value::DefaultValue,
    ) -> MacroResult<TokenStream2> {
        use crate::parser::default_value::DefaultValueType;

        match &default_value.value_type {
            DefaultValueType::String(s) => Ok(quote! { serde_json::json!(#s) }),
            DefaultValueType::Integer(i) => {
                Ok(quote! { serde_json::json!(#i) })
            },
            DefaultValueType::Float(f) => Ok(quote! { serde_json::json!(#f) }),
            DefaultValueType::Boolean(b) => {
                Ok(quote! { serde_json::json!(#b) })
            },
            DefaultValueType::Json(json_value) => {
                // 对于 JSON 值，转换为字符串然后在运行时解析
                let json_str = serde_json::to_string(json_value)
                    .unwrap_or_else(|_| "null".to_string());
                Ok(quote! {
                    serde_json::from_str(#json_str).unwrap_or_else(|_| serde_json::json!(null))
                })
            },
            DefaultValueType::CustomType(expr) => {
                // 对于自定义类型表达式，直接执行表达式并序列化结果
                let expr_tokens =
                    syn::parse_str::<syn::Expr>(expr).map_err(|_| {
                        MacroError::parse_error(
                            &format!("无效的自定义类型表达式: {expr}"),
                            self.input,
                        )
                    })?;
                Ok(quote! {
                    serde_json::to_value(#expr_tokens).unwrap_or_else(|_| serde_json::json!(null))
                })
            },
            DefaultValueType::Null => Ok(quote! { serde_json::json!(null) }),
        }
    }

    /// 生成类型的默认值表达式
    ///
    /// 为没有 default 属性的字段生成类型默认值的 JSON 表达式。
    /// 支持泛型类型（如 Option<String>、Vec<u8>）和自定义类型。
    /// 自定义类型必须实现 Default + Serialize traits。
    ///
    /// # 参数
    ///
    /// * `type_name` - 字段类型名称（支持泛型，如 "Option<String>"）
    ///
    /// # 返回值
    ///
    /// 返回类型默认值的 JSON 表达式代码
    ///
    /// # 支持的类型示例
    ///
    /// ```rust
    /// // 基本类型
    /// "String" => serde_json::json!(String::default())
    /// "i32" => serde_json::json!(0)
    /// "bool" => serde_json::json!(false)
    ///
    /// // 泛型类型
    /// "Option<String>" => serde_json::json!(null)
    /// "Vec<u8>" => serde_json::json!(Vec::<u8>::new())
    ///
    /// // 自定义类型 (需要 Default + Serialize)
    /// "CustomStruct" => serde_json::to_value(<CustomStruct as Default>::default())
    /// ```
    fn generate_type_default_value(
        &self,
        type_name: &str,
    ) -> MacroResult<TokenStream2> {
        let default_expr = match type_name {
            "String" => quote! { serde_json::json!(String::default()) },
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => {
                quote! { serde_json::json!(0) }
            },
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => {
                quote! { serde_json::json!(0) }
            },
            "f32" | "f64" => quote! { serde_json::json!(0.0) },
            "bool" => quote! { serde_json::json!(false) },
            "serde_json::Value" | "Value" => quote! { serde_json::json!(null) },
            "uuid::Uuid" | "Uuid" => {
                quote! { serde_json::json!(uuid::Uuid::new_v4().to_string()) }
            },
            "Vec<u8>" => quote! { serde_json::json!(Vec::<u8>::new()) },
            "Vec<String>" => quote! { serde_json::json!(Vec::<String>::new()) },
            _ if type_name.starts_with("Option<") => {
                // Option 类型默认为 None，在 JSON 中表示为 null
                quote! { serde_json::json!(null) }
            },
            _ => {
                // 对于其他自定义类型，尝试使用 Default trait 并序列化
                // 这要求类型实现 Default + Serialize traits
                if let Ok(type_ident) = syn::parse_str::<syn::Type>(type_name) {
                    quote! {
                        serde_json::to_value(<#type_ident as Default>::default())
                            .unwrap_or_else(|_| serde_json::json!(null))
                    }
                } else {
                    // 如果类型解析失败，回退到 null
                    quote! { serde_json::json!(null) }
                }
            },
        };

        Ok(default_expr)
    }

    /// 提取所有字段信息
    ///
    /// 从 DeriveInput 中提取所有字段，包括有和没有 #[attr] 标记的字段。
    ///
    /// # 返回值
    ///
    /// 返回包含字段信息的向量
    fn extract_all_fields(&self) -> MacroResult<Vec<FieldInfo>> {
        use syn::{Data, Fields};

        let mut all_fields = Vec::new();

        match &self.input.data {
            Data::Struct(data_struct) => {
                match &data_struct.fields {
                    Fields::Named(fields_named) => {
                        for field in &fields_named.named {
                            if let Some(field_name) = &field.ident {
                                // 检查是否是有 #[attr] 标记的字段
                                let field_config =
                                    self.config.attr_fields.iter().find(
                                        |config| *field_name == config.name,
                                    );

                                let field_info = FieldInfo {
                                    name: field_name.to_string(),
                                    type_name: self
                                        .extract_type_name(&field.ty),
                                    config: field_config.cloned(),
                                };

                                all_fields.push(field_info);
                            }
                        }
                    },
                    Fields::Unnamed(_) => {
                        return Err(MacroError::validation_error(
                            "不支持元组结构体",
                            self.input,
                        ));
                    },
                    Fields::Unit => {
                        // 单元结构体，没有字段
                    },
                }
            },
            _ => {
                return Err(MacroError::validation_error(
                    "只支持结构体类型",
                    self.input,
                ));
            },
        }

        Ok(all_fields)
    }

    /// 从类型中提取完整类型名称（包含泛型参数）
    ///
    /// 递归解析类型结构，构建包含完整泛型信息的类型名称字符串。
    /// 这对于正确处理 Option<String>、Vec<u8> 等泛型类型至关重要。
    ///
    /// # 参数
    ///
    /// * `ty` - syn::Type 类型引用
    ///
    /// # 返回值
    ///
    /// 包含完整泛型信息的类型名称字符串
    fn extract_type_name(
        &self,
        ty: &syn::Type,
    ) -> String {
        use syn::{
            Type, TypePath, PathArguments, GenericArgument,
            AngleBracketedGenericArguments,
        };

        match ty {
            Type::Tuple(tuple) if tuple.elems.is_empty() => {
                // Handle unit type ()
                "()".to_string()
            },
            Type::Path(TypePath { path, .. }) => {
                // 构建完整的类型名称，包括泛型参数
                let segments: Vec<String> = path
                    .segments
                    .iter()
                    .map(|seg| {
                        let ident = seg.ident.to_string();
                        match &seg.arguments {
                            PathArguments::AngleBracketed(
                                AngleBracketedGenericArguments { args, .. },
                            ) => {
                                let type_args: Vec<String> = args
                                    .iter()
                                    .map(|arg| {
                                        match arg {
                                            GenericArgument::Type(inner_ty) => {
                                                self.extract_type_name(inner_ty)
                                            },
                                            GenericArgument::Lifetime(_) => {
                                                "".to_string()
                                            }, // Skip lifetimes
                                            GenericArgument::Const(_) => {
                                                "".to_string()
                                            }, // Skip const generics
                                            GenericArgument::AssocType(_) => {
                                                "".to_string()
                                            }, // Skip associated types
                                            GenericArgument::AssocConst(_) => {
                                                "".to_string()
                                            }, // Skip associated consts
                                            GenericArgument::Constraint(_) => {
                                                "".to_string()
                                            }, // Skip constraints
                                            _ => "".to_string(), // Handle any other cases
                                        }
                                    })
                                    .collect();
                                if type_args.is_empty() {
                                    ident
                                } else {
                                    format!(
                                        "{}<{}>",
                                        ident,
                                        type_args.join(", ")
                                    )
                                }
                            },
                            _ => ident,
                        }
                    })
                    .collect();
                segments.join("::")
            },
            _ => "Unknown".to_string(),
        }
    }

    /// 生成 from 方法的实现代码
    ///
    /// 根据配置信息生成 from 方法，该方法接受 mf_model::mark::Mark 参数
    /// 并返回当前结构体的实例。处理所有字段（包括有和没有 #[attr] 标记的）。
    ///
    /// # 返回值
    ///
    /// 成功时返回生成的代码 TokenStream，失败时返回生成错误
    pub fn generate_from_method(&self) -> MacroResult<TokenStream2> {
        let _struct_name = &self.input.ident;
        let mark_type = self.config.mark_type.as_ref().ok_or_else(|| {
            MacroError::validation_error(
                "Mark 配置缺少必需的 mark_type 属性",
                self.input,
            )
        })?;

        // 生成字段初始化代码
        let field_inits = self.generate_field_initializers()?;

        // 只返回 from 方法的实现，不包含 impl 块
        let method_impl = quote! {
            /// 从 mf_model::mark::Mark 创建结构体实例
            ///
            /// 此方法由 #[derive(Mark)] 宏自动生成，根据 Mark 的属性
            /// 创建相应的结构体实例。
            ///
            /// # 参数
            ///
            /// * `mark` - 要转换的 Mark 实例
            ///
            /// # 返回值
            ///
            /// 成功时返回结构体实例，失败时返回错误信息
            ///
            /// # 错误
            ///
            /// 当标记类型不匹配时，返回包含错误信息的 Result
            ///
            /// # 生成说明
            ///
            /// 这个方法是由 ModuForge-RS 宏系统自动生成的，
            /// 它遵循以下设计原则：
            /// - **单一职责**: 只负责从 Mark 创建结构体实例
            /// - **错误安全**: 使用 Result 类型处理类型不匹配错误
            /// - **字段分离**: #[attr] 字段从 attrs 提取，非 #[attr] 字段使用默认值
            /// - **类型安全**: 支持泛型类型和自定义类型的安全转换
            pub fn from(mark: &mf_model::mark::Mark) -> Result<Self, String> {
                use serde_json::Value as JsonValue;

                // 验证标记类型匹配
                if mark.r#type != #mark_type {
                    return Err(format!("标记类型不匹配: 期望 '{}', 实际 '{}'", #mark_type, mark.r#type));
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
    /// 为所有字段生成初始化代码，包括有和没有 #[attr] 标记的字段。
    /// 保持与 to_mark() 方法的一致性。
    ///
    /// # 返回值
    ///
    /// 成功时返回字段初始化代码，失败时返回生成错误
    fn generate_field_initializers(&self) -> MacroResult<TokenStream2> {
        // 获取所有字段（包括有和没有 #[attr] 标记的）
        let all_fields = self.extract_all_fields()?;
        let mut field_inits = Vec::new();

        for field_info in all_fields {
            let field_init =
                self.generate_field_initialization_from_info(&field_info)?;
            field_inits.push(field_init);
        }

        Ok(quote! {
            #(#field_inits),*
        })
    }

    /// 基于字段信息生成字段初始化代码
    ///
    /// 为任意字段（有或没有 #[attr] 标记）生成从 Mark 属性中提取值的初始化代码。
    ///
    /// # 参数
    ///
    /// * `field_info` - 字段信息（包含可选的配置）
    ///
    /// # 返回值
    ///
    /// 成功时返回字段初始化代码，失败时返回转换错误
    fn generate_field_initialization_from_info(
        &self,
        field_info: &FieldInfo,
    ) -> MacroResult<TokenStream2> {
        let field_name = &field_info.name;
        let field_ident =
            syn::parse_str::<syn::Ident>(field_name).map_err(|_| {
                MacroError::parse_error(
                    &format!("无效的字段名称: {field_name}"),
                    self.input,
                )
            })?;

        // 生成字段值提取代码
        let extraction_code = if let Some(config) = &field_info.config {
            // 有 #[attr] 标记的字段，从 Mark 的 attrs 中提取
            self.generate_field_extraction_code(config)?
        } else {
            // 没有 #[attr] 标记的字段，使用默认值
            self.generate_non_attr_field_default(&field_info.type_name)?
        };

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
    fn generate_field_extraction_code(
        &self,
        field_config: &FieldConfig,
    ) -> MacroResult<TokenStream2> {
        let field_name = &field_config.name;
        let type_name = &field_config.type_name;

        // 为不同类型生成不同的提取逻辑
        let extraction = match type_name.as_str() {
            "String" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default()
            },
            "i8" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_i64())
                    .map(|i| i as i8)
                    .unwrap_or_default()
            },
            "i16" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_i64())
                    .map(|i| i as i16)
                    .unwrap_or_default()
            },
            "i32" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_i64())
                    .map(|i| i as i32)
                    .unwrap_or_default()
            },
            "i64" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_i64())
                    .unwrap_or_default()
            },
            "i128" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_i64())
                    .map(|i| i as i128)
                    .unwrap_or_default()
            },
            "isize" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_i64())
                    .map(|i| i as isize)
                    .unwrap_or_default()
            },
            "u8" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_u64())
                    .map(|u| u as u8)
                    .unwrap_or_default()
            },
            "u16" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_u64())
                    .map(|u| u as u16)
                    .unwrap_or_default()
            },
            "u32" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_u64())
                    .map(|u| u as u32)
                    .unwrap_or_default()
            },
            "u64" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_u64())
                    .unwrap_or_default()
            },
            "u128" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_u64())
                    .map(|u| u as u128)
                    .unwrap_or_default()
            },
            "usize" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_u64())
                    .map(|u| u as usize)
                    .unwrap_or_default()
            },
            "f32" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
                    .unwrap_or_default()
            },
            "f64" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_f64())
                    .unwrap_or_default()
            },
            "bool" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_bool())
                    .unwrap_or_default()
            },
            "serde_json::Value" | "Value" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null)
            },
            "uuid::Uuid" | "Uuid" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_str())
                    .and_then(|s| uuid::Uuid::parse_str(s).ok())
                    .unwrap_or_else(uuid::Uuid::new_v4)
            },
            "Vec<u8>" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_u64().map(|u| u as u8)).collect())
                    .unwrap_or_default()
            },
            "Vec<String>" => quote! {
                mark.attrs.attrs.get(#field_name)
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default()
            },
            _ if type_name.starts_with("Option<") => {
                // 处理 Option 类型
                let inner_type = self.extract_option_inner_type(type_name);
                match inner_type.as_str() {
                    "String" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    },
                    "i8" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_i64())
                            .map(|i| i as i8)
                    },
                    "i16" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_i64())
                            .map(|i| i as i16)
                    },
                    "i32" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_i64())
                            .map(|i| i as i32)
                    },
                    "i64" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_i64())
                    },
                    "i128" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_i64())
                            .map(|i| i as i128)
                    },
                    "isize" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_i64())
                            .map(|i| i as isize)
                    },
                    "u8" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_u64())
                            .map(|u| u as u8)
                    },
                    "u16" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_u64())
                            .map(|u| u as u16)
                    },
                    "u32" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_u64())
                            .map(|u| u as u32)
                    },
                    "u64" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_u64())
                    },
                    "u128" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_u64())
                            .map(|u| u as u128)
                    },
                    "usize" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_u64())
                            .map(|u| u as usize)
                    },
                    "f32" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_f64())
                            .map(|f| f as f32)
                    },
                    "f64" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_f64())
                    },
                    "bool" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_bool())
                    },
                    "serde_json::Value" | "Value" => quote! {
                        mark.attrs.attrs.get(#field_name).cloned()
                    },
                    "uuid::Uuid" | "Uuid" => quote! {
                        mark.attrs.attrs.get(#field_name)
                            .and_then(|v| v.as_str())
                            .and_then(|s| uuid::Uuid::parse_str(s).ok())
                    },
                    _ => {
                        // 对于其他自定义类型的Option，尝试反序列化
                        quote! {
                            mark.attrs.attrs.get(#field_name)
                                .and_then(|v| serde_json::from_value(v.clone()).ok())
                        }
                    },
                }
            },
            _ => {
                // 对于自定义类型，尝试从JSON反序列化
                quote! {
                    mark.attrs.attrs.get(#field_name)
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default()
                }
            },
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
    fn extract_option_inner_type(
        &self,
        type_name: &str,
    ) -> String {
        if let Some(start) = type_name.find('<') {
            if let Some(end) = type_name.rfind('>') {
                if start < end {
                    return type_name[start + 1..end].to_string();
                }
            }
        }
        "String".to_string() // 默认返回 String
    }

    /// 生成非 attr 字段的默认值
    ///
    /// 为没有 #[attr] 标记的字段生成默认值表达式。
    /// 这些字段不会从 Mark 的 attrs 中提取，而是使用类型默认值。
    /// 支持泛型类型和自定义类型的类型安全处理。
    ///
    /// # 参数
    ///
    /// * `type_name` - 字段类型名称（支持泛型，如 "Option<String>"）
    ///
    /// # 返回值
    ///
    /// 返回默认值表达式代码
    fn generate_non_attr_field_default(
        &self,
        type_name: &str,
    ) -> MacroResult<TokenStream2> {
        let default_expr = match type_name {
            "String" => quote! { String::default() },
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => quote! { 0 },
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => quote! { 0 },
            "f32" | "f64" => quote! { 0.0 },
            "bool" => quote! { false },
            "uuid::Uuid" | "Uuid" => quote! { uuid::Uuid::new_v4() },
            "Vec<u8>" => quote! { Vec::new() },
            "Vec<String>" => quote! { Vec::new() },
            "()" => quote! { () },
            _ if type_name.starts_with("Option<") => {
                quote! { None }
            },
            _ if type_name.contains("PhantomData") => {
                quote! { std::marker::PhantomData }
            },
            "Unknown" => {
                // 如果类型名称无法识别，使用 Default::default()
                quote! { Default::default() }
            },
            _ => {
                // 对于自定义类型，尝试使用 Default trait
                if let Ok(type_ident) = syn::parse_str::<syn::Type>(type_name) {
                    quote! { <#type_ident as Default>::default() }
                } else {
                    // 如果类型解析失败，使用通用 Default
                    quote! { Default::default() }
                }
            },
        };

        Ok(default_expr)
    }

    /// 生成 default_instance 方法的实现代码
    ///
    /// 生成一个创建默认实例的方法，当 From 转换失败时使用。
    ///
    /// # 返回值
    ///
    /// 成功时返回生成的代码 TokenStream，失败时返回生成错误
    pub fn generate_default_instance_method(
        &self
    ) -> MacroResult<TokenStream2> {
        // 获取所有字段信息
        let all_fields = self.extract_all_fields()?;

        let mut field_inits = Vec::new();

        for field_info in all_fields {
            let field_name = syn::parse_str::<syn::Ident>(&field_info.name)
                .map_err(|_| {
                    MacroError::parse_error(
                        &format!("无效的字段名称: {}", field_info.name),
                        self.input,
                    )
                })?;

            // 生成字段的默认值
            let default_value = if let Some(config) = &field_info.config {
                if config.default_value.is_some() {
                    self.generate_default_value_for_instance(config)?
                } else {
                    self.generate_type_default_for_instance(
                        &field_info.type_name,
                    )?
                }
            } else {
                self.generate_type_default_for_instance(&field_info.type_name)?
            };

            field_inits.push(quote! {
                #field_name: #default_value
            });
        }

        let method_impl = quote! {
            /// 创建默认实例
            ///
            /// 当从 Mark 转换失败时使用此方法创建默认实例。
            /// 此方法由 #[derive(Mark)] 宏自动生成。
            ///
            /// # 返回值
            ///
            /// 返回使用默认值初始化的结构体实例
            fn default_instance() -> Self {
                Self {
                    #(#field_inits),*
                }
            }
        };

        Ok(method_impl)
    }

    /// 为实例生成默认值表达式（用于字段初始化）
    fn generate_default_value_for_instance(
        &self,
        field_config: &FieldConfig,
    ) -> MacroResult<TokenStream2> {
        if let Some(default_value) = &field_config.default_value {
            return self.generate_default_value_from_attr_for_instance(
                default_value,
                &field_config.type_name,
            );
        }

        self.generate_type_default_for_instance(&field_config.type_name)
    }

    /// 从 attr 的 default 属性生成实例默认值表达式
    fn generate_default_value_from_attr_for_instance(
        &self,
        default_value: &crate::parser::default_value::DefaultValue,
        target_type: &str,
    ) -> MacroResult<TokenStream2> {
        use crate::parser::default_value::DefaultValueType;

        match &default_value.value_type {
            DefaultValueType::String(s) => Ok(quote! { #s.to_string() }),
            DefaultValueType::Integer(i) => {
                // 根据目标类型进行适当的转换
                match target_type {
                    "i8" => Ok(quote! { #i as i8 }),
                    "i16" => Ok(quote! { #i as i16 }),
                    "i32" => Ok(quote! { #i as i32 }),
                    "i64" => Ok(quote! { #i }),
                    "i128" => Ok(quote! { #i as i128 }),
                    "isize" => Ok(quote! { #i as isize }),
                    "u8" => Ok(quote! { #i as u8 }),
                    "u16" => Ok(quote! { #i as u16 }),
                    "u32" => Ok(quote! { #i as u32 }),
                    "u64" => Ok(quote! { #i as u64 }),
                    "u128" => Ok(quote! { #i as u128 }),
                    "usize" => Ok(quote! { #i as usize }),
                    "f32" => Ok(quote! { #i as f32 }),
                    "f64" => Ok(quote! { #i as f64 }),
                    _ => Ok(quote! { #i as i32 }), // 默认转换为 i32
                }
            },
            DefaultValueType::Float(f) => {
                // 根据目标类型进行适当的转换
                match target_type {
                    "f32" => Ok(quote! { #f as f32 }),
                    "f64" => Ok(quote! { #f }),
                    _ => Ok(quote! { #f }),
                }
            },
            DefaultValueType::Boolean(b) => Ok(quote! { #b }),
            DefaultValueType::Json(_) => {
                // 对于复杂的 JSON，使用字符串表示
                Ok(quote! { String::default() })
            },
            DefaultValueType::CustomType(expr) => {
                // 对于自定义类型表达式，直接执行表达式
                let expr_tokens =
                    syn::parse_str::<syn::Expr>(expr).map_err(|_| {
                        MacroError::parse_error(
                            &format!("无效的自定义类型表达式: {expr}"),
                            self.input,
                        )
                    })?;
                Ok(quote! { #expr_tokens })
            },
            DefaultValueType::Null => Ok(quote! { String::default() }),
        }
    }

    /// 生成类型的默认值表达式（用于实例创建）
    fn generate_type_default_for_instance(
        &self,
        type_name: &str,
    ) -> MacroResult<TokenStream2> {
        let default_expr = match type_name {
            "String" => quote! { String::default() },
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => quote! { 0 },
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => quote! { 0 },
            "f32" | "f64" => quote! { 0.0 },
            "bool" => quote! { false },
            "serde_json::Value" | "Value" => quote! { serde_json::Value::Null },
            "uuid::Uuid" | "Uuid" => quote! { uuid::Uuid::new_v4() },
            "Vec<u8>" => quote! { Vec::new() },
            "Vec<String>" => quote! { Vec::new() },
            "()" => quote! { () },
            _ if type_name.starts_with("Option<") => {
                quote! { None }
            },
            "Unknown" => {
                // 如果类型名称无法识别，使用 Default::default()
                quote! { Default::default() }
            },
            _ => {
                // 对于其他自定义类型，尝试使用 Default trait，并提供更好的类型安全性
                let type_ident = syn::parse_str::<syn::Type>(type_name)
                    .map_err(|_| {
                        MacroError::parse_error(
                            &format!("无效的类型名称: {type_name}"),
                            self.input,
                        )
                    })?;
                quote! { <#type_ident as Default>::default() }
            },
        };

        Ok(default_expr)
    }
}

/// 字段信息结构体
///
/// 包含字段的基本信息和可选的配置信息
#[derive(Debug, Clone)]
struct FieldInfo {
    /// 字段名称
    name: String,
    /// 字段类型名称
    type_name: String,
    /// 可选的字段配置（如果有 #[attr] 标记）
    config: Option<FieldConfig>,
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
        let mark_definition_method = self.generate_mark_definition_method()?;
        let to_mark_method = self.generate_to_mark_method()?;
        let from_method = self.generate_from_method()?;
        let default_instance_method =
            self.generate_default_instance_method()?;

        Ok(quote! {
            impl #struct_name {
                #mark_definition_method

                #to_mark_method

                #from_method

                #default_instance_method
            }

            impl From<#struct_name> for mf_core::mark::Mark {
                /// 将结构体实例转换为 mf_core::mark::Mark
                ///
                /// 实现标准的 From trait，支持使用 `.into()` 方法进行转换。
                /// 此实现由 #[derive(Mark)] 宏自动生成。
                ///
                /// # 参数
                ///
                /// * `_value` - 结构体实例（当前实现中使用定义而非实例值）
                ///
                /// # 返回值
                ///
                /// 返回配置好的 `mf_core::mark::Mark` 定义
                ///
                /// # 使用示例
                ///
                /// ```rust
                /// let my_struct = MyStruct { /* fields */ };
                /// let mark: mf_core::mark::Mark = my_struct.into();
                /// // 或者
                /// let mark = mf_core::mark::Mark::from(my_struct);
                /// ```
                fn from(_value: #struct_name) -> Self {
                    #struct_name::mark_definition()
                }
            }

            impl From<mf_model::mark::Mark> for #struct_name {
                /// 从 mf_model::mark::Mark 转换为结构体实例
                ///
                /// 实现标准的 From trait，支持使用 `.into()` 方法进行反向转换。
                /// 此实现由 #[derive(Mark)] 宏自动生成。
                ///
                /// # 参数
                ///
                /// * `mark` - mf_model::mark::Mark 实例
                ///
                /// # 返回值
                ///
                /// 返回结构体实例，如果转换失败则使用默认值
                ///
                /// # 使用示例
                ///
                /// ```rust
                /// let mark: mf_model::mark::Mark = /* ... */;
                /// let my_struct: MyStruct = mark.into();
                /// // 或者
                /// let my_struct = MyStruct::from(mark);
                /// ```
                fn from(mark: mf_model::mark::Mark) -> Self {
                    #struct_name::from(&mark).unwrap_or_else(|_| {
                        // 如果转换失败，使用默认值创建实例
                        Self::default_instance()
                    })
                }
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
        assert!(
            code_str.contains("imbl")
                && code_str.contains("HashMap")
                && code_str.contains("new")
        );
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
        assert!(
            imports_str.contains("HashMap")
                || imports_str.contains("JsonValue")
        );
    }
}
