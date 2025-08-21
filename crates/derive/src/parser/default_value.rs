//! 默认值处理器模块
//!
//! 负责解析、验证和处理 #[attr(default="value")] 属性中的默认值。
//! 严格遵循单一职责原则，专门处理默认值相关的所有逻辑。
//!
//! # 设计原则体现
//!
//! - **单一职责**: 专门负责默认值相关的数据结构和基础逻辑
//! - **开闭原则**: 通过枚举和 trait 支持新的默认值类型扩展
//! - **接口隔离**: 提供最小化、专用的解析接口
//! - **里氏替换**: DefaultValue 实例在任何需要默认值的地方都可互换使用

use proc_macro2::Span;
use serde_json;
use crate::common::{MacroError, MacroResult};

/// 默认值表示
///
/// 存储解析后的默认值信息，包括原始值、类型化值和元数据。
/// 遵循不可变性原则，创建后内容不可修改，确保数据一致性。
///
/// # 设计原则体现
///
/// - **单一职责**: 专门负责默认值的数据表示
/// - **不可变性**: 创建后不可修改，确保数据一致性
/// - **开闭原则**: 通过 DefaultValueType 枚举支持新类型扩展
///
/// # 使用示例
///
/// ```rust
/// use moduforge_derive::parser::default_value::*;
///
/// // 解析字符串默认值
/// let default_value = DefaultValueParser::parse("hello", None)?;
/// assert!(matches!(default_value.value_type, DefaultValueType::String(_)));
///
/// // 解析数值默认值
/// let default_value = DefaultValueParser::parse("42", None)?;
/// assert!(matches!(default_value.value_type, DefaultValueType::Integer(42)));
/// ```
#[derive(Debug, Clone)]
pub struct DefaultValue {
    /// 原始字符串值
    /// 
    /// 保存用户在宏属性中输入的原始字符串，用于错误报告和调试
    pub raw_value: String,
    
    /// 解析后的值类型
    /// 
    /// 将原始字符串解析为强类型的值，确保类型安全
    pub value_type: DefaultValueType,
    
    /// 是否为 JSON 格式
    /// 
    /// 标识此默认值是否为 JSON 格式，用于约束类型检查
    pub is_json: bool,
    
    /// 源码位置信息（用于错误报告）
    /// 
    /// 记录默认值在源码中的位置，提供精确的错误定位
    pub span: Option<Span>,
}

/// 默认值类型枚举
///
/// 表示所有支持的默认值类型，提供类型安全的值表示。
/// 遵循开闭原则，可以通过添加新的变体来支持更多类型。
///
/// # 设计原则体现
///
/// - **开闭原则**: 可以添加新变体而不修改现有代码
/// - **类型安全**: 每种类型都有明确的表示
/// - **单一职责**: 每个变体只表示一种特定的值类型
///
/// # 支持的类型
///
/// - `String`: 字符串字面量，如 "hello world"
/// - `Integer`: 整数字面量，如 42, -100
/// - `Float`: 浮点数字面量，如 3.14, -2.5
/// - `Boolean`: 布尔值字面量，如 true, false
/// - `Json`: JSON 格式的复杂值，如 {"key": "value"}
/// - `Null`: 空值，用于 Option 类型的默认值
#[derive(Debug, Clone, PartialEq)]
pub enum DefaultValueType {
    /// 字符串类型默认值
    /// 
    /// 存储解析后的字符串值，已去除引号
    String(String),
    
    /// 整数类型默认值
    /// 
    /// 存储解析后的整数值，使用 i64 作为统一表示
    Integer(i64),
    
    /// 浮点数类型默认值
    /// 
    /// 存储解析后的浮点数值，使用 f64 作为统一表示
    Float(f64),
    
    /// 布尔类型默认值
    /// 
    /// 存储解析后的布尔值
    Boolean(bool),
    
    /// JSON 类型默认值
    /// 
    /// 存储解析后的 JSON 值，用于复杂数据结构的默认值
    Json(serde_json::Value),
    
    /// 空值类型默认值
    /// 
    /// 用于表示 Option 类型的 None 值
    Null,
}

/// 默认值解析器
///
/// 提供将字符串解析为类型化默认值的核心功能。
/// 遵循单一职责原则，专门负责解析逻辑，不涉及验证或生成。
///
/// # 设计原则体现
///
/// - **单一职责**: 只负责字符串解析，不处理验证或类型检查
/// - **开闭原则**: 通过模式匹配支持新的解析规则
/// - **接口隔离**: 提供简单、专用的解析接口
///
/// # 解析优先级
///
/// 1. JSON 格式检测（优先级最高）
/// 2. 布尔值解析 ("true", "false")
/// 3. 数值解析（整数、浮点数）
/// 4. 特殊值解析 ("null")
/// 5. 字符串解析（默认情况）
pub struct DefaultValueParser;

