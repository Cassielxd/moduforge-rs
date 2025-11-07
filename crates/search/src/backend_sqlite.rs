use crate::model::IndexDoc;
use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::TransactionBehavior;
use std::path::PathBuf;

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
    /// 标记类型（简单查询：只检查是否包含某类型 mark）
    pub marks: Vec<String>,
    /// 带属性的精确 mark 查询（mark_type, attr_key, attr_value）
    /// 例如：("link", "href", "https://example.com")
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
    pool: Pool<SqliteConnectionManager>,
    index_dir: PathBuf,
    /// 临时目录对象（如果使用临时目录）
    /// 保存此对象以确保临时目录在 Backend 生命周期内不被删除
    _temp_dir: Option<tempfile::TempDir>,
}

impl SqliteBackend {
    /// 在指定目录创建或打开数据库
    pub fn new_in_dir(dir: &std::path::Path) -> Result<Self> {
        std::fs::create_dir_all(dir)?;
        let db_path = dir.join("index.db");

        // 创建连接池管理器
        let manager = SqliteConnectionManager::file(&db_path);
        let pool = Pool::new(manager)?;

        // 初始化表结构
        pool.get()?.execute_batch(
            "-- 开启 WAL 模式（高并发）
             PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA cache_size=-64000;  -- 64MB cache
             PRAGMA temp_store=MEMORY;

             -- 创建主表
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

             -- 创建索引
             CREATE INDEX IF NOT EXISTS idx_node_type ON nodes(node_type);
             CREATE INDEX IF NOT EXISTS idx_parent_id ON nodes(parent_id);
             CREATE INDEX IF NOT EXISTS idx_path ON nodes(path);
             CREATE INDEX IF NOT EXISTS idx_created_at ON nodes(created_at_i64);
             CREATE INDEX IF NOT EXISTS idx_updated_at ON nodes(updated_at_i64);
             CREATE INDEX IF NOT EXISTS idx_order ON nodes(order_i64);

             -- 创建 FTS5 全文索引表
             CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(
                 id UNINDEXED,
                 text,
                 content='nodes',
                 content_rowid='rowid'
             );

             -- 创建触发器：自动同步 FTS5
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
             END;"
        )?;

        Ok(Self {
            pool,
            index_dir: dir.to_path_buf(),
            _temp_dir: None,  // 持久化目录，不使用临时目录
        })
    }

    /// 使用系统临时目录
    pub fn new_in_system_temp() -> Result<Self> {
        let temp_dir = tempfile::Builder::new()
            .prefix("mf_index_")
            .tempdir()?;

        let db_path = temp_dir.path().join("index.db");

        // 创建连接池管理器
        let manager = SqliteConnectionManager::file(&db_path);
        let pool = Pool::new(manager)?;

        // 初始化表结构
        pool.get()?.execute_batch(
            "-- 开启 WAL 模式（高并发）
             PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA cache_size=-64000;  -- 64MB cache
             PRAGMA temp_store=MEMORY;

             -- 创建主表
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

             -- 创建索引
             CREATE INDEX IF NOT EXISTS idx_node_type ON nodes(node_type);
             CREATE INDEX IF NOT EXISTS idx_parent_id ON nodes(parent_id);
             CREATE INDEX IF NOT EXISTS idx_path ON nodes(path);
             CREATE INDEX IF NOT EXISTS idx_created_at ON nodes(created_at_i64);
             CREATE INDEX IF NOT EXISTS idx_updated_at ON nodes(updated_at_i64);
             CREATE INDEX IF NOT EXISTS idx_order ON nodes(order_i64);

             -- 创建 FTS5 全文索引表
             CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(
                 id UNINDEXED,
                 text,
                 content='nodes',
                 content_rowid='rowid'
             );

             -- 创建触发器：自动同步 FTS5
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
             END;"
        )?;

        Ok(Self {
            pool,
            index_dir: temp_dir.path().to_path_buf(),
            _temp_dir: Some(temp_dir),  // 保存临时目录对象，确保生命周期内不被删除
        })
    }

    /// 在指定临时根目录下创建
    pub fn new_in_temp_root(temp_root: &std::path::Path) -> Result<Self> {
        let temp_dir = tempfile::Builder::new()
            .prefix("mf_index_")
            .tempdir_in(temp_root)?;

        let db_path = temp_dir.path().join("index.db");

        // 创建连接池管理器
        let manager = SqliteConnectionManager::file(&db_path);
        let pool = Pool::new(manager)?;

        // 初始化表结构
        pool.get()?.execute_batch(
            "-- 开启 WAL 模式（高并发）
             PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA cache_size=-64000;  -- 64MB cache
             PRAGMA temp_store=MEMORY;

             -- 创建主表
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

             -- 创建索引
             CREATE INDEX IF NOT EXISTS idx_node_type ON nodes(node_type);
             CREATE INDEX IF NOT EXISTS idx_parent_id ON nodes(parent_id);
             CREATE INDEX IF NOT EXISTS idx_path ON nodes(path);
             CREATE INDEX IF NOT EXISTS idx_created_at ON nodes(created_at_i64);
             CREATE INDEX IF NOT EXISTS idx_updated_at ON nodes(updated_at_i64);
             CREATE INDEX IF NOT EXISTS idx_order ON nodes(order_i64);

             -- 创建 FTS5 全文索引表
             CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(
                 id UNINDEXED,
                 text,
                 content='nodes',
                 content_rowid='rowid'
             );

             -- 创建触发器：自动同步 FTS5
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
             END;"
        )?;

        Ok(Self {
            pool,
            index_dir: temp_dir.path().to_path_buf(),
            _temp_dir: Some(temp_dir),  // 保存临时目录对象，确保生命周期内不被删除
        })
    }

