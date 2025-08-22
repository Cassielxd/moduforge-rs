use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use mf_model::node_pool::NodePool;
use mf_state::plugin::{
    Plugin, PluginMetadata, PluginSpec, PluginTrait, StateField,
};
use mf_state::state::State;
use mf_state::transaction::Transaction;

// TantivyBackend imported indirectly via create_tantivy_service
use crate::service::{IndexEvent, IndexService};
use crate::step_registry::ensure_default_step_indexers;
use crate::create_tantivy_service;

// Resource wrappers
pub struct TantivySearchIndexResource {
    pub service: Arc<IndexService>,
}
impl mf_state::resource::Resource for TantivySearchIndexResource {}

struct TantivyIndexStateField {
    service: Arc<IndexService>,
}

impl std::fmt::Debug for TantivyIndexStateField {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("TantivyIndexStateField").finish()
    }
}

// in-memory backend removed

#[async_trait]
impl StateField for TantivyIndexStateField {
    async fn init(
        &self,
        _config: &mf_state::state::StateConfig,
        _instance: &State,
    ) -> Arc<dyn mf_state::resource::Resource> {
        Arc::new(TantivySearchIndexResource { service: self.service.clone() })
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn mf_state::resource::Resource>,
        old_state: &State,
        new_state: &State,
    ) -> Arc<dyn mf_state::resource::Resource> {
        if let Some(res) = value.downcast_arc::<TantivySearchIndexResource>() {
            let svc = res.service.clone();
            let steps: Vec<Arc<dyn mf_transform::step::Step>> =
                tr.steps.iter().cloned().collect();
            let pool_before: Arc<NodePool> = old_state.doc();
            let pool_after: Arc<NodePool> = new_state.doc();
            let _ = svc.handle(IndexEvent::TransactionCommitted {
                pool_before: Some(pool_before),
                pool_after: pool_after,
                steps,
            });
        }
        value
    }
}
#[derive(Debug)]
struct TantivyIndexPluginTrait {}

impl PluginTrait for TantivyIndexPluginTrait {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "tantivy_index".to_string(),
            version: "1.0.0".to_string(),
            description: "Tantivy 索引插件".to_string(),
            author: "Tantivy".to_string(),
            dependencies: vec![],
            conflicts: vec![],
            state_fields: vec![],
            tags: vec![],
        }
    }
}
// Create a plugin that maintains a Tantivy index and updates it on each transaction.
pub fn create_tantivy_index_plugin(
    index_dir: &std::path::Path
) -> Result<Arc<Plugin>> {
    ensure_default_step_indexers();
    let service = Arc::new(create_tantivy_service(index_dir)?);
    let field: Arc<dyn StateField> =
        Arc::new(TantivyIndexStateField { service });
    let spec = PluginSpec {
        state_field: Some(field),
        tr: Arc::new(TantivyIndexPluginTrait {}),
    };
    Ok(Arc::new(Plugin::new(spec)))
}
