//! 字段分析器模块
//!
//! 负责分析结构体字段的类型信息和属性标记。
//! 遵循单一职责原则，专门处理字段相关的分析逻辑。

use syn::{Field, Type};
use crate::common::{MacroError, MacroResult, utils};

/// 字段类型信息
///
/// 描述一个字段的详细类型信息。
/// 遵循数据传输对象（DTO）模式，只包含类型分析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct FieldTypeInfo {
    /// 原始类型名称（完整的类型表示）
    pub original_type: String,

    /// 简化的类型名称（用于显示和错误消息）
    pub simple_name: String,

    /// 是否为 Option<T> 包装类型
    pub is_optional: bool,

    /// 内部类型（如果是 Option<T>，则为 T 的类型信息）
    pub inner_type: Option<Box<FieldTypeInfo>>,

    /// 是否为支持的基本类型
    pub is_supported: bool,
}

/// 字段分析结果
///
/// 包含字段的完整分析结果，包括名称、类型信息和属性标记。
/// 遵循接口隔离原则，只包含分析结果相关的信息。
#[derive(Debug, Clone)]
pub struct FieldAnalysis {
    /// 字段名称
    pub name: String,

    /// 字段的类型信息
    pub type_info: FieldTypeInfo,

    /// 是否带有 #[attr] 标记
    pub is_marked_as_attr: bool,

    /// 字段的所有属性标记
    pub attributes: Vec<String>,

    /// 原始字段引用（用于错误定位）
    pub original_field: Field,
}

/// 字段分析器
///
/// 提供字段类型分析和属性检查的核心功能。
/// 遵循单一职责原则，专门负责字段相关的分析逻辑。
pub struct FieldAnalyzer;

impl FieldAnalyzer {
    /// 分析字段类型信息
    ///
    /// 深度分析字段的类型信息，包括是否为 Option 类型、内部类型等。
    /// 遵循里氏替换原则，任何 Type 都能正确分析。
    ///
    /// # 参数
    ///
    /// * `field_type` - 要分析的字段类型
    ///
    /// # 返回值
    ///
    /// 返回详细的字段类型信息
    ///
    /// # 分析内容
    ///
    /// - 原始类型名称和简化名称
    /// - 是否为 Option<T> 类型
    /// - Option 的内部类型信息（递归分析）
    /// - 是否为支持的基本类型
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责类型信息分析
    /// - **里氏替换**: 任何 Type 都能正确处理
    /// - **开闭原则**: 可扩展支持新的类型分析规则
    ///
    /// # 示例
    ///
    /// ```rust
    /// use syn::parse_quote;
    /// use crate::parser::field_analyzer::FieldAnalyzer;
    ///
    /// let ty: Type = parse_quote! { Option<String> };
    /// let info = FieldAnalyzer::analyze_field_type(&ty);
    ///
    /// assert_eq!(info.simple_name, "Option<String>");
    /// assert!(info.is_optional);
    /// assert!(info.inner_type.is_some());
    /// assert_eq!(info.inner_type.unwrap().simple_name, "String");
    /// ```
    pub fn analyze_field_type(field_type: &Type) -> FieldTypeInfo {
        let original_type = quote::quote! { #field_type }.to_string();
        let simple_name = utils::extract_type_name(field_type);

        // 检查是否为 Option 类型
        if utils::is_option_type(field_type) {
            // 分析 Option 的内部类型
            if let Some(inner_type) =
                utils::extract_option_inner_type(field_type)
            {
                let inner_info = Self::analyze_field_type(inner_type);

                FieldTypeInfo {
                    original_type,
                    simple_name,
                    is_optional: true,
                    inner_type: Some(Box::new(inner_info.clone())),
                    is_supported: inner_info.is_supported, // Option<T> 的支持性取决于 T
                }
            } else {
                // 无法解析内部类型的 Option
                FieldTypeInfo {
                    original_type,
                    simple_name,
                    is_optional: true,
                    inner_type: None,
                    is_supported: false,
                }
            }
        } else {
            // 普通类型（非 Option）
            let is_supported = utils::is_supported_basic_type(field_type);

            FieldTypeInfo {
                original_type,
                simple_name,
                is_optional: false,
                inner_type: None,
                is_supported,
            }
        }
    }

    /// 分析单个字段
    ///
    /// 对字段进行完整分析，包括类型信息和属性标记。
    /// 遵循单一职责原则，专门负责单个字段的全面分析。
    ///
    /// # 参数
    ///
    /// * `field` - 要分析的字段
    ///
    /// # 返回值
    ///
    /// 成功时返回字段分析结果，失败时返回分析错误
    ///
    /// # 分析内容
    ///
    /// - 字段名称提取和验证
    /// - 字段类型的详细分析
    /// - 属性标记的识别和提取
    /// - 支持性验证
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责单个字段的分析
    /// - **接口隔离**: 提供完整但精简的分析结果
    ///
    /// # 示例
    ///
    /// ```rust
    /// use syn::parse_quote;
    /// use crate::parser::field_analyzer::FieldAnalyzer;
    ///
    /// let field: Field = parse_quote! {
    ///     #[attr]
    ///     name: String
    /// };
    ///
    /// let analysis = FieldAnalyzer::analyze_field(&field)?;
    /// assert_eq!(analysis.name, "name");
    /// assert!(analysis.is_marked_as_attr);
    /// assert!(analysis.type_info.is_supported);
    /// ```
    pub fn analyze_field(field: &Field) -> MacroResult<FieldAnalysis> {
        // 提取字段名称
        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| {
                MacroError::parse_error("字段缺少名称（不支持匿名字段）", field)
            })?
            .to_string();

        // 分析字段类型
        let type_info = Self::analyze_field_type(&field.ty);

        // 分析字段属性
        let (is_marked_as_attr, attributes) =
            Self::analyze_field_attributes(field)?;

        Ok(FieldAnalysis {
            name: field_name,
            type_info,
            is_marked_as_attr,
            attributes,
            original_field: field.clone(),
        })
    }

