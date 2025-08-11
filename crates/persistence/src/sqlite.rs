//! 基于 SQLite 的 `EventStore` 实现。
//!
//! 默认使用 WAL 日志模式与 IMMEDIATE 事务以获得低延迟写入。
//! 采用“单连接 + 互斥”顺序写，确保追加顺序并避免写锁竞争；
//! 读取按需准备语句，受益于 SQLite 页缓存。

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::Mutex;
use rusqlite::{params, Connection, TransactionBehavior};
 

use crate::api::{CommitMode, EventStore, PersistedEvent, Snapshot};

/// `EventStore` 的 SQLite 具体实现。
pub struct SqliteEventStore {
    _db_path: PathBuf,
    conn: Mutex<Connection>,
    commit_mode: CommitMode,
}

impl SqliteEventStore {
    /// 打开（或创建）数据库并初始化表结构与 PRAGMA。
    pub fn open(db_path: impl Into<PathBuf>, commit_mode: CommitMode) -> anyhow::Result<Arc<Self>> {
        let db_path = db_path.into();
        // 确保父目录存在
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open(&db_path)?;
        conn.pragma_update(None, "journal_mode", &"WAL")?;
        conn.pragma_update(None, "synchronous", &"NORMAL")?;
        conn.pragma_update(None, "busy_timeout", &5000i64)?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS events (
              lsn INTEGER PRIMARY KEY AUTOINCREMENT,
              tr_id INTEGER NOT NULL,
              doc_id TEXT NOT NULL,
              ts INTEGER NOT NULL,
              actor TEXT,
              idempotency_key TEXT NOT NULL UNIQUE,
              meta TEXT NOT NULL,
              payload BLOB NOT NULL,
              checksum INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS ix_events_doc_lsn ON events(doc_id, lsn);

            CREATE TABLE IF NOT EXISTS snapshots (
              doc_id TEXT NOT NULL,
              upto_lsn INTEGER NOT NULL,
              created_at INTEGER NOT NULL,
              state_blob BLOB NOT NULL,
              version INTEGER NOT NULL,
              PRIMARY KEY (doc_id, upto_lsn)
            );
            CREATE INDEX IF NOT EXISTS ix_snapshots_doc_created ON snapshots(doc_id, created_at DESC);
            "#,
        )?;

        Ok(Arc::new(Self { _db_path: db_path, conn: Mutex::new(conn), commit_mode }))
    }
}

#[async_trait]
impl EventStore for SqliteEventStore {
    /// 以独立事务追加一条事件记录。
    async fn append(&self, ev: PersistedEvent) -> anyhow::Result<i64> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO events (tr_id, doc_id, ts, actor, idempotency_key, meta, payload, checksum) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                ev.tr_id as i64,
                ev.doc_id,
                ev.ts as i64,
                ev.actor,
                ev.idempotency_key,
                ev.meta.to_string(),
                ev.payload,
                ev.checksum as i64
            ],
        )?;
        tx.commit()?;
        let lsn = conn.last_insert_rowid();
        match self.commit_mode {
            CommitMode::SyncDurable => { conn.pragma_update(None, "wal_checkpoint", &"TRUNCATE").ok(); },
            _ => {}
        }
        Ok(lsn)
    }

    /// 在单事务内批量追加多条事件以提升吞吐。
    async fn append_batch(&self, evs: Vec<PersistedEvent>) -> anyhow::Result<i64> {
        if evs.is_empty() { return Ok(0); }
        let mut conn = self.conn.lock();
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        {
            let mut stmt = tx.prepare("INSERT INTO events (tr_id, doc_id, ts, actor, idempotency_key, meta, payload, checksum) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)")?;
            for ev in evs.into_iter() {
                stmt.execute(params![
                    ev.tr_id as i64,
                    ev.doc_id,
                    ev.ts as i64,
                    ev.actor,
                    ev.idempotency_key,
                    ev.meta.to_string(),
                    ev.payload,
                    ev.checksum as i64
                ])?;
            }
        }
        tx.commit()?;
        let lsn = conn.last_insert_rowid();
        Ok(lsn)
    }

    /// 读取指定文档在 `from_lsn` 之后的有序事件流。
    async fn load_since(&self, doc_id: &str, from_lsn: i64, limit: u32) -> anyhow::Result<Vec<PersistedEvent>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT lsn, tr_id, doc_id, ts, actor, idempotency_key, meta, payload, checksum FROM events WHERE doc_id = ?1 AND lsn > ?2 ORDER BY lsn ASC LIMIT ?3")?;
        let rows = stmt.query_map(params![doc_id, from_lsn, limit as i64], |row| {
            let meta_str: String = row.get(6)?;
            let meta: serde_json::Value = serde_json::from_str(&meta_str).unwrap_or(serde_json::json!({}));
            Ok(PersistedEvent {
                lsn: row.get(0)?,
                tr_id: row.get::<_, i64>(1)? as u64,
                doc_id: row.get(2)?,
                ts: row.get(3)?,
                actor: row.get(4).ok(),
                idempotency_key: row.get(5)?,
                meta,
                payload: row.get(7)?,
                checksum: row.get::<_, i64>(8)? as u32,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// 返回该文档的最新快照（若存在）。
    async fn latest_snapshot(&self, doc_id: &str) -> anyhow::Result<Option<Snapshot>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT doc_id, upto_lsn, created_at, state_blob, version FROM snapshots WHERE doc_id = ?1 ORDER BY created_at DESC LIMIT 1")?;
        let mut rows = stmt.query(params![doc_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Snapshot {
                doc_id: row.get(0)?,
                upto_lsn: row.get(1)?,
                created_at: row.get(2)?,
                state_blob: row.get(3)?,
                version: row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// 原子写入/替换快照。
    async fn write_snapshot(&self, snap: Snapshot) -> anyhow::Result<()> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT OR REPLACE INTO snapshots (doc_id, upto_lsn, created_at, state_blob, version) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![snap.doc_id, snap.upto_lsn, snap.created_at, snap.state_blob, snap.version],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// 最佳努力的历史压缩：删除最新快照之前的事件。
    async fn compact(&self, doc_id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock();
        // 删除最老的事件到最近快照
        let upto: Option<i64> = conn.query_row(
            "SELECT upto_lsn FROM snapshots WHERE doc_id = ?1 ORDER BY upto_lsn DESC LIMIT 1",
            params![doc_id],
            |r| r.get(0),
        ).ok();
        if let Some(upto_lsn) = upto {
            conn.execute("DELETE FROM events WHERE doc_id = ?1 AND lsn <= ?2", params![doc_id, upto_lsn])?;
        }
        Ok(())
    }
}


