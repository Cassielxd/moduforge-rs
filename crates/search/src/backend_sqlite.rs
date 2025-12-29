use crate::model::IndexDoc;
use anyhow::Result;
use rbatis::{executor::Executor, RBatis};
use rbdc_sqlite::Driver;
use rbs::Value;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

const SCHEMA_SQL: &str = r#"
    PRAGMA journal_mode=WAL;
    PRAGMA synchronous=NORMAL;
    PRAGMA cache_size=-64000;
    PRAGMA temp_store=MEMORY;

    CREATE TABLE IF NOT EXISTS nodes (
        id TEXT PRIMARY KEY,
        node_type TEXT NOT NULL,
        parent_id TEXT,
        path TEXT NOT NULL,
        marks TEXT,
        marks_json TEXT,
        attrs TEXT,
        attrs_json TEXT,
        text TEXT,
        order_i64 INTEGER,
        created_at_i64 INTEGER,
        updated_at_i64 INTEGER
    );

    CREATE INDEX IF NOT EXISTS idx_node_type ON nodes(node_type);
    CREATE INDEX IF NOT EXISTS idx_parent_id ON nodes(parent_id);
    CREATE INDEX IF NOT EXISTS idx_path ON nodes(path);
    CREATE INDEX IF NOT EXISTS idx_created_at ON nodes(created_at_i64);
    CREATE INDEX IF NOT EXISTS idx_updated_at ON nodes(updated_at_i64);
    CREATE INDEX IF NOT EXISTS idx_order ON nodes(order_i64);

    CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(
        id UNINDEXED,
        text,
        content='nodes',
        content_rowid='rowid'
    );

    CREATE TRIGGER IF NOT EXISTS nodes_ai AFTER INSERT ON nodes BEGIN
        INSERT INTO nodes_fts(rowid, id, text)
        VALUES (new.rowid, new.id, new.text);
    END;

    CREATE TRIGGER IF NOT EXISTS nodes_ad AFTER DELETE ON nodes BEGIN
        INSERT INTO nodes_fts(nodes_fts, rowid, id, text)
        VALUES('delete', old.rowid, old.id, old.text);
    END;

    CREATE TRIGGER IF NOT EXISTS nodes_au AFTER UPDATE ON nodes BEGIN
        INSERT INTO nodes_fts(nodes_fts, rowid, id, text)
        VALUES('delete', old.rowid, old.id, old.text);
        INSERT INTO nodes_fts(rowid, id, text)
        VALUES (new.rowid, new.id, new.text);
    END;
"#;

/// SQLite 后端索引增量变更
#[derive(Debug, Clone)]
pub enum IndexMutation {
    Add(IndexDoc),
    Upsert(IndexDoc),
    DeleteById(String),
    DeleteManyById(Vec<String>),
}

/// 查询条件
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    /// 全文查询（走 FTS5）
    pub text: Option<String>,
    /// 节点类型精确匹配
    pub node_type: Option<String>,
    /// 父节点精确匹配
    pub parent_id: Option<String>,
    /// 路径前缀匹配（如 "/root/section"）
    pub path_prefix: Option<String>,
    /// 标记类型（简单查询：只检查是否包含某类型 marks）
    pub marks: Vec<String>,
    /// 带属性的精确 mark 查询（mark_type, attr_key, attr_value）
    pub mark_attrs: Vec<(String, String, String)>,
    /// JSON 属性匹配（支持复杂查询）
    pub attrs: Vec<(String, String)>,
    /// 返回条数限制
    pub limit: usize,
    /// 偏移量
    pub offset: usize,
    /// 排序字段
    pub sort_by: Option<String>,
    /// 排序方向 true=升序，false=降序
    pub sort_asc: bool,
    /// 树形查询：返回子树所有节点
    pub include_descendants: bool,
    /// 范围查询
    pub range_field: Option<String>,
    pub range_min: Option<i64>,
    pub range_max: Option<i64>,
}

