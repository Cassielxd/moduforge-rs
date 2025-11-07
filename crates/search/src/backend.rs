// SQLite backend - 完整替换 Tantivy

pub use crate::backend_sqlite::{IndexMutation, SearchQuery, SqliteBackend};

// 类型别名，保持向后兼容
pub type Backend = SqliteBackend;
