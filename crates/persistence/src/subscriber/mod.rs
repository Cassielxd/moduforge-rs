//! 基于事件总线的持久化订阅者：在 TrApply 事件中持久化事务并按策略写入快照
use std::{collections::HashSet, fmt, sync::Arc};

use async_trait::async_trait;
use dashmap::DashMap;
use mf_core::{
    event::{Event, EventHandler},
    ForgeResult,
};
use mf_state::transaction::Transaction;
use serde_json::json;
use uuid::Uuid;

use crate::api::{CommitMode, EventStore, PersistOptions, PersistedEvent, Snapshot};
use crate::ser::{
    checksum32, compress_if_needed, frame_steps, SnapshotData, TypeWrapper,
};

#[derive(Default)]
struct SnapshotCounters {
    last_snapshot_ms: i64,
    last_snapshot_upto_lsn: i64,
    events_since: u32,
    bytes_since: u64,
}

pub struct SnapshotSubscriber<E: EventStore + 'static> {
    store: Arc<E>,
    options: PersistOptions,
    commit_mode: CommitMode,
    default_doc_id: String,
    // 进程内已持久化的事务ID集合，避免重复写入（进程重启后靠数据库幂等键）
    persisted: DashMap<Uuid, ()>, // ✅ 改用 UUID 作为 key
    // 每个文档的快照计数器
    snap_counters: DashMap<String, SnapshotCounters>,
    // 待写快照的 upto_lsn
    pending_snapshot_lsn: DashMap<String, i64>,
}

impl<E: EventStore + 'static> fmt::Debug for SnapshotSubscriber<E> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.write_str("SnapshotSubscriber")
    }
}

impl<E: EventStore + 'static> Clone for SnapshotSubscriber<E> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            options: self.options.clone(),
            commit_mode: self.commit_mode,
            default_doc_id: self.default_doc_id.clone(),
            persisted: DashMap::new(),
            snap_counters: DashMap::new(),
            pending_snapshot_lsn: DashMap::new(),
        }
    }
}

impl<E: EventStore + 'static> SnapshotSubscriber<E> {
    pub fn new(
        store: Arc<E>,
        options: PersistOptions,
        default_doc_id: impl Into<String>,
    ) -> Self {
        Self {
            commit_mode: options.commit_mode,
            store,
            options,
            default_doc_id: default_doc_id.into(),
            persisted: DashMap::new(),
            snap_counters: DashMap::new(),
            pending_snapshot_lsn: DashMap::new(),
        }
    }

    async fn persist_one(
        &self,
        tr_id: Uuid, // ✅ 改用 UUID
        doc_id: &str,
        transaction: &Transaction,
    ) -> ForgeResult<Option<(i64, usize)>> {
        // 将每个步骤包装为 {type_id, data}
        let frames: Vec<TypeWrapper> = frame_steps(transaction);
        let framed = serde_json::to_vec(&frames).map_err(|e| {
            mf_core::error::error_utils::middleware_error(format!(
                "步骤编码失败: {e}"
            ))
        })?;
        let payload = compress_if_needed(&framed, self.options.compression)
            .map_err(|e| {
                mf_core::error::error_utils::middleware_error(e.to_string())
            })?;
        let checksum = checksum32(&payload);

        let ev = PersistedEvent {
            lsn: 0,
            tr_id,
            doc_id: doc_id.to_string(),
            ts: chrono::Utc::now().timestamp_millis(),
            actor: None,
            idempotency_key: format!("tr:{tr_id}"),
            payload,
            meta: json!({}),
            checksum,
        };

        match self.commit_mode {
            CommitMode::MemoryOnly => Ok(None),
            _ => {
                let bytes = ev.payload.len();
                self.store
                    .append(ev)
                    .await
                    .map(|lsn| Some((lsn, bytes)))
                    .map_err(|e| {
                        mf_core::error::error_utils::middleware_error(
                            e.to_string(),
                        )
                    })
            },
        }
    }

    fn should_snapshot(
        &self,
        doc_id: &str,
    ) -> bool {
        if let Some(counters) = self.snap_counters.get(doc_id) {
            let now = chrono::Utc::now().timestamp_millis();
            let time_ok = self.options.snapshot_every_ms > 0
                && now - counters.last_snapshot_ms
                    >= self.options.snapshot_every_ms as i64;
            let count_ok = self.options.snapshot_every_n_events > 0
                && counters.events_since
                    >= self.options.snapshot_every_n_events;
            let bytes_ok = self.options.snapshot_every_bytes > 0
                && counters.bytes_since >= self.options.snapshot_every_bytes;
            time_ok || count_ok || bytes_ok
        } else {
            false
        }
    }

    async fn write_snapshot(
        &self,
        doc_id: &str,
        upto_lsn: i64,
        state: &mf_state::State,
    ) -> ForgeResult<()> {
        let mut ser = state.serialize().await.map_err(|e| {
            mf_core::error::error_utils::middleware_error(format!(
                "状态序列化失败: {e}"
            ))
        })?;
        let snap = SnapshotData {
            node_pool: std::mem::take(&mut ser.node_pool),
            state_fields: std::mem::take(&mut ser.state_fields),
        };
        let blob = serde_json::to_vec(&snap).map_err(|e| {
            mf_core::error::error_utils::middleware_error(format!(
                "快照编码失败: {e}"
            ))
        })?;
        let blob = compress_if_needed(&blob, self.options.compression)
            .map_err(|e| {
                mf_core::error::error_utils::middleware_error(format!(
                    "快照压缩失败: {e}"
                ))
            })?;

        let snap = Snapshot {
            doc_id: doc_id.to_string(),
            upto_lsn,
            created_at: chrono::Utc::now().timestamp_millis(),
            state_blob: blob,
            version: 1,
        };

        self.store.write_snapshot(snap).await.map_err(|e| {
            mf_core::error::error_utils::middleware_error(e.to_string())
        })?;

        // 重置计数器
        let mut entry =
            self.snap_counters.entry(doc_id.to_string()).or_default();
        entry.last_snapshot_ms = chrono::Utc::now().timestamp_millis();
        entry.last_snapshot_upto_lsn = upto_lsn;
        entry.events_since = 0;
        entry.bytes_since = 0;
        Ok(())
    }
}