    /// 获取索引目录
    pub fn index_dir(&self) -> &std::path::Path {
        &self.index_dir
    }

    /// 应用增量变更
    pub async fn apply(&self, mutations: Vec<IndexMutation>) -> Result<()> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

        for mutation in mutations {
            match mutation {
                IndexMutation::Add(doc) | IndexMutation::Upsert(doc) => {
                    self.upsert_doc(&tx, &doc)?;
                }
                IndexMutation::DeleteById(id) => {
                    tx.execute("DELETE FROM nodes WHERE id = ?", [&id])?;
                }
                IndexMutation::DeleteManyById(ids) => {
                    for id in ids {
                        tx.execute("DELETE FROM nodes WHERE id = ?", [&id])?;
                    }
                }
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// 插入或更新文档
    fn upsert_doc(
        &self,
        tx: &rusqlite::Transaction,
        doc: &IndexDoc,
    ) -> Result<()> {
        let marks_types_json = serde_json::to_string(&doc.marks)?;
        let attrs_flat_json = serde_json::to_string(&doc.attrs_flat)?;
        let path_str = format!("/{}", doc.path.join("/"));

        tx.execute(
            "INSERT OR REPLACE INTO nodes
             (id, node_type, parent_id, path, marks, marks_json, attrs, attrs_json, text,
              order_i64, created_at_i64, updated_at_i64)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                &doc.node_id,
                &doc.node_type,
                &doc.parent_id,
                &path_str,
                &marks_types_json,
                &doc.marks_json,
                &attrs_flat_json,
                &doc.attrs_json,
                &doc.text,
                doc.order_i64,
                doc.created_at_i64,
                doc.updated_at_i64,
            ],
        )?;

        Ok(())
    }

    /// 重建全部索引
    pub async fn rebuild_all(&self, docs: Vec<IndexDoc>) -> Result<()> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

        // 清空表
        tx.execute("DELETE FROM nodes", [])?;

        // 批量插入
        for doc in docs {
            self.upsert_doc(&tx, &doc)?;
        }

        tx.commit()?;
        Ok(())
    }

    /// 搜索节点 ID
    pub async fn search_ids(&self, query: SearchQuery) -> Result<Vec<String>> {
        let conn = self.pool.get()?;

        // 树形查询
        if query.include_descendants && query.parent_id.is_some() {
            return self.search_tree(&conn, &query);
        }

        // 全文搜索
        if query.text.is_some() {
            return self.search_fulltext(&conn, &query);
        }

        // 结构化查询
        self.search_structured(&conn, &query)
    }

    /// 树形递归查询
    fn search_tree(
        &self,
        conn: &rusqlite::Connection,
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
            SELECT id FROM tree WHERE 1=1"
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(parent_id.clone())];

        // 添加过滤条件
        if let Some(node_type) = &query.node_type {
            sql.push_str(" AND id IN (SELECT id FROM nodes WHERE node_type = ?)");
            params.push(Box::new(node_type.clone()));
        }

        // 排序和分页
        if let Some(sort_by) = &query.sort_by {
            let direction = if query.sort_asc { "ASC" } else { "DESC" };
            sql.push_str(&format!(
                " ORDER BY (SELECT {} FROM nodes WHERE nodes.id = tree.id) {}",
                sort_by, direction
            ));
        }