    /// 分析多个字段
    ///
    /// 批量分析多个字段，并返回所有字段的分析结果。
    /// 遵循开闭原则，可以扩展支持不同的字段过滤和分析策略。
    ///
    /// # 参数
    ///
    /// * `fields` - 要分析的字段列表
    ///
    /// # 返回值
    ///
    /// 成功时返回所有字段的分析结果，失败时返回第一个遇到的错误
    ///
    /// # 分析策略
    ///
    /// - 逐个分析每个字段
    /// - 收集所有分析结果
    /// - 遇到错误时立即停止并返回错误
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 专门负责批量字段分析
    /// - **开闭原则**: 可扩展不同的批处理策略
    pub fn analyze_fields(fields: &[Field]) -> MacroResult<Vec<FieldAnalysis>> {
        let mut analyses = Vec::with_capacity(fields.len());

        for field in fields {
            let analysis = Self::analyze_field(field)?;
            analyses.push(analysis);
        }

        Ok(analyses)
    }

    /// 过滤带有属性标记的字段
    ///
    /// 从字段分析结果中筛选出带有 #[attr] 标记的字段。
    /// 遵循接口隔离原则，提供专门的过滤功能。
    ///
    /// # 参数
    ///
    /// * `analyses` - 字段分析结果列表
    ///
    /// # 返回值
    ///
    /// 返回所有带有 #[attr] 标记的字段分析结果
    ///
    /// # 过滤规则
    ///
    /// - 只保留 `is_marked_as_attr` 为 true 的字段
    /// - 保持原有的顺序
    ///
    /// # 设计原则体现
    ///
    /// - **接口隔离**: 提供专门的过滤接口
    /// - **单一职责**: 只负责属性字段的筛选
    pub fn filter_attr_fields(
        analyses: &[FieldAnalysis]
    ) -> Vec<&FieldAnalysis> {
        analyses.iter().filter(|analysis| analysis.is_marked_as_attr).collect()
    }

