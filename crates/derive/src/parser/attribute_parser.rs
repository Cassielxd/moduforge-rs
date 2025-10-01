//! 属性解析器模块
//!
//! 负责解析 #[derive(Node)] 和 #[derive(Mark)] 派生宏的各种属性配置。
//! 严格遵循单一职责原则，专门负责宏属性的解析和结构化表示。

use syn::{Attribute, DeriveInput, Field, Lit, Meta};
use syn::spanned::Spanned;
use crate::common::{MacroError, MacroResult};
use crate::parser::default_value::DefaultValue;

/// Node 属性配置
///
/// 存储 #[derive(Node)] 派生宏解析后的所有属性配置。
/// 遵循数据传输对象（DTO）模式，只包含数据不包含业务逻辑。
#[derive(Debug, Clone, Default)]
pub struct NodeConfig {
    /// 节点类型标识符（必需）
    ///
    /// 对应 #[node_type = "类型名"] 属性
    pub node_type: Option<String>,
    ///描述
    pub desc: Option<String>,

    /// 支持的标记类型列表（可选）
    ///
    /// 对应 #[marks = "mark1 mark2"] 属性，直接存储空格分隔的字符串
    pub marks: Option<String>,

    /// 内容约束表达式（可选）
    ///
    /// 对应 #[content = "表达式"] 属性
    pub content: Option<String>,

    /// 标记为属性的字段列表
    ///
    /// 包含所有带有 #[attr] 标记的字段信息
    pub attr_fields: Vec<FieldConfig>,

    /// 标记为 ID 映射的字段（可选）
    ///
    /// 包含带有 #[id] 标记的字段信息，用于映射 Node 的 id 字段
    pub id_field: Option<FieldConfig>,
}

/// Mark 属性配置
///
/// 存储 #[derive(Mark)] 派生宏解析后的所有属性配置。
/// 相比 Node 配置更简单，只包含基本的标记类型和属性字段。
#[derive(Debug, Clone, Default)]
pub struct MarkConfig {
    /// 标记类型标识符（必需）
    ///
    /// 对应 #[mark_type = "类型名"] 属性
    pub mark_type: Option<String>,

    /// 标记为属性的字段列表
    ///
    /// 包含所有带有 #[attr] 标记的字段信息
    pub attr_fields: Vec<FieldConfig>,
}

/// 字段配置
///
/// 描述结构体中一个字段的属性和类型信息。
/// 遵循接口隔离原则，只包含属性解析需要的最少信息。
///
/// # 设计原则体现
///
/// - **开闭原则**: 通过添加可选的默认值字段扩展功能，不修改现有代码
/// - **里氏替换**: 现有代码可以忽略新字段继续正常工作
#[derive(Debug, Clone)]
pub struct FieldConfig {
    /// 字段名称
    pub name: String,

    /// 字段的类型名称（字符串表示）
    pub type_name: String,

    /// 是否为可选类型（Option<T>）
    pub is_optional: bool,

    /// 是否标记为属性（带有 #[attr]）
    pub is_attr: bool,

    /// 原始字段引用（用于获取 span 信息）
    pub field: Field,

    /// 默认值配置（None 表示无默认值，保持现有行为）
    ///
    /// # 设计原则体现
    ///
    /// - **开闭原则**: 通过 Option 类型实现无破坏性扩展
    /// - **里氏替换**: 现有代码可以忽略此字段继续工作
    pub default_value: Option<DefaultValue>,
}

/// 属性解析器
///
/// 提供解析宏属性的核心功能。
/// 遵循单一职责原则，专门负责属性解析逻辑，不涉及验证或代码生成。
pub struct AttributeParser;

impl AttributeParser {
    /// 解析 Node 相关属性
    ///
    /// 从 DeriveInput 中提取和解析所有与 Node 相关的宏属性。
    /// 包括结构体级别的属性和字段级别的属性。
    ///
    /// # 参数
    ///
    /// * `input` - 派生宏的输入，包含结构体定义和属性
    ///
    /// # 返回值
    ///
    /// 成功时返回 Ok(NodeConfig)，失败时返回解析错误
    ///
    /// # 解析的属性
    ///
    /// - `#[node_type = "类型名"]` - 必需，节点类型标识符
    /// - `#[marks = "mark1 mark2"]` - 可选，支持的标记类型列表
    /// - `#[content = "表达式"]` -可选，内容约束表达式
    /// - `#[attr]` - 字段级属性，标记字段为节点属性
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责解析 Node 属性，不处理验证或生成
    /// - **里氏替换**: 任何 DeriveInput 都能正确处理
    ///
    /// # 示例
    ///
    /// ```rust
    /// use syn::parse_quote;
    /// use crate::parser::attribute_parser::AttributeParser;
    ///
    /// let input = parse_quote! {
    ///     #[derive(Node)]
    ///     #[node_type = "paragraph"]
    ///     #[marks = "bold italic"]
    ///     struct ParagraphNode {
    ///         #[attr]
    ///         content: String,
    ///     }
    /// };
    ///
    /// let config = AttributeParser::parse_node_attributes(&input)?;
    /// assert_eq!(config.node_type, Some("paragraph".to_string()));
    /// ```
    pub fn parse_node_attributes(
        input: &DeriveInput
    ) -> MacroResult<NodeConfig> {
        let mut config = NodeConfig::default();

        // 解析结构体级别的属性
        for attr in &input.attrs {
            match attr.path().get_ident().map(|i| i.to_string()).as_deref() {
                Some("node_type") => {
                    config.node_type =
                        Some(Self::parse_string_attribute(attr)?);
                },
                Some("marks") => {
                    let marks_str = Self::parse_string_attribute(attr)?;
                    config.marks = Some(marks_str);
                },
                Some("content") => {
                    config.content = Some(Self::parse_string_attribute(attr)?);
                },
                Some("desc") => {
                    config.desc = Some(Self::parse_string_attribute(attr)?);
                },
                _ => {
                    // 忽略不相关的属性
                },
            }
        }

        // 验证必需属性
        if config.node_type.is_none() {
            return Err(MacroError::missing_attribute("node_type", input));
        }

        // 解析字段级别的属性
        config.attr_fields = Self::parse_field_attributes(input)?;

        // 解析 ID 字段
        config.id_field = Self::parse_id_field(input)?;

        Ok(config)
    }