        let limit = if query.limit == 0 { 1000 } else { query.limit };
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, query.offset));

        let mut stmt = conn.prepare(&sql)?;
        let params_ref: Vec<&dyn rusqlite::ToSql> = params
            .iter()
            .map(|p| p.as_ref() as &dyn rusqlite::ToSql)
            .collect();

        let ids: Vec<String> = stmt
            .query_map(&params_ref[..], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ids)
    }

    /// 全文搜索
    fn search_fulltext(
        &self,
        conn: &rusqlite::Connection,
        query: &SearchQuery,
    ) -> Result<Vec<String>> {
        let text = query.text.as_ref().unwrap();

        let mut sql = String::from(
            "SELECT nodes.id FROM nodes_fts
             JOIN nodes ON nodes_fts.id = nodes.id
             WHERE nodes_fts.text MATCH ?1"
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(text.clone())];
        let mut param_index = 2;

        // 添加过滤条件
        if let Some(node_type) = &query.node_type {
            sql.push_str(&format!(" AND nodes.node_type = ?{}", param_index));
            params.push(Box::new(node_type.clone()));
            param_index += 1;
        }

        if let Some(parent_id) = &query.parent_id {
            sql.push_str(&format!(" AND nodes.parent_id = ?{}", param_index));
            params.push(Box::new(parent_id.clone()));
            param_index += 1;
        }

        // 标记过滤
        for mark in &query.marks {
            sql.push_str(&format!(" AND nodes.marks LIKE ?{}", param_index));
            params.push(Box::new(format!("%\"{}\"%%", mark)));
            param_index += 1;
        }

        // 排序
        if let Some(sort_by) = &query.sort_by {
            let direction = if query.sort_asc { "ASC" } else { "DESC" };
            sql.push_str(&format!(" ORDER BY nodes.{} {}", sort_by, direction));
        } else {
            sql.push_str(" ORDER BY rank"); // FTS5 相关性排序
        }

        let limit = if query.limit == 0 { 50 } else { query.limit };
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, query.offset));

        let mut stmt = conn.prepare(&sql)?;
        let params_ref: Vec<&dyn rusqlite::ToSql> = params
            .iter()
            .map(|p| p.as_ref() as &dyn rusqlite::ToSql)
            .collect();

        let ids: Vec<String> = stmt
            .query_map(&params_ref[..], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ids)
    }

    /// 结构化查询
    fn search_structured(
        &self,
        conn: &rusqlite::Connection,
        query: &SearchQuery,
    ) -> Result<Vec<String>> {
        let mut sql = String::from("SELECT id FROM nodes WHERE 1=1");
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
        let mut param_index = 1;

        // 节点类型
        if let Some(node_type) = &query.node_type {
            sql.push_str(&format!(" AND node_type = ?{}", param_index));
            params.push(Box::new(node_type.clone()));
            param_index += 1;
        }

        // 父节点
        if let Some(parent_id) = &query.parent_id {
            sql.push_str(&format!(" AND parent_id = ?{}", param_index));
            params.push(Box::new(parent_id.clone()));
            param_index += 1;
        }

        // 路径前缀
        if let Some(path_prefix) = &query.path_prefix {
            sql.push_str(&format!(" AND path LIKE ?{}", param_index));
            params.push(Box::new(format!("{}%", path_prefix)));
            param_index += 1;
        }

        // 标记过滤（简单：只检查类型）
        for mark in &query.marks {
            sql.push_str(&format!(" AND marks LIKE ?{}", param_index));
            params.push(Box::new(format!("%\"{}\"%%", mark)));
            param_index += 1;
        }

        // 带属性的精确 mark 查询（使用 marks_json）
        for (mark_type, attr_key, attr_value) in &query.mark_attrs {
            // 使用 json_each 查询 marks_json 数组中符合条件的 mark
            sql.push_str(&format!(
                " AND EXISTS (
                    SELECT 1 FROM json_each(marks_json)
                    WHERE json_extract(value, '$.type') = ?{}
                    AND json_extract(value, '$.attrs.{}') = ?{}
                )",
                param_index, attr_key, param_index + 1
            ));
            params.push(Box::new(mark_type.clone()));
            params.push(Box::new(attr_value.clone()));
            param_index += 2;
        }

        // 属性过滤（简单：使用 attrs 字段）
        for (key, value) in &query.attrs {
            sql.push_str(&format!(
                " AND json_extract(attrs, '$.{}') = ?{}",
                key, param_index
            ));
            params.push(Box::new(value.clone()));
            param_index += 1;
        }

        // 范围查询
        if let Some(field) = &query.range_field {
            if let Some(min) = query.range_min {
                sql.push_str(&format!(" AND {} >= ?{}", field, param_index));
                params.push(Box::new(min));
                param_index += 1;
            }
            if let Some(max) = query.range_max {
                sql.push_str(&format!(" AND {} <= ?{}", field, param_index));
                params.push(Box::new(max));
            }
        }

        // 排序
        if let Some(sort_by) = &query.sort_by {
            let direction = if query.sort_asc { "ASC" } else { "DESC" };
            sql.push_str(&format!(" ORDER BY {} {}", sort_by, direction));
        }

        // 分页
        let limit = if query.limit == 0 { 50 } else { query.limit };
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, query.offset));

        let mut stmt = conn.prepare(&sql)?;
        let params_ref: Vec<&dyn rusqlite::ToSql> = params
            .iter()
            .map(|p| p.as_ref() as &dyn rusqlite::ToSql)
            .collect();

        let ids: Vec<String> = stmt
            .query_map(&params_ref[..], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_operations() {
        let backend = SqliteBackend::new_in_system_temp().unwrap();

        // 插入文档
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

        backend.apply(vec![IndexMutation::Add(doc)]).await.unwrap();

        // 查询
        let results = backend
            .search_ids(SearchQuery {
                node_type: Some("paragraph".to_string()),
                limit: 10,
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "test1");
    }

    #[tokio::test]
    async fn test_tree_query() {
        let backend = SqliteBackend::new_in_system_temp().unwrap();

        // 插入树形结构
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
                node_type: "section".to_string(),
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
                parent_id: Some("child1".to_string()),
                path: vec!["root".to_string(), "child1".to_string(), "child2".to_string()],
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

        // 查询子树
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
}