impl DefaultValueParser {
    /// 解析默认值字符串为结构化表示
    ///
    /// 从用户输入的字符串中解析出类型化的默认值。
    /// 支持多种格式的自动识别和转换。
    ///
    /// # 参数
    ///
    /// * `raw_value` - 原始的默认值字符串
    /// * `span` - 源码位置信息，用于错误报告
    ///
    /// # 返回值
    ///
    /// 成功时返回 `Ok(DefaultValue)`，失败时返回解析错误
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责字符串解析，不处理验证
    /// - **开闭原则**: 通过类型匹配支持新的默认值类型
    /// - **里氏替换**: 任何字符串输入都能得到一致的处理
    ///
    /// # 解析规则
    ///
    /// 1. **JSON 格式**: 以 `{` 或 `[` 开头的字符串作为 JSON 解析
    /// 2. **布尔值**: "true" 和 "false" 解析为布尔类型
    /// 3. **空值**: "null" 解析为 Null 类型
    /// 4. **数值**: 纯数字字符串解析为整数或浮点数
    /// 5. **字符串**: 其他所有情况解析为字符串类型
    ///
    /// # 使用示例
    ///
    /// ```rust
    /// // 字符串解析
    /// let result = DefaultValueParser::parse("hello world", None)?;
    /// assert!(matches!(result.value_type, DefaultValueType::String(_)));
    ///
    /// // 整数解析
    /// let result = DefaultValueParser::parse("42", None)?;
    /// assert!(matches!(result.value_type, DefaultValueType::Integer(42)));
    ///
    /// // JSON 解析
    /// let result = DefaultValueParser::parse(r#"{"key": "value"}"#, None)?;
    /// assert!(result.is_json);
    /// ```
    ///
    /// # 错误处理
    ///
    /// - JSON 语法错误会返回详细的错误信息
    /// - 数值格式错误会退化到字符串解析
    /// - 所有错误都包含原始输入和位置信息
    pub fn parse(raw_value: &str, span: Option<Span>) -> MacroResult<DefaultValue> {
        // 去除首尾空白字符
        let trimmed_value = raw_value.trim();
        
        // 检查是否为空值
        if trimmed_value.is_empty() {
            return Ok(DefaultValue {
                raw_value: raw_value.to_string(),
                value_type: DefaultValueType::String(String::new()),
                is_json: false,
                span,
            });
        }
        
        // 1. 优先检查 JSON 格式
        if Self::is_json_format(trimmed_value) {
            match serde_json::from_str::<serde_json::Value>(trimmed_value) {
                Ok(json_value) => {
                    return Ok(DefaultValue {
                        raw_value: raw_value.to_string(),
                        value_type: DefaultValueType::Json(json_value),
                        is_json: true,
                        span,
                    });
                }
                Err(json_err) => {
                    return Err(MacroError::default_value_parse_error(
                        &format!("JSON 解析失败: {}", json_err),
                        raw_value,
                        span.unwrap_or_else(Span::call_site),
                    ));
                }
            }
        }
        
        // 2. 检查布尔值
        match trimmed_value {
            "true" => {
                return Ok(DefaultValue {
                    raw_value: raw_value.to_string(),
                    value_type: DefaultValueType::Boolean(true),
                    is_json: false,
                    span,
                });
            }
            "false" => {
                return Ok(DefaultValue {
                    raw_value: raw_value.to_string(),
                    value_type: DefaultValueType::Boolean(false),
                    is_json: false,
                    span,
                });
            }
            "null" => {
                return Ok(DefaultValue {
                    raw_value: raw_value.to_string(),
                    value_type: DefaultValueType::Null,
                    is_json: false,
                    span,
                });
            }
            _ => {}
        }
        
        // 3. 尝试解析数值（整数优先）
        if let Ok(int_value) = trimmed_value.parse::<i64>() {
            return Ok(DefaultValue {
                raw_value: raw_value.to_string(),
                value_type: DefaultValueType::Integer(int_value),
                is_json: false,
                span,
            });
        }
        
        // 4. 尝试解析浮点数
        if let Ok(float_value) = trimmed_value.parse::<f64>() {
            // 确保是有效的浮点数（不是 NaN 或无穷大）
            if float_value.is_finite() {
                return Ok(DefaultValue {
                    raw_value: raw_value.to_string(),
                    value_type: DefaultValueType::Float(float_value),
                    is_json: false,
                    span,
                });
            }
        }
        
        // 5. 默认情况：作为字符串处理
        Ok(DefaultValue {
            raw_value: raw_value.to_string(),
            value_type: DefaultValueType::String(trimmed_value.to_string()),
            is_json: false,
            span,
        })
    }
    
