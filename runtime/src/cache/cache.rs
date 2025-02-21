// cache.rs

use moduforge_core::model::node_pool::NodePool;
use moduforge_delta::from_binary;
use moduforge_delta::snapshot::FullSnapshot;

use std::fs;
use std::sync::Arc;

use crate::types::StorageOptions;

use super::CacheKey;
use super::l1::L1Cache;
use super::l2::L2Cache;

#[derive(Clone, Debug)]
pub struct DocumentCache {
    pub l1: Arc<L1Cache>,
    pub l2: Arc<L2Cache>,
    pub storage_option: StorageOptions,
}
impl DocumentCache {
    pub fn new(path: &StorageOptions) -> Arc<Self> {
        Arc::new(DocumentCache {
            storage_option: path.clone(),
            l1: Arc::new(L1Cache::new(10)),
            l2: Arc::new(L2Cache::open(path.l2_path.as_path()).unwrap()),
        })
    }

    /// 分级读取流程
    pub fn get(&self, key: &CacheKey) -> Option<Arc<NodePool>> {
        // 1. 尝试L1读取
        if let Some(v) = self.l1.get(key) {
            return Some(v);
        }

        // 2. 尝试L2读取
        if let Ok(v) = self
            .l2
            .get(format!("{}{}", key.doc_id.clone(), key.version))
        {
            // 3. 回填L1
            self.l1.put(key.clone(), v.clone());
            return Some(v);
        }

        // 4. 回源加载

        self.load_from_storage(key)
    }

    fn load_from_storage(&self, key: &CacheKey) -> Option<Arc<NodePool>> {
        // 从全量快照+增量日志重构文档
        let base_path = self
            .storage_option
            .snapshot_path
            .join(key.clone().doc_id.as_str());
        let path = base_path.join(format!("snapshot_v{}.bin", key.version));
        let snapshot_data = fs::read(path).unwrap();
        let f = from_binary::<FullSnapshot>(&snapshot_data).unwrap();
        Some(f.node_pool)
    }
}
