use std::{
    fmt::format,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use moduforge_core::{
    model::node_pool::NodePool,
    state::{self, state::State},
};

use crate::cache::{cache::DocumentCache, CacheKey};
#[derive(Debug)]
pub struct SnapshotManager {
    document_cache: Arc<DocumentCache>,
    list: Arc<Mutex<Vec<CacheKey>>>,
}
impl SnapshotManager {
    pub fn create(document_cache: Arc<DocumentCache>) -> Arc<SnapshotManager> {
        Arc::new(SnapshotManager {
            document_cache,
            list: Arc::new(Mutex::new(vec![])),
        })
    }
    pub fn get_snapshot(&self, key: &CacheKey) -> Option<Arc<NodePool>> {
        self.document_cache.get(key)
    }

    pub fn put(&self, state: &Arc<State>) {
        let key = CacheKey {
            doc_id: state.doc().inner.root_id.clone(),
            version: state.version,
        };
        {
            self.list.lock().unwrap().push(key.clone());
        }
        self.document_cache.l1.put(key.clone(), state.doc());
        self.document_cache.l2.put(
            format!("{}{}", key.doc_id.clone(), state.version),
            state.doc(),
        );
    }
}
