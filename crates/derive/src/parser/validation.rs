//! 验证逻辑模块
//!
//! 提供属性配置和字段信息的验证功能。
//! 遵循单一职责原则，专门负责配置有效性的验证逻辑。

use syn::spanned::Spanned;
use crate::common::{MacroError, MacroResult, utils, constants::validation as limits};
use crate::parser::attribute_parser::{NodeConfig, MarkConfig, FieldConfig};
use crate::parser::field_analyzer::FieldAnalysis;

/// 验证器
///
/// 提供全面的配置验证功能，确保生成的代码正确无误。
/// 遵循单一职责原则，专门负责验证逻辑而不涉及解析或生成。
pub struct Validator;

impl Validator {
    /// 验证 Node 配置
    ///
    /// 对 Node 配置进行全面验证，确保所有属性和字段配置正确。
    /// 遵循里氏替换原则，任何 NodeConfig 都能正确验证。
    ///
    /// # 参数
    ///
    /// * `config` - 要验证的 Node 配置
    ///
    /// # 返回值
    ///
    /// 配置有效时返回 Ok(())，否则返回第一个验证错误
    ///
    /// # 验证内容
    ///
    /// - 必需属性的存在性检查
    /// - 属性值的格式和有效性验证
    /// - 字段类型的支持性验证
    /// - 标识符的合法性检查
    /// - 配置的一致性验证
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责 Node 配置验证
    /// - **里氏替换**: 任何 NodeConfig 都能正确处理
    /// - **接口隔离**: 提供专门的 Node 验证接口
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::parser::{attribute_parser::NodeConfig, validation::Validator};
    ///
    /// let mut config = NodeConfig::default();
    /// config.node_type = Some("paragraph".to_string());
    ///
    /// let result = Validator::validate_node_config(&config);
    /// assert!(result.is_ok());
    /// ```
    pub fn validate_node_config(config: &NodeConfig) -> MacroResult<()> {
        // 1. 验证必需属性
        Self::validate_required_node_attributes(config)?;
        
        // 2. 验证 node_type
        Self::validate_node_type(config)?;
        
        // 3. 验证 marks 配置
        Self::validate_marks_config(config)?;
        
        // 4. 验证 content 配置
        Self::validate_content_config(config)?;
        
        // 5. 验证字段配置
        Self::validate_node_field_configs(config)?;
        
        // 6. 验证配置的一致性
        Self::validate_node_config_consistency(config)?;
        
        Ok(())
    }
    
    /// 验证 Mark 配置
    ///
    /// 对 Mark 配置进行全面验证，确保标记定义的正确性。
    /// 遵循里氏替换原则，任何 MarkConfig 都能正确验证。
    ///
    /// # 参数
    ///
    /// * `config` - 要验证的 Mark 配置
    ///
    /// # 返回值
    ///
    /// 配置有效时返回 Ok(())，否则返回第一个验证错误
    ///
    /// # 验证内容
    ///
    /// - mark_type 的存在性和格式验证
    /// - 字段类型的支持性验证
    /// - 标识符的合法性检查
    /// - 配置的完整性验证
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责 Mark 配置验证
    /// - **接口隔离**: 提供专门的 Mark 验证接口
    /// - **里氏替换**: 与 Node 配置验证方法可互换使用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::parser::{attribute_parser::MarkConfig, validation::Validator};
    ///
    /// let mut config = MarkConfig::default();
    /// config.mark_type = Some("bold".to_string());
    ///
    /// let result = Validator::validate_mark_config(&config);
    /// assert!(result.is_ok());
    /// ```
    pub fn validate_mark_config(config: &MarkConfig) -> MacroResult<()> {
        // 1. 验证必需属性
        Self::validate_required_mark_attributes(config)?;
        
        // 2. 验证 mark_type
        Self::validate_mark_type(config)?;
        
        // 3. 验证字段配置
        Self::validate_mark_field_configs(config)?;
        
        // 4. 验证配置的一致性
        Self::validate_mark_config_consistency(config)?;
        
        Ok(())
    }
    
