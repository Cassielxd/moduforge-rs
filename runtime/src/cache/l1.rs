// cache/l1.rs
use lru::LruCache;
use moduforge_core::model::node_pool::NodePool;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use super::CacheKey;
/// 基于LRU策略的内存缓存
#[derive(Debug)]
pub struct L1Cache {
    inner: Mutex<LruCache<CacheKey, Arc<NodePool>>>,
}

impl L1Cache {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Mutex::new(LruCache::new(NonZeroUsize::new(capacity).unwrap())),
        }
    }

    /// 读取缓存
    pub fn get(&self, key: &CacheKey) -> Option<Arc<NodePool>> {
        let mut guard = self.inner.lock().expect("获取锁失败");
        let value = guard.get(key).cloned();
        value
    }

    /// 写入缓存
    pub fn put(&self, key: CacheKey, value: Arc<NodePool>) {
        let mut guard = self.inner.lock().expect("获取锁失败");
        guard.put(key, value);
    }

    /// 淘汰策略
    pub fn evict(&self, count: usize) {
        let mut guard = self.inner.lock().expect("获取锁失败");
        for _ in 0..count {
            guard.pop_lru();
        }
    }
}