/// SQLite 后端实现
pub struct SqliteBackend {
    pool: Arc<RBatis>,
    index_dir: PathBuf,
    _temp_dir: Option<tempfile::TempDir>,
}

impl SqliteBackend {
    /// 在指定目录创建或打开数据库
    pub async fn new_in_dir(dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(dir)?;
        let db_path = dir.join("index.db");
        Self::new_with_path(db_path, None).await
    }

    /// 使用系统临时目录
    pub async fn new_in_system_temp() -> Result<Self> {
        let temp_dir = tempfile::Builder::new().prefix("index").tempdir()?;
        let db_path = temp_dir.path().join("index.db");
        Self::new_with_path(db_path, Some(temp_dir)).await
    }

    /// 在指定临时根目录下创建临时索引
    pub async fn new_in_temp_root(temp_root: &Path) -> Result<Self> {
        let temp_dir =
            tempfile::Builder::new().prefix("index").tempdir_in(temp_root)?;
        let db_path = temp_dir.path().join("index.db");
        Self::new_with_path(db_path, Some(temp_dir)).await
    }

    async fn new_with_path(
        db_path: PathBuf,
        temp_dir: Option<tempfile::TempDir>,
    ) -> Result<Self> {
        let rb = RBatis::new();
        rb.link(Driver {}, &format!("sqlite://{}", db_path.display())).await?;
        rb.acquire().await?.exec(SCHEMA_SQL, vec![]).await?;

        Ok(Self {
            pool: Arc::new(rb),
            index_dir: db_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from(".")),
            _temp_dir: temp_dir,
        })
    }

    /// 获取索引目录
    pub fn index_dir(&self) -> &Path {
        &self.index_dir
    }

    /// 应用增量变更
    pub async fn apply(
        &self,
        mutations: Vec<IndexMutation>,
    ) -> Result<()> {
        if mutations.is_empty() {
            return Ok(());
        }

        let tx = self.pool.acquire_begin().await?;
        for mutation in mutations {
            match mutation {
                IndexMutation::Add(doc) | IndexMutation::Upsert(doc) => {
                    self.upsert_doc(&tx, &doc).await?;
                },
                IndexMutation::DeleteById(id) => {
                    tx.exec(
                        "DELETE FROM nodes WHERE id = ?1",
                        vec![to_value(id)],
                    )
                    .await?;
                },
                IndexMutation::DeleteManyById(ids) => {
                    for id in ids {
                        tx.exec(
                            "DELETE FROM nodes WHERE id = ?1",
                            vec![to_value(id)],
                        )
                        .await?;
                    }
                },
            }
        }
        tx.commit().await?;
        Ok(())
    }

    async fn upsert_doc<E>(
        &self,
        exec: &E,
        doc: &IndexDoc,
    ) -> Result<()>
    where
        E: Executor + ?Sized,
    {
        let marks_types_json = serde_json::to_string(&doc.marks)?;
        let attrs_map: serde_json::Map<String, serde_json::Value> = doc
            .attrs_flat
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();
        let attrs_flat_json = serde_json::to_string(&attrs_map)?;
        let path_str = format!("/{}", doc.path.join("/"));

        exec.exec(
            "INSERT OR REPLACE INTO nodes
             (id, node_type, parent_id, path, marks, marks_json, attrs, attrs_json, text,
              order_i64, created_at_i64, updated_at_i64)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            vec![
                to_value(doc.node_id.clone()),
                to_value(doc.node_type.clone()),
                to_value(doc.parent_id.clone()),
                to_value(path_str),
                to_value(marks_types_json),
                to_value(doc.marks_json.clone()),
                to_value(attrs_flat_json),
                to_value(doc.attrs_json.clone()),
                to_value(doc.text.clone()),
                to_value(doc.order_i64),
                to_value(doc.created_at_i64),
                to_value(doc.updated_at_i64),
            ],
        )
        .await?;
        Ok(())
    }

    /// 重建全部索引
    pub async fn rebuild_all(
        &self,
        docs: Vec<IndexDoc>,
    ) -> Result<()> {
        let tx = self.pool.acquire_begin().await?;
        tx.exec("DELETE FROM nodes", vec![]).await?;
        for doc in &docs {
            self.upsert_doc(&tx, doc).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// 搜索节点 ID
    pub async fn search_ids(
        &self,
        query: SearchQuery,
    ) -> Result<Vec<String>> {
        if query.include_descendants && query.parent_id.is_some() {
            return self.search_tree(&query).await;
        }
        if query.text.is_some() {
            return self.search_fulltext(&query).await;
        }
        self.search_structured(&query).await
    }

    /// 搜索并返回完整文档
    pub async fn search_docs(
        &self,
        query: SearchQuery,
    ) -> Result<Vec<IndexDoc>> {
        let ids = self.search_ids(query).await?;
        self.get_docs_by_ids(&ids).await
    }

    /// 根据 ID 列表获取完整文档
    pub async fn get_docs_by_ids(
        &self,
        ids: &[String],
    ) -> Result<Vec<IndexDoc>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = std::iter::repeat("?")
            .take(ids.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT id, node_type, parent_id, path, marks_json, attrs_json, text,
                    order_i64, created_at_i64, updated_at_i64
             FROM nodes WHERE id IN ({})",
            placeholders
        );
        let params = ids.iter().cloned().map(to_value).collect::<Vec<Value>>();

        let conn = self.pool.acquire().await?;
        let rows: Vec<NodeRow> = conn.query_decode(&sql, params).await?;
        let mut docs_by_id: HashMap<String, IndexDoc> =
            HashMap::with_capacity(rows.len());
        for row in rows {
            let doc = IndexDoc::try_from(row)?;
            docs_by_id.insert(doc.node_id.clone(), doc);
        }

        let mut ordered = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(doc) = docs_by_id.remove(id.as_str()) {
                ordered.push(doc);
            }
        }
        Ok(ordered)
    }

    async fn search_tree(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<String>> {
        let parent_id = query.parent_id.as_ref().unwrap();
        let mut sql = String::from(
            "WITH RECURSIVE tree(id, level) AS (
                SELECT id, 0 as level FROM nodes WHERE id = ?1
                UNION ALL
                SELECT n.id, t.level + 1
                FROM nodes n
                JOIN tree t ON n.parent_id = t.id
                WHERE t.level < 100
            )
            SELECT id FROM tree WHERE 1=1",
        );

        let mut params = vec![to_value(parent_id.clone())];
        if let Some(node_type) = &query.node_type {
            sql.push_str(
                " AND id IN (SELECT id FROM nodes WHERE node_type = ?)",
            );
            params.push(to_value(node_type.clone()));
        }

        if let Some(sort_by) = &query.sort_by {
            let direction = if query.sort_asc { "ASC" } else { "DESC" };
            sql.push_str(&format!(
                " ORDER BY (SELECT {} FROM nodes WHERE nodes.id = tree.id) {}",
                sort_by, direction
            ));
        }

        let limit = if query.limit == 0 { 1000 } else { query.limit };
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, query.offset));

        let conn = self.pool.acquire().await?;
        let rows: Vec<IdRow> = conn.query_decode(&sql, params).await?;
        Ok(rows.into_iter().map(|r| r.id).collect())
    }

    async fn search_fulltext(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<String>> {
        let text = query.text.as_ref().unwrap();
        let mut sql = String::from(
            "SELECT nodes.id FROM nodes_fts
             JOIN nodes ON nodes_fts.id = nodes.id
             WHERE nodes_fts.text MATCH ?",
        );
        let mut params = vec![to_value(text.clone())];

        if let Some(node_type) = &query.node_type {
            sql.push_str(" AND nodes.node_type = ?");
            params.push(to_value(node_type.clone()));
        }
        if let Some(parent_id) = &query.parent_id {
            sql.push_str(" AND nodes.parent_id = ?");
            params.push(to_value(parent_id.clone()));
        }

        for mark in &query.marks {
            sql.push_str(" AND nodes.marks LIKE ?");
            params.push(to_value(format!("%\"{}\"%%", mark)));
        }

        for (mark_type, attr_key, attr_value) in &query.mark_attrs {
            sql.push_str(&format!(
                " AND EXISTS (
                    SELECT 1 FROM json_each(nodes.marks_json)
                    WHERE json_extract(value, '$.type') = ?
                    AND json_extract(value, '$.attrs.{}') = ?
                )",
                attr_key
            ));
            params.push(to_value(mark_type.clone()));
            params.push(to_value(attr_value.clone()));
        }

        if let Some(sort_by) = &query.sort_by {
            let direction = if query.sort_asc { "ASC" } else { "DESC" };
            sql.push_str(&format!(" ORDER BY nodes.{} {}", sort_by, direction));
        } else {
            sql.push_str(" ORDER BY rank");
        }

        let limit = if query.limit == 0 { 50 } else { query.limit };
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, query.offset));

        let conn = self.pool.acquire().await?;
        let rows: Vec<IdRow> = conn.query_decode(&sql, params).await?;
        Ok(rows.into_iter().map(|r| r.id).collect())
    }

    async fn search_structured(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<String>> {
        let mut sql = String::from("SELECT id FROM nodes WHERE 1=1");
        let mut params: Vec<Value> = Vec::new();

        if let Some(node_type) = &query.node_type {
            sql.push_str(" AND node_type = ?");
            params.push(to_value(node_type.clone()));
        }
        if let Some(parent_id) = &query.parent_id {
            sql.push_str(" AND parent_id = ?");
            params.push(to_value(parent_id.clone()));
        }
        if let Some(path_prefix) = &query.path_prefix {
            sql.push_str(" AND path LIKE ?");
            params.push(to_value(format!("{}%", path_prefix)));
        }

        for mark in &query.marks {
            sql.push_str(" AND marks LIKE ?");
            params.push(to_value(format!("%\"{}\"%%", mark)));
        }

        for (mark_type, attr_key, attr_value) in &query.mark_attrs {
            sql.push_str(&format!(
                " AND EXISTS (
                    SELECT 1 FROM json_each(marks_json)
                    WHERE json_extract(value, '$.type') = ?
                    AND json_extract(value, '$.attrs.{}') = ?
                )",
                attr_key
            ));
            params.push(to_value(mark_type.clone()));
            params.push(to_value(attr_value.clone()));
        }

        for (key, value) in &query.attrs {
            sql.push_str(&format!(" AND json_extract(attrs, '$.{}') = ?", key));
            params.push(to_value(value.clone()));
        }

        if let Some(field) = &query.range_field {
            if let Some(min) = query.range_min {
                sql.push_str(&format!(" AND {} >= ?", field));
                params.push(to_value(min));
            }
            if let Some(max) = query.range_max {
                sql.push_str(&format!(" AND {} <= ?", field));
                params.push(to_value(max));
            }
        }

        if let Some(sort_by) = &query.sort_by {
            let direction = if query.sort_asc { "ASC" } else { "DESC" };
            sql.push_str(&format!(" ORDER BY {} {}", sort_by, direction));
        }

        let limit = if query.limit == 0 { 50 } else { query.limit };
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, query.offset));

        let conn = self.pool.acquire().await?;
        let rows: Vec<IdRow> = conn.query_decode(&sql, params).await?;
        Ok(rows.into_iter().map(|r| r.id).collect())
    }
}

