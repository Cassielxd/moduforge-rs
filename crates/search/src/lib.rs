pub mod model;
pub mod backend;
pub mod indexer;
pub mod service;
pub mod step_registry;
pub mod state_plugin;
// 分词器：使用 tantivy-jieba 集成
pub use backend::{IndexMutation, TantivyBackend, SearchQuery};
pub use service::{IndexService, IndexEvent, RebuildScope, event_from_transaction};

use std::sync::Arc;
use std::path::Path;
use crate::step_registry::ensure_default_step_indexers;

/// 创建默认的 Tantivy 后端索引服务
pub fn create_tantivy_service(index_dir: &Path) -> anyhow::Result<IndexService> {
    ensure_default_step_indexers();
    let backend = Arc::new(TantivyBackend::new_in_dir(index_dir)?);
    Ok(IndexService::new(backend))
}

/// 在指定临时根目录下创建索引服务（会在根目录下生成唯一子目录）
pub fn create_tantivy_service_in_temp_root(temp_root: &Path) -> anyhow::Result<IndexService> {
    ensure_default_step_indexers();
    let backend = Arc::new(TantivyBackend::new_in_temp_root(temp_root)?);
    Ok(IndexService::new(backend))
}

// In-memory service removed; Tantivy is the default and only backend.

