use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    signal,
};

use crate::{
    delta::{
        delta::{create_tr_snapshot, to_delta, TransactionDelta},
        snapshot::create_full_snapshot,
    },
    event::{Event, EventHandler},
    snapshot_manager::SnapshotManager,
    types::StorageOptions,
};

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
pub fn create_delta_handler(storage_option: StorageOptions) -> Arc<DeltaHandler> {
    let (tx, rx) = async_channel::bounded::<(TransactionDelta, PathBuf)>(100);
    tokio::spawn(async move {
        loop {
            tokio::select! {
                event = rx.recv() => match event {
                    Ok((data,path)) => {
                        if let Ok(data) = create_tr_snapshot(data) {
                            match File::create(&path).await {
                                Ok(mut file) => {
                                    file.write_all(&data).await.unwrap();

                                },
                                Err(e) => {
                                    println!("write file error:{}",e);
                                },
                            }
                        }
                    },
                    Err(_) => {
                        println!("跳出了");
                        break;
                    },
                },
                shutdown_signal = Box::pin(signal::ctrl_c()) => {
                    match shutdown_signal {
                        Ok(()) => {
                            println!("增量事务服务 接收到关闭信号，正在退出...");
                            break;
                        },
                        Err(e) => {
                            eprintln!("增量事务服务 处理关闭信号时出错: {}", e);
                            break;
                        }
                    }
                },
            }
        }
    });
    Arc::new(DeltaHandler { storage_option, tx })
}
#[derive(Debug)]
pub struct DeltaHandler {
    pub storage_option: StorageOptions,
    tx: async_channel::Sender<(TransactionDelta, PathBuf)>,
}
#[async_trait::async_trait]
impl EventHandler for DeltaHandler {
    async fn handle(&self, event: &Event) {
        if let Event::TrApply(tx, state) = event {
            let base_path = self
                .storage_option
                .delta_path
                .join(state.doc().inner.root_id.clone());
            let path = base_path.join(format!("delta_{}_{}.bin", tx.time, state.version));
            let _ = fs::create_dir_all(base_path).await;
            let tx_clone = tx.clone();
            let state_version = state.version;
            let path_clone = path.clone();
            let _ = self
                .tx
                .send((to_delta(&tx_clone, state_version), path_clone))
                .await;
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
pub fn create_snapshot_handler(
    storage_option: StorageOptions,
    snapshot_interval: usize,
    snapshot_manager: Arc<SnapshotManager>,
) -> Arc<SnapshotHandler> {
    Arc::new(SnapshotHandler {
        storage_option,
        snapshot_interval,
        counter: AtomicUsize::new(0),
        snapshot_manager,
    })
}
#[derive(Debug)]
pub struct SnapshotHandler {
    storage_option: StorageOptions,
    snapshot_interval: usize,
    counter: AtomicUsize,
    snapshot_manager: Arc<SnapshotManager>,
}
#[async_trait::async_trait]
impl EventHandler for SnapshotHandler {
    async fn handle(&self, event: &Event) {
        if let Event::TrApply(tr, state) = event {
            let count = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
            if count % self.snapshot_interval == 0 {
                let state_clone = state.clone();
                let base_path = self
                    .storage_option
                    .snapshot_path
                    .join(state_clone.doc().inner.root_id.clone());
                let path = base_path.join(format!("snapshot_v{}.bin", state_clone.version));
                let delta_path = self
                    .storage_option
                    .delta_path
                    .join(state_clone.doc().inner.root_id.clone());
                let max_version = state_clone.version;
                let _ = fs::create_dir_all(base_path).await;
                let cache_ref: Arc<SnapshotManager> = self.snapshot_manager.clone();
                let time = tr.time;
                tokio::spawn(async move {
                    cache_ref.put(&state_clone, time);
                    match create_full_snapshot(&state_clone) {
                        Ok(data) => match File::create(&path).await {
                            Ok(mut file) => {
                                file.write_all(&data).await.unwrap();
                                cleanup_old_deltas(&delta_path, max_version).await;
                            }
                            Err(e) => {
                                println!("write file error:{}", e);
                            }
                        },
                        Err(error) => {
                            println!("Error creating snapshot: {}", error);
                        }
                    }
                });
            }
        }
    }
}

/// 清理旧的事务增量文件
async fn cleanup_old_deltas(delta_dir: &PathBuf, max_version: u64) {
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
        }
        Err(e) => {
            println!("无法读取增量目录 {}: {}", delta_dir.display(), e);
        }
    }
}

/// 从文件名中提取版本号
fn extract_version(file_name: &str) -> Option<u64> {
    // 文件名格式: delta_{timestamp}_{version}.bin
    let parts: Vec<&str> = file_name.split('_').collect();
    if parts.len() >= 3 && parts[0] == "delta" {
        parts[2].split('.').next().and_then(|v| v.parse().ok())
    } else {
        None
    }
}