    /// 验证字段分析结果
    ///
    /// 对字段分析结果进行验证，确保字段能够正确用作属性。
    /// 遵循开闭原则，可以扩展支持新的字段验证规则。
    ///
    /// # 参数
    ///
    /// * `analyses` - 字段分析结果列表
    ///
    /// # 返回值
    ///
    /// 所有字段都有效时返回 Ok(())，否则返回第一个验证错误
    ///
    /// # 验证内容
    ///
    /// - 字段类型的支持性验证
    /// - 字段名称的合法性检查
    /// - 属性标记的正确性验证
    /// - 字段配置的完整性检查
    ///
    /// # 设计原则体现
    ///
    /// - **开闭原则**: 可扩展新的字段验证规则
    /// - **单一职责**: 只负责字段分析结果验证
    pub fn validate_field_analyses(analyses: &[FieldAnalysis]) -> MacroResult<()> {
        for analysis in analyses {
            // 验证字段名称
            Self::validate_field_name(&analysis.name)?;
            
            // 验证字段类型支持性
            if analysis.is_marked_as_attr {
                Self::validate_field_type_support(analysis)?;
            }
            
            // 验证字段配置的完整性
            Self::validate_field_config_completeness(analysis)?;
        }
        
        Ok(())
    }
    
    /// 验证 Node 的必需属性
    ///
    /// 检查 Node 配置是否包含所有必需的属性。
    /// 遵循单一职责原则，专门验证必需属性的存在性。
    ///
    /// # 参数
    ///
    /// * `config` - Node 配置
    ///
    /// # 返回值
    ///
    /// 所有必需属性都存在时返回 Ok(())，否则返回缺少属性错误
    fn validate_required_node_attributes(config: &NodeConfig) -> MacroResult<()> {
        if config.node_type.is_none() {
            return Err(MacroError::ValidationError {
                message: "缺少必需的 node_type 属性".to_string(),
                span: None,
            });
        }
        
        Ok(())
    }
    
    /// 验证 Mark 的必需属性
    ///
    /// 检查 Mark 配置是否包含所有必需的属性。
    /// 遵循单一职责原则，专门验证必需属性的存在性。
    ///
    /// # 参数
    ///
    /// * `config` - Mark 配置
    ///
    /// # 返回值
    ///
    /// 所有必需属性都存在时返回 Ok(())，否则返回缺少属性错误
    fn validate_required_mark_attributes(config: &MarkConfig) -> MacroResult<()> {
        if config.mark_type.is_none() {
            return Err(MacroError::ValidationError {
                message: "缺少必需的 mark_type 属性".to_string(),
                span: None,
            });
        }
        
        Ok(())
    }
    
    /// 验证 node_type 属性
    ///
    /// 验证 node_type 的格式和有效性。
    /// 遵循单一职责原则，专门验证节点类型属性。
    ///
    /// # 参数
    ///
    /// * `config` - Node 配置
    ///
    /// # 返回值
    ///
    /// node_type 有效时返回 Ok(())，否则返回验证错误
    ///
    /// # 验证规则
    ///
    /// - 必须是非空字符串
    /// - 必须是有效的标识符格式
    /// - 长度在限制范围内
    /// - 不包含保留字符
    fn validate_node_type(config: &NodeConfig) -> MacroResult<()> {
        let node_type = config.node_type.as_ref().unwrap(); // 已在前面验证存在性
        
        // 验证长度
        if node_type.len() < limits::MIN_IDENTIFIER_LENGTH {
            return Err(MacroError::ValidationError {
                message: format!(
                    "node_type '{}' 太短，最少需要 {} 个字符",
                    node_type, limits::MIN_IDENTIFIER_LENGTH
                ),
                span: None,
            });
        }
        
        if node_type.len() > limits::MAX_IDENTIFIER_LENGTH {
            return Err(MacroError::ValidationError {
                message: format!(
                    "node_type '{}' 太长，最多允许 {} 个字符",
                    node_type, limits::MAX_IDENTIFIER_LENGTH
                ),
                span: None,
            });
        }
        
        // 验证标识符格式
        if !utils::is_valid_identifier(node_type) {
            return Err(MacroError::ValidationError {
                message: format!(
                    "node_type '{}' 不是有效的标识符格式",
                    node_type
                ),
                span: None,
            });
        }
        
        Ok(())
    }
    
