use std::sync::{Arc};
use tokio::sync::broadcast;
use yrs::sync::{Message, SyncMessage};
use yrs::types::ToJson;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Subscription, Transact, Update};
use url::Url;
use yrs_warp::AwarenessRef;
use crate::types::*;
use crate::conn::ClientConn;
use futures_util::SinkExt;

pub struct WebsocketProvider {
    pub server_url: String,
    pub room_name: String,
    pub awareness: AwarenessRef,
    client_conn: Option<ClientConn>,
    event_sender: broadcast::Sender<ProviderEvent>,
    pub status: ConnectionStatus,
    pub ws_reconnect_attempts: u32,
    pub max_backoff_time: u64,
    pub ws_url: Option<Url>,
    pub client_id: u64,
    // 消息处理器
    message_handler: Option<Arc<dyn MessageHandler>>,
    // 新增：监听器订阅管理
    local_update_subscription: Option<Subscription>,
    awareness_subscription: Option<Box<dyn std::any::Any + Send + Sync>>,
}

impl WebsocketProvider {
    pub async fn new(
        server_url: String,
        room_name: String,
        awareness: AwarenessRef,
    ) -> Self {
        tracing::info!("Creating new WebsocketProvider");

        Self::new_with_options(
            server_url,
            room_name,
            awareness,
            WebsocketProviderOptions::default(),
        )
        .await
    }

    pub async fn new_with_options(
        server_url: String,
        room_name: String,
        awareness: AwarenessRef,
        options: WebsocketProviderOptions,
    ) -> Self {
        let (event_sender, _) = broadcast::channel(16);
        let ws_url = Url::parse(&format!(
            "{}/{}",
            server_url.trim_end_matches('/'),
            room_name
        ))
        .ok();
        let client_id = awareness.read().await.doc().client_id();
        Self {
            client_id,
            server_url,
            room_name,
            awareness,
            client_conn: None,
            event_sender,
            status: ConnectionStatus::Disconnected,
            ws_reconnect_attempts: 0,
            max_backoff_time: options.max_backoff_time,
            ws_url,
            message_handler: None,
            local_update_subscription: None,
            awareness_subscription: None,
        }
    }

