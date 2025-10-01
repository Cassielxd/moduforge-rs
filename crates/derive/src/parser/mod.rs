//! 解析模块
//!
//! 提供宏属性解析、字段分析和配置验证的完整功能。
//! 严格遵循 SOLID 设计原则，确保解析逻辑的清晰性和可维护性。
//!
//! # 模块组成
//!
//! - `attribute_parser`: 属性解析器，负责解析宏属性并构建配置对象
//! - `field_analyzer`: 字段分析器，负责分析结构体字段的类型和属性信息
//! - `validation`: 验证器，负责验证配置的正确性和完整性
//! - `default_value`: 默认值处理器，负责解析和处理字段默认值
//!
//! # 设计原则体现
//!
//! - **单一职责原则 (SRP)**: 每个子模块都有明确的单一功能
//! - **接口隔离原则 (ISP)**: 提供专门的解析、分析和验证接口
//! - **开闭原则 (OCP)**: 通过配置化支持扩展新的解析规则
//! - **里氏替换原则 (LSP)**: 所有解析器都可以无缝替换使用
//!
//! # 使用流程
//!
//! 1. **属性解析**: 使用 `AttributeParser` 解析宏属性
//! 2. **字段分析**: 使用 `FieldAnalyzer` 分析字段类型和属性
//! 3. **配置验证**: 使用 `Validator` 验证解析结果的正确性
//! 4. **错误处理**: 统一的错误类型和友好的错误消息
//!
//! # 示例
//!
//! ```rust
//! use syn::parse_quote;
//! use crate::parser::{AttributeParser, FieldAnalyzer, Validator};
//!
//! let input: syn::DeriveInput = parse_quote! {
//!     #[derive(Node)]
//!     #[node_type = "paragraph"]
//!     struct ParagraphNode {
//!         #[attr]
//!         content: String,
//!     }
//! };
//!
//! // 1. 解析属性
//! let config = AttributeParser::parse_node_attributes(&input)?;
//!
//! // 2. 验证配置
//! Validator::validate_node_config(&config)?;
//!
//! // 3. 分析字段（如果需要更详细的分析）
//! let analyses = FieldAnalyzer::analyze_fields(&get_fields(&input))?;
//! Validator::validate_field_analyses(&analyses)?;
//! ```

/// 属性解析器模块
///
/// 提供 Node 和 Mark 宏属性的解析功能。
/// 遵循单一职责原则，专门负责属性解析和配置构建。
pub mod attribute_parser;

/// 字段分析器模块
///
/// 提供结构体字段的类型分析和属性检查功能。
/// 遵循接口隔离原则，为字段分析提供专门的接口。
pub mod field_analyzer;

/// 验证器模块
///
/// 提供配置验证和错误检查功能。
/// 遵循开闭原则，支持扩展新的验证规则。
pub mod validation;

/// 默认值处理器模块
///
/// 提供默认值的解析、验证和类型化表示功能。
/// 遵循单一职责原则，专门负责默认值相关的所有逻辑。
pub mod default_value;

// 重新导出核心类型和函数，遵循接口隔离原则
pub use attribute_parser::{AttributeParser, NodeConfig, MarkConfig, FieldConfig};
pub use validation::Validator;