    /// 验证 mark_type 属性
    ///
    /// 验证 mark_type 的格式和有效性。
    /// 遵循单一职责原则，专门验证标记类型属性。
    ///
    /// # 参数
    ///
    /// * `config` - Mark 配置
    ///
    /// # 返回值
    ///
    /// mark_type 有效时返回 Ok(())，否则返回验证错误
    ///
    /// # 验证规则
    ///
    /// - 必须是非空字符串
    /// - 必须是有效的标识符格式
    /// - 长度在限制范围内
    /// - 不包含保留字符
    fn validate_mark_type(config: &MarkConfig) -> MacroResult<()> {
        let mark_type = config.mark_type.as_ref().unwrap(); // 已在前面验证存在性
        
        // 验证长度
        if mark_type.len() < limits::MIN_IDENTIFIER_LENGTH {
            return Err(MacroError::ValidationError {
                message: format!(
                    "mark_type '{}' 太短，最少需要 {} 个字符",
                    mark_type, limits::MIN_IDENTIFIER_LENGTH
                ),
                span: None,
            });
        }
        
        if mark_type.len() > limits::MAX_IDENTIFIER_LENGTH {
            return Err(MacroError::ValidationError {
                message: format!(
                    "mark_type '{}' 太长，最多允许 {} 个字符",
                    mark_type, limits::MAX_IDENTIFIER_LENGTH
                ),
                span: None,
            });
        }
        
        // 验证标识符格式
        if !utils::is_valid_identifier(mark_type) {
            return Err(MacroError::ValidationError {
                message: format!(
                    "mark_type '{}' 不是有效的标识符格式",
                    mark_type
                ),
                span: None,
            });
        }
        
        Ok(())
    }
    