fn to_value<T: Serialize>(value: T) -> Value {
    rbs::value_def(value)
}

#[derive(Debug, Deserialize)]
struct IdRow {
    id: String,
}

#[derive(Debug, Deserialize)]
struct NodeRow {
    id: String,
    node_type: String,
    parent_id: Option<String>,
    path: String,
    marks_json: serde_json::Value,
    attrs_json: serde_json::Value,
    text: Option<String>,
    order_i64: Option<i64>,
    created_at_i64: Option<i64>,
    updated_at_i64: Option<i64>,
}

impl TryFrom<NodeRow> for IndexDoc {
    type Error = anyhow::Error;

    fn try_from(row: NodeRow) -> Result<Self> {
        let marks_json_str = json_to_string(&row.marks_json);
        let attrs_json_str = json_to_string(&row.attrs_json);

        Ok(IndexDoc {
            node_id: row.id,
            node_type: row.node_type,
            parent_id: row.parent_id,
            path: parse_path(&row.path),
            marks: marks_from_value(&row.marks_json),
            marks_json: marks_json_str,
            attrs_flat: flatten_attrs(&row.attrs_json),
            attrs_json: attrs_json_str,
            text: row.text,
            order_i64: row.order_i64,
            created_at_i64: row.created_at_i64,
            updated_at_i64: row.updated_at_i64,
        })
    }
}

