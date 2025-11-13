//! 基于 SQLite 的 `EventStore` 实现，使用 RBatis 异步访问。
//!
//! 默认启用 WAL 模式配合 IMMEDIATE 事务，保证顺序写入与良好吞吐，
//! 并在 `CommitMode::SyncDurable` 下触发 WAL checkpoint 以获得更强持久性。

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use rbatis::{executor::Executor, RBatis};
use rbdc_sqlite::Driver;
use rbs::Value;
use serde::{Deserialize, Serialize};

use crate::api::{CommitMode, EventStore, PersistedEvent, Snapshot};

const INIT_SQL: &str = r#"
    PRAGMA journal_mode = WAL;
    PRAGMA synchronous = NORMAL;
    PRAGMA cache_size = -64000;
    PRAGMA temp_store = MEMORY;

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
    CREATE INDEX IF NOT EXISTS ix_snapshots_doc_created
        ON snapshots(doc_id, created_at DESC);
"#;

const INSERT_EVENT_SQL: &str = "\
    INSERT INTO events \
    (tr_id, doc_id, ts, actor, idempotency_key, meta, payload, checksum) \
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";

const UPSERT_SNAPSHOT_SQL: &str = "\
    INSERT OR REPLACE INTO snapshots \
    (doc_id, upto_lsn, created_at, state_blob, version) \
    VALUES (?1, ?2, ?3, ?4, ?5)";

/// `EventStore` 的 SQLite 具体实现。
pub struct SqliteEventStore {
    _db_path: PathBuf,
    pool: Arc<RBatis>,
    commit_mode: CommitMode,
}

impl SqliteEventStore {
    /// 打开（或创建）数据库并初始化表结构与 PRAGMA。
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(db_path), fields(
        crate_name = "persistence",
        commit_mode = ?commit_mode
    )))]
    pub async fn open(
        db_path: impl Into<PathBuf>,
        commit_mode: CommitMode,
    ) -> anyhow::Result<Arc<Self>> {
        let db_path = db_path.into();
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let rb = RBatis::new();
        rb.link(Driver {}, &format!("sqlite://{}", db_path.display())).await?;
        Self::init_schema(&rb).await?;

        Ok(Arc::new(Self {
            _db_path: db_path,
            pool: Arc::new(rb),
            commit_mode,
        }))
    }

    async fn init_schema(rb: &RBatis) -> anyhow::Result<()> {
        let conn = rb.acquire().await?;
        conn.exec(INIT_SQL, vec![]).await?;
        Ok(())
    }

    async fn insert_event<E>(
        &self,
        exec: &E,
        ev: &PersistedEvent,
    ) -> anyhow::Result<i64>
    where
        E: Executor + ?Sized,
    {
        let params = vec![
            to_value(ev.tr_id.to_string()),
            to_value(ev.doc_id.clone()),
            to_value(ev.ts),
            to_value(ev.actor.clone()),
            to_value(ev.idempotency_key.clone()),
            to_value(serde_json::to_string(&ev.meta)?),
            to_value(ev.payload.clone()),
            to_value(ev.checksum as i64),
        ];
        let exec_result = exec.exec(INSERT_EVENT_SQL, params).await?;
        Ok(exec_result.last_insert_id.as_i64().unwrap_or_default())
    }

    async fn ensure_sync(&self) -> anyhow::Result<()> {
        if matches!(self.commit_mode, CommitMode::SyncDurable) {
            let conn = self.pool.acquire().await?;
            conn.exec("PRAGMA wal_checkpoint(TRUNCATE)", vec![]).await.ok();
        }
        Ok(())
    }
}

