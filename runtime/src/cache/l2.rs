use std::fs;
use std::path::Path;

use moduforge_delta::{from_binary, to_binary};
// cache/l2.rs
use moduforge_core::model::node_pool::NodePool;
use rocksdb::{Options, DB};
use std::io::Error;
use std::sync::{Arc, Mutex};
use zstd::stream::decode_all;
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
        let _=fs::create_dir_all(path);
        Ok(Self {
            db: DB::open(&opts, path).unwrap(),
            compression_level: 3,
        })
    }

    /// 读取数据（自动解压）
    pub fn get(&self, key: &[u8]) -> Result<Arc<NodePool>, Error> {
        let compressed = self.db.get(key).unwrap().unwrap();
        let data = decode_all(&compressed[..])?;
        Ok(from_binary::<Arc<NodePool>>(&data).unwrap())
    }

    /// 写入数据（自动压缩）
    pub fn put(&self, key: &[u8], value: Arc<NodePool>) {
        let data = to_binary(value).unwrap();
        let compressed = zstd::encode_all(&data[..], self.compression_level).unwrap();
        self.db.put(key, compressed);
    }

    /// 批量写入优化
    pub fn batch_put(&self, batch: Vec<(Vec<u8>, NodePool)>) {
        let mut wb = rocksdb::WriteBatch::default();
        for (k, v) in batch {
            let data = to_binary(v).unwrap();
            let compressed = zstd::encode_all(&data[..], self.compression_level).unwrap();
            wb.put(k, compressed);
        }
        self.db.write(wb);
    }
}
