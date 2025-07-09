use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

use async_channel::{Receiver, Sender};
use mf_state::{debug, state::State, Transaction};
use tokio::{signal};

use crate::error::{ForgeResult, error_utils};

// 事件类型定义
#[derive(Clone)]
pub enum Event {
    Create(Arc<State>),
    TrApply(u64, Arc<Vec<Transaction>>, Arc<State>), // 事务应用后 + 是否成功
    Destroy,                                         // 销毁事件
    Stop,                                            // 停止后需要重启
}

impl Event {
    pub fn name(&self) -> &'static str {
        match self {
            Event::Create(_) => "Create",
            Event::TrApply(_, _, _) => "TrApply",
            Event::Destroy => "Destroy",
            Event::Stop => "Stop",
        }
    }
}

/// 事件总线
#[derive(Clone)]
pub struct EventBus<T: Send + 'static> {
    tx: Sender<T>,
    rt: Receiver<T>,
    event_handlers: Arc<RwLock<Vec<Arc<dyn EventHandler<T>>>>>,
    shutdown: (Sender<()>, Receiver<()>),
}

impl<T: Send + 'static> Default for EventBus<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send + 'static> EventBus<T> {
    pub fn add_event_handler(
        &self,
        event_handler: Arc<dyn EventHandler<T>>,
    ) -> ForgeResult<()> {
        let mut write = self.event_handlers.write().unwrap();
        write.push(event_handler);
        Ok(())
    }
    pub fn add_event_handlers(
        &self,
        event_handlers: Vec<Arc<dyn EventHandler<T>>>,
    ) -> ForgeResult<()> {
        let mut write = self.event_handlers.write().unwrap();
        write.extend(event_handlers);
        Ok(())
    }
    pub fn destroy(&self) {
        let _ = self.shutdown.0.send_blocking(());
    }
    /// 启动事件循环
    pub fn start_event_loop(&self) {
        let rx: async_channel::Receiver<T> = self.subscribe();
        let event_handlers = self.event_handlers.clone();
        let shutdown_rt = self.shutdown.1.clone();
        tokio::spawn(async move {
            let mut join_set = tokio::task::JoinSet::new();
              
            // 定义清理函数，确保所有任务都被正确清理
            async fn cleanup_tasks(join_set: &mut tokio::task::JoinSet<()>) {
                debug!("开始清理事件处理任务...");
                // 首先停止接受新任务
                join_set.shutdown().await;
                // 然后等待所有现有任务完成，设置超时防止无限等待
                let cleanup_timeout = std::time::Duration::from_secs(5);
                match tokio::time::timeout(cleanup_timeout, async {
                    while let Some(result) = join_set.join_next().await {
                        if let Err(e) = result {
                            debug!("事件处理任务错误: {}", e);
                        }
                    }
                }).await {
                    Ok(_) => debug!("所有事件处理任务已正常清理"),
                    Err(_) => debug!("事件处理任务清理超时"),
                }
            }
            loop {
                tokio::select! {
                    event = rx.recv() => match event {
                        Ok(event) => {
                            // 限制并发任务数量，防止无限制spawning
                            const MAX_CONCURRENT_TASKS: usize = 100;
                            if join_set.len() >= MAX_CONCURRENT_TASKS {
                                debug!("事件处理任务数量达到上限，等待部分任务完成...");
                                // 等待至少一个任务完成
                                if let Some(result) = join_set.join_next().await {
                                    if let Err(e) = result {
                                        debug!("事件处理任务错误: {}", e);
                                    }
                                }
                            }
                                                       
                            // 并发处理所有handler
                            let handlers_clone = event_handlers.read().unwrap().clone();
                            join_set.spawn(async move {
                                // 并发处理所有handler
                                // 并发处理所有handler，添加超时保护
                                let handler_timeout = std::time::Duration::from_secs(30);
                                for handler in &handlers_clone {
                                    //let _ = handler.handle(&event).await;
                                    match tokio::time::timeout(handler_timeout, handler.handle(&event)).await {
                                        Ok(Ok(_)) => {}, // 处理成功
                                        Ok(Err(e)) => debug!("事件处理器执行失败: {}", e),
                                        Err(_) => debug!("事件处理器执行超时"),
                                    }
                                }
                            });
                        },
                        Err(e) => {
                            debug!("事件接收错误: {}", e);
                            cleanup_tasks(&mut join_set).await;
                            break;
                        },
                    },
                    _ = shutdown_rt.recv() => {
                        let _ = join_set.join_all().await;
                        debug!("事件管理器,接收到关闭信号，正在退出...");
                        break;
                    },
                    shutdown_signal = Box::pin(signal::ctrl_c()) => {
                        match shutdown_signal {
                            Ok(()) => {
                                debug!("事件管理器,接收到关闭信号，正在退出...");
                                cleanup_tasks(&mut join_set).await;
                                break;
                            },
                            Err(e) => {
                                debug!("事件管理器,处理关闭信号时出错: {}", e);
                                cleanup_tasks(&mut join_set).await;
                                let _ = join_set.shutdown().await;
                                break;
                            }
                        }
                    }
                    // 定期清理已完成的任务，防止JoinSet无限增长
                    _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                        // 非阻塞地清理已完成的任务
                        while let Some(result) = join_set.try_join_next() {
                            if let Err(e) = result {
                                debug!("事件处理任务错误: {}", e);
                            }
                        }
                    },
                }
            }
        });
    }

    pub fn new() -> Self {
        let (tx, rt) = async_channel::bounded(100);
        let (shutdown_tx, shutdown_rt) = async_channel::bounded(1);
        Self {
            tx,
            rt,
            event_handlers: Arc::new(RwLock::new(vec![])),
            shutdown: (shutdown_tx, shutdown_rt),
        }
    }

    pub fn subscribe(&self) -> Receiver<T> {
        self.rt.clone()
    }

    pub async fn broadcast(
        &self,
        event: T,
    ) -> ForgeResult<()> {
        self.tx.send(event).await.map_err(|e| {
            error_utils::event_error(format!("广播事件失败: {}", e))
        })
    }
    pub fn broadcast_blocking(
        &self,
        event: T,
    ) -> ForgeResult<()> {
        self.tx.send_blocking(event).map_err(|e| {
            error_utils::event_error(format!("广播事件失败: {}", e))
        })
    }
}

// 事件处理器特征
#[async_trait::async_trait]
pub trait EventHandler<T>: Send + Sync + Debug {
    async fn handle(
        &self,
        event: &T,
    ) -> ForgeResult<()>;
}