#[async_trait]
impl EventStore for SqliteEventStore {
    /// 以独立事务追加一条事件记录。
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, ev), fields(
        crate_name = "persistence",
        tr_id = %ev.tr_id,
        doc_id = %ev.doc_id
    )))]
    async fn append(
        &self,
        ev: PersistedEvent,
    ) -> anyhow::Result<i64> {
        let tx = self.pool.acquire_begin().await?;
        let lsn = self.insert_event(&tx, &ev).await?;
        tx.commit().await?;
        self.ensure_sync().await?;
        Ok(lsn)
    }

    /// 在单事务内批量追加多条事件以提升吞吐。
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, evs), fields(
        crate_name = "persistence",
        batch_size = evs.len()
    )))]
    async fn append_batch(
        &self,
        evs: Vec<PersistedEvent>,
    ) -> anyhow::Result<i64> {
        if evs.is_empty() {
            return Ok(0);
        }
        let tx = self.pool.acquire_begin().await?;
        let mut last_lsn = 0;
        for ev in &evs {
            last_lsn = self.insert_event(&tx, ev).await?;
        }
        tx.commit().await?;
        self.ensure_sync().await?;
        Ok(last_lsn)
    }

    /// 读取指定文档在 `from_lsn` 之后的有序事件流。
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self), fields(
        crate_name = "persistence",
        doc_id = %doc_id,
        from_lsn = from_lsn,
        limit = limit
    )))]
    async fn load_since(
        &self,
        doc_id: &str,
        from_lsn: i64,
        limit: u32,
    ) -> anyhow::Result<Vec<PersistedEvent>> {
        let conn = self.pool.acquire().await?;
        let rows: Vec<EventRow> = conn
            .query_decode(
                "SELECT lsn, tr_id, doc_id, ts, actor, idempotency_key, \
                 meta, payload, checksum \
                 FROM events \
                 WHERE doc_id = ?1 AND lsn > ?2 \
                 ORDER BY lsn ASC LIMIT ?3",
                vec![
                    to_value(doc_id),
                    to_value(from_lsn),
                    to_value(limit as i64),
                ],
            )
            .await?;
        Ok(rows
            .into_iter()
            .map(PersistedEvent::try_from)
            .collect::<anyhow::Result<Vec<_>>>()?)
    }

    /// 返回该文档的最新快照（若存在）。
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self), fields(
        crate_name = "persistence",
        doc_id = %doc_id
    )))]
    async fn latest_snapshot(
        &self,
        doc_id: &str,
    ) -> anyhow::Result<Option<Snapshot>> {
        let conn = self.pool.acquire().await?;
        let rows: Vec<SnapshotRow> = conn
            .query_decode(
                "SELECT doc_id, upto_lsn, created_at, state_blob, version \
                 FROM snapshots \
                 WHERE doc_id = ?1 \
                 ORDER BY created_at DESC LIMIT 1",
                vec![to_value(doc_id)],
            )
            .await?;
        Ok(rows.into_iter().next().map(Into::into))
    }

    /// 原子写入/替换快照。
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, snap), fields(
        crate_name = "persistence",
        doc_id = %snap.doc_id,
        upto_lsn = snap.upto_lsn
    )))]
    async fn write_snapshot(
        &self,
        snap: Snapshot,
    ) -> anyhow::Result<()> {
        let tx = self.pool.acquire_begin().await?;
        tx.exec(
            UPSERT_SNAPSHOT_SQL,
            vec![
                to_value(snap.doc_id),
                to_value(snap.upto_lsn),
                to_value(snap.created_at),
                to_value(snap.state_blob),
                to_value(snap.version),
            ],
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    /// 最佳努力的历史压缩：删除最新快照之前的事件。
    async fn compact(
        &self,
        doc_id: &str,
    ) -> anyhow::Result<()> {
        let conn = self.pool.acquire().await?;
        let upto_rows: Vec<UptoRow> = conn
            .query_decode(
                "SELECT upto_lsn FROM snapshots \
                 WHERE doc_id = ?1 \
                 ORDER BY upto_lsn DESC LIMIT 1",
                vec![to_value(doc_id)],
            )
            .await?;
        if let Some(row) = upto_rows.into_iter().next() {
            conn.exec(
                "DELETE FROM events WHERE doc_id = ?1 AND lsn <= ?2",
                vec![to_value(doc_id), to_value(row.upto_lsn)],
            )
            .await?;
        }
        Ok(())
    }
}

fn to_value<T: serde::Serialize>(value: T) -> Value {
    rbs::value_def(value)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventRow {
    lsn: i64,
    tr_id: u64,
    doc_id: String,
    ts: i64,
    actor: Option<String>,
    idempotency_key: String,
    meta: String,
    payload: Vec<u8>,
    checksum: i64,
}

impl TryFrom<EventRow> for PersistedEvent {
    type Error = anyhow::Error;

    fn try_from(row: EventRow) -> anyhow::Result<Self> {
        Ok(Self {
            lsn: row.lsn,
            tr_id: row.tr_id,
            doc_id: row.doc_id,
            ts: row.ts,
            actor: row.actor,
            idempotency_key: row.idempotency_key,
            meta: serde_json::from_str(&row.meta)?,
            payload: row.payload,
            checksum: row.checksum as u32,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotRow {
    doc_id: String,
    upto_lsn: i64,
    created_at: i64,
    state_blob: Vec<u8>,
    version: i32,
}

impl From<SnapshotRow> for Snapshot {
    fn from(row: SnapshotRow) -> Self {
        Self {
            doc_id: row.doc_id,
            upto_lsn: row.upto_lsn,
            created_at: row.created_at,
            state_blob: row.state_blob,
            version: row.version,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UptoRow {
    upto_lsn: i64,
}
