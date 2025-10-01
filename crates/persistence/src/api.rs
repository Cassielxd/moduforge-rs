//! 持久化抽象层的公共 API。
//!
//! 本模块定义了持久化的可调一致性选项、事件/快照的持久化格式，
//! 以及具体后端需要实现的 trait。默认实现为基于 SQLite 的 WAL，
//! 但上层仅依赖这些 trait 即可与具体后端解耦。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 写入持久性与延迟之间的权衡。
///
/// - `MemoryOnly`：事件入内存队列即返回，最低延迟、不具持久性，仅适合演示/开发。
/// - `AsyncDurable`：写入 WAL/系统缓存后返回，fsync 由后台分组处理；
///   在桌面环境具有很好的实际持久性与极低延迟（默认）。
/// - `SyncDurable`：提交前强制 fsync，延迟最高、持久性最强。
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum CommitMode {
    MemoryOnly,
    AsyncDurable { group_window_ms: u32 },
    SyncDurable,
}

/// 快照节奏与压缩等可调参数。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistOptions {
    pub commit_mode: CommitMode,
    pub snapshot_every_n_events: u32,
    pub snapshot_every_bytes: u64,
    pub snapshot_every_ms: u64,
    pub compression: bool,
}

/// 事件存储的"仅追加"记录。
///
/// 不变式：
/// - `lsn` 为后端分配的单调递增日志序号
/// - `idempotency_key` 全局唯一，用于请求重试的幂等去重
/// - `checksum` 对（压缩/加密后的）`payload` 计算得到
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistedEvent {
    pub lsn: i64,
    pub tr_id: Uuid, // ✅ 改用 UUID 作为事务唯一标识
    pub doc_id: String,
    pub ts: i64,
    pub actor: Option<String>,
    pub idempotency_key: String,
    pub payload: Vec<u8>,
    pub meta: serde_json::Value,
    pub checksum: u32,
}

/// 截止到 `upto_lsn` 的物化状态快照。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub doc_id: String,
    pub upto_lsn: i64,
    pub created_at: i64,
    pub state_blob: Vec<u8>,
    pub version: i32,
}

/// 具体后端必须实现的最小持久化接口。
#[async_trait]
pub trait EventStore: Send + Sync {
    /// 追加单条事件。返回分配的 `lsn`。
    async fn append(
        &self,
        ev: PersistedEvent,
    ) -> anyhow::Result<i64>;
    /// 在单事务内批量追加事件。返回最后一条的 `lsn`。
    async fn append_batch(
        &self,
        evs: Vec<PersistedEvent>,
    ) -> anyhow::Result<i64>;
    /// 读取指定文档分片自 `from_lsn` 之后的增量事件，上限为 `limit` 条。
    async fn load_since(
        &self,
        doc_id: &str,
        from_lsn: i64,
        limit: u32,
    ) -> anyhow::Result<Vec<PersistedEvent>>;
    /// 获取指定文档分片的最新快照（如果存在）。
    async fn latest_snapshot(
        &self,
        doc_id: &str,
    ) -> anyhow::Result<Option<Snapshot>>;
    /// 写入/替换快照，应保证原子性。
    async fn write_snapshot(
        &self,
        snap: Snapshot,
    ) -> anyhow::Result<()>;
    /// 可选压缩：删除已被最新快照覆盖的历史事件。
    async fn compact(
        &self,
        doc_id: &str,
    ) -> anyhow::Result<()>;
}

/// 用于启动引导的状态加载结果（快照 + 增量事件）。
#[derive(Clone, Debug)]
pub struct LoadedState {
    pub upto_lsn: i64,
    pub snapshot: Option<Snapshot>,
    pub events: Vec<PersistedEvent>,
}

/// 高层持久化编排：事务持久化与快照检查点。
#[async_trait]
pub trait Persistence: Send + Sync {
    async fn persist_transaction(
        &self,
        tr: &mf_state::Transaction,
        doc_id: &str,
        context_meta: &serde_json::Value,
    ) -> anyhow::Result<i64>;

    async fn load_state(
        &self,
        doc_id: &str,
    ) -> anyhow::Result<LoadedState>;

    async fn checkpoint_if_needed(
        &self,
        doc_id: &str,
    ) -> anyhow::Result<()>;
}
