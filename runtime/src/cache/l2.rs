use std::fs;
use std::path::Path;

// cache/l2.rs
use moduforge_core::model::node_pool::NodePool;
use rocksdb::{Options, DB};
use std::io::Error;
use std::sync::Arc;

use crate::delta::{from_binary, to_binary};
/// 基于RocksDB的磁盘缓存
#[derive(Debug)]
pub struct L2Cache {
    db: DB,
    compression_level: i32,
}

impl L2Cache {
    pub fn open(path: &Path) -> Result<Self, Error> {
        let mut opts = Options::default();
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        opts.create_if_missing(true);
        let _ = fs::create_dir_all(path);
        Ok(Self {
            db: DB::open(&opts, path).unwrap(),
            compression_level: 3,
        })
    }

    /// 读取数据（自动解压）
    pub fn get(&self, key: String) -> Result<Arc<NodePool>, Error> {
        let data = self.db.get(key).unwrap().unwrap();
        Ok(from_binary::<Arc<NodePool>>(&data).unwrap())
    }

    /// 写入数据（自动压缩）
    pub fn put(&self, key: String, value: Arc<NodePool>) {
        let data = to_binary(value).unwrap();
        let _ = self.db.put(key, data);
    }

    /// 批量写入优化
    pub fn batch_put(&self, batch: Vec<(Vec<u8>, NodePool)>) {
        let mut wb = rocksdb::WriteBatch::default();
        for (k, v) in batch {
            let data = to_binary(v).unwrap();
            wb.put(k, data);
        }
        let _ = self.db.write(wb);
    }
}
