use std::{
    sync::{Arc, Mutex},
};

use moduforge_core::{
    model::node_pool::NodePool,
    state::{self, state::State},
};

use crate::cache::{cache::DocumentCache, CacheKey};
/// 快照管理器
#[derive(Debug)]
pub struct SnapshotManager {
    document_cache: Arc<DocumentCache>,
    list: Arc<Mutex<Vec<CacheKey>>>,
}
impl SnapshotManager {
    pub fn get_snapshot_list(&self) -> Vec<CacheKey> {
        let mut list = self.list.lock().unwrap().clone();
        list.sort_by(|a,b| b.time.cmp(&a.time));
        list
    }
    /// 创建快照管理器
    pub fn create(document_cache: Arc<DocumentCache>) -> Arc<SnapshotManager> {
        Arc::new(SnapshotManager {
            document_cache,
            list: Arc::new(Mutex::new(vec![])),
        })
    }
    /// 获取全量
    pub fn get_snapshot(&self, key: &CacheKey) -> Option<Arc<NodePool>> {
        self.document_cache.get(key)
    }

    pub fn put(&self, state: &Arc<State>,time: u64) {
        let key = CacheKey {
            doc_id: state.doc().inner.root_id.clone(),
            version: state.version,
            time
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
