use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use moduforge_delta::{
    delta::{create_tr_snapshot, to_delta},
    snapshot::create_full_snapshot,
};

use crate::event::{Event, EventHandler};

/// 创建一个DeltaHandler，用于处理事务事件，并生成增量记录。
///
/// # Arguments
///
/// * `storage_path`: 增量记录存储路径
///
/// # Returns
///
/// 返回一个DeltaHandler实例
///
pub fn create_delta_handler(storage_path: PathBuf) -> DeltaHandler {
    DeltaHandler { storage_path }
}
pub struct DeltaHandler {
    storage_path: PathBuf,
}
#[async_trait::async_trait]
impl EventHandler for DeltaHandler {
    async fn handle(&self, event: &Event) {
        match event {
            Event::Apply(tx, state) => {
                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let path = self.storage_path.join(format!("delta_{}.bin", timestamp));
                let tx_clone = tx.clone();
                let state_version = state.version.clone();
                let path_clone = path.clone();
                tokio::spawn(async move {
                    if let Ok(data) = create_tr_snapshot(to_delta(&tx_clone, state_version)) {
                        let _ = tokio::fs::write(path_clone, data).await;
                    }
                });
            }
            _ => {}
        }
    }
}

/// 创建一个SnapshotHandler，用于处理事务事件，并生成快照。
///
/// # Arguments
///
/// * `storage_path`: 快照存储路径
/// * `snapshot_interval`: 快照生成间隔
///
/// # Returns
///
/// 返回一个SnapshotHandler实例
pub fn create_snapshot_handler(storage_path: PathBuf, snapshot_interval: usize) -> SnapshotHandler {
    SnapshotHandler {
        storage_path,
        snapshot_interval,
        counter: AtomicUsize::new(0),
    }
}
pub struct SnapshotHandler {
    storage_path: PathBuf,
    snapshot_interval: usize,
    counter: AtomicUsize,
}
#[async_trait::async_trait]
impl EventHandler for SnapshotHandler {
   async fn handle(&self, event: &Event) {
        match event {
            Event::Apply(_, state) => {
                let count = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
                if count % self.snapshot_interval == 0 {
                    let state_clone = state.clone();
                    let path = self.storage_path.join(format!("snapshot_v{}.bin", count));
                    tokio::spawn(async move {
                        match create_full_snapshot(&state_clone) {
                            Ok(data) => {
                                let _ = tokio::fs::write(path, data).await;
                            }
                            Err(_) => {}
                        }
                    });
                }
            }
            _ => {}
        }
    }
}