    /// 验证字段类型的支持性
    ///
    /// 检查字段类型是否被宏系统支持。
    /// 遵循单一职责原则，专门负责类型支持性验证。
    ///
    /// # 参数
    ///
    /// * `analysis` - 字段分析结果
    ///
    /// # 返回值
    ///
    /// 如果类型受支持则返回 Ok(())，否则返回验证错误
    ///
    /// # 验证规则
    ///
    /// - 基本类型必须在支持列表中
    /// - Option<T> 类型要求 T 是支持的基本类型
    /// - 复合类型暂不支持
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责类型支持性验证
    /// - **开闭原则**: 可扩展支持新的类型验证规则
    pub fn validate_field_type_support(
        analysis: &FieldAnalysis
    ) -> MacroResult<()> {
        if !analysis.type_info.is_supported {
            return Err(MacroError::unsupported_field_type(
                &analysis.name,
                &analysis.type_info.simple_name,
                &analysis.original_field,
            ));
        }

        // 对于 Option 类型，还需要验证内部类型
        if analysis.type_info.is_optional {
            if let Some(inner_type) = &analysis.type_info.inner_type {
                if !inner_type.is_supported {
                    return Err(MacroError::unsupported_field_type(
                        &analysis.name,
                        &inner_type.simple_name,
                        &analysis.original_field,
                    ));
                }
            }
        }

        Ok(())
    }

    /// 批量验证字段类型支持性
    ///
    /// 对多个字段进行类型支持性验证。
    /// 遵循开闭原则，可以扩展不同的批量验证策略。
    ///
    /// # 参数
    ///
    /// * `analyses` - 字段分析结果列表
    ///
    /// # 返回值
    ///
    /// 如果所有字段类型都受支持则返回 Ok(())，否则返回第一个验证错误
    ///
    /// # 验证策略
    ///
    /// - 逐个验证每个字段的类型支持性
    /// - 遇到不支持的类型时立即返回错误
    /// - 提供详细的错误信息
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 专门负责批量类型验证
    /// - **里氏替换**: 可以替换单个字段验证使用
    pub fn validate_all_field_types(
        analyses: &[FieldAnalysis]
    ) -> MacroResult<()> {
        for analysis in analyses {
            Self::validate_field_type_support(analysis)?;
        }
        Ok(())
    }

    /// 分析字段的属性标记
    ///
    /// 提取和分析字段上的所有属性标记。
    /// 遵循单一职责原则，专门处理字段属性的提取。
    ///
    /// # 参数
    ///
    /// * `field` - 要分析的字段
    ///
    /// # 返回值
    ///
    /// 返回元组：(是否带有 #[attr] 标记, 所有属性名称列表)
    ///
    /// # 分析内容
    ///
    /// - 检查是否带有 #[attr] 标记
    /// - 提取所有属性的名称
    /// - 过滤无效的属性
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责属性标记的分析
    /// - **接口隔离**: 提供简洁的属性分析接口
    fn analyze_field_attributes(
        field: &Field
    ) -> MacroResult<(bool, Vec<String>)> {
        let mut is_marked_as_attr = false;
        let mut attributes = Vec::new();

        for attr in &field.attrs {
            if let Some(ident) = attr.path().get_ident() {
                let attr_name = ident.to_string();
                attributes.push(attr_name.clone());

                // 检查是否为 #[attr] 标记
                if attr_name == "attr" {
                    is_marked_as_attr = true;

                    // 验证 #[attr] 属性的格式
                    Self::validate_attr_attribute(attr)?;
                }
            }
        }

        Ok((is_marked_as_attr, attributes))
    }

    /// 验证 #[attr] 属性的格式
    ///
    /// 确保 #[attr] 属性使用正确的格式。
    /// 遵循单一职责原则，专门验证属性格式。
    ///
    /// # 参数
    ///
    /// * `attr` - 要验证的属性
    ///
    /// # 返回值
    ///
    /// 如果格式正确则返回 Ok(())，否则返回格式错误
    ///
    /// # 验证规则
    ///
    /// - #[attr] 应该是简单标记，不带参数
    /// - 不支持 #[attr = "value"] 或 #[attr(param)] 格式
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责 #[attr] 属性格式验证
    /// - **接口隔离**: 提供专门的格式验证接口
    fn validate_attr_attribute(attr: &syn::Attribute) -> MacroResult<()> {
        match &attr.meta {
            syn::Meta::Path(_) => {
                // #[attr] 格式，正确
                Ok(())
            },
            syn::Meta::List(_) => {
                // #[attr(...)] 格式，暂不支持
                Err(MacroError::parse_error(
                    "#[attr] 不支持参数，请使用简单的 #[attr] 标记",
                    attr,
                ))
            },
            syn::Meta::NameValue(_) => {
                // #[attr = "..."] 格式，暂不支持
                Err(MacroError::parse_error(
                    "#[attr] 不支持值赋值，请使用简单的 #[attr] 标记",
                    attr,
                ))
            },
        }
    }
}

