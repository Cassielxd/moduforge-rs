//! 常量定义模块
//!
//! 定义宏系统中使用的各种常量，包括支持的类型、默认值、错误消息模板等。
//! 遵循单一职责原则，专门负责常量的定义和管理。

/// 支持的基本类型列表
///
/// 定义宏系统支持的所有基本数据类型。
/// 遵循开闭原则，可以通过添加新的类型来扩展功能而不修改现有代码。
pub const SUPPORTED_BASIC_TYPES: &[&str] = &[
    // 字符串类型
    "String",
    "str",
    "&str",
    
    // 有符号整数类型  
    "i8",
    "i16", 
    "i32",
    "i64",
    "i128",
    "isize",
    
    // 无符号整数类型
    "u8",
    "u16",
    "u32", 
    "u64",
    "u128",
    "usize",
    
    // 浮点数类型
    "f32",
    "f64",
    
    // 布尔类型
    "bool",
];

/// 常用的节点类型示例
///
/// 提供常见的节点类型名称作为示例和文档参考。
/// 遵循接口隔离原则，为不同用途提供专门的常量集合。
pub const COMMON_NODE_TYPES: &[&str] = &[
    "paragraph",     // 段落节点
    "heading",       // 标题节点
    "text",          // 文本节点
    "image",         // 图片节点
    "link",          // 链接节点
    "list",          // 列表节点
    "list_item",     // 列表项节点
    "table",         // 表格节点
    "table_row",     // 表格行节点
    "table_cell",    // 表格单元格节点
    "code_block",    // 代码块节点
    "blockquote",    // 引用块节点
    "horizontal_rule", // 水平分割线节点
];

/// 常用的标记类型示例
///
/// 提供常见的标记类型名称作为示例和文档参考。
/// 这些标记通常用于文本格式化和样式设置。
pub const COMMON_MARK_TYPES: &[&str] = &[
    "bold",          // 粗体标记
    "italic",        // 斜体标记
    "underline",     // 下划线标记
    "strikethrough", // 删除线标记
    "code",          // 代码标记
    "link",          // 链接标记
    "color",         // 颜色标记
    "background",    // 背景色标记
    "font_size",     // 字体大小标记
    "font_family",   // 字体族标记
];

/// 必需的宏属性列表
///
/// 定义各种派生宏必需的属性名称。
/// 遵循单一职责原则，为每种宏类型明确定义必需的属性。
pub mod required_attributes {
    /// Node 派生宏必需的属性
    pub const NODE_REQUIRED: &[&str] = &[
        "node_type",     // 节点类型，必需
    ];
    
    /// Mark 派生宏必需的属性
    pub const MARK_REQUIRED: &[&str] = &[
        "mark_type",     // 标记类型，必需
    ];
}

/// 可选的宏属性列表
///
/// 定义各种派生宏可选的属性名称。
/// 提供完整的属性支持列表，便于验证和文档生成。
pub mod optional_attributes {
    /// Node 派生宏可选的属性
    pub const NODE_OPTIONAL: &[&str] = &[
        "marks",         // 支持的标记列表，可选
        "content",       // 内容约束表达式，可选
    ];
    
    /// Mark 派生宏可选的属性（暂时没有）
    pub const MARK_OPTIONAL: &[&str] = &[
        // 暂时没有可选属性
    ];
    
    /// 字段级通用属性
    pub const FIELD_ATTRIBUTES: &[&str] = &[
        "attr",          // 标记字段为属性，可用于字段级
    ];
}

/// 错误消息模板
///
/// 定义标准化的错误消息模板，确保错误消息的一致性和友好性。
/// 遵循接口隔离原则，为不同类型的错误提供专门的消息模板。
pub mod error_messages {
    /// 缺少必需属性的错误消息模板
    pub const MISSING_ATTRIBUTE_TEMPLATE: &str = 
        "缺少必需的宏属性: {attribute}";
    
    /// 无效属性值的错误消息模板
    pub const INVALID_ATTRIBUTE_VALUE_TEMPLATE: &str = 
        "无效的属性值 '{value}' 用于属性 '{attribute}': {reason}";
    
    /// 不支持字段类型的错误消息模板
    pub const UNSUPPORTED_FIELD_TYPE_TEMPLATE: &str = 
        "不支持的字段类型 '{field_type}' 在字段 '{field_name}' 中";
    
    /// 属性解析错误的消息模板
    pub const PARSE_ERROR_TEMPLATE: &str = 
        "属性解析错误: {message}";
    
    /// 代码生成错误的消息模板
    pub const GENERATION_ERROR_TEMPLATE: &str = 
        "代码生成错误: {message}";
    
    /// 验证错误的消息模板
    pub const VALIDATION_ERROR_TEMPLATE: &str = 
        "验证错误: {message}";
}

