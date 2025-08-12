//! XML Schema 解析与序列化模块
//!
//! 拆分自历史上的大型 `schema_parser.rs`，模块化为：
//! - `error`: 错误类型与结果别名
//! - `types`: XML 映射结构与自定义反序列化器
//! - `parser`: 解析器与多文件解析逻辑
//! - `serializer`: 从 `SchemaSpec`/`Extensions` 生成 XML
//!
//! 对外导出兼容的公共 API：
//! - `XmlSchemaParser`（解析）
//! - `XmlSchemaSerializer`（序列化）
//! - `XmlSchemaError`、`XmlSchemaResult`

pub mod error;
pub mod types;
pub mod parser;
pub mod serializer;

pub use error::{XmlSchemaError, XmlSchemaResult};
pub use parser::{MultiFileParseContext, XmlSchemaParser};
pub use serializer::XmlSchemaSerializer;