    /// 解析 Mark 相关属性
    ///
    /// 从 DeriveInput 中提取和解析所有与 Mark 相关的宏属性。
    /// 相比 Node 解析更简单，只需要处理 mark_type 和字段属性。
    ///
    /// # 参数
    ///
    /// * `input` - 派生宏的输入，包含结构体定义和属性
    ///
    /// # 返回值
    ///
    /// 成功时返回 Ok(MarkConfig)，失败时返回解析错误
    ///
    /// # 解析的属性
    ///
    /// - `#[mark_type = "类型名"]` - 必需，标记类型标识符
    /// - `#[attr]` - 字段级属性，标记字段为标记属性
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责解析 Mark 属性
    /// - **接口隔离**: 提供专门的 Mark 属性解析接口
    ///
    /// # 示例
    ///
    /// ```rust
    /// use syn::parse_quote;
    /// use crate::parser::attribute_parser::AttributeParser;
    ///
    /// let input = parse_quote! {
    ///     #[derive(Mark)]
    ///     #[mark_type = "bold"]
    ///     struct BoldMark {
    ///         #[attr]
    ///         strength: i32,
    ///     }
    /// };
    ///
    /// let config = AttributeParser::parse_mark_attributes(&input)?;
    /// assert_eq!(config.mark_type, Some("bold".to_string()));
    /// ```
    pub fn parse_mark_attributes(
        input: &DeriveInput
    ) -> MacroResult<MarkConfig> {
        let mut config = MarkConfig::default();

        // 解析结构体级别的属性
        for attr in &input.attrs {
            if let Some(ident) = attr.path().get_ident() {
                if ident == "mark_type" {
                    config.mark_type =
                        Some(Self::parse_string_attribute(attr)?);
                }
                // 忽略其他属性
            }
        }

        // 验证必需属性
        if config.mark_type.is_none() {
            return Err(MacroError::missing_attribute("mark_type", input));
        }

        // 解析字段级别的属性
        config.attr_fields = Self::parse_field_attributes(input)?;

        Ok(config)
    }