/// 修复建议模板
///
/// 为常见错误提供具体的修复建议模板，帮助开发者快速解决问题。
/// 体现了友好的用户体验设计。
pub mod suggestion_templates {
    /// 缺少 node_type 属性的修复建议
    pub const MISSING_NODE_TYPE_SUGGESTION: &str = 
        "请在结构体上添加 #[node_type = \"类型名\"] 属性，例如: #[node_type = \"paragraph\"]";
    
    /// 缺少 mark_type 属性的修复建议
    pub const MISSING_MARK_TYPE_SUGGESTION: &str = 
        "请在结构体上添加 #[mark_type = \"类型名\"] 属性，例如: #[mark_type = \"bold\"]";
    
    /// 无效属性值的修复建议
    pub const INVALID_ATTRIBUTE_VALUE_SUGGESTION: &str = 
        "请检查属性值格式是否正确，确保使用双引号包围字符串值";
    
    /// 不支持字段类型的修复建议
    pub const UNSUPPORTED_FIELD_TYPE_SUGGESTION: &str = 
        "请使用支持的基本类型：String, i32, i64, f32, f64, bool 或其 Option 包装版本";
    
    /// 通用修复建议
    pub const GENERAL_SUGGESTION: &str = 
        "请检查宏的使用方式是否符合文档要求";
}

/// 默认值配置
///
/// 定义各种情况下使用的默认值。
/// 遵循开闭原则，可以通过修改这些常量来调整默认行为。
pub mod defaults {
    /// 默认的 Node 内容约束
    pub const DEFAULT_NODE_CONTENT: &str = "*";
    
    /// 默认的属性映射初始容量
    pub const DEFAULT_ATTRS_CAPACITY: usize = 4;
    
    /// 默认的标记列表分隔符
    pub const DEFAULT_MARKS_SEPARATOR: &str = ",";
}

/// 代码生成相关常量
///
/// 定义代码生成过程中使用的常量，包括方法名称、变量名等。
/// 遵循单一职责原则，专门管理代码生成相关的命名规范。
pub mod codegen {
    /// 生成的 Node 转换方法名称
    pub const NODE_CONVERTER_METHOD: &str = "to_node";
    
    /// 生成的 Mark 转换方法名称  
    pub const MARK_CONVERTER_METHOD: &str = "to_mark";
    
    /// 生成代码中的 Node 实例变量名
    pub const NODE_INSTANCE_VAR: &str = "node";
    
    /// 生成代码中的 Mark 实例变量名
    pub const MARK_INSTANCE_VAR: &str = "mark";
    
    /// 生成代码中的 NodeSpec 实例变量名
    pub const NODE_SPEC_VAR: &str = "node_spec";
    
    /// 生成代码中的 MarkSpec 实例变量名
    pub const MARK_SPEC_VAR: &str = "mark_spec";
    
    /// 生成代码中的属性映射变量名
    pub const ATTRS_MAP_VAR: &str = "attrs";
}

/// 文档相关常量
///
/// 定义生成的文档注释中使用的常量文本。
/// 确保生成的文档具有一致的格式和内容。
pub mod documentation {
    /// Node 转换方法的文档注释
    pub const NODE_CONVERTER_DOC: &str = 
        "将结构体转换为 mf_core::node::Node 实例\n\
         \n\
         此方法由 #[derive(Node)] 宏自动生成，\n\
         根据结构体的字段和宏属性配置创建相应的 Node 实例。";
    
    /// Mark 转换方法的文档注释
    pub const MARK_CONVERTER_DOC: &str = 
        "将结构体转换为 mf_core::mark::Mark 实例\n\
         \n\
         此方法由 #[derive(Mark)] 宏自动生成，\n\
         根据结构体的字段和宏属性配置创建相应的 Mark 实例。";
    
    /// 返回值说明
    pub const RETURN_VALUE_DOC: &str = 
        "# 返回值\n\
         \n\
         返回配置好的实例";
    
    /// 示例代码说明
    pub const EXAMPLE_DOC: &str = 
        "# 示例\n\
         \n\
         ```rust\n\
         // 创建实例\n\
         let instance = MyStruct { /* 字段初始化 */ };\n\
         // 转换为相应类型\n\
         let converted = instance.to_node(); // 或 to_mark()\n\
         ```";
}

/// 验证规则常量
///
/// 定义各种验证规则的参数和阈值。
/// 遵循开闭原则，可以通过修改这些常量来调整验证行为。
pub mod validation {
    /// 标识符最小长度
    pub const MIN_IDENTIFIER_LENGTH: usize = 1;
    
    /// 标识符最大长度
    pub const MAX_IDENTIFIER_LENGTH: usize = 64;
    