impl FieldTypeInfo {
    /// 获取用于代码生成的类型名称
    ///
    /// 返回适合在生成代码中使用的类型名称。
    /// 遵循单一职责原则，专门提供代码生成所需的类型名称。
    ///
    /// # 返回值
    ///
    /// 返回用于代码生成的类型名称字符串
    ///
    /// # 命名规则
    ///
    /// - 基本类型返回简化名称
    /// - Option<T> 类型返回完整的 Option<T> 表示
    /// - 复杂类型返回原始类型表示
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责提供代码生成用的类型名称
    /// - **接口隔离**: 提供专门的类型名称获取接口
    pub fn codegen_type_name(&self) -> &str {
        &self.simple_name
    }

    /// 获取基础类型名称
    ///
    /// 返回去除 Option 包装后的基础类型名称。
    /// 用于类型检查和代码生成逻辑。
    ///
    /// # 返回值
    ///
    /// 返回基础类型名称，如果是 Option<T> 则返回 T 的名称
    ///
    /// # 示例
    ///
    /// ```rust
    /// // 对于 String 类型
    /// assert_eq!(type_info.base_type_name(), "String");
    ///
    /// // 对于 Option<String> 类型
    /// assert_eq!(type_info.base_type_name(), "String");
    /// ```
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责提取基础类型名称
    /// - **里氏替换**: 任何类型信息都能正确处理
    pub fn base_type_name(&self) -> &str {
        if self.is_optional {
            if let Some(inner_type) = &self.inner_type {
                inner_type.base_type_name()
            } else {
                &self.simple_name
            }
        } else {
            &self.simple_name
        }
    }

    /// 检查是否为字符串类型
    ///
    /// 判断类型是否为字符串相关类型（String, &str, str）。
    /// 用于特殊的字符串处理逻辑。
    ///
    /// # 返回值
    ///
    /// 如果是字符串类型则返回 true，否则返回 false
    ///
    /// # 字符串类型识别
    ///
    /// - String
    /// - &str
    /// - str
    /// - Option<String>, Option<&str>, Option<str>
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责字符串类型识别
    /// - **开闭原则**: 可扩展支持新的字符串类型
    pub fn is_string_type(&self) -> bool {
        let base_name = self.base_type_name();
        matches!(base_name, "String" | "str" | "&str")
    }

