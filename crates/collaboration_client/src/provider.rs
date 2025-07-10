use tokio_tungstenite::connect_async;
use yrs::sync::{Message, SyncMessage};
use yrs::updates::encoder::Encode;
use yrs::{Subscription};
use url::Url;
use crate::AwarenessRef;
use crate::conn::Connection;
use crate::types::*;
use crate::client::{ClientSink, ClientStream};
use futures_util::{SinkExt, StreamExt};

pub struct WebsocketProvider {
    pub server_url: String,
    pub room_name: String,
    pub awareness: AwarenessRef,
    client_conn: Option<Connection<ClientSink, ClientStream>>,
    pub status: ConnectionStatus,
    // 同步检测相关
    sync_event_sender: Option<SyncEventSender>,
    sync_event_receiver: Option<SyncEventReceiver>,

    pub ws_reconnect_attempts: u32,
    pub max_backoff_time: u64,
    pub ws_url: Option<Url>,
    pub client_id: u64,
    subscriptions: Vec<Subscription>,
}

impl WebsocketProvider {
    pub async fn new(
        server_url: String,
        room_name: String,
        awareness: AwarenessRef,
    ) -> Self {
        let (event_sender, event_receiver) =
            tokio::sync::broadcast::channel(100);

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
            status: ConnectionStatus::Disconnected,
            sync_event_sender: Some(event_sender),
            sync_event_receiver: Some(event_receiver),
            ws_reconnect_attempts: 0,
            max_backoff_time: 2500,
            ws_url,
            subscriptions: Vec::new(),
        }
    }

    pub fn subscription(
        &mut self,
        subscription: Subscription,
    ) {
        self.subscriptions.push(subscription);
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

        let ws_url = match &self.ws_url {
            Some(url) => url.as_str(),
            None => {
                self.status = ConnectionStatus::Disconnected;
                return;
            },
        };

        // 建立 WebSocket 连接
        let (ws_stream, _) = match connect_async(ws_url).await {
            Ok(result) => result,
            Err(e) => {
                self.status = ConnectionStatus::Disconnected;
                self.ws_reconnect_attempts += 1;
                tracing::error!("WebSocket 连接失败: {}", e);
                return;
            },
        };

        let (sink, stream) = ws_stream.split();

        // 🔥 使用带同步检测的连接
        let client_conn = Connection::new_with_sync_detection(
            self.awareness.clone(),
            ClientSink(sink),
            ClientStream(stream),
            self.sync_event_sender.clone(),
        );

        self.client_conn = Some(client_conn);
        self.status = ConnectionStatus::Connected;
        self.ws_reconnect_attempts = 0;

        // 设置统一的变更监听器
        self.setup_update_listeners().await;
    }

    /// 设置统一的文档变更监听器
    /// 监听所有文档变更并发送事件通知
    async fn setup_update_listeners(&mut self) {
        // 延迟 100 毫秒，以避免与 yrs-warp 的初始事务发生竞争
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // 1. 监听文档变更
        let doc_subscription = {
            let sink = self.client_conn.as_ref().unwrap().sink();
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
            self.subscriptions.push(subscription);
        }

        // 2. 监听本地 awareness 变更

        {
            let awareness_lock = self.awareness.write().await;
            let sink = self.client_conn.as_ref().unwrap().sink();

            // 修复 on_update 签名以匹配 yrs v0.18.8
            let awareness_subscription =
                awareness_lock.on_update(move |event| {
                    let awareness_update = event.awareness_update().unwrap();
                    let sink_weak = sink.clone();
                    tokio::spawn(async move {
                        let msg: Vec<u8> =
                            Message::Awareness(awareness_update).encode_v1();
                        let binding = sink_weak.upgrade().unwrap();
                        let mut sink = binding.lock().await;
                        sink.send(msg).await.unwrap();
                    });
                });
            self.subscriptions.push(awareness_subscription);
            tracing::info!("✅ 本地 Awareness 变更监听器已设置");
        }
    }

    /// 等待协议级同步完成（包括空房间）
    pub async fn wait_for_protocol_sync(
        &self,
        timeout_ms: u64,
    ) -> anyhow::Result<bool> {
        match &self.client_conn {
            Some(conn) => Ok(conn.wait_for_initial_sync(timeout_ms).await),
            None => Err(anyhow::anyhow!("连接未建立")),
        }
    }

    /// 获取协议同步状态
    pub async fn get_protocol_sync_state(&self) -> Option<ProtocolSyncState> {
        match &self.client_conn {
            Some(conn) => Some(conn.get_protocol_sync_state().await),
            None => None,
        }
    }

    /// 订阅同步事件
    pub fn subscribe_sync_events(&mut self) -> Option<SyncEventReceiver> {
        self.sync_event_receiver.take()
    }

    /// 断开连接并清理资源
    pub async fn disconnect(&mut self) {
        tracing::info!("🔌 断开 WebSocket 连接...");

        // 清理连接
        self.client_conn = None;
        self.status = ConnectionStatus::Disconnected;
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
        tracing::debug!("🧹 WebsocketProvider 已清理");
    }
}