    /// 验证 marks 配置
    ///
    /// 验证 marks 列表的格式和有效性。
    /// 遵循单一职责原则，专门验证标记列表配置。
    ///
    /// # 参数
    ///
    /// * `config` - Node 配置
    ///
    /// # 返回值
    ///
    /// marks 配置有效时返回 Ok(())，否则返回验证错误
    ///
    /// # 验证规则
    ///
    /// - 如果存在，不能为空列表
    /// - 每个标记名称必须是有效标识符
    /// - 标记数量在限制范围内
    /// - 不能有重复的标记名称
    fn validate_marks_config(config: &NodeConfig) -> MacroResult<()> {
        if let Some(marks) = &config.marks {
            // 验证不为空
            if marks.trim().is_empty() {
                return Err(MacroError::ValidationError {
                    message: "marks 列表不能为空，如果不需要标记请移除 marks 属性".to_string(),
                    span: None,
                });
            }
            
            // 分割成数组用于验证
            let mark_list: Vec<&str> = marks.split_whitespace().collect();
            
            // 验证数量限制
            if mark_list.len() > limits::MAX_MARKS_COUNT {
                return Err(MacroError::ValidationError {
                    message: format!(
                        "marks 列表太长，最多允许 {} 个标记，当前有 {} 个",
                        limits::MAX_MARKS_COUNT, mark_list.len()
                    ),
                    span: None,
                });
            }
            
            // 验证每个标记名称
            for (index, mark) in mark_list.iter().enumerate() {
                // 验证标识符格式
                if !utils::is_valid_identifier(mark) {
                    return Err(MacroError::ValidationError {
                        message: format!(
                            "marks 列表中第 {} 个标记 '{}' 不是有效的标识符格式",
                            index + 1, mark
                        ),
                        span: None,
                    });
                }
                
                // 验证长度
                if mark.len() > limits::MAX_IDENTIFIER_LENGTH {
                    return Err(MacroError::ValidationError {
                        message: format!(
                            "marks 列表中的标记 '{}' 太长，最多允许 {} 个字符",
                            mark, limits::MAX_IDENTIFIER_LENGTH
                        ),
                        span: None,
                    });
                }
            }
            
            // 验证没有重复
            let mut unique_marks = std::collections::HashSet::new();
            for mark in mark_list {
                if !unique_marks.insert(mark) {
                    return Err(MacroError::ValidationError {
                        message: format!("marks 列表中存在重复的标记: '{}'", mark),
                        span: None,
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// 验证 content 配置
    ///
    /// 验证 content 表达式的格式和有效性。
    /// 遵循单一职责原则，专门验证内容约束表达式。
    ///
    /// # 参数
    ///
    /// * `config` - Node 配置
    ///
    /// # 返回值
    ///
    /// content 配置有效时返回 Ok(())，否则返回验证错误
    ///
    /// # 验证规则
    ///
    /// - 如果存在，不能为空字符串
    /// - 长度在合理范围内
    /// - 格式符合内容表达式语法（基本验证）
    fn validate_content_config(config: &NodeConfig) -> MacroResult<()> {
        if let Some(content) = &config.content {
            // 验证不为空
            if content.trim().is_empty() {
                return Err(MacroError::ValidationError {
                    message: "content 表达式不能为空，如果不需要内容约束请移除 content 属性".to_string(),
                    span: None,
                });
            }
            
            // 验证长度
            if content.len() > limits::MAX_ATTRIBUTE_VALUE_LENGTH {
                return Err(MacroError::ValidationError {
                    message: format!(
                        "content 表达式太长，最多允许 {} 个字符，当前有 {} 个",
                        limits::MAX_ATTRIBUTE_VALUE_LENGTH, content.len()
                    ),
                    span: None,
                });
            }
            
            // 基本的格式验证
            Self::validate_content_expression_syntax(content)?;
        }
        
        Ok(())
    }
    
    /// 验证内容表达式语法
    ///
    /// 对内容约束表达式进行基本的语法验证。
    /// 遵循单一职责原则，专门验证表达式语法。
    ///
    /// # 参数
    ///
    /// * `expression` - 内容表达式字符串
    ///
    /// # 返回值
    ///
    /// 语法有效时返回 Ok(())，否则返回语法错误
    ///
    /// # 验证规则
    ///
    /// - 基本的字符合法性检查
    /// - 括号匹配验证
    /// - 保留字符的使用检查
    fn validate_content_expression_syntax(expression: &str) -> MacroResult<()> {
        // 检查是否包含不允许的字符
        let invalid_chars = ['<', '>', '"', '\'', '\\'];
        for ch in invalid_chars.iter() {
            if expression.contains(*ch) {
                return Err(MacroError::ValidationError {
                    message: format!(
                        "content 表达式包含不允许的字符 '{}'",
                        ch
                    ),
                    span: None,
                });
            }
        }
        
        // 基本的括号匹配检查
        let mut paren_count = 0;
        let mut bracket_count = 0;
        let mut brace_count = 0;
        
        for ch in expression.chars() {
            match ch {
                '(' => paren_count += 1,
                ')' => {
                    paren_count -= 1;
                    if paren_count < 0 {
                        return Err(MacroError::ValidationError {
                            message: "content 表达式中的括号不匹配".to_string(),
                            span: None,
                        });
                    }
                }
                '[' => bracket_count += 1,
                ']' => {
                    bracket_count -= 1;
                    if bracket_count < 0 {
                        return Err(MacroError::ValidationError {
                            message: "content 表达式中的方括号不匹配".to_string(),
                            span: None,
                        });
                    }
                }
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count < 0 {
                        return Err(MacroError::ValidationError {
                            message: "content 表达式中的花括号不匹配".to_string(),
                            span: None,
                        });
                    }
                }
                _ => {}
            }
        }
        
        // 检查所有括号都已闭合
        if paren_count != 0 || bracket_count != 0 || brace_count != 0 {
            return Err(MacroError::ValidationError {
                message: "content 表达式中存在未闭合的括号".to_string(),
                span: None,
            });
        }
        
        Ok(())
    }
    
    /// 验证 Node 字段配置
    ///
    /// 验证 Node 的所有字段配置。
    /// 遵循单一职责原则，专门验证字段配置的有效性。
    ///
    /// # 参数
    ///
    /// * `config` - Node 配置
    ///
    /// # 返回值
    ///
    /// 所有字段配置都有效时返回 Ok(())，否则返回第一个验证错误
    fn validate_node_field_configs(config: &NodeConfig) -> MacroResult<()> {
        // 验证属性字段数量
        if config.attr_fields.len() > limits::MAX_FIELD_ATTRIBUTES {
            return Err(MacroError::ValidationError {
                message: format!(
                    "属性字段太多，最多允许 {} 个，当前有 {} 个",
                    limits::MAX_FIELD_ATTRIBUTES, config.attr_fields.len()
                ),
                span: None,
            });
        }
        
        // 验证每个字段配置
        for field_config in &config.attr_fields {
            Self::validate_field_config(field_config)?;
        }
        
        // 验证字段名称无重复
        Self::validate_no_duplicate_field_names(&config.attr_fields)?;
        
        Ok(())
    }
    
    /// 验证 Mark 字段配置
    ///
    /// 验证 Mark 的所有字段配置。
    /// 遵循单一职责原则，专门验证字段配置的有效性。
    ///
    /// # 参数
    ///
    /// * `config` - Mark 配置
    ///
    /// # 返回值
    ///
    /// 所有字段配置都有效时返回 Ok(())，否则返回第一个验证错误
    fn validate_mark_field_configs(config: &MarkConfig) -> MacroResult<()> {
        // 验证属性字段数量
        if config.attr_fields.len() > limits::MAX_FIELD_ATTRIBUTES {
            return Err(MacroError::ValidationError {
                message: format!(
                    "属性字段太多，最多允许 {} 个，当前有 {} 个",
                    limits::MAX_FIELD_ATTRIBUTES, config.attr_fields.len()
                ),
                span: None,
            });
        }
        
        // 验证每个字段配置
        for field_config in &config.attr_fields {
            Self::validate_field_config(field_config)?;
        }
        
        // 验证字段名称无重复
        Self::validate_no_duplicate_field_names(&config.attr_fields)?;
        
        Ok(())
    }
    
    /// 验证单个字段配置
    ///
    /// 对单个字段配置进行详细验证。
    /// 遵循单一职责原则，专门验证字段配置的各项属性。
    ///
    /// # 参数
    ///
    /// * `field_config` - 字段配置
    ///
    /// # 返回值
    ///
    /// 字段配置有效时返回 Ok(())，否则返回验证错误
    fn validate_field_config(field_config: &FieldConfig) -> MacroResult<()> {
        // 验证字段名称
        Self::validate_field_name(&field_config.name)?;
        
        // 验证字段类型（基本检查）
        if field_config.type_name.trim().is_empty() {
            return Err(MacroError::ValidationError {
                message: format!("字段 '{}' 的类型名称为空", field_config.name),
                span: None,
            });
        }
        
        // 验证属性标记的一致性
        if field_config.is_attr {
            // 带有 #[attr] 标记的字段必须是支持的类型
            if !utils::is_supported_type(&field_config.field.ty) {
                return Err(MacroError::UnsupportedFieldType {
                    field_name: field_config.name.clone(),
                    field_type: field_config.type_name.clone(),
                    span: Some(field_config.field.ty.span()),
                });
            }
        }
        
        Ok(())
    }
    
    /// 验证字段名称
    ///
    /// 验证字段名称的格式和有效性。
    /// 遵循单一职责原则，专门验证字段名称。
    ///
    /// # 参数
    ///
    /// * `field_name` - 字段名称
    ///
    /// # 返回值
    ///
    /// 字段名称有效时返回 Ok(())，否则返回验证错误
    fn validate_field_name(field_name: &str) -> MacroResult<()> {
        // 验证不为空
        if field_name.trim().is_empty() {
            return Err(MacroError::ValidationError {
                message: "字段名称不能为空".to_string(),
                span: None,
            });
        }
        
        // 验证标识符格式
        if !utils::is_valid_identifier(field_name) {
            return Err(MacroError::ValidationError {
                message: format!("字段名称 '{}' 不是有效的标识符格式", field_name),
                span: None,
            });
        }
        
        // 验证长度
        if field_name.len() > limits::MAX_IDENTIFIER_LENGTH {
            return Err(MacroError::ValidationError {
                message: format!(
                    "字段名称 '{}' 太长，最多允许 {} 个字符",
                    field_name, limits::MAX_IDENTIFIER_LENGTH
                ),
                span: None,
            });
        }
        
        Ok(())
    }
    
    /// 验证字段类型支持性
    ///
    /// 验证字段的类型是否被宏系统支持。
    /// 遵循单一职责原则，专门验证类型支持性。
    ///
    /// # 参数
    ///
    /// * `analysis` - 字段分析结果
    ///
    /// # 返回值
    ///
    /// 字段类型受支持时返回 Ok(())，否则返回支持性错误
    fn validate_field_type_support(analysis: &FieldAnalysis) -> MacroResult<()> {
        if !analysis.type_info.is_supported {
            return Err(MacroError::UnsupportedFieldType {
                field_name: analysis.name.clone(),
                field_type: analysis.type_info.simple_name.clone(),
                span: Some(analysis.original_field.ty.span()),
            });
        }
        
        Ok(())
    }
    
    /// 验证字段配置的完整性
    ///
    /// 验证字段分析结果的完整性和一致性。
    /// 遵循单一职责原则，专门验证配置完整性。
    ///
    /// # 参数
    ///
    /// * `analysis` - 字段分析结果
    ///
    /// # 返回值
    ///
    /// 配置完整时返回 Ok(())，否则返回完整性错误
    fn validate_field_config_completeness(analysis: &FieldAnalysis) -> MacroResult<()> {
        // 验证字段名称不为空
        if analysis.name.trim().is_empty() {
            return Err(MacroError::ValidationError {
                message: "字段名称不能为空".to_string(),
                span: Some(analysis.original_field.span()),
            });
        }
        
        // 验证类型信息的一致性
        if analysis.type_info.simple_name.trim().is_empty() {
            return Err(MacroError::ValidationError {
                message: format!("字段 '{}' 的类型名称为空", analysis.name),
                span: Some(analysis.original_field.ty.span()),
            });
        }
        
        Ok(())
    }
    
    /// 验证字段名称无重复
    ///
    /// 检查字段列表中是否存在重复的字段名称。
    /// 遵循单一职责原则，专门检查名称重复性。
    ///
    /// # 参数
    ///
    /// * `field_configs` - 字段配置列表
    ///
    /// # 返回值
    ///
    /// 无重复名称时返回 Ok(())，否则返回重复错误
    fn validate_no_duplicate_field_names(field_configs: &[FieldConfig]) -> MacroResult<()> {
        let mut seen_names = std::collections::HashSet::new();
        
        for field_config in field_configs {
            if !seen_names.insert(&field_config.name) {
                return Err(MacroError::ValidationError {
                    message: format!("存在重复的字段名称: '{}'", field_config.name),
                    span: Some(field_config.field.span()),
                });
            }
        }
        
        Ok(())
    }
    
    /// 验证 Node 配置的一致性
    ///
    /// 检查 Node 配置各部分之间的一致性。
    /// 遵循单一职责原则，专门验证配置一致性。
    ///
    /// # 参数
    ///
    /// * `config` - Node 配置
    ///
    /// # 返回值
    ///
    /// 配置一致时返回 Ok(())，否则返回一致性错误
    fn validate_node_config_consistency(_config: &NodeConfig) -> MacroResult<()> {
        // 这里可以添加配置一致性检查
        // 例如：验证 marks 和字段类型的兼容性
        // 暂时返回 Ok，为将来的一致性检查预留接口
        Ok(())
    }
    
    /// 验证 Mark 配置的一致性
    ///
    /// 检查 Mark 配置各部分之间的一致性。
    /// 遵循单一职责原则，专门验证配置一致性。
    ///
    /// # 参数
    ///
    /// * `config` - Mark 配置
    ///
    /// # 返回值
    ///
    /// 配置一致时返回 Ok(())，否则返回一致性错误
    fn validate_mark_config_consistency(_config: &MarkConfig) -> MacroResult<()> {
        // 这里可以添加配置一致性检查
        // 例如：验证标记类型与字段的兼容性
        // 暂时返回 Ok，为将来的一致性检查预留接口
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;
    use crate::parser::attribute_parser::AttributeParser;
    use crate::parser::field_analyzer::FieldAnalyzer;

    /// 测试有效的 Node 配置验证
    #[test]
    fn test_valid_node_config_validation() {
        let mut config = NodeConfig::default();
        config.node_type = Some("paragraph".to_string());
        
        let result = Validator::validate_node_config(&config);
        assert!(result.is_ok());
    }
    
    /// 测试缺少必需属性的 Node 配置
    #[test]
    fn test_missing_required_node_attribute() {
        let config = NodeConfig::default(); // 缺少 node_type
        
        let result = Validator::validate_node_config(&config);
        assert!(result.is_err());
        
        if let Err(MacroError::ValidationError { message, .. }) = result {
            assert!(message.contains("node_type"));
        } else {
            panic!("期望 ValidationError");
        }
    }
    
    /// 测试有效的 Mark 配置验证
    #[test]
    fn test_valid_mark_config_validation() {
        let mut config = MarkConfig::default();
        config.mark_type = Some("bold".to_string());
        
        let result = Validator::validate_mark_config(&config);
        assert!(result.is_ok());
    }
    
    /// 测试缺少必需属性的 Mark 配置
    #[test]
    fn test_missing_required_mark_attribute() {
        let config = MarkConfig::default(); // 缺少 mark_type
        
        let result = Validator::validate_mark_config(&config);
        assert!(result.is_err());
        
        if let Err(MacroError::ValidationError { message, .. }) = result {
            assert!(message.contains("mark_type"));
        } else {
            panic!("期望 ValidationError");
        }
    }
    
    /// 测试无效的标识符验证
    #[test]
    fn test_invalid_identifier_validation() {
        let mut config = NodeConfig::default();
        config.node_type = Some("invalid-identifier".to_string()); // 包含连字符
        
        let result = Validator::validate_node_config(&config);
        assert!(result.is_err());
    }
    
    /// 测试 marks 列表验证
    #[test]
    fn test_marks_list_validation() {
        // 测试有效的 marks 列表
        let mut config = NodeConfig::default();
        config.node_type = Some("paragraph".to_string());
        config.marks = Some("bold italic".to_string());
        
        let result = Validator::validate_node_config(&config);
        assert!(result.is_ok());
        
        // 测试空的 marks 列表
        config.marks = Some("".to_string());
        let result = Validator::validate_node_config(&config);
        assert!(result.is_err());
        
        // 测试重复的 marks
        config.marks = Some("bold bold".to_string());
        let result = Validator::validate_node_config(&config);
        assert!(result.is_err());
    }
    
    /// 测试 content 表达式验证
    #[test]
    fn test_content_expression_validation() {
        let mut config = NodeConfig::default();
        config.node_type = Some("paragraph".to_string());
        
        // 测试有效的 content 表达式
        config.content = Some("text*".to_string());
        let result = Validator::validate_node_config(&config);
        assert!(result.is_ok());
        
        // 测试空的 content 表达式
        config.content = Some("".to_string());
        let result = Validator::validate_node_config(&config);
        assert!(result.is_err());
        
        // 测试包含无效字符的 content 表达式
        config.content = Some("text<invalid>".to_string());
        let result = Validator::validate_node_config(&config);
        assert!(result.is_err());
    }
    
    /// 测试字段分析结果验证
    #[test]
    fn test_field_analyses_validation() {
        let fields: Vec<syn::Field> = vec![
            parse_quote! {
                #[attr]
                name: String
            },
            parse_quote! {
                #[attr]
                age: Option<i32>
            },
        ];
        
        let analyses = FieldAnalyzer::analyze_fields(&fields).unwrap();
        let result = Validator::validate_field_analyses(&analyses);
        assert!(result.is_ok());
    }
    
    /// 测试不支持字段类型的验证
    #[test]
    fn test_unsupported_field_type_validation() {
        let field: syn::Field = parse_quote! {
            #[attr]
            data: Vec<String>
        };
        
        let analysis = FieldAnalyzer::analyze_field(&field).unwrap();
        let result = Validator::validate_field_analyses(&[analysis]);
        assert!(result.is_err());
        
        if let Err(MacroError::UnsupportedFieldType { .. }) = result {
            // 正确的错误类型
        } else {
            panic!("期望 UnsupportedFieldType 错误");
        }
    }
    
    /// 测试字段名称验证
    #[test]
    fn test_field_name_validation() {
        // 测试有效的字段名称
        let result = Validator::validate_field_name("valid_name");
        assert!(result.is_ok());
        
        // 测试空字段名称
        let result = Validator::validate_field_name("");
        assert!(result.is_err());
        
        // 测试无效的字段名称
        let result = Validator::validate_field_name("invalid-name");
        assert!(result.is_err());
    }
    
    /// 测试长度限制验证
    #[test]
    fn test_length_limit_validation() {
        let mut config = NodeConfig::default();
        
        // 测试过长的 node_type
        let long_name = "a".repeat(limits::MAX_IDENTIFIER_LENGTH + 1);
        config.node_type = Some(long_name);
        
        let result = Validator::validate_node_config(&config);
        assert!(result.is_err());
    }
    
    /// 测试括号匹配验证
    #[test]
    fn test_bracket_matching_validation() {
        // 测试匹配的括号
        let result = Validator::validate_content_expression_syntax("text(content)");
        assert!(result.is_ok());
        
        // 测试不匹配的括号
        let result = Validator::validate_content_expression_syntax("text(content");
        assert!(result.is_err());
        
        // 测试嵌套括号
        let result = Validator::validate_content_expression_syntax("text([{content}])");
        assert!(result.is_ok());
    }
    
    /// 测试完整的配置验证流程
    #[test]
    fn test_complete_validation_flow() {
        // 创建一个完整的有效配置
        let input: syn::DeriveInput = parse_quote! {
            #[derive(Node)]
            #[node_type = "paragraph"]
            #[marks = "bold italic"]
            #[content = "text*"]
            struct ParagraphNode {
                #[attr]
                content: String,
                
                #[attr]
                alignment: Option<String>,
                
                // 非属性字段
                private_data: i32,
            }
        };
        
        // 解析配置
        let config = AttributeParser::parse_node_attributes(&input).unwrap();
        
        // 验证配置
        let result = Validator::validate_node_config(&config);
        assert!(result.is_ok());
    }
}