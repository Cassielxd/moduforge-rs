use std::{
    fs::{self},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use moduforge_delta::{
    delta::{create_tr_snapshot, to_delta, TransactionDelta},
    snapshot::create_full_snapshot,
};
use tokio::{fs::File, io::AsyncWriteExt, signal};

use crate::{
    cache::{cache::DocumentCache, l2, CacheKey},
    event::{Event, EventHandler}, snapshot_manager::SnapshotManager,
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
pub fn create_delta_handler(storage_path: PathBuf) -> Arc<DeltaHandler> {
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
    Arc::new(DeltaHandler { storage_path, tx })
}
#[derive(Debug)]
pub struct DeltaHandler {
    storage_path: PathBuf,
    tx: async_channel::Sender<(TransactionDelta, PathBuf)>,
}
#[async_trait::async_trait]
impl EventHandler for DeltaHandler {
    async fn handle(&self, event: &Event) {
        match event {
            Event::Apply(tx, state) => {
                let base_path = self
                        .storage_path.join(state.doc().inner.root_id.clone());
                    let path =  base_path .join(format!("delta_{}_{}.bin", tx.time, state.version));
                    fs::create_dir_all(base_path).unwrap();
                let tx_clone = tx.clone();
                let state_version = state.version.clone();
                let path_clone = path.clone();
                let _ = self
                    .tx
                    .send((to_delta(&tx_clone, state_version), path_clone))
                    .await;
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
pub fn create_snapshot_handler(
    storage_path: PathBuf,
    snapshot_interval: usize,
    snapshot_manager: Arc<SnapshotManager>,
) -> Arc<SnapshotHandler> {
    Arc::new(SnapshotHandler {
        storage_path,
        snapshot_interval,
        counter: AtomicUsize::new(0),
        snapshot_manager,
    })
}
#[derive(Debug)]
pub struct SnapshotHandler {
    storage_path: PathBuf,
    snapshot_interval: usize,
    counter: AtomicUsize,
    snapshot_manager: Arc<SnapshotManager>,
}
#[async_trait::async_trait]
impl EventHandler for SnapshotHandler {
    async fn handle(&self, event: &Event) {
        match event {
            Event::Apply(_, state) => {
                let count = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
                if count % self.snapshot_interval == 0 {
                    let state_clone = state.clone();
                    let base_path = self
                        .storage_path.join(state_clone.doc().inner.root_id.clone());
                    let path =  base_path .join(format!("snapshot_v{}.bin", state_clone.version));
                    let _=fs::create_dir_all(base_path);
                    let cache_ref: Arc<SnapshotManager> = self.snapshot_manager.clone();
                    tokio::spawn(async move {
                        cache_ref.put(&path,&state_clone);
                        match create_full_snapshot(&state_clone) {
                            Ok(data) => match File::create(&path).await {
                                Ok(mut file) => {
                                    file.write_all(&data).await.unwrap();
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
            _ => {}
        }
    }
}