fn parse_path(path: &str) -> Vec<String> {
    path.trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn marks_from_value(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|mark| {
                mark.get("type").and_then(|v| v.as_str()).map(ToOwned::to_owned)
            })
            .collect(),
        serde_json::Value::String(raw) => serde_json::from_str::<
            serde_json::Value,
        >(raw)
        .ok()
        .and_then(|parsed| parsed.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|mark| {
            mark.get("type").and_then(|v| v.as_str()).map(ToOwned::to_owned)
        })
        .collect(),
        _ => Vec::new(),
    }
}

fn flatten_attrs(value: &serde_json::Value) -> Vec<(String, String)> {
    match value {
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(k, v)| (k.clone(), json_value_to_string(v)))
            .collect(),
        serde_json::Value::String(raw) => serde_json::from_str::<
            serde_json::Map<String, serde_json::Value>,
        >(raw)
        .map(|map| {
            map.into_iter()
                .map(|(k, v)| (k, json_value_to_string(&v)))
                .collect()
        })
        .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn json_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn json_value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_operations() {
        let backend = SqliteBackend::new_in_system_temp().await.unwrap();

        let doc = IndexDoc {
            node_id: "test1".to_string(),
            node_type: "paragraph".to_string(),
            parent_id: Some("root".to_string()),
            path: vec!["root".to_string(), "test1".to_string()],
            marks: vec!["bold".to_string()],
            marks_json: r#"[{"type":"bold","attrs":{}}]"#.to_string(),
            attrs_flat: vec![("status".to_string(), "published".to_string())],
            attrs_json: r#"{"status":"published"}"#.to_string(),
            text: Some("测试文本".to_string()),
            order_i64: Some(1),
            created_at_i64: Some(1000),
            updated_at_i64: Some(2000),
        };

        backend.apply(vec![IndexMutation::Add(doc.clone())]).await.unwrap();

        let ids = backend
            .search_ids(SearchQuery {
                node_type: Some("paragraph".to_string()),
                limit: 10,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(ids, vec!["test1"]);

        let docs = backend.get_docs_by_ids(&ids).await.unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].node_id, "test1");
    }

    #[tokio::test]
    async fn test_tree_query() {
        let backend = SqliteBackend::new_in_system_temp().await.unwrap();

        let docs = vec![
            IndexDoc {
                node_id: "root".to_string(),
                node_type: "doc".to_string(),
                parent_id: None,
                path: vec!["root".to_string()],
                marks: vec![],
                marks_json: "[]".to_string(),
                attrs_flat: vec![],
                attrs_json: "{}".to_string(),
                text: None,
                order_i64: None,
                created_at_i64: None,
                updated_at_i64: None,
            },
            IndexDoc {
                node_id: "child1".to_string(),
                node_type: "paragraph".to_string(),
                parent_id: Some("root".to_string()),
                path: vec!["root".to_string(), "child1".to_string()],
                marks: vec![],
                marks_json: "[]".to_string(),
                attrs_flat: vec![],
                attrs_json: "{}".to_string(),
                text: None,
                order_i64: None,
                created_at_i64: None,
                updated_at_i64: None,
            },
            IndexDoc {
                node_id: "child2".to_string(),
                node_type: "paragraph".to_string(),
                parent_id: Some("root".to_string()),
                path: vec!["root".to_string(), "child2".to_string()],
                marks: vec![],
                marks_json: "[]".to_string(),
                attrs_flat: vec![],
                attrs_json: "{}".to_string(),
                text: None,
                order_i64: None,
                created_at_i64: None,
                updated_at_i64: None,
            },
        ];

        backend.rebuild_all(docs).await.unwrap();

        let results = backend
            .search_ids(SearchQuery {
                parent_id: Some("root".to_string()),
                include_descendants: true,
                limit: 100,
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 3); // root + 2 children
    }

    #[tokio::test]
    async fn test_search_docs() {
        let backend = SqliteBackend::new_in_system_temp().await.unwrap();

        let docs = vec![
            IndexDoc {
                node_id: "doc1".to_string(),
                node_type: "paragraph".to_string(),
                parent_id: Some("root".to_string()),
                path: vec!["root".to_string(), "doc1".to_string()],
                marks: vec!["bold".to_string()],
                marks_json: r#"[{"type":"bold","attrs":{}}]"#.to_string(),
                attrs_flat: vec![(
                    "status".to_string(),
                    "published".to_string(),
                )],
                attrs_json: r#"{"status":"published"}"#.to_string(),
                text: Some("第一篇文章".to_string()),
                order_i64: Some(1),
                created_at_i64: Some(1000),
                updated_at_i64: Some(1500),
            },
            IndexDoc {
                node_id: "doc2".to_string(),
                node_type: "paragraph".to_string(),
                parent_id: Some("root".to_string()),
                path: vec!["root".to_string(), "doc2".to_string()],
                marks: vec!["italic".to_string()],
                marks_json: r#"[{"type":"italic","attrs":{}}]"#.to_string(),
                attrs_flat: vec![("status".to_string(), "draft".to_string())],
                attrs_json: r#"{"status":"draft"}"#.to_string(),
                text: Some("第二篇文章".to_string()),
                order_i64: Some(2),
                created_at_i64: Some(2000),
                updated_at_i64: Some(2500),
            },
        ];

        backend.rebuild_all(docs).await.unwrap();

        let results = backend
            .search_docs(SearchQuery {
                node_type: Some("paragraph".to_string()),
                limit: 10,
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        let doc1 = results.iter().find(|d| d.node_id == "doc1").unwrap();
        assert_eq!(doc1.text, Some("第一篇文章".to_string()));
        assert_eq!(doc1.marks, vec!["bold".to_string()]);

        let doc2 = results.iter().find(|d| d.node_id == "doc2").unwrap();
        assert_eq!(doc2.text, Some("第二篇文章".to_string()));
        assert_eq!(doc2.marks, vec!["italic".to_string()]);
    }

    #[tokio::test]
    async fn test_get_docs_by_ids() {
        let backend = SqliteBackend::new_in_system_temp().await.unwrap();

        let doc = IndexDoc {
            node_id: "test123".to_string(),
            node_type: "heading".to_string(),
            parent_id: None,
            path: vec!["test123".to_string()],
            marks: vec![],
            marks_json: "[]".to_string(),
            attrs_flat: vec![("level".to_string(), "1".to_string())],
            attrs_json: r#"{"level":"1"}"#.to_string(),
            text: Some("标题文本".to_string()),
            order_i64: None,
            created_at_i64: None,
            updated_at_i64: None,
        };

        backend.apply(vec![IndexMutation::Add(doc)]).await.unwrap();

        let docs =
            backend.get_docs_by_ids(&["test123".to_string()]).await.unwrap();

        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].node_id, "test123");
        assert_eq!(docs[0].node_type, "heading");
        assert_eq!(docs[0].text, Some("标题文本".to_string()));

        let empty = backend.get_docs_by_ids(&[]).await.unwrap();
        assert_eq!(empty.len(), 0);
    }
}
