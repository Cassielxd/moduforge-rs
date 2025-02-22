use std::{
    io::{Error, Write},
    path::Path,
    sync::{Arc, Mutex},
};

use moduforge_core::{model::node_pool::NodePool, state::state::State};
use moduforge_delta::snapshot::create_full_snapshot;
use zip::write::SimpleFileOptions;

use crate::cache::{CacheKey, cache::DocumentCache};
/// 快照管理器
///
#[derive(Debug)]
pub struct StorageManager {
    document_cache: Arc<DocumentCache>,
    list: Arc<Mutex<Vec<CacheKey>>>,
}
impl StorageManager {
    pub fn get_snapshot_list(&self) -> Vec<CacheKey> {
        let mut list = self.list.lock().unwrap().clone();
        list.sort_by(|a, b| b.time.cmp(&a.time));
        list
    }
    /// 创建快照管理器
    pub fn create(document_cache: Arc<DocumentCache>) -> Arc<StorageManager> {
        Arc::new(StorageManager {
            document_cache,
            list: Arc::new(Mutex::new(vec![])),
        })
    }
    /// 获取全量
    pub fn get_snapshot(&self, key: &CacheKey) -> Option<Arc<NodePool>> {
        self.document_cache.get(key)
    }

    pub fn put(&self, state: &Arc<State>, time: u64) {
        let key = CacheKey {
            doc_id: state.doc().inner.root_id.clone(),
            version: state.version,
            time,
        };
        {
            self.list.lock().unwrap().push(key.clone());
        }
        self.document_cache.put(state, &key);
    }
    // 导出zip 文件使用
    pub async fn export_zip(&self, state: &Arc<State>, output_path: &Path) -> Result<(), Error> {
        if let Ok(buf) = create_full_snapshot(state) {
            let file = std::fs::File::create(output_path)?;
            let mut zip = zip::ZipWriter::new(file);
            // 1. 筛选需要保留的快照和增量
            zip.add_directory("snapshot", SimpleFileOptions::default())?;
            zip.start_file(
                format!("snapshot/snapshot_v_{}.bin", state.version),
                SimpleFileOptions::default(),
            )?;
            zip.write_all(&buf)?;
            // 2. 将筛选后的文件写入ZIP
            // 写入快照
            // 写入增量
            // 写入更新后的manifest（仅包含保留的快照）
            // 3. 完成并重命名
            zip.finish()?;
            println!("{:?}", output_path.display());
        }

        Ok(())
    }
}
