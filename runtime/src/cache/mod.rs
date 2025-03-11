pub mod cache;
pub mod l1;
pub mod l2;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
  pub doc_id: String, // 文档唯一标识
  pub version: u64,
  pub time: u64,
}
