use std::{fmt::Debug, sync::Arc};

use async_channel::{Receiver, Sender};
use moduforge_core::state::{state::State, transaction::Transaction};
use tokio::{signal, sync::RwLock};

// 事件类型定义
#[derive(Clone)]
pub enum Event {
    Create(Arc<State>),
    TrApply(Arc<Transaction>, Arc<State>), // 事务应用后 + 是否成功
    Stop,// 停止后需要重启
}
/// 事件总线
#[derive(Clone)]
pub struct EventBus {
    tx: Sender<Event>,
    rt: Receiver<Event>,
    event_handlers: Arc<RwLock<Vec<Arc<dyn EventHandler>>>>,
}

impl EventBus {
    pub async fn restart(&self){
        let _= self.broadcast(Event::Stop).await;
        //由于是异步的 延迟50毫秒启动
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        self.start_event_loop();
    }
    pub async fn add_event_handler(&mut self, event_handler: Arc<dyn EventHandler>) {
        let mut write = self.event_handlers.write().await;
        write.push(event_handler);
    }
    pub async fn add_event_handlers(&mut self, event_handlers: Vec<Arc<dyn EventHandler>>) {
        let mut write = self.event_handlers.write().await;
        write.extend(event_handlers);
    }
    /// 启动事件循环
    pub fn start_event_loop(&self) {
        let rx: async_channel::Receiver<Event> = self.subscribe();
        let event_handlers = self.event_handlers.clone();
        tokio::spawn(async move {
            let handlers_clone = {
                let handlers = event_handlers.read().await;
                handlers.clone() // 克隆以避免长时间持有锁
            };
            loop {
                tokio::select! {
                    event = rx.recv() => match event {
                        Ok(Event::Stop) => {
                            println!("接收到停止事件，正在退出...");
                            break;
                        },
                        Ok(event) => {
                            for handler in &handlers_clone {
                                handler.handle(&event).await;
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
                                println!("事件管理器,接收到关闭信号，正在退出...");
                                break;
                            },
                            Err(e) => {
                                eprintln!("事件管理器,处理关闭信号时出错: {}", e);
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
        Self {
            tx,
            rt,
            event_handlers: Arc::new(RwLock::new(vec![])),
        }
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        self.rt.clone()
    }

    pub async fn broadcast(&self, event: Event) -> Result<(), async_channel::SendError<Event>> {
        self.tx.send(event).await?;
        Ok(())
    }
    pub fn broadcast_blocking(&self, event: Event) -> Result<(), async_channel::SendError<Event>> {
        self.tx.send_blocking(event)?;
        Ok(())
    }
}

// 事件处理器特征
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync + Debug {
    async fn handle(&self, event: &Event);
}

// 事件上下文
pub struct EventContext {
    pub state: Arc<State>,
}