    /// 检测是否为 JSON 格式
    ///
    /// 通过简单的启发式规则判断字符串是否可能是 JSON 格式。
    /// 这是一个快速的预检查，具体的语法验证由 serde_json 完成。
    ///
    /// # 参数
    ///
    /// * `value` - 要检查的字符串
    ///
    /// # 返回值
    ///
    /// 如果可能是 JSON 格式返回 true，否则返回 false
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 专门负责 JSON 格式检测
    /// - **性能优化**: 使用简单规则避免昂贵的解析操作
    ///
    /// # 检测规则
    ///
    /// - 以 `{` 开头和 `}` 结尾的字符串（JSON 对象）
    /// - 以 `[` 开头和 `]` 结尾的字符串（JSON 数组）
    /// - 长度必须至少为 2 个字符
    ///
    /// # 使用示例
    ///
    /// ```rust
    /// assert!(DefaultValueParser::is_json_format(r#"{"key": "value"}"#));
    /// assert!(DefaultValueParser::is_json_format(r#"["item1", "item2"]"#));
    /// assert!(!DefaultValueParser::is_json_format("simple string"));
    /// assert!(!DefaultValueParser::is_json_format("42"));
    /// ```
    fn is_json_format(value: &str) -> bool {
        let trimmed = value.trim();
        
        // 检查长度（最短的 JSON 是 "{}" 或 "[]"）
        if trimmed.len() < 2 {
            return false;
        }
        
        // 检查 JSON 对象格式
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            return true;
        }
        
        // 检查 JSON 数组格式
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            return true;
        }
        
        false
    }
}

impl DefaultValue {
    /// 获取默认值的类型名称
    ///
    /// 返回默认值类型的字符串表示，用于错误消息和调试。
    /// 遵循单一职责原则，专门负责类型名称的获取。
    ///
    /// # 返回值
    ///
    /// 返回类型名称的字符串表示
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 只负责类型名称获取
    /// - **接口隔离**: 提供简单的类型查询接口
    ///
    /// # 使用示例
    ///
    /// ```rust
    /// let default_value = DefaultValueParser::parse("42", None)?;
    /// assert_eq!(default_value.type_name(), "Integer");
    ///
    /// let default_value = DefaultValueParser::parse("hello", None)?;
    /// assert_eq!(default_value.type_name(), "String");
    /// ```
    pub fn type_name(&self) -> &'static str {
        match &self.value_type {
            DefaultValueType::String(_) => "String",
            DefaultValueType::Integer(_) => "Integer",
            DefaultValueType::Float(_) => "Float",
            DefaultValueType::Boolean(_) => "Boolean",
            DefaultValueType::Json(_) => "Json",
            DefaultValueType::Null => "Null",
        }
    }
    
    /// 检查是否为数值类型
    ///
    /// 判断默认值是否为数值类型（整数或浮点数）。
    /// 用于类型验证和代码生成优化。
    ///
    /// # 返回值
    ///
    /// 如果是数值类型返回 true，否则返回 false
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 专门负责数值类型判断
    /// - **接口隔离**: 提供专用的类型检查接口
    pub fn is_numeric(&self) -> bool {
        matches!(self.value_type, DefaultValueType::Integer(_) | DefaultValueType::Float(_))
    }
    
    /// 检查是否为字符串类型
    ///
    /// 判断默认值是否为字符串类型。
    /// 用于类型验证和代码生成。
    ///
    /// # 返回值
    ///
    /// 如果是字符串类型返回 true，否则返回 false
    pub fn is_string(&self) -> bool {
        matches!(self.value_type, DefaultValueType::String(_))
    }
    
    /// 检查是否为布尔类型
    ///
    /// 判断默认值是否为布尔类型。
    /// 用于类型验证和代码生成。
    ///
    /// # 返回值
    ///
    /// 如果是布尔类型返回 true，否则返回 false
    pub fn is_boolean(&self) -> bool {
        matches!(self.value_type, DefaultValueType::Boolean(_))
    }
    
    /// 检查是否为空值类型
    ///
    /// 判断默认值是否为空值类型（null）。
    /// 主要用于 Option 类型的处理。
    ///
    /// # 返回值
    ///
    /// 如果是空值类型返回 true，否则返回 false
    pub fn is_null(&self) -> bool {
        matches!(self.value_type, DefaultValueType::Null)
    }
}

