//! 属性解析器模块
//!
//! 负责解析 #[derive(Node)] 和 #[derive(Mark)] 派生宏的各种属性配置。
//! 严格遵循单一职责原则，专门负责宏属性的解析和结构化表示。

use syn::{Attribute, DeriveInput, Field, Lit, Meta};
use crate::common::{MacroError, MacroResult};

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
    pub fn parse_node_attributes(input: &DeriveInput) -> MacroResult<NodeConfig> {
        let mut config = NodeConfig::default();
        
        // 解析结构体级别的属性
        for attr in &input.attrs {
            match attr.path().get_ident().map(|i| i.to_string()).as_deref() {
                Some("node_type") => {
                    config.node_type = Some(Self::parse_string_attribute(attr)?);
                }
                Some("marks") => {
                    let marks_str = Self::parse_string_attribute(attr)?;
                    config.marks = Some(marks_str);
                }
                Some("content") => {
                    config.content = Some(Self::parse_string_attribute(attr)?);
                }
                _ => {
                    // 忽略不相关的属性
                }
            }
        }
        
        // 验证必需属性
        if config.node_type.is_none() {
            return Err(MacroError::missing_attribute("node_type", input));
        }
        
        // 解析字段级别的属性
        config.attr_fields = Self::parse_field_attributes(input)?;
        
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
    pub fn parse_mark_attributes(input: &DeriveInput) -> MacroResult<MarkConfig> {
        let mut config = MarkConfig::default();
        
        // 解析结构体级别的属性
        for attr in &input.attrs {
            if let Some(ident) = attr.path().get_ident() {
                if ident == "mark_type" {
                    config.mark_type = Some(Self::parse_string_attribute(attr)?);
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
            Meta::NameValue(meta) => {
                match &meta.value {
                    syn::Expr::Lit(expr_lit) => {
                        match &expr_lit.lit {
                            Lit::Str(lit_str) => {
                                let value = lit_str.value();
                                if value.is_empty() {
                                    return Err(MacroError::invalid_attribute_value(
                                        &attr.path().get_ident()
                                            .map(|i| i.to_string())
                                            .unwrap_or_default(),
                                        &value,
                                        "属性值不能为空字符串",
                                        attr,
                                    ));
                                }
                                Ok(value)
                            }
                            _ => Err(MacroError::invalid_attribute_value(
                                &attr.path().get_ident()
                                    .map(|i| i.to_string())
                                    .unwrap_or_default(),
                                "非字符串值",
                                "属性值必须是字符串字面量",
                                attr,
                            )),
                        }
                    }
                    _ => Err(MacroError::invalid_attribute_value(
                        &attr.path().get_ident()
                            .map(|i| i.to_string())
                            .unwrap_or_default(),
                        "复杂表达式",
                        "属性值必须是字符串字面量",
                        attr,
                    )),
                }
            }
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
    fn parse_field_attributes(input: &DeriveInput) -> MacroResult<Vec<FieldConfig>> {
        let mut fields = Vec::new();
        
        // 只处理结构体类型
        match &input.data {
            syn::Data::Struct(data_struct) => {
                match &data_struct.fields {
                    syn::Fields::Named(named_fields) => {
                        // 遍历所有具名字段
                        for field in &named_fields.named {
                            if let Some(field_name) = &field.ident {
                                // 检查字段是否带有 #[attr] 标记
                                let is_attr = field.attrs.iter()
                                    .any(|attr| {
                                        attr.path().get_ident()
                                            .map(|ident| ident == "attr")
                                            .unwrap_or(false)
                                    });
                                
                                if is_attr {
                                    // 提取类型信息
                                    let field_ty = &field.ty;
                                    let type_name = quote::quote! { #field_ty }.to_string().replace(" ", "");
                                    let is_optional = crate::common::utils::is_option_type(&field.ty);
                                    
                                    fields.push(FieldConfig {
                                        name: field_name.to_string(),
                                        type_name,
                                        is_optional,
                                        is_attr: true,
                                        field: field.clone(),
                                    });
                                }
                            }
                        }
                    }
                    syn::Fields::Unnamed(_) => {
                        return Err(MacroError::parse_error(
                            "不支持元组结构体，请使用具名字段的结构体",
                            input,
                        ));
                    }
                    syn::Fields::Unit => {
                        // 单元结构体没有字段，直接返回空列表
                    }
                }
            }
            syn::Data::Enum(_) => {
                return Err(MacroError::parse_error(
                    "不支持枚举类型，请使用结构体",
                    input,
                ));
            }
            syn::Data::Union(_) => {
                return Err(MacroError::parse_error(
                    "不支持联合体类型，请使用结构体",
                    input,
                ));
            }
        }
        
        Ok(fields)
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
                        message: format!("无效的标记名称: '{}'", mark),
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
                    message: format!("无效的标记类型名称: '{}'", mark_type),
                    span: None,
                });
            }
        }
        
        Ok(())
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
        let result = AttributeParser::parse_space_separated_list("bold italic underline");
        assert_eq!(result, vec!["bold", "italic", "underline"]);
        
        // 测试多个空格的情况
        let result = AttributeParser::parse_space_separated_list("bold  italic   underline");
        assert_eq!(result, vec!["bold", "italic", "underline"]);
        
        // 测试带前后空格的情况
        let result = AttributeParser::parse_space_separated_list("  bold italic underline  ");
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
}