    /// 属性值最大长度
    pub const MAX_ATTRIBUTE_VALUE_LENGTH: usize = 256;
    
    /// 标记列表最大数量
    pub const MAX_MARKS_COUNT: usize = 10;
    
    /// 字段属性最大数量
    pub const MAX_FIELD_ATTRIBUTES: usize = 20;
}

/// 性能优化相关常量
///
/// 定义性能优化相关的参数和阈值。
/// 用于控制缓存大小、批处理数量等性能相关设置。
pub mod performance {
    /// 类型分析结果缓存大小
    pub const TYPE_ANALYSIS_CACHE_SIZE: usize = 100;
    
    /// 代码生成缓存大小
    pub const CODEGEN_CACHE_SIZE: usize = 50;
    
    /// 批处理字段数量阈值
    pub const BATCH_PROCESS_THRESHOLD: usize = 5;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试支持类型列表的完整性
    #[test]
    fn test_supported_basic_types() {
        assert!(SUPPORTED_BASIC_TYPES.contains(&"String"));
        assert!(SUPPORTED_BASIC_TYPES.contains(&"i32"));
        assert!(SUPPORTED_BASIC_TYPES.contains(&"f64"));
        assert!(SUPPORTED_BASIC_TYPES.contains(&"bool"));
        assert!(!SUPPORTED_BASIC_TYPES.is_empty());
    }

    /// 测试常用节点类型列表
    #[test]
    fn test_common_node_types() {
        assert!(COMMON_NODE_TYPES.contains(&"paragraph"));
        assert!(COMMON_NODE_TYPES.contains(&"heading"));
        assert!(COMMON_NODE_TYPES.contains(&"text"));
        assert!(!COMMON_NODE_TYPES.is_empty());
    }

    /// 测试常用标记类型列表
    #[test] 
    fn test_common_mark_types() {
        assert!(COMMON_MARK_TYPES.contains(&"bold"));
        assert!(COMMON_MARK_TYPES.contains(&"italic"));
        assert!(COMMON_MARK_TYPES.contains(&"code"));
        assert!(!COMMON_MARK_TYPES.is_empty());
    }

    /// 测试必需属性列表
    #[test]
    fn test_required_attributes() {
        assert!(required_attributes::NODE_REQUIRED.contains(&"node_type"));
        assert!(required_attributes::MARK_REQUIRED.contains(&"mark_type"));
    }

    /// 测试可选属性列表
    #[test]
    fn test_optional_attributes() {
        assert!(optional_attributes::NODE_OPTIONAL.contains(&"marks"));
        assert!(optional_attributes::NODE_OPTIONAL.contains(&"content"));
        assert!(optional_attributes::FIELD_ATTRIBUTES.contains(&"attr"));
    }

    /// 测试错误消息模板格式
    #[test]
    fn test_error_message_templates() {
        assert!(error_messages::MISSING_ATTRIBUTE_TEMPLATE.contains("{attribute}"));
        assert!(error_messages::INVALID_ATTRIBUTE_VALUE_TEMPLATE.contains("{value}"));
        assert!(error_messages::UNSUPPORTED_FIELD_TYPE_TEMPLATE.contains("{field_type}"));
    }

    /// 测试修复建议模板
    #[test]
    fn test_suggestion_templates() {
        assert!(suggestion_templates::MISSING_NODE_TYPE_SUGGESTION.contains("node_type"));
        assert!(suggestion_templates::MISSING_MARK_TYPE_SUGGESTION.contains("mark_type"));
        assert!(!suggestion_templates::GENERAL_SUGGESTION.is_empty());
    }

    /// 测试代码生成常量
    #[test]
    fn test_codegen_constants() {
        assert_eq!(codegen::NODE_CONVERTER_METHOD, "to_node");
        assert_eq!(codegen::MARK_CONVERTER_METHOD, "to_mark");
        assert_eq!(codegen::NODE_INSTANCE_VAR, "node");
        assert_eq!(codegen::MARK_INSTANCE_VAR, "mark");
    }

    /// 测试验证规则常量的合理性
    #[test]
    fn test_validation_constants() {
        assert!(validation::MIN_IDENTIFIER_LENGTH > 0);
        assert!(validation::MAX_IDENTIFIER_LENGTH > validation::MIN_IDENTIFIER_LENGTH);
        assert!(validation::MAX_MARKS_COUNT > 0);
        assert!(validation::MAX_FIELD_ATTRIBUTES > 0);
    }

    /// 测试性能常量的合理性
    #[test]
    fn test_performance_constants() {
        assert!(performance::TYPE_ANALYSIS_CACHE_SIZE > 0);
        assert!(performance::CODEGEN_CACHE_SIZE > 0);
        assert!(performance::BATCH_PROCESS_THRESHOLD > 0);
    }
}