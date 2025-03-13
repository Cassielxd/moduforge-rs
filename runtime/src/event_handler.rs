use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use moduforge_core::state::{state::State, transaction::Transaction};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    signal,
};

use crate::{
    event::{Event, EventHandler},
    storage_manager::StorageManager,
    types::StorageOptions, EditorResult,
};

use moduforge_delta::{
    delta::{TransactionDelta, create_tr_snapshot, to_delta},
    snapshot::create_full_snapshot,
};

enum SnapshotData {
    Tr(TransactionDelta),
    State(Arc<State>),
}

#[derive(Debug)]
pub struct SnapshotHandler {
    storage_option: StorageOptions,
    snapshot_interval: usize,
    counter: AtomicUsize,
    storage_manager: Arc<StorageManager>,
    tx: async_channel::Sender<(SnapshotData, PathBuf, PathBuf)>,
}

impl SnapshotHandler {
    pub fn new(
        storage_option: StorageOptions,
        snapshot_interval: usize,
        storage_manager: Arc<StorageManager>,
    ) -> Arc<SnapshotHandler> {
        let (tx, rx) = async_channel::bounded::<(SnapshotData, PathBuf, PathBuf)>(100);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = rx.recv() => match event {
                        Ok((data,tr_path,all_path)) => {
                            match data {
                                SnapshotData::Tr(transaction_delta) => {
                                    // 创建增量快照
                                    if let Ok(data) = create_tr_snapshot(transaction_delta) {
                                        match File::create(&tr_path).await {
                                            Ok(mut file) => {
                                                file.write_all(&data).await.unwrap();

                                            },
                                            Err(e) => {
                                                println!("write file error:{}",e);
                                            },
                                        }
                                    }
                                },
                                SnapshotData::State(state) => {
                                    // 创建全量快照
                                    match create_full_snapshot(&state) {
                                        Ok(data) => match File::create(&all_path).await {
                                            Ok(mut file) => {
                                                file.write_all(&data).await.unwrap();
                                                //清除冗余增量快照
                                                cleanup_old(&tr_path, state.version).await;
                                            }
                                            Err(e) => {
                                                println!("write file error:{}", e);
                                            }
                                        },
                                        Err(error) => {
                                            println!("Error creating snapshot: {}", error);
                                        }
                                    }
                                },
                            }

                        },
                        Err(_) => {
                            break;
                        },
                    },
                    _ = Box::pin(signal::ctrl_c()) => {
                        eprintln!("快照服务正在退出");
                        break;
                    },
                }
            }
        });
        Arc::new(SnapshotHandler {
            storage_option,
            snapshot_interval,
            counter: AtomicUsize::new(0),
            storage_manager,
            tx,
        })
    }
    async fn get_delta_path(
        &self,
        id: String,
    ) -> PathBuf {
        let base_path = self.storage_option.storage_path.join(id).join("delta");
        if !Path::exists(&base_path) {
            let _ = fs::create_dir_all(base_path.clone()).await;
        }
        base_path
    }
    async fn get_snapshot_path(
        &self,
        id: String,
    ) -> PathBuf {
        let base_path = self.storage_option.storage_path.join(id).join("snapshot");
        if !Path::exists(&base_path) {
            let _ = fs::create_dir_all(base_path.clone()).await;
        }
        base_path
    }
    /// 处理全量快照
    pub async fn handle_full_snapshot(
        &self,
        tr: &Arc<Transaction>,
        state: &Arc<State>,
    ) {
        let state_clone = state.clone();

        let id: String = state_clone.doc().inner.root_id.clone();

        let snapshot_path = self.get_snapshot_path(id.clone()).await;
        let delta_path = self.get_delta_path(id.clone()).await;

        let path = snapshot_path.join(format!("snapshot_v_{}.bin", state_clone.version));

        let cache_ref: Arc<StorageManager> = self.storage_manager.clone();
        let time: u64 = tr.id;
        cache_ref.put(&state_clone, time);
        let _ = self.tx.send((SnapshotData::State(state_clone), delta_path, path)).await;
    }
    ///处理增量事务
    pub async fn handle_tr_snapshot(
        &self,
        tr: &Arc<Transaction>,
        state: &Arc<State>,
    ) {
        let id: String = state.doc().inner.root_id.clone();
        let delta_path = self.get_delta_path(id.clone()).await;
        let path = delta_path.join(format!("delta_{}_{}.bin", tr.id, state.version));

        let tr_clone = tr.clone();
        let path_clone = path.clone();
        let _ = self
            .tx
            .send((
                SnapshotData::Tr(to_delta(&tr_clone, state.version)),
                path_clone.clone(),
                path_clone,
            ))
            .await;
    }
}

#[async_trait::async_trait]
impl EventHandler for SnapshotHandler {
    async fn handle(
        &self,
        event: &Event,
    ) -> EditorResult<()> {
        if let Event::TrApply(tr, state) = event {
            let count = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
            // 处理增量事务快照
            self.handle_tr_snapshot(tr, state).await;
            if count % self.snapshot_interval == 0 {
                // 处理全量快照
                self.handle_full_snapshot(tr, state).await;
            }
        }
        Ok(())
    }
}

/// 清理旧的事务增量文件
async fn cleanup_old(
    delta_dir: &PathBuf,
    max_version: u64,
) {
    match fs::read_dir(delta_dir).await {
        Ok(mut entries) => {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        if let Some(version) = extract_version(file_name) {
                            if version <= max_version {
                                if let Err(e) = fs::remove_file(&path).await {
                                    println!("删除文件失败 {}: {}", path.display(), e);
                                } else {
                                    println!("已清理冗余事务文件: {}", path.display());
                                }
                            }
                        }
                    }
                }
            }
        },
        Err(e) => {
            println!("无法读取增量目录 {}: {}", delta_dir.display(), e);
        },
    }
}

/// 从文件名中提取版本号
fn extract_version(file_name: &str) -> Option<u64> {
    // 文件名格式: delta_{timestamp}_{version}.bin
    let parts: Vec<&str> = file_name.split('_').collect();
    if parts.len() >= 3 { parts[2].split('.').next().and_then(|v| v.parse().ok()) } else { None }
}