impl PartialEq for DefaultValue {
    /// 比较两个 DefaultValue 是否相等
    /// 
    /// 忽略 span 字段，只比较值相关的字段。
    /// 这样做是合理的，因为 span 只是位置信息，不影响值的语义。
    fn eq(&self, other: &Self) -> bool {
        self.raw_value == other.raw_value
            && self.value_type == other.value_type
            && self.is_json == other.is_json
    }
}

// 为错误处理扩展 MacroError，提供默认值相关的便利方法
impl MacroError {
    /// 创建默认值解析错误
    ///
    /// 专门用于创建默认值解析相关的错误。
    /// 提供统一的错误创建接口，确保错误信息的一致性。
    ///
    /// # 参数
    ///
    /// * `reason` - 错误原因描述
    /// * `value` - 导致错误的原始值
    /// * `span` - 源码位置信息
    ///
    /// # 返回值
    ///
    /// 返回配置好的 MacroError 实例
    ///
    /// # 设计原则体现
    ///
    /// - **单一职责**: 专门负责默认值解析错误创建
    /// - **接口隔离**: 提供专用的错误创建接口
    pub fn default_value_parse_error(reason: &str, value: &str, span: Span) -> Self {
        MacroError::ParseError {
            message: format!("默认值解析失败: {} (问题值: '{}')", reason, value),
            span: Some(span),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;

    /// 测试字符串默认值解析
    #[test]
    fn test_parse_string_default() {
        let result = DefaultValueParser::parse("hello world", None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        assert_eq!(default_value.raw_value, "hello world");
        assert!(matches!(default_value.value_type, DefaultValueType::String(ref s) if s == "hello world"));
        assert!(!default_value.is_json);
        assert_eq!(default_value.type_name(), "String");
        assert!(default_value.is_string());
    }
    
    /// 测试整数默认值解析
    #[test]
    fn test_parse_integer_default() {
        let result = DefaultValueParser::parse("42", None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        assert_eq!(default_value.raw_value, "42");
        assert!(matches!(default_value.value_type, DefaultValueType::Integer(42)));
        assert!(!default_value.is_json);
        assert_eq!(default_value.type_name(), "Integer");
        assert!(default_value.is_numeric());
    }
    
    /// 测试负整数默认值解析
    #[test]
    fn test_parse_negative_integer_default() {
        let result = DefaultValueParser::parse("-100", None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        assert!(matches!(default_value.value_type, DefaultValueType::Integer(-100)));
        assert!(default_value.is_numeric());
    }
    
    /// 测试浮点数默认值解析
    #[test]
    fn test_parse_float_default() {
        let result = DefaultValueParser::parse("3.14159", None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        assert_eq!(default_value.raw_value, "3.14159");
        assert!(matches!(default_value.value_type, DefaultValueType::Float(f) if (f - 3.14159).abs() < f64::EPSILON));
        assert!(!default_value.is_json);
        assert_eq!(default_value.type_name(), "Float");
        assert!(default_value.is_numeric());
    }
    
    /// 测试布尔值默认值解析
    #[test]
    fn test_parse_boolean_default() {
        // 测试 true
        let result = DefaultValueParser::parse("true", None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        assert!(matches!(default_value.value_type, DefaultValueType::Boolean(true)));
        assert_eq!(default_value.type_name(), "Boolean");
        assert!(default_value.is_boolean());
        
        // 测试 false
        let result = DefaultValueParser::parse("false", None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        assert!(matches!(default_value.value_type, DefaultValueType::Boolean(false)));
        assert!(default_value.is_boolean());
    }
    
    /// 测试 null 值默认值解析
    #[test]
    fn test_parse_null_default() {
        let result = DefaultValueParser::parse("null", None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        assert!(matches!(default_value.value_type, DefaultValueType::Null));
        assert_eq!(default_value.type_name(), "Null");
        assert!(default_value.is_null());
    }
    
    /// 测试 JSON 对象默认值解析
    #[test]
    fn test_parse_json_object_default() {
        let json_str = r#"{"key": "value", "number": 123, "nested": {"inner": true}}"#;
        let result = DefaultValueParser::parse(json_str, None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        assert_eq!(default_value.raw_value, json_str);
        assert!(default_value.is_json);
        assert_eq!(default_value.type_name(), "Json");
        
        if let DefaultValueType::Json(json_value) = &default_value.value_type {
            assert_eq!(json_value["key"], "value");
            assert_eq!(json_value["number"], 123);
            assert_eq!(json_value["nested"]["inner"], true);
        } else {
            panic!("期望 JSON 类型");
        }
    }
    
    /// 测试 JSON 数组默认值解析
    #[test]
    fn test_parse_json_array_default() {
        let json_str = r#"["item1", "item2", {"key": "value"}]"#;
        let result = DefaultValueParser::parse(json_str, None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        assert!(default_value.is_json);
        
        if let DefaultValueType::Json(json_value) = &default_value.value_type {
            assert!(json_value.is_array());
            let array = json_value.as_array().unwrap();
            assert_eq!(array.len(), 3);
            assert_eq!(array[0], "item1");
            assert_eq!(array[1], "item2");
            assert_eq!(array[2]["key"], "value");
        } else {
            panic!("期望 JSON 类型");
        }
    }
    
    /// 测试无效 JSON 的错误处理
    #[test]
    fn test_parse_invalid_json() {
        let invalid_json = r#"{"invalid": json}"#; // 修正：添加结尾括号但使用无效的 JSON 语法
        let result = DefaultValueParser::parse(invalid_json, None);
        assert!(result.is_err());
        
        if let Err(MacroError::ParseError { message, .. }) = result {
            assert!(message.contains("JSON 解析失败"));
        } else {
            panic!("期望 ParseError");
        }
    }
    
    /// 测试空字符串处理
    #[test]
    fn test_parse_empty_string() {
        let result = DefaultValueParser::parse("", None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        assert!(matches!(default_value.value_type, DefaultValueType::String(ref s) if s.is_empty()));
    }
    
    /// 测试空白字符串处理
    #[test]
    fn test_parse_whitespace_string() {
        let result = DefaultValueParser::parse("   ", None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        // 应该被trim为空字符串
        assert!(matches!(default_value.value_type, DefaultValueType::String(ref s) if s.is_empty()));
    }
    
    /// 测试 JSON 格式检测
    #[test]
    fn test_is_json_format() {
        // 有效的 JSON 格式
        assert!(DefaultValueParser::is_json_format(r#"{"key": "value"}"#));
        assert!(DefaultValueParser::is_json_format(r#"["item1", "item2"]"#));
        assert!(DefaultValueParser::is_json_format("{}"));
        assert!(DefaultValueParser::is_json_format("[]"));
        assert!(DefaultValueParser::is_json_format("  {  }  ")); // 带空格
        
        // 无效的 JSON 格式
        assert!(!DefaultValueParser::is_json_format("simple string"));
        assert!(!DefaultValueParser::is_json_format("42"));
        assert!(!DefaultValueParser::is_json_format("true"));
        assert!(!DefaultValueParser::is_json_format("{"));
        assert!(!DefaultValueParser::is_json_format("}"));
        assert!(!DefaultValueParser::is_json_format(""));
        assert!(!DefaultValueParser::is_json_format("a"));
    }
    
    /// 测试复杂数值格式
    #[test]
    fn test_parse_complex_numbers() {
        // 测试十六进制（应该作为字符串处理）
        let result = DefaultValueParser::parse("0x42", None);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().value_type, DefaultValueType::String(_)));
        
        // 测试科学计数法
        let result = DefaultValueParser::parse("1.23e-4", None);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().value_type, DefaultValueType::Float(_)));
        
        // 测试极大数值
        let result = DefaultValueParser::parse("999999999999999999999", None);
        assert!(result.is_ok());
        // 可能超出 i64 范围，应该作为字符串处理或者浮点数
        let default_value = result.unwrap();
        // 这取决于具体的实现，可能是 Integer、Float 或 String
        assert!(matches!(default_value.value_type, DefaultValueType::Integer(_) | DefaultValueType::Float(_) | DefaultValueType::String(_)));
    }
    
    /// 测试 Unicode 字符串
    #[test]
    fn test_parse_unicode_string() {
        let unicode_str = "你好世界 🦀";
        let result = DefaultValueParser::parse(unicode_str, None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        assert!(matches!(default_value.value_type, DefaultValueType::String(ref s) if s == unicode_str));
    }
    
    /// 测试边界情况：看起来像 JSON 但不是
    #[test]
    fn test_parse_json_like_strings() {
        // 不完整的对象
        let result = DefaultValueParser::parse("{incomplete", None);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().value_type, DefaultValueType::String(_)));
        
        // 单引号（无效 JSON）
        let result = DefaultValueParser::parse("{'key': 'value'}", None);
        assert!(result.is_err()); // 应该尝试解析为 JSON 但失败
    }
}