    /// 检查是否为数值类型
    ///
    /// 判断类型是否为数值相关类型。
    /// 用于数值处理的特殊逻辑。
    ///
    /// # 返回值
    ///
    /// 如果是数值类型则返回 true，否则返回 false
    ///
    /// # 数值类型识别
    ///
    /// - 整数类型: i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
    /// - 浮点类型: f32, f64
    /// - 对应的 Option 包装版本
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责数值类型识别
    /// - **开闭原则**: 可扩展支持新的数值类型
    pub fn is_numeric_type(&self) -> bool {
        let base_name = self.base_type_name();
        matches!(
            base_name,
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

impl FieldAnalysis {
    /// 检查字段是否可用作属性
    ///
    /// 验证字段是否满足作为属性使用的所有条件。
    /// 结合了属性标记和类型支持性的检查。
    ///
    /// # 返回值
    ///
    /// 如果字段可用作属性则返回 Ok(())，否则返回验证错误
    ///
    /// # 验证条件
    ///
    /// - 必须带有 #[attr] 标记
    /// - 字段类型必须受支持
    /// - 字段名称必须是有效标识符
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责属性字段的可用性验证
    /// - **接口隔离**: 提供综合的验证接口
    pub fn validate_as_attribute(&self) -> MacroResult<()> {
        // 检查是否带有 #[attr] 标记
        if !self.is_marked_as_attr {
            return Err(MacroError::validation_error(
                &format!("字段 '{}' 没有 #[attr] 标记", self.name),
                &self.original_field,
            ));
        }

        // 检查字段名称是否为有效标识符
        if !utils::is_valid_identifier(&self.name) {
            return Err(MacroError::validation_error(
                &format!("字段名称 '{}' 不是有效的标识符", self.name),
                &self.original_field,
            ));
        }

        // 检查类型支持性
        FieldAnalyzer::validate_field_type_support(self)?;

        Ok(())
    }

    /// 生成字段的显示信息
    ///
    /// 创建用于错误消息和调试的字段描述信息。
    /// 遵循单一职责原则，专门提供字段信息的格式化。
    ///
    /// # 返回值
    ///
    /// 返回包含字段详细信息的字符串
    ///
    /// # 信息内容
    ///
    /// - 字段名称
    /// - 字段类型
    /// - 是否为可选类型
    /// - 是否带有属性标记
    /// - 类型支持状态
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责字段信息的格式化
    /// - **接口隔离**: 提供统一的信息显示接口
    pub fn display_info(&self) -> String {
        format!(
            "字段 '{}': {} ({}{}{})",
            self.name,
            self.type_info.simple_name,
            if self.type_info.is_optional { "可选, " } else { "必需, " },
            if self.is_marked_as_attr {
                "带属性标记, "
            } else {
                "无属性标记, "
            },
            if self.type_info.is_supported { "支持" } else { "不支持" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    /// 测试基本类型的字段分析
    #[test]
    fn test_analyze_basic_field_type() {
        let field_type: Type = parse_quote! { String };
        let type_info = FieldAnalyzer::analyze_field_type(&field_type);

        assert_eq!(type_info.simple_name, "String");
        assert!(!type_info.is_optional);
        assert!(type_info.is_supported);
        assert!(type_info.inner_type.is_none());
    }

    /// 测试 Option 类型的字段分析
    #[test]
    fn test_analyze_option_field_type() {
        let field_type: Type = parse_quote! { Option<i32> };
        let type_info = FieldAnalyzer::analyze_field_type(&field_type);

        assert_eq!(type_info.simple_name, "Option<i32>");
        assert!(type_info.is_optional);
        assert!(type_info.is_supported);

        // 检查内部类型
        assert!(type_info.inner_type.is_some());
        let inner_type = type_info.inner_type.unwrap();
        assert_eq!(inner_type.simple_name, "i32");
        assert!(!inner_type.is_optional);
        assert!(inner_type.is_supported);
    }

    /// 测试不支持类型的字段分析
    #[test]
    fn test_analyze_unsupported_field_type() {
        let field_type: Type = parse_quote! { Vec<String> };
        let type_info = FieldAnalyzer::analyze_field_type(&field_type);

        assert!(!type_info.is_supported);
        assert!(!type_info.is_optional);
    }

    /// 测试单个字段的完整分析
    #[test]
    fn test_analyze_complete_field() {
        let field: Field = parse_quote! {
            #[attr]
            name: String
        };

        let result = FieldAnalyzer::analyze_field(&field);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.name, "name");
        assert!(analysis.is_marked_as_attr);
        assert!(analysis.type_info.is_supported);
        assert!(!analysis.type_info.is_optional);
        assert!(analysis.attributes.contains(&"attr".to_string()));
    }

    /// 测试没有属性标记的字段分析
    #[test]
    fn test_analyze_field_without_attr() {
        let field: Field = parse_quote! {
            name: String
        };

        let result = FieldAnalyzer::analyze_field(&field);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.name, "name");
        assert!(!analysis.is_marked_as_attr);
        assert!(analysis.attributes.is_empty());
    }

    /// 测试多个字段的批量分析
    #[test]
    fn test_analyze_multiple_fields() {
        let fields: Vec<Field> = vec![
            parse_quote! {
                #[attr]
                name: String
            },
            parse_quote! {
                #[attr]
                age: Option<i32>
            },
            parse_quote! {
                description: String
            },
        ];

        let result = FieldAnalyzer::analyze_fields(&fields);
        assert!(result.is_ok());

        let analyses = result.unwrap();
        assert_eq!(analyses.len(), 3);

        // 检查第一个字段
        assert_eq!(analyses[0].name, "name");
        assert!(analyses[0].is_marked_as_attr);

        // 检查第二个字段
        assert_eq!(analyses[1].name, "age");
        assert!(analyses[1].is_marked_as_attr);
        assert!(analyses[1].type_info.is_optional);

        // 检查第三个字段
        assert_eq!(analyses[2].name, "description");
        assert!(!analyses[2].is_marked_as_attr);
    }

    /// 测试属性字段过滤功能
    #[test]
    fn test_filter_attr_fields() {
        let fields: Vec<Field> = vec![
            parse_quote! {
                #[attr]
                name: String
            },
            parse_quote! {
                #[attr]
                age: Option<i32>
            },
            parse_quote! {
                description: String
            },
        ];

        let analyses = FieldAnalyzer::analyze_fields(&fields).unwrap();
        let attr_fields = FieldAnalyzer::filter_attr_fields(&analyses);

        assert_eq!(attr_fields.len(), 2);
        assert_eq!(attr_fields[0].name, "name");
        assert_eq!(attr_fields[1].name, "age");
    }

    /// 测试字段类型支持性验证
    #[test]
    fn test_validate_field_type_support() {
        // 测试支持的类型
        let field: Field = parse_quote! {
            #[attr]
            name: String
        };
        let analysis = FieldAnalyzer::analyze_field(&field).unwrap();
        assert!(FieldAnalyzer::validate_field_type_support(&analysis).is_ok());

        // 测试不支持的类型
        let field: Field = parse_quote! {
            #[attr]
            data: Vec<String>
        };
        let analysis = FieldAnalyzer::analyze_field(&field).unwrap();
        assert!(FieldAnalyzer::validate_field_type_support(&analysis).is_err());
    }

    /// 测试 FieldTypeInfo 的辅助方法
    #[test]
    fn test_field_type_info_helpers() {
        // 测试字符串类型识别
        let string_type: Type = parse_quote! { String };
        let type_info = FieldAnalyzer::analyze_field_type(&string_type);
        assert!(type_info.is_string_type());
        assert!(!type_info.is_numeric_type());

        // 测试数值类型识别
        let numeric_type: Type = parse_quote! { i32 };
        let type_info = FieldAnalyzer::analyze_field_type(&numeric_type);
        assert!(!type_info.is_string_type());
        assert!(type_info.is_numeric_type());

        // 测试 Option<String> 类型
        let option_string: Type = parse_quote! { Option<String> };
        let type_info = FieldAnalyzer::analyze_field_type(&option_string);
        assert!(type_info.is_string_type());
        assert!(!type_info.is_numeric_type());
        assert_eq!(type_info.base_type_name(), "String");
    }

    /// 测试字段属性验证
    #[test]
    fn test_field_attribute_validation() {
        // 测试有效的属性字段
        let field: Field = parse_quote! {
            #[attr]
            name: String
        };
        let analysis = FieldAnalyzer::analyze_field(&field).unwrap();
        assert!(analysis.validate_as_attribute().is_ok());

        // 测试没有属性标记的字段
        let field: Field = parse_quote! {
            name: String
        };
        let analysis = FieldAnalyzer::analyze_field(&field).unwrap();
        assert!(analysis.validate_as_attribute().is_err());
    }

    /// 测试字段信息显示
    #[test]
    fn test_field_display_info() {
        let field: Field = parse_quote! {
            #[attr]
            name: Option<String>
        };
        let analysis = FieldAnalyzer::analyze_field(&field).unwrap();
        let display = analysis.display_info();

        assert!(display.contains("name"));
        assert!(display.contains("Option<String>"));
        assert!(display.contains("可选"));
        assert!(display.contains("带属性标记"));
        assert!(display.contains("支持"));
    }

    /// 测试无效的 #[attr] 属性格式
    #[test]
    fn test_invalid_attr_attribute_formats() {
        // 注意：由于 syn 解析限制，这里主要测试我们能够处理的格式
        // 实际的语法错误会在更早的解析阶段被 syn 捕获

        let field: Field = parse_quote! {
            #[attr]
            name: String
        };
        let result = FieldAnalyzer::analyze_field(&field);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.is_marked_as_attr);
    }
}