#[async_trait]
impl<E: EventStore + 'static> EventHandler<Event> for SnapshotSubscriber<E> {
    async fn handle(
        &self,
        event: &Event,
    ) -> ForgeResult<()> {
        if let Event::Create(state) = event {
            // 初始化时若不存在任何快照，则写入基础快照（upto_lsn = 0）
            let doc_id = state.doc().root_id().to_string();
            let has_snapshot = self
                .store
                .latest_snapshot(&doc_id)
                .await
                .map(|s| s.is_some())
                .unwrap_or(false);
            if !has_snapshot {
                self.write_snapshot(&doc_id, 0, state).await?;
            }
        } else if let Event::TrApply { old_state: _, new_state, transactions } =
            event
        {
            let state = new_state;
            let trs = transactions;
            let mut touched_docs: HashSet<String> = HashSet::new();
            let max_lsn_by_doc: DashMap<String, i64> = DashMap::new();

            for tr in trs.iter() {
                let tr_id = tr.id;
                if self.persisted.contains_key(&tr_id) {
                    continue;
                }
                // 选择 doc_id：优先 meta 的 doc_id，否则使用当前状态根ID
                let doc_id = tr
                    .get_meta::<String>("doc_id")
                    .unwrap_or_else(|| state.doc().root_id().to_string());
                let doc_id = if doc_id.is_empty() {
                    self.default_doc_id.clone()
                } else {
                    doc_id
                };

                match self.persist_one(tr_id, &doc_id, tr).await {
                    Ok(Some((lsn, bytes))) => {
                        self.persisted.insert(tr_id, ());
                        touched_docs.insert(doc_id.clone());
                        let mut entry = self
                            .snap_counters
                            .entry(doc_id.clone())
                            .or_default();
                        entry.events_since += 1;
                        entry.bytes_since += bytes as u64;
                        // 更新 up-to LSN
                        let cur = self
                            .pending_snapshot_lsn
                            .get(&doc_id)
                            .map(|v| *v.value())
                            .unwrap_or(-1);
                        if lsn > cur {
                            self.pending_snapshot_lsn
                                .insert(doc_id.clone(), lsn);
                        }
                        // 记录每个文档的最大 LSN（备用）
                        let m = max_lsn_by_doc
                            .get(&doc_id)
                            .map(|v| *v.value())
                            .unwrap_or(-1);
                        if lsn > m {
                            max_lsn_by_doc.insert(doc_id.clone(), lsn);
                        }
                    },
                    Ok(None) => {},
                    Err(e) => {
                        return Err(
                            mf_core::error::error_utils::middleware_error(
                                format!("持久化失败: {e}"),
                            ),
                        );
                    },
                }
            }

            // 对触达的文档执行快照策略
            for doc_id in touched_docs.into_iter() {
                if let Some(upto) =
                    self.pending_snapshot_lsn.get(&doc_id).map(|v| *v.value())
                {
                    if upto >= 0 {
                        let has_snapshot = self
                            .store
                            .latest_snapshot(&doc_id)
                            .await
                            .map(|s| s.is_some())
                            .unwrap_or(false);
                        if !has_snapshot || self.should_snapshot(&doc_id) {
                            self.write_snapshot(&doc_id, upto, state).await?;
                            self.pending_snapshot_lsn.remove(&doc_id);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
