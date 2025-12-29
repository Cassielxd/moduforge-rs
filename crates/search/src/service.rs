use crate::backend::SqliteBackend;
use crate::indexer::mutations_from_step;
use crate::model::IndexDoc;
use anyhow::Result;
use mf_model::node_pool::NodePool;
use mf_model::schema::Schema;
use mf_transform::step::StepGeneric;
use std::sync::Arc;
use mf_state::transaction::Transaction;

/// 外部可投递的事件
#[derive(Debug, Clone)]
pub enum IndexEvent {
    /// 增量：来自单个 step（可选地提供变更前的 `NodePool` 以支持删除子树精确处理）
    StepApplied {
        pool_before: Option<Arc<NodePool>>,
        pool_after: Arc<NodePool>,
        step: Arc<dyn StepGeneric<NodePool, Schema>>,
    },
    /// 增量：来自一整个事务（可选地提供 before 池）
    TransactionCommitted {
        pool_before: Option<Arc<NodePool>>,
        pool_after: Arc<NodePool>,
        steps: Vec<Arc<dyn StepGeneric<NodePool, Schema>>>,
    },
    /// 全量重建
    Rebuild { pool: Arc<NodePool>, scope: RebuildScope },
}

#[derive(Debug, Clone, Copy)]
pub enum RebuildScope {
    Full,
}

/// 索引服务：桥接 `Transaction/Step` 与后端
pub struct IndexService {
    backend: Arc<SqliteBackend>,
}

impl IndexService {
    pub fn new(backend: Arc<SqliteBackend>) -> Self {
        Self { backend }
    }

    /// 处理事件（调度后端执行）
    pub async fn handle(
        &self,
        event: IndexEvent,
    ) -> Result<()> {
        match event {
            IndexEvent::StepApplied { pool_before, pool_after, step } => {
                let pool_b = pool_before.as_deref().unwrap_or(&pool_after);
                let muts = mutations_from_step(pool_b, &pool_after, &step);
                self.backend.apply(muts).await
            },
            IndexEvent::TransactionCommitted {
                pool_before,
                pool_after,
                steps,
            } => {
                let pool_b = pool_before.as_deref().unwrap_or(&pool_after);
                // 合并事务中所有 step 的增量（可能有覆盖）
                let mut all = Vec::new();
                for s in &steps {
                    all.extend(mutations_from_step(pool_b, &pool_after, s));
                }
                self.backend.apply(all).await
            },
            IndexEvent::Rebuild { pool, scope: RebuildScope::Full } => {
                // 并行/顺序遍历整个池，构造文档集合
                let mut docs: Vec<IndexDoc> = Vec::new();
                for shard in &pool.get_inner().nodes {
                    for node in shard.values() {
                        docs.push(IndexDoc::from_node(&pool, node));
                    }
                }
                self.backend.rebuild_all(docs).await
            },
        }
    }
}

/// 搜索服务：提供高层查询接口
pub struct SearchService {
    backend: Arc<SqliteBackend>,
}

impl SearchService {
    pub fn new(backend: Arc<SqliteBackend>) -> Self {
        Self { backend }
    }

    /// 简单查询：返回节点 ID 列表
    pub async fn search(
        &self,
        query: crate::backend::SearchQuery,
    ) -> Result<Vec<String>> {
        self.backend.search_ids(query).await
    }

    /// 查询并返回完整文档
    pub async fn search_docs(
        &self,
        query: crate::backend::SearchQuery,
    ) -> Result<Vec<IndexDoc>> {
        self.backend.search_docs(query).await
    }

    /// 全文搜索
    pub async fn search_text(
        &self,
        text: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        self.backend
            .search_ids(crate::backend::SearchQuery {
                text: Some(text.to_string()),
                limit,
                ..Default::default()
            })
            .await
    }

    /// 全文搜索（返回完整文档）
    pub async fn search_text_docs(
        &self,
        text: &str,
        limit: usize,
    ) -> Result<Vec<IndexDoc>> {
        self.backend
            .search_docs(crate::backend::SearchQuery {
                text: Some(text.to_string()),
                limit,
                ..Default::default()
            })
            .await
    }

    /// 查询子树（递归）
    pub async fn query_descendants(
        &self,
        parent_id: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        self.backend
            .search_ids(crate::backend::SearchQuery {
                parent_id: Some(parent_id.to_string()),
                include_descendants: true,
                limit,
                ..Default::default()
            })
            .await
    }

    /// 按类型查询
    pub async fn query_by_type(
        &self,
        node_type: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        self.backend
            .search_ids(crate::backend::SearchQuery {
                node_type: Some(node_type.to_string()),
                limit,
                ..Default::default()
            })
            .await
    }
}

#[allow(dead_code)]
pub fn event_from_transaction(
    pool_after: Arc<NodePool>,
    tr: &Transaction,
) -> IndexEvent {
    let steps: Vec<Arc<dyn StepGeneric<NodePool, Schema>>> = tr.steps.iter().cloned().collect();
    IndexEvent::TransactionCommitted { pool_before: None, pool_after, steps }
}