    /// 创建带扩展的WebSocket客户端
    pub async fn new_with_ext<T: MessageHandlerExt + 'static>(
        server_url: String,
        room_name: String,
        awareness: AwarenessRef,
        ext: T,
    ) -> Self {
        let provider = Self::new_with_options(
            server_url,
            room_name,
            awareness,
            WebsocketProviderOptions::default(),
        )
        .await;
        provider
    }

    /// 创建带扩展和选项的WebSocket客户端
    pub async fn new_with_ext_and_options<T: MessageHandlerExt + 'static>(
        server_url: String,
        room_name: String,
        awareness: AwarenessRef,
        ext: T,
        options: WebsocketProviderOptions,
    ) -> Self {
        let mut provider =
            Self::new_with_options(server_url, room_name, awareness, options)
                .await;
        provider
    }

    pub async fn connect(&mut self) {
        if self.status == ConnectionStatus::Connected
            || self.status == ConnectionStatus::Connecting
        {
            return;
        }
        self.setup_connection().await;
    }

    async fn setup_connection(&mut self) {
        self.status = ConnectionStatus::Connecting;
        let _ = self
            .event_sender
            .send(ProviderEvent::Status("connecting".to_string()));

        let ws_url = match &self.ws_url {
            Some(url) => url.as_str(),
            None => {
                let _ = self.event_sender.send(ProviderEvent::ConnectionError(
                    "Invalid WebSocket URL".to_string(),
                ));
                self.status = ConnectionStatus::Disconnected;
                return;
            },
        };

        // 使用 ClientConn 建立连接，直接传递 awareness
        let client_conn =
            match ClientConn::connect(ws_url, self.awareness.clone()).await {
                Ok(conn) => conn,
                Err(e) => {
                    let _ =
                        self.event_sender.send(ProviderEvent::ConnectionError(
                            format!("ClientConn connect error: {}", e),
                        ));
                    self.status = ConnectionStatus::Disconnected;
                    self.ws_reconnect_attempts += 1;

                    // 指数退避重连
                    let backoff = std::cmp::min(
                        100 * 2u64.pow(self.ws_reconnect_attempts),
                        self.max_backoff_time,
                    );
                    let event_sender = self.event_sender.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            backoff,
                        ))
                        .await;
                        let _ = event_sender.send(ProviderEvent::Status(
                            "reconnect".to_string(),
                        ));
                    });
                    return;
                },
            };
        self.client_conn = Some(client_conn);

        self.status = ConnectionStatus::Connected;
        self.ws_reconnect_attempts = 0;
        let _ = self
            .event_sender
            .send(ProviderEvent::Status("connected".to_string()));

        // 设置统一的变更监听器
        self.setup_update_listeners().await;

        let _ = self.event_sender.send(ProviderEvent::Sync(true));
    }

    /// 设置统一的文档变更监听器
    /// 监听所有文档变更并发送事件通知
    async fn setup_update_listeners(&mut self) {
        // 清理之前的监听器
        self.cleanup_listeners();

        // 延迟 100 毫秒，以避免与 yrs-warp 的初始事务发生竞争
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // 1. 监听文档变更
        let doc_subscription = {
            let sink = self.client_conn.as_ref().unwrap().0.sink();
            let client_id = self.client_id.clone();
            let awareness_lock = self.awareness.read().await;
            let doc = awareness_lock.doc();
            doc.observe_update_v1(move |txn, event| {
                let origin = txn.origin();

                if let Some(origin_ref) = origin {
                    let origin_bytes = origin_ref.as_ref();
                    if let Ok(origin_str) = std::str::from_utf8(origin_bytes) {
                        let update = event.update.to_owned();
                        if origin_str == client_id.to_string() {
                            let sink_weak = sink.clone();
                            tokio::spawn(async move {
                                let msg =
                                    Message::Sync(SyncMessage::Update(update))
                                        .encode_v1();
                                let binding = sink_weak.upgrade().unwrap();
                                let mut sink = binding.lock().await;
                                sink.send(msg).await.unwrap();
                            });
                        }
                    }
                }
            })
        };

        // 保存订阅
        if let Ok(subscription) = doc_subscription {
            self.local_update_subscription = Some(subscription);
        }

        // 2. 监听本地 awareness 变更
        let event_sender_awareness = self.event_sender.clone();

        {
            let awareness_lock = self.awareness.write().await;
            let client_id: u64 = awareness_lock.doc().client_id();
            let sink = self.client_conn.as_ref().unwrap().0.sink();

            // 修复 on_update 签名以匹配 yrs v0.18.8
            let awareness_subscription =
                awareness_lock.on_update(move |event| {
                    let preview = format!(
                        "added: {:?}, updated: {:?}, removed: {:?}",
                        event.added(),
                        event.updated(),
                        event.removed()
                    );
                    let awareness_update = event.awareness_update().unwrap();
                    let sink_weak = sink.clone();
                    tokio::spawn(async move {
                        let msg: Vec<u8> =
                            Message::Awareness(awareness_update).encode_v1();
                        let binding = sink_weak.upgrade().unwrap();
                        let mut sink = binding.lock().await;
                        sink.send(msg).await.unwrap();
                    });

                    let data_length = event.added().len()
                        + event.updated().len()
                        + event.removed().len();

                    // 发送 Awareness 更新事件
                    let _ = event_sender_awareness.send(
                        ProviderEvent::AwarenessMessage {
                            data_length,
                            preview,
                        },
                    );
                });

            // 将 awareness subscription 保存为 Any 类型
            self.awareness_subscription =
                Some(Box::new(awareness_subscription));
            tracing::info!("✅ 本地 Awareness 变更监听器已设置");
        }
    }

    /// 清理所有监听器
    fn cleanup_listeners(&mut self) {
        // 清理本地文档订阅
        if let Some(_subscription) = self.local_update_subscription.take() {
            // subscription 会在 drop 时自动清理
            tracing::debug!("🧹 清理本地文档变更订阅");
        }

        // 清理 awareness 订阅
        if let Some(_subscription) = self.awareness_subscription.take() {
            // subscription 会在 drop 时自动清理
            tracing::debug!("🧹 清理 Awareness 变更订阅");
        }
    }

    pub fn broadcast_message(
        &self,
        _buf: Vec<u8>,
    ) {
        // TODO: 广播消息到 ws/bc
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ProviderEvent> {
        self.event_sender.subscribe()
    }

    /// 断开连接并清理资源
    pub async fn disconnect(&mut self) {
        tracing::info!("🔌 断开 WebSocket 连接...");

        // 清理监听器
        self.cleanup_listeners();

        // 清理连接
        self.client_conn = None;
        self.status = ConnectionStatus::Disconnected;

        // 发送断开连接事件
        let _ = self.event_sender.send(ProviderEvent::ConnectionClose);
        let _ = self
            .event_sender
            .send(ProviderEvent::Status("disconnected".to_string()));

        tracing::info!("✅ WebSocket 连接已断开");
    }

    /// 检查连接状态
    pub fn is_connected(&self) -> bool {
        self.status == ConnectionStatus::Connected && self.client_conn.is_some()
    }

    /// 获取连接状态
    pub fn get_status(&self) -> &ConnectionStatus {
        &self.status
    }
}

impl Drop for WebsocketProvider {
    fn drop(&mut self) {
        // 在析构时清理监听器
        self.cleanup_listeners();
        tracing::debug!("🧹 WebsocketProvider 已清理");
    }
}
