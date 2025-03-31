use std::{fmt::Debug, sync::Arc, time::Duration};

use async_channel::{Receiver, Sender};
use futures::future::join_all;
use moduforge_core::{
    debug,
    state::{state::State, transaction::Transaction},
};
use tokio::{signal, sync::RwLock, time::timeout};

use crate::error::{EditorResult, error_utils};

// 事件类型定义
#[derive(Clone)]
pub enum Event {
    Create(Arc<State>),
    TrApply(Arc<Vec<Transaction>>, Arc<State>), // 事务应用后 + 是否成功
    Destroy,                                     // 销毁事件
    Stop,                                       // 停止后需要重启
}
/// 事件总线
#[derive(Clone)]
pub struct EventBus {
    tx: Sender<Event>,
    rt: Receiver<Event>,
    event_handlers: Arc<RwLock<Vec<Arc<dyn EventHandler>>>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub async fn restart(&self) -> EditorResult<()> {
        self.broadcast(Event::Stop).await?;
        //由于是异步的 延迟50毫秒启动
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        self.start_event_loop();
        Ok(())
    }
    pub async fn add_event_handler(
        &mut self,
        event_handler: Arc<dyn EventHandler>,
    ) -> EditorResult<()> {
        let mut write = self.event_handlers.write().await;
        write.push(event_handler);
        Ok(())
    }
    pub async fn add_event_handlers(
        &mut self,
        event_handlers: Vec<Arc<dyn EventHandler>>,
    ) -> EditorResult<()> {
        let mut write = self.event_handlers.write().await;
        write.extend(event_handlers);
        Ok(())
    }
    /// 启动事件循环
    pub fn start_event_loop(&self) {
        let rx: async_channel::Receiver<Event> = self.subscribe();
        let event_handlers = self.event_handlers.clone();
        tokio::spawn(async move {
            let handlers_clone = {
                let handlers = event_handlers.read().await;
                handlers.clone()
            };
            loop {
                tokio::select! {
                    event = rx.recv() => match event {
                        Ok(Event::Stop) => {
                            debug!("接收到停止事件，等待所有处理器完成...");
                            // 等待所有正在进行的处理完成
                            let mut pending_handles = Vec::new();
                            for handler in &handlers_clone {
                                let handle = handler.handle(&Event::Stop);
                                pending_handles.push(handle);
                            }
                            // 设置超时时间为5秒
                            if let Err(e) = timeout(Duration::from_secs(5), join_all(pending_handles)).await {
                                debug!("等待处理器完成超时: {}", e);
                            }
                            break;
                        },
                        Ok(event) => {
                            // 并发处理所有handler
                            let mut handles = Vec::new();
                            for handler in &handlers_clone {
                                let handle = handler.handle(&event);
                                handles.push(handle);
                            }
                            
                            // 设置每个handler的超时时间为3秒
                            let results = join_all(handles.into_iter().map(|handle| {
                                timeout(Duration::from_secs(3), handle)
                            })).await;
                            
                            // 处理结果
                            for result in results {
                                match result {
                                    Ok(Ok(())) => continue,
                                    Ok(Err(e)) => debug!("事件处理错误: {}", e),
                                    Err(e) => debug!("事件处理超时: {}", e),
                                }
                            }
                        },
                        Err(e) => {
                            debug!("事件接收错误: {}", e);
                            break;
                        },
                    },
                    shutdown_signal = Box::pin(signal::ctrl_c()) => {
                        match shutdown_signal {
                            Ok(()) => {
                                debug!("事件管理器,接收到关闭信号，正在退出...");
                                break;
                            },
                            Err(e) => {
                                debug!("事件管理器,处理关闭信号时出错: {}", e);
                                break;
                            }
                        }
                    },
                }
            }
        });
    }

    pub fn new() -> Self {
        let (tx, rt) = async_channel::bounded(100);
        Self { tx, rt, event_handlers: Arc::new(RwLock::new(vec![])) }
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        self.rt.clone()
    }

    pub async fn broadcast(
        &self,
        event: Event,
    ) -> EditorResult<()> {
        self.tx.send(event).await.map_err(|e| {
            error_utils::event_error(format!(
                "Failed to broadcast event: {}",
                e
            ))
        })
    }
    pub fn broadcast_blocking(
        &self,
        event: Event,
    ) -> EditorResult<()> {
        self.tx.send_blocking(event).map_err(|e| {
            error_utils::event_error(format!(
                "Failed to broadcast event: {}",
                e
            ))
        })
    }
}

impl Drop for EventBus {
    fn drop(&mut self) {
        // Create a new runtime to handle async operations during drop
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Broadcast Stop event to signal handlers to complete
            if let Err(e) = self.broadcast_blocking(Event::Stop) {
                debug!("Failed to broadcast stop event during drop: {}", e);
            }
            
            // Wait for handlers to complete with a timeout
            let handlers = self.event_handlers.read().await;
            let mut pending_handles = Vec::new();
            for handler in handlers.iter() {
                let handle = handler.handle(&Event::Stop);
                pending_handles.push(handle);
            }
            
            // Wait up to 5 seconds for all handlers to complete
            if let Err(e) = timeout(Duration::from_secs(5), join_all(pending_handles)).await {
                debug!("Timeout waiting for handlers to complete during drop: {}", e);
            }
        });
    }
}

// 事件处理器特征
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync + Debug {
    async fn handle(
        &self,
        event: &Event,
    ) -> EditorResult<()>;
}

// 事件上下文
pub struct EventContext {
    pub state: Arc<State>,
}