    /// 解析字符串类型的属性值
    ///
    /// 从属性中提取字符串值，处理 `#[key = "value"]` 格式的属性。
    /// 遵循单一职责原则，专门负责字符串属性值的提取。
    ///
    /// # 参数
    ///
    /// * `attr` - 要解析的属性
    ///
    /// # 返回值
    ///
    /// 成功时返回字符串值，失败时返回解析错误
    ///
    /// # 支持的格式
    ///
    /// - `#[key = "value"]` - 标准字符串属性
    /// - `#[key("value")]` - 函数调用风格（暂不支持）
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责字符串属性值解析
    /// - **里氏替换**: 任何字符串属性都能正确处理
    fn parse_string_attribute(attr: &Attribute) -> MacroResult<String> {
        match &attr.meta {
            Meta::NameValue(meta) => match &meta.value {
                syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
                    Lit::Str(lit_str) => {
                        let value = lit_str.value();
                        if value.is_empty() {
                            return Err(MacroError::invalid_attribute_value(
                                &attr
                                    .path()
                                    .get_ident()
                                    .map(|i| i.to_string())
                                    .unwrap_or_default(),
                                &value,
                                "属性值不能为空字符串",
                                attr,
                            ));
                        }
                        Ok(value)
                    },
                    _ => Err(MacroError::invalid_attribute_value(
                        &attr
                            .path()
                            .get_ident()
                            .map(|i| i.to_string())
                            .unwrap_or_default(),
                        "非字符串值",
                        "属性值必须是字符串字面量",
                        attr,
                    )),
                },
                _ => Err(MacroError::invalid_attribute_value(
                    &attr
                        .path()
                        .get_ident()
                        .map(|i| i.to_string())
                        .unwrap_or_default(),
                    "复杂表达式",
                    "属性值必须是字符串字面量",
                    attr,
                )),
            },
            _ => Err(MacroError::parse_error(
                "属性格式错误，期望 key = \"value\" 格式",
                attr,
            )),
        }
    }

    /// 解析空格分隔的字符串列表
    ///
    /// 将空格分隔的字符串解析为字符串向量，并去除空白字符。
    /// 遵循单一职责原则，专门处理列表字符串的分割和清理。
    ///
    /// # 参数
    ///
    /// * `input` - 空格分隔的字符串
    ///
    /// # 返回值
    ///
    /// 返回分割后的字符串向量，已去除空白和空项
    ///
    /// # 示例
    ///
    /// ```rust
    /// let marks = AttributeParser::parse_space_separated_list("bold italic underline");
    /// assert_eq!(marks, vec!["bold", "italic", "underline"]);
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责字符串分割和清理
    /// - **开闭原则**: 可扩展支持其他分隔符
    fn parse_space_separated_list(input: &str) -> Vec<String> {
        input
            .split_whitespace()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 解析字段的 attr 属性（可能包含默认值）
    ///
    /// 解析字段上的 #[attr] 或 #[attr(default="value")] 属性。
    /// 遵循单一职责原则，专门负责字段属性的解析。
    ///
    /// # 参数
    ///
    /// * `field` - 要解析的字段
    ///
    /// # 返回值
    ///
    /// 返回 `(is_attr, default_value)` 元组：
    /// - `is_attr`: 是否有 attr 属性标记
    /// - `default_value`: 解析到的默认值（如果有）
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责字段 attr 属性解析
    /// - **开闭原则**: 扩展支持默认值而不破坏现有行为
    /// - **接口隔离**: 提供专门的字段属性解析接口
    ///
    /// # 支持的语法
    ///
    /// - `#[attr]` - 标记为属性字段，无默认值
    /// - `#[attr(default="string_value")]` - 带字符串默认值
    /// - `#[attr(default=42)]` - 带数字默认值
    /// - `#[attr(default=true)]` - 带布尔默认值
    /// - `#[attr(default=null)]` - 带空值
    /// - `#[attr(default={"key": "value"})]` - 带 JSON 默认值
    ///
    /// # 错误处理
    ///
    /// - 无效的默认值语法会返回解析错误
    /// - 多个 attr 属性会返回错误
    /// - 无效的 JSON 格式会返回错误
    fn parse_field_attr_attribute(
        field: &Field
    ) -> MacroResult<(bool, Option<DefaultValue>)> {
        use syn::{Meta};
        

        let mut is_attr = false;
        let mut default_value = None;
        let mut attr_count = 0;

        // 遍历字段的所有属性
        for attr in &field.attrs {
            // 检查是否为 attr 属性
            if let Some(ident) = attr.path().get_ident() {
                if ident == "attr" {
                    attr_count += 1;
                    is_attr = true;

                    // 防止重复的 attr 属性
                    if attr_count > 1 {
                        return Err(MacroError::parse_error(
                            "字段不能有多个 #[attr] 属性",
                            field,
                        ));
                    }

                    // 解析属性参数（如果有）
                    match &attr.meta {
                        // #[attr] - 简单形式，无参数
                        Meta::Path(_) => {
                            // 保持现有行为，无默认值
                        },

                        // #[attr(default="value")] - 带参数形式
                        Meta::List(meta_list) => {
                            // 解析参数列表
                            default_value =
                                Self::parse_attr_meta_list(meta_list, field)?;
                        },

                        // #[attr = "value"] - 名值对形式（不支持，避免歧义）
                        Meta::NameValue(_) => {
                            return Err(MacroError::parse_error(
                                "不支持 #[attr = \"value\"] 语法，请使用 #[attr(default=\"value\")]",
                                field,
                            ));
                        },
                    }
                }
            }
        }

        Ok((is_attr, default_value))
    }

    /// 解析 attr 属性的参数列表
    ///
    /// 解析 #[attr(default="value")] 中的参数部分。
    /// 专门处理 default 参数的解析。
    ///
    /// # 参数
    ///
    /// * `meta_list` - syn::MetaList 参数列表
    /// * `field` - 字段引用（用于错误报告）
    ///
    /// # 返回值
    ///
    /// 返回解析得到的默认值（如果有）
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责参数列表解析
    /// - **接口隔离**: 提供专门的参数解析接口
    fn parse_attr_meta_list(
        meta_list: &syn::MetaList,
        field: &Field,
    ) -> MacroResult<Option<DefaultValue>> {
        use syn::{Meta, Token, parse::ParseStream, parse::Parse};
        use crate::parser::default_value::DefaultValueParser;

        // 自定义解析器来解析参数列表
        struct MetaArgs {
            metas: Vec<syn::Meta>,
        }

        impl Parse for MetaArgs {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let mut metas = Vec::new();

                while !input.is_empty() {
                    metas.push(input.parse::<syn::Meta>()?);

                    // 如果后面还有内容，消费逗号
                    if !input.is_empty() {
                        input.parse::<Token![,]>()?;
                    }
                }

                Ok(MetaArgs { metas })
            }
        }

        // 解析参数列表
        let args: MetaArgs = meta_list.parse_args().map_err(|e| {
            MacroError::parse_error(
                &format!("无法解析 attr 属性参数: {e}"),
                field,
            )
        })?;

        let mut default_value = None;

        // 遍历所有参数
        for nested_meta in args.metas {
            match nested_meta {
                // default="value" 形式
                Meta::NameValue(name_value) => {
                    if let Some(ident) = name_value.path.get_ident() {
                        if ident == "default" {
                            if default_value.is_some() {
                                return Err(MacroError::parse_error(
                                    "不能有多个 default 参数",
                                    field,
                                ));
                            }

                            // 解析默认值
                            let value_str = Self::extract_value_from_expr(
                                &name_value.value,
                            )?;
                            default_value = Some(DefaultValueParser::parse(
                                &value_str,
                                Some(name_value.value.span()),
                            )?);
                        } else {
                            return Err(MacroError::parse_error(
                                &format!("不支持的 attr 参数: {ident}"),
                                field,
                            ));
                        }
                    }
                },

                // 不支持其他形式的参数
                _ => {
                    return Err(MacroError::parse_error(
                        "attr 参数必须是 name=value 形式，如 default=\"value\"",
                        field,
                    ));
                },
            }
        }

        Ok(default_value)
    }

    /// 从表达式中提取字面量值
    ///
    /// 将 syn::Expr 转换为字符串表示，用于默认值解析。
    /// 支持各种类型的字面量表达式。
    ///
    /// # 参数
    ///
    /// * `expr` - 表达式引用
    ///
    /// # 返回值
    ///
    /// 返回表达式的字符串表示
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责表达式到字符串的转换
    /// - **开闭原则**: 支持扩展新的表达式类型
    fn extract_value_from_expr(expr: &syn::Expr) -> MacroResult<String> {
        use syn::Lit;

        match expr {
            // 字符串字面量: "hello"
            syn::Expr::Lit(expr_lit) => {
                match &expr_lit.lit {
                    Lit::Str(lit_str) => Ok(lit_str.value()),
                    Lit::Int(lit_int) => {
                        Ok(lit_int.base10_digits().to_string())
                    },
                    Lit::Float(lit_float) => {
                        Ok(lit_float.base10_digits().to_string())
                    },
                    Lit::Bool(lit_bool) => Ok(lit_bool.value.to_string()),
                    _ => {
                        // 对于其他字面量类型，使用 quote 转换
                        Ok(quote::quote! { #expr_lit }.to_string())
                    },
                }
            },

            // 路径表达式: null, true, false 等
            syn::Expr::Path(expr_path) => {
                if let Some(ident) = expr_path.path.get_ident() {
                    match ident.to_string().as_str() {
                        "true" => Ok("true".to_string()),
                        "false" => Ok("false".to_string()),
                        "null" => Ok("null".to_string()),
                        other => Ok(other.to_string()),
                    }
                } else {
                    Ok(quote::quote! { #expr_path }.to_string())
                }
            },

            // 负数: -42
            syn::Expr::Unary(expr_unary) => {
                if matches!(expr_unary.op, syn::UnOp::Neg(_)) {
                    let inner =
                        Self::extract_value_from_expr(&expr_unary.expr)?;
                    Ok(format!("-{inner}"))
                } else {
                    Ok(quote::quote! { #expr_unary }.to_string())
                }
            },

            // 其他表达式（包括 JSON 对象/数组）
            _ => {
                // 使用 quote 将表达式转换为字符串
                let token_stream = quote::quote! { #expr };
                let mut result = token_stream.to_string();

                // 移除不必要的空格（quote 生成的代码可能有额外空格）
                result = result.replace(" ", "");

                // 如果看起来像 JSON，恢复必要的空格
                if (result.starts_with('{') && result.ends_with('}'))
                    || (result.starts_with('[') && result.ends_with(']'))
                {
                    // 对于 JSON，保持原始格式
                    result = quote::quote! { #expr }.to_string();
                }

                Ok(result)
            },
        }
    }

    /// 解析字段级别的属性
    ///
    /// 分析结构体的所有字段，提取带有 #[attr] 标记的字段信息。
    /// 遵循单一职责原则，专门负责字段属性的识别和信息提取。
    ///
    /// # 参数
    ///
    /// * `input` - 派生宏的输入，包含结构体定义
    ///
    /// # 返回值
    ///
    /// 成功时返回字段配置向量，失败时返回解析错误
    ///
    /// # 提取的信息
    ///
    /// - 字段名称
    /// - 字段类型（字符串表示）
    /// - 是否为 Option 类型
    /// - 是否带有 #[attr] 标记
    /// - 原始字段引用
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责字段属性分析
    /// - **里氏替换**: 任何结构体字段都能正确处理
    ///
    /// # 示例
    ///
    /// ```rust
    /// let input = parse_quote! {
    ///     struct Example {
    ///         #[attr]
    ///         name: String,
    ///         
    ///         #[attr]
    ///         age: Option<i32>,
    ///         
    ///         description: String, // 不带 #[attr]，会被忽略
    ///     }
    /// };
    ///
    /// let fields = AttributeParser::parse_field_attributes(&input)?;
    /// assert_eq!(fields.len(), 2); // 只有带 #[attr] 的字段
    /// ```
    fn parse_field_attributes(
        input: &DeriveInput
    ) -> MacroResult<Vec<FieldConfig>> {
        let mut fields = Vec::new();

        // 只处理结构体类型
        match &input.data {
            syn::Data::Struct(data_struct) => {
                match &data_struct.fields {
                    syn::Fields::Named(named_fields) => {
                        // 遍历所有具名字段
                        for field in &named_fields.named {
                            if let Some(field_name) = &field.ident {
                                // 解析字段的 attr 属性（可能包含默认值）
                                let (is_attr, default_value) =
                                    Self::parse_field_attr_attribute(field)?;

                                if is_attr {
                                    // 提取类型信息
                                    let field_ty = &field.ty;
                                    let type_name = quote::quote! { #field_ty }
                                        .to_string()
                                        .replace(" ", "");
                                    let is_optional =
                                        crate::common::utils::is_option_type(
                                            &field.ty,
                                        );

                                    fields.push(FieldConfig {
                                        name: field_name.to_string(),
                                        type_name,
                                        is_optional,
                                        is_attr: true,
                                        field: field.clone(),
                                        default_value, // 从属性解析得到的默认值
                                    });
                                }
                            }
                        }
                    },
                    syn::Fields::Unnamed(_) => {
                        return Err(MacroError::parse_error(
                            "不支持元组结构体，请使用具名字段的结构体",
                            input,
                        ));
                    },
                    syn::Fields::Unit => {
                        // 单元结构体没有字段，直接返回空列表
                    },
                }
            },
            syn::Data::Enum(_) => {
                return Err(MacroError::parse_error(
                    "不支持枚举类型，请使用结构体",
                    input,
                ));
            },
            syn::Data::Union(_) => {
                return Err(MacroError::parse_error(
                    "不支持联合体类型，请使用结构体",
                    input,
                ));
            },
        }

        Ok(fields)
    }

    /// 解析 ID 字段
    ///
    /// 查找带有 #[id] 标记的字段，用于映射 Node 的 id 属性。
    /// 每个结构体最多只能有一个 #[id] 字段。
    ///
    /// # 参数
    ///
    /// * `input` - 派生宏的输入，包含结构体定义
    ///
    /// # 返回值
    ///
    /// 成功时返回 ID 字段配置（如果有），失败时返回解析错误
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责 ID 字段的解析
    /// - **接口隔离**: 提供专门的 ID 字段解析接口
    /// - **错误安全**: 防止多个 ID 字段冲突
    ///
    /// # 示例
    ///
    /// ```rust
    /// let input = parse_quote! {
    ///     struct Example {
    ///         #[id]
    ///         node_id: String,
    ///         
    ///         #[attr]
    ///         content: String,
    ///     }
    /// };
    ///
    /// let id_field = AttributeParser::parse_id_field(&input)?;
    /// assert!(id_field.is_some());
    /// assert_eq!(id_field.unwrap().name, "node_id");
    /// ```
    fn parse_id_field(input: &DeriveInput) -> MacroResult<Option<FieldConfig>> {
        let mut id_field = None;

        // 只处理结构体类型
        match &input.data {
            syn::Data::Struct(data_struct) => {
                match &data_struct.fields {
                    syn::Fields::Named(named_fields) => {
                        // 遍历所有具名字段
                        for field in &named_fields.named {
                            if let Some(field_name) = &field.ident {
                                // 检查是否有 #[id] 属性
                                let has_id_attr =
                                    Self::check_id_attribute(field)?;

                                if has_id_attr {
                                    // 确保不能有多个 ID 字段
                                    if id_field.is_some() {
                                        return Err(MacroError::parse_error(
                                            "一个结构体只能有一个 #[id] 字段",
                                            field,
                                        ));
                                    }

                                    // 提取类型信息
                                    let field_ty = &field.ty;
                                    let type_name = quote::quote! { #field_ty }
                                        .to_string()
                                        .replace(" ", "");
                                    let is_optional =
                                        crate::common::utils::is_option_type(
                                            &field.ty,
                                        );

                                    id_field = Some(FieldConfig {
                                        name: field_name.to_string(),
                                        type_name,
                                        is_optional,
                                        is_attr: false, // ID 字段不是普通属性
                                        field: field.clone(),
                                        default_value: None, // ID 字段不支持默认值
                                    });
                                }
                            }
                        }
                    },
                    syn::Fields::Unnamed(_) => {
                        return Err(MacroError::parse_error(
                            "不支持元组结构体，请使用具名字段的结构体",
                            input,
                        ));
                    },
                    syn::Fields::Unit => {
                        // 单元结构体没有字段，直接返回 None
                    },
                }
            },
            syn::Data::Enum(_) => {
                return Err(MacroError::parse_error(
                    "不支持枚举类型，请使用结构体",
                    input,
                ));
            },
            syn::Data::Union(_) => {
                return Err(MacroError::parse_error(
                    "不支持联合体类型，请使用结构体",
                    input,
                ));
            },
        }

        Ok(id_field)
    }

    /// 检查字段是否有 #[id] 属性
    ///
    /// 检查字段的属性列表中是否包含 #[id] 标记。
    ///
    /// # 参数
    ///
    /// * `field` - 要检查的字段
    ///
    /// # 返回值
    ///
    /// 如果字段有 #[id] 属性返回 true，否则返回 false
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责检查 ID 属性的存在
    /// - **接口隔离**: 提供简单的布尔查询接口
    fn check_id_attribute(field: &Field) -> MacroResult<bool> {
        let mut id_count = 0;

        // 遍历字段的所有属性
        for attr in &field.attrs {
            // 检查是否为 id 属性
            if let Some(ident) = attr.path().get_ident() {
                if ident == "id" {
                    id_count += 1;

                    // 防止重复的 id 属性
                    if id_count > 1 {
                        return Err(MacroError::parse_error(
                            "字段不能有多个 #[id] 属性",
                            field,
                        ));
                    }

                    // 验证 id 属性格式（应该是简单的 #[id]，不支持参数）
                    match &attr.meta {
                        syn::Meta::Path(_) => {
                            // #[id] - 正确格式
                        },
                        syn::Meta::List(_) => {
                            return Err(MacroError::parse_error(
                                "#[id] 属性不支持参数，请使用简单的 #[id] 格式",
                                field,
                            ));
                        },
                        syn::Meta::NameValue(_) => {
                            return Err(MacroError::parse_error(
                                "#[id] 属性不支持赋值，请使用简单的 #[id] 格式",
                                field,
                            ));
                        },
                    }
                }
            }
        }

        Ok(id_count > 0)
    }
}

impl NodeConfig {
    /// 验证 Node 配置的完整性
    ///
    /// 检查 Node 配置是否包含所有必需的信息。
    /// 遵循单一职责原则，专门负责配置完整性验证。
    ///
    /// # 返回值
    ///
    /// 配置有效时返回 Ok(())，否则返回验证错误
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责配置完整性检查
    /// - **接口隔离**: 提供简单的验证接口
    pub fn validate(&self) -> MacroResult<()> {
        // 验证必需属性
        if self.node_type.is_none() {
            return Err(MacroError::ValidationError {
                message: "缺少必需的 node_type 属性".to_string(),
                span: None,
            });
        }

        // 验证 marks 字符串（如果存在）
        if let Some(marks) = &self.marks {
            if marks.trim().is_empty() {
                return Err(MacroError::ValidationError {
                    message: "marks 属性不能为空字符串".to_string(),
                    span: None,
                });
            }

            // 检查每个 mark 是否为有效标识符
            for mark in marks.split_whitespace() {
                if !crate::common::utils::is_valid_identifier(mark) {
                    return Err(MacroError::ValidationError {
                        message: format!("无效的标记名称: '{mark}'"),
                        span: None,
                    });
                }
            }
        }

        Ok(())
    }

    /// 获取 marks 字符串表示
    ///
    /// 将 marks 列表转换为逗号分隔的字符串，用于代码生成。
    /// 遵循单一职责原则，专门负责格式转换。
    ///
    /// # 返回值
    ///
    /// 返回空格分隔的字符串，如果没有 marks 则返回 None
    pub fn marks_string(&self) -> Option<String> {
        self.marks.clone()
    }
}

impl MarkConfig {
    /// 验证 Mark 配置的完整性
    ///
    /// 检查 Mark 配置是否包含所有必需的信息。
    /// 遵循单一职责原则，专门负责配置完整性验证。
    ///
    /// # 返回值
    ///
    /// 配置有效时返回 Ok(())，否则返回验证错误
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责配置完整性检查
    /// - **里氏替换**: 与 NodeConfig 的验证方法可互换使用
    pub fn validate(&self) -> MacroResult<()> {
        // 验证必需属性
        if self.mark_type.is_none() {
            return Err(MacroError::ValidationError {
                message: "缺少必需的 mark_type 属性".to_string(),
                span: None,
            });
        }

        // 验证 mark_type 是否为有效标识符
        if let Some(mark_type) = &self.mark_type {
            if !crate::common::utils::is_valid_identifier(mark_type) {
                return Err(MacroError::ValidationError {
                    message: format!("无效的标记类型名称: '{mark_type}'"),
                    span: None,
                });
            }
        }

        Ok(())
    }
}

impl FieldConfig {
    /// 创建新的 FieldConfig 实例（保持现有接口不变）
    ///
    /// 此方法保持完全的向后兼容性，新的 default_value 字段默认为 None。
    ///
    /// # 参数
    ///
    /// * `name` - 字段名称
    /// * `type_name` - 字段类型名称
    /// * `is_optional` - 是否为 Option 类型
    /// * `is_attr` - 是否为属性字段
    /// * `field` - 原始字段引用
    ///
    /// # 返回值
    ///
    /// 返回新的 FieldConfig 实例
    ///
    /// # 设计原则体现
    ///
    /// - **里氏替换**: 与现有构造函数完全兼容
    /// - **开闭原则**: 新字段使用默认值，不影响现有行为
    pub fn new(
        name: String,
        type_name: String,
        is_optional: bool,
        is_attr: bool,
        field: Field,
    ) -> Self {
        Self {
            name,
            type_name,
            is_optional,
            is_attr,
            field,
            default_value: None, // 保持向后兼容
        }
    }

    /// 设置默认值（链式调用方式）
    ///
    /// 提供 builder 模式的便利方法，支持链式设置默认值。
    ///
    /// # 参数
    ///
    /// * `default_value` - 要设置的默认值
    ///
    /// # 返回值
    ///
    /// 返回设置了默认值的 Self 实例
    ///
    /// # 设计原则体现
    ///
    /// - **接口隔离**: 提供专门的默认值设置接口
    /// - **开闭原则**: 扩展功能而不修改现有结构
    ///
    /// # 使用示例
    ///
    /// ```rust
    /// let field_config = FieldConfig::new(...)
    ///     .with_default_value(default_value);
    /// ```
    pub fn with_default_value(
        mut self,
        default_value: DefaultValue,
    ) -> Self {
        self.default_value = Some(default_value);
        self
    }

    /// 检查是否有默认值
    ///
    /// 提供简单的布尔查询接口，检查字段是否配置了默认值。
    ///
    /// # 返回值
    ///
    /// 如果有默认值返回 true，否则返回 false
    ///
    /// # 设计原则体现
    ///
    /// - **接口隔离**: 提供简单的查询接口
    /// - **单一职责**: 专门负责默认值存在性检查
    ///
    /// # 使用示例
    ///
    /// ```rust
    /// if field_config.has_default_value() {
    ///     // 处理有默认值的字段
    /// }
    /// ```
    pub fn has_default_value(&self) -> bool {
        self.default_value.is_some()
    }

    /// 获取默认值引用
    ///
    /// 提供对默认值的只读访问，遵循借用检查规则。
    ///
    /// # 返回值
    ///
    /// 返回默认值的可选引用
    ///
    /// # 设计原则体现
    ///
    /// - **接口隔离**: 提供专门的默认值访问接口
    /// - **单一职责**: 专门负责默认值的只读访问
    ///
    /// # 使用示例
    ///
    /// ```rust
    /// if let Some(default_value) = field_config.get_default_value() {
    ///     // 使用默认值
    /// }
    /// ```
    pub fn get_default_value(&self) -> Option<&DefaultValue> {
        self.default_value.as_ref()
    }

    /// 获取默认值的可变引用
    ///
    /// 提供对默认值的可变访问，用于在解析过程中修改默认值。
    ///
    /// # 返回值
    ///
    /// 返回默认值的可选可变引用
    ///
    /// # 设计原则体现
    ///
    /// - **接口隔离**: 提供专门的默认值修改接口
    /// - **单一职责**: 专门负责默认值的可变访问
    pub fn get_default_value_mut(&mut self) -> Option<&mut DefaultValue> {
        self.default_value.as_mut()
    }

    /// 设置默认值（直接赋值方式）
    ///
    /// 提供直接设置默认值的方法，不使用链式调用。
    ///
    /// # 参数
    ///
    /// * `default_value` - 要设置的默认值（使用 Option 允许清空）
    ///
    /// # 设计原则体现
    ///
    /// - **接口隔离**: 提供专门的默认值设置接口
    /// - **里氏替换**: 可以与链式调用方法互换使用
    pub fn set_default_value(
        &mut self,
        default_value: Option<DefaultValue>,
    ) {
        self.default_value = default_value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    /// 测试基本的 Node 属性解析功能
    #[test]
    fn test_parse_basic_node_attributes() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "paragraph"]
            struct ParagraphNode {
                #[attr]
                content: String,
            }
        };

        let result = AttributeParser::parse_node_attributes(&input);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.node_type, Some("paragraph".to_string()));
        assert_eq!(config.attr_fields.len(), 1);
        assert_eq!(config.attr_fields[0].name, "content");
        assert!(!config.attr_fields[0].is_optional);
    }

    /// 测试完整的 Node 属性解析功能
    #[test]
    fn test_parse_full_node_attributes() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "paragraph"]
            #[marks = "bold italic underline"]
            #[content = "text*"]
            struct ParagraphNode {
                #[attr]
                content: String,

                #[attr]
                alignment: Option<String>,

                // 没有 #[attr] 的字段应该被忽略
                private_field: i32,
            }
        };

        let result = AttributeParser::parse_node_attributes(&input);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.node_type, Some("paragraph".to_string()));
        assert_eq!(config.marks, Some("bold italic underline".to_string()));
        assert_eq!(config.content, Some("text*".to_string()));
        assert_eq!(config.attr_fields.len(), 2);

        // 检查第一个字段
        assert_eq!(config.attr_fields[0].name, "content");
        assert!(!config.attr_fields[0].is_optional);

        // 检查第二个字段
        assert_eq!(config.attr_fields[1].name, "alignment");
        assert!(config.attr_fields[1].is_optional);
    }

    /// 测试基本的 Mark 属性解析功能
    #[test]
    fn test_parse_basic_mark_attributes() {
        let input: DeriveInput = parse_quote! {
            #[derive(Mark)]
            #[mark_type = "bold"]
            struct BoldMark {
                #[attr]
                strength: i32,
            }
        };

        let result = AttributeParser::parse_mark_attributes(&input);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.mark_type, Some("bold".to_string()));
        assert_eq!(config.attr_fields.len(), 1);
        assert_eq!(config.attr_fields[0].name, "strength");
    }

    /// 测试缺少必需属性的错误处理
    #[test]
    fn test_missing_required_attribute_error() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            // 缺少 node_type 属性
            struct InvalidNode {
                #[attr]
                content: String,
            }
        };

        let result = AttributeParser::parse_node_attributes(&input);
        assert!(result.is_err());

        if let Err(MacroError::MissingAttribute { attribute, .. }) = result {
            assert_eq!(attribute, "node_type");
        } else {
            panic!("期望 MissingAttribute 错误");
        }
    }

    /// 测试空属性值的错误处理
    #[test]
    fn test_empty_attribute_value_error() {
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = ""] // 空字符串
            struct InvalidNode {
                #[attr]
                content: String,
            }
        };

        let result = AttributeParser::parse_node_attributes(&input);
        assert!(result.is_err());

        if let Err(MacroError::InvalidAttributeValue { reason, .. }) = result {
            assert!(reason.contains("不能为空"));
        } else {
            panic!("期望 InvalidAttributeValue 错误");
        }
    }

    /// 测试空格分隔列表解析功能
    #[test]
    fn test_parse_space_separated_list() {
        // 测试正常情况
        let result = AttributeParser::parse_space_separated_list(
            "bold italic underline",
        );
        assert_eq!(result, vec!["bold", "italic", "underline"]);

        // 测试多个空格的情况
        let result = AttributeParser::parse_space_separated_list(
            "bold  italic   underline",
        );
        assert_eq!(result, vec!["bold", "italic", "underline"]);

        // 测试带前后空格的情况
        let result = AttributeParser::parse_space_separated_list(
            "  bold italic underline  ",
        );
        assert_eq!(result, vec!["bold", "italic", "underline"]);

        // 测试单个项目
        let result = AttributeParser::parse_space_separated_list("bold");
        assert_eq!(result, vec!["bold"]);

        // 测试空字符串
        let result = AttributeParser::parse_space_separated_list("");
        assert_eq!(result, Vec::<String>::new());

        // 测试只有空格的情况
        let result = AttributeParser::parse_space_separated_list("   ");
        assert_eq!(result, Vec::<String>::new());
    }

    /// 测试不支持的结构体类型
    #[test]
    fn test_unsupported_struct_types() {
        // 测试元组结构体
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "tuple"]
            struct TupleStruct(String, i32);
        };

        let result = AttributeParser::parse_node_attributes(&input);
        assert!(result.is_err());

        // 测试枚举类型
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "enum"]
            enum EnumType {
                Variant1,
                Variant2,
            }
        };

        let result = AttributeParser::parse_node_attributes(&input);
        assert!(result.is_err());
    }

    /// 测试 NodeConfig 验证功能
    #[test]
    fn test_node_config_validation() {
        // 测试有效配置
        let mut config = NodeConfig::default();
        config.node_type = Some("paragraph".to_string());
        assert!(config.validate().is_ok());

        // 测试缺少必需属性
        let config = NodeConfig::default();
        assert!(config.validate().is_err());

        // 测试空 marks 列表
        let mut config = NodeConfig::default();
        config.node_type = Some("paragraph".to_string());
        config.marks = Some("".to_string());
        assert!(config.validate().is_err());

        // 测试无效的 mark 名称
        let mut config = NodeConfig::default();
        config.node_type = Some("paragraph".to_string());
        config.marks = Some("invalid-mark".to_string());
        assert!(config.validate().is_err());
    }

    /// 测试 MarkConfig 验证功能
    #[test]
    fn test_mark_config_validation() {
        // 测试有效配置
        let mut config = MarkConfig::default();
        config.mark_type = Some("bold".to_string());
        assert!(config.validate().is_ok());

        // 测试缺少必需属性
        let config = MarkConfig::default();
        assert!(config.validate().is_err());

        // 测试无效的 mark_type 名称
        let mut config = MarkConfig::default();
        config.mark_type = Some("invalid-type".to_string());
        assert!(config.validate().is_err());
    }

    /// 测试 NodeConfig marks_string 方法
    #[test]
    fn test_node_config_marks_string() {
        let mut config = NodeConfig::default();

        // 没有 marks
        assert_eq!(config.marks_string(), None);

        // 有 marks
        config.marks = Some("bold italic".to_string());
        assert_eq!(config.marks_string(), Some("bold italic".to_string()));
    }

    /// 测试 FieldConfig 的向后兼容性
    #[test]
    fn test_field_config_backward_compatibility() {
        use syn::parse_quote;

        // 创建一个测试字段
        let field: Field = parse_quote! { content: String };

        // 使用新的构造函数
        let field_config = FieldConfig::new(
            "content".to_string(),
            "String".to_string(),
            false,
            true,
            field,
        );

        // 验证向后兼容性
        assert_eq!(field_config.name, "content");
        assert_eq!(field_config.type_name, "String");
        assert!(!field_config.is_optional);
        assert!(field_config.is_attr);
        assert!(!field_config.has_default_value()); // 新字段默认为 None
        assert!(field_config.get_default_value().is_none());
    }

    /// 测试 FieldConfig 的默认值相关方法
    #[test]
    fn test_field_config_default_value_methods() {
        use syn::parse_quote;
        use crate::parser::default_value::{
            DefaultValueType, DefaultValueParser,
        };

        // 创建一个测试字段
        let field: Field = parse_quote! { content: String };

        // 创建 FieldConfig
        let mut field_config = FieldConfig::new(
            "content".to_string(),
            "String".to_string(),
            false,
            true,
            field,
        );

        // 初始状态：没有默认值
        assert!(!field_config.has_default_value());
        assert!(field_config.get_default_value().is_none());

        // 创建一个默认值
        let default_value =
            DefaultValueParser::parse("hello world", None).unwrap();

        // 测试直接设置方法
        field_config.set_default_value(Some(default_value.clone()));
        assert!(field_config.has_default_value());
        assert!(field_config.get_default_value().is_some());

        // 验证默认值内容
        let stored_value = field_config.get_default_value().unwrap();
        assert_eq!(stored_value.raw_value, "hello world");
        assert!(
            matches!(stored_value.value_type, DefaultValueType::String(ref s) if s == "hello world")
        );

        // 测试清空默认值
        field_config.set_default_value(None);
        assert!(!field_config.has_default_value());

        // 测试链式调用方法
        let field2: Field = parse_quote! { title: String };
        let field_config2 = FieldConfig::new(
            "title".to_string(),
            "String".to_string(),
            false,
            true,
            field2,
        )
        .with_default_value(default_value);

        assert!(field_config2.has_default_value());
        assert_eq!(
            field_config2.get_default_value().unwrap().raw_value,
            "hello world"
        );
    }

    /// 测试 FieldConfig 的可变引用功能
    #[test]
    fn test_field_config_mutable_default_value() {
        use syn::parse_quote;
        use crate::parser::default_value::{DefaultValueParser};

        // 创建一个测试字段
        let field: Field = parse_quote! { content: String };

        // 创建带默认值的 FieldConfig
        let default_value = DefaultValueParser::parse("initial", None).unwrap();
        let mut field_config = FieldConfig::new(
            "content".to_string(),
            "String".to_string(),
            false,
            true,
            field,
        )
        .with_default_value(default_value);

        // 获取可变引用并修改
        if let Some(default_value_mut) = field_config.get_default_value_mut() {
            // 这里我们可以修改默认值的内容
            // 注意：DefaultValue 的字段都是公开的，可以直接修改
            assert_eq!(default_value_mut.raw_value, "initial");
        } else {
            panic!("应该有默认值");
        }
    }

    /// 测试字段属性解析的新功能
    #[test]
    fn test_parse_field_attr_with_default_values() {
        use syn::parse_quote;
        use crate::parser::default_value::{DefaultValueType};

        // 测试简单的 #[attr] 语法（向后兼容）
        let field: Field = parse_quote! {
            #[attr]
            content: String
        };

        let (is_attr, default_value) =
            AttributeParser::parse_field_attr_attribute(&field).unwrap();
        assert!(is_attr);
        assert!(default_value.is_none());

        // 测试带字符串默认值的语法
        let field: Field = parse_quote! {
            #[attr(default = "hello world")]
            content: String
        };

        let (is_attr, default_value) =
            AttributeParser::parse_field_attr_attribute(&field).unwrap();
        assert!(is_attr);
        assert!(default_value.is_some());

        let default_val = default_value.unwrap();
        assert_eq!(default_val.raw_value, "hello world");
        assert!(
            matches!(default_val.value_type, DefaultValueType::String(ref s) if s == "hello world")
        );

        // 测试带数字默认值的语法
        let field: Field = parse_quote! {
            #[attr(default = 42)]
            count: i32
        };

        let (is_attr, default_value) =
            AttributeParser::parse_field_attr_attribute(&field).unwrap();
        assert!(is_attr);
        assert!(default_value.is_some());

        let default_val = default_value.unwrap();
        assert_eq!(default_val.raw_value, "42");
        assert!(matches!(
            default_val.value_type,
            DefaultValueType::Integer(42)
        ));

        // 测试带布尔默认值的语法
        let field: Field = parse_quote! {
            #[attr(default = true)]
            enabled: bool
        };

        let (is_attr, default_value) =
            AttributeParser::parse_field_attr_attribute(&field).unwrap();
        assert!(is_attr);
        assert!(default_value.is_some());

        let default_val = default_value.unwrap();
        assert_eq!(default_val.raw_value, "true");
        assert!(matches!(
            default_val.value_type,
            DefaultValueType::Boolean(true)
        ));
    }

    /// 测试 ID 字段解析功能
    #[test]
    fn test_parse_id_field() {
        use syn::parse_quote;

        // 测试有 ID 字段的情况
        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "test_node"]
            struct TestNode {
                #[id]
                node_id: String,

                #[attr]
                content: String,
            }
        };

        let config = AttributeParser::parse_node_attributes(&input).unwrap();
        assert!(config.id_field.is_some());

        let id_field = config.id_field.unwrap();
        assert_eq!(id_field.name, "node_id");
        assert_eq!(id_field.type_name, "String");
        assert!(!id_field.is_optional);
        assert!(!id_field.is_attr); // ID 字段不是 attr
        assert!(id_field.default_value.is_none()); // ID 字段不支持默认值
    }

    /// 测试 Option<String> 类型的 ID 字段
    #[test]
    fn test_parse_optional_id_field() {
        use syn::parse_quote;

        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "test_node"]
            struct TestNode {
                #[id]
                node_id: Option<String>,

                #[attr]
                content: String,
            }
        };

        let config = AttributeParser::parse_node_attributes(&input).unwrap();
        assert!(config.id_field.is_some());

        let id_field = config.id_field.unwrap();
        assert_eq!(id_field.name, "node_id");
        assert_eq!(id_field.type_name, "Option<String>");
        assert!(id_field.is_optional);
    }

    /// 测试没有 ID 字段的情况
    #[test]
    fn test_parse_no_id_field() {
        use syn::parse_quote;

        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "test_node"]
            struct TestNode {
                #[attr]
                content: String,
            }
        };

        let config = AttributeParser::parse_node_attributes(&input).unwrap();
        assert!(config.id_field.is_none());
    }

    /// 测试多个 ID 字段的错误处理
    #[test]
    fn test_multiple_id_fields_error() {
        use syn::parse_quote;

        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "test_node"]
            struct TestNode {
                #[id]
                node_id1: String,

                #[id]
                node_id2: String,

                #[attr]
                content: String,
            }
        };

        let result = AttributeParser::parse_node_attributes(&input);
        assert!(result.is_err());

        if let Err(error) = result {
            let error_msg = format!("{error:?}");
            assert!(error_msg.contains("一个结构体只能有一个"));
        }
    }

    /// 测试 ID 字段的重复属性错误
    #[test]
    fn test_duplicate_id_attribute_error() {
        use syn::parse_quote;

        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "test_node"]
            struct TestNode {
                #[id]
                #[id]
                node_id: String,

                #[attr]
                content: String,
            }
        };

        let result = AttributeParser::parse_node_attributes(&input);
        assert!(result.is_err());

        if let Err(error) = result {
            let error_msg = format!("{error:?}");
            assert!(error_msg.contains("多个 #[id] 属性"));
        }
    }

    /// 测试 ID 属性不支持参数的错误处理
    #[test]
    fn test_id_attribute_with_params_error() {
        use syn::parse_quote;

        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "test_node"]
            struct TestNode {
                #[id(param = "value")]
                node_id: String,

                #[attr]
                content: String,
            }
        };

        let result = AttributeParser::parse_node_attributes(&input);
        assert!(result.is_err());

        if let Err(error) = result {
            let error_msg = format!("{error:?}");
            assert!(error_msg.contains("不支持参数"));
        }
    }

    /// 测试 ID 属性不支持赋值的错误处理
    #[test]
    fn test_id_attribute_with_value_error() {
        use syn::parse_quote;

        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "test_node"]
            struct TestNode {
                #[id = "value"]
                node_id: String,

                #[attr]
                content: String,
            }
        };

        let result = AttributeParser::parse_node_attributes(&input);
        assert!(result.is_err());

        if let Err(error) = result {
            let error_msg = format!("{error:?}");
            assert!(error_msg.contains("不支持赋值"));
        }
    }

    /// 测试同时有 ID 字段和属性字段的完整解析
    #[test]
    fn test_complete_parsing_with_id_and_attr_fields() {
        use syn::parse_quote;

        let input: DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "complex_node"]
            #[marks = "bold italic"]
            #[content = "text*"]
            struct ComplexNode {
                #[id]
                node_id: String,

                #[attr]
                title: String,

                #[attr(default = "default content")]
                content: String,

                #[attr]
                optional_field: Option<String>,

                // 普通字段（无标记）
                internal_data: Vec<u8>,
            }
        };

        let config = AttributeParser::parse_node_attributes(&input).unwrap();

        // 验证基本配置
        assert_eq!(config.node_type, Some("complex_node".to_string()));
        assert_eq!(config.marks, Some("bold italic".to_string()));
        assert_eq!(config.content, Some("text*".to_string()));

        // 验证 ID 字段
        assert!(config.id_field.is_some());
        let id_field = config.id_field.unwrap();
        assert_eq!(id_field.name, "node_id");
        assert_eq!(id_field.type_name, "String");

        // 验证属性字段
        assert_eq!(config.attr_fields.len(), 3);

        let title_field =
            config.attr_fields.iter().find(|f| f.name == "title").unwrap();
        assert_eq!(title_field.type_name, "String");
        assert!(!title_field.has_default_value());

        let content_field =
            config.attr_fields.iter().find(|f| f.name == "content").unwrap();
        assert_eq!(content_field.type_name, "String");
        assert!(content_field.has_default_value());
        assert_eq!(
            content_field.get_default_value().unwrap().raw_value,
            "default content"
        );

        let optional_field = config
            .attr_fields
            .iter()
            .find(|f| f.name == "optional_field")
            .unwrap();
        assert_eq!(optional_field.type_name, "Option<String>");
        assert!(optional_field.is_optional);
        assert!(!optional_field.has_default_value());
    }

    /// 测试字段属性解析的错误处理
    #[test]
    fn test_parse_field_attr_error_handling() {
        use syn::parse_quote;

        // 测试多个 #[attr] 属性的错误
        let field: Field = parse_quote! {
            #[attr]
            #[attr(default = "test")]
            content: String
        };

        let result = AttributeParser::parse_field_attr_attribute(&field);
        assert!(result.is_err());

        // 测试不支持的 #[attr = "value"] 语法
        let field: Field = parse_quote! {
            #[attr = "value"]
            content: String
        };

        let result = AttributeParser::parse_field_attr_attribute(&field);
        assert!(result.is_err());

        // 测试重复的 default 参数
        // 注意：这个测试可能会因为语法解析失败而不能正确测试，但我们可以测试逻辑
    }

    /// 测试表达式值提取
    #[test]
    fn test_extract_value_from_expr() {
        use syn::parse_quote;

        // 测试字符串字面量
        let expr: syn::Expr = parse_quote! { "hello" };
        let result = AttributeParser::extract_value_from_expr(&expr).unwrap();
        assert_eq!(result, "hello");

        // 测试整数字面量
        let expr: syn::Expr = parse_quote! { 42 };
        let result = AttributeParser::extract_value_from_expr(&expr).unwrap();
        assert_eq!(result, "42");

        // 测试浮点数字面量
        let expr: syn::Expr = parse_quote! { 3.14 };
        let result = AttributeParser::extract_value_from_expr(&expr).unwrap();
        assert_eq!(result, "3.14");

        // 测试布尔字面量
        let expr: syn::Expr = parse_quote! { true };
        let result = AttributeParser::extract_value_from_expr(&expr).unwrap();
        assert_eq!(result, "true");

        let expr: syn::Expr = parse_quote! { false };
        let result = AttributeParser::extract_value_from_expr(&expr).unwrap();
        assert_eq!(result, "false");

        // 测试 null 路径
        let expr: syn::Expr = parse_quote! { null };
        let result = AttributeParser::extract_value_from_expr(&expr).unwrap();
        assert_eq!(result, "null");

        // 测试负数
        let expr: syn::Expr = parse_quote! { -42 };
        let result = AttributeParser::extract_value_from_expr(&expr).unwrap();
        assert_eq!(result, "-42");
    }

    /// 测试完整的字段解析过程
    #[test]
    fn test_complete_field_parsing_with_defaults() {
        use syn::parse_quote;

        // 创建一个测试结构体
        let input: syn::DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "test_node"]
            struct TestNode {
                #[attr]
                simple_field: String,

                #[attr(default = "default value")]
                field_with_default: String,

                #[attr(default = 42)]
                numeric_field: i32,

                #[attr(default = true)]
                boolean_field: bool,

                regular_field: String,
            }
        };

        // 解析 Node 配置
        let config = AttributeParser::parse_node_attributes(&input).unwrap();

        // 验证字段数量（应该有 4 个 attr 字段）
        assert_eq!(config.attr_fields.len(), 4);

        // 验证各个字段的默认值设置
        let simple_field = config
            .attr_fields
            .iter()
            .find(|f| f.name == "simple_field")
            .expect("应该找到 simple_field");
        assert!(!simple_field.has_default_value());

        let field_with_default = config
            .attr_fields
            .iter()
            .find(|f| f.name == "field_with_default")
            .expect("应该找到 field_with_default");
        assert!(field_with_default.has_default_value());
        assert_eq!(
            field_with_default.get_default_value().unwrap().raw_value,
            "default value"
        );

        let numeric_field = config
            .attr_fields
            .iter()
            .find(|f| f.name == "numeric_field")
            .expect("应该找到 numeric_field");
        assert!(numeric_field.has_default_value());
        assert_eq!(numeric_field.get_default_value().unwrap().raw_value, "42");

        let boolean_field = config
            .attr_fields
            .iter()
            .find(|f| f.name == "boolean_field")
            .expect("应该找到 boolean_field");
        assert!(boolean_field.has_default_value());
        assert_eq!(
            boolean_field.get_default_value().unwrap().raw_value,
            "true"
        );
    }
}
