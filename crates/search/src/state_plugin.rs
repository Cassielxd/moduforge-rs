use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use mf_model::node_pool::NodePool;
use mf_state::plugin::{
    Plugin, PluginMetadata, PluginSpec, PluginTrait, StateField,
};
use mf_state::state::State;
use mf_state::transaction::Transaction;

use crate::backend::SqliteBackend;
use crate::service::{IndexEvent, IndexService};
use crate::step_registry::ensure_default_step_indexers;

// Resource wrappers
pub struct SearchIndexResource {
    pub service: Arc<IndexService>,
}
impl mf_state::resource::Resource for SearchIndexResource {}

struct SearchIndexStateField {
    service: Arc<IndexService>,
}

impl std::fmt::Debug for SearchIndexStateField {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("SearchIndexStateField").finish()
    }
}

#[async_trait]
impl StateField for SearchIndexStateField {
    type Value = SearchIndexResource;

    async fn init(
        &self,
        _config: &mf_state::state::StateConfig,
        _instance: &State,
    ) -> Arc<Self::Value> {
        Arc::new(SearchIndexResource { service: self.service.clone() })
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<Self::Value>,
        old_state: &State,
        new_state: &State,
    ) -> Arc<Self::Value> {
        let svc = value.service.clone();
        let steps: Vec<Arc<dyn mf_transform::step::Step>> =
            tr.steps.iter().cloned().collect();
        let pool_before: Arc<NodePool> = old_state.doc();
        let pool_after: Arc<NodePool> = new_state.doc();

        // 异步处理索引更新（不阻塞事务）
        tokio::spawn(async move {
            let _ = svc.handle(IndexEvent::TransactionCommitted {
                pool_before: Some(pool_before),
                pool_after,
                steps,
            }).await;
        });

        value
    }
}

#[derive(Debug)]
struct SearchIndexPluginTrait {}

impl PluginTrait for SearchIndexPluginTrait {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "search_index".to_string(),
            version: "2.0.0".to_string(),
            description: "SQLite 搜索索引插件".to_string(),
            author: "ModuForge".to_string(),
            dependencies: vec![],
            conflicts: vec![],
            state_fields: vec![],
            tags: vec![],
        }
    }
}

/// 创建搜索索引插件（使用 SQLite 后端）
pub fn create_search_index_plugin(
    index_dir: &std::path::Path
) -> Result<Arc<Plugin>> {
    ensure_default_step_indexers();
    let backend = Arc::new(SqliteBackend::new_in_dir(index_dir)?);
    let service = Arc::new(IndexService::new(backend));

    let field = Arc::new(SearchIndexStateField { service });
    let spec = PluginSpec {
        state_field: Some(field),
        tr: Arc::new(SearchIndexPluginTrait {}),
    };
    Ok(Arc::new(Plugin::new(spec)))
}

/// 创建临时搜索索引插件（用于测试）
pub fn create_temp_search_index_plugin() -> Result<Arc<Plugin>> {
    ensure_default_step_indexers();
    let backend = Arc::new(SqliteBackend::new_in_system_temp()?);
    let service = Arc::new(IndexService::new(backend));

    let field = Arc::new(SearchIndexStateField { service });
    let spec = PluginSpec {
        state_field: Some(field),
        tr: Arc::new(SearchIndexPluginTrait {}),
    };
    Ok(Arc::new(Plugin::new(spec)))
}

// 向后兼容的别名
pub use create_search_index_plugin as SearchPlugin;
