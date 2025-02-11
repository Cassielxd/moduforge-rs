// cache.rs

use moduforge_core::model::node_pool::NodePool;
use moduforge_delta::from_binary;
use moduforge_delta::snapshot::{FullSnapshot};

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::l1::L1Cache;
use super::l2::L2Cache;
use super::CacheKey;

#[derive(Clone, Debug)]
pub struct DocumentCache {
    pub l1: Arc<L1Cache>,
    pub l2: Arc<L2Cache>,
}
impl DocumentCache {
    pub fn new(path: &PathBuf) -> Arc<Self> {
        Arc::new(DocumentCache {
            l1: Arc::new(L1Cache::new(10)),
            l2: Arc::new(L2Cache::open(path.join("db").as_path()).unwrap()),
        })
    }

    /// 分级读取流程
    pub fn get(&self, key: &CacheKey) -> Option<Arc<NodePool>> {
        // 1. 尝试L1读取
        if let Some(v) = self.l1.get(key) {
            return Some(v);
        }

        // 2. 尝试L2读取
        if let Ok(v) = self.l2.get(&key.doc_id.as_bytes()) {
            // 3. 回填L1
            self.l1.put(key.clone(), v.clone());
            return Some(v);
        }

        // 4. 回源加载
        let value = self.load_from_storage(&key.path);
        value
    }

    fn load_from_storage(&self, path: &PathBuf) -> Option<Arc<NodePool>> {
        // 从全量快照+增量日志重构文档
        let snapshot_data = fs::read(path).unwrap();
        let f = from_binary::<FullSnapshot>(&snapshot_data).unwrap();
        Some(f.node_pool)
    }
}
