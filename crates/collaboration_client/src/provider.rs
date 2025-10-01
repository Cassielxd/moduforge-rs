use std::time::Duration;

use tokio::time::timeout;
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
        if let Err(e) = self.smart_connect().await {
            tracing::error!("{}", e);
        }
    }
    pub async fn connect_with_retry(
        &mut self,
        config: Option<ConnectionRetryConfig>,
    ) -> anyhow::Result<()> {
        let config = config.unwrap_or_default();
        let mut attempt = 0;
        let mut delay = config.initial_delay_ms;

        while attempt < config.max_attempts {
            attempt += 1;
            self.update_status(ConnectionStatus::Retrying {
                attempt,
                max_attempts: config.max_attempts,
            });

            tracing::info!("🔄 连接尝试 {}/{}", attempt, config.max_attempts);

            match self.try_connect().await {
                Ok(()) => {
                    self.update_status(ConnectionStatus::Connected);
                    return Ok(());
                },
                Err(e) => {
                    let error = self.classify_connection_error(&e);

                    if attempt >= config.max_attempts {
                        // 🔥 发送连接失败事件
                        if let Some(sender) = &self.sync_event_sender {
                            let _ = sender.send(SyncEvent::ConnectionFailed(
                                error.clone(),
                            ));
                        }
                        self.update_status(ConnectionStatus::Failed(
                            error.clone(),
                        ));
                        tracing::error!(
                            "❌ 连接失败，已达到最大重试次数: {}",
                            error
                        );
                        return Err(anyhow::anyhow!("连接失败: {}", error));
                    }

                    tracing::warn!(
                        "⚠️ 连接失败 (尝试 {}/{}): {}",
                        attempt,
                        config.max_attempts,
                        error
                    );

                    // 指数退避延迟
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                    delay = (delay as f64 * config.backoff_multiplier) as u64;
                    delay = delay.min(config.max_delay_ms);
                },
            }
        }

        Err(anyhow::anyhow!("连接失败，已达到最大重试次数"))
    }
    async fn try_connect(&mut self) -> anyhow::Result<()> {
        if self.status == ConnectionStatus::Connected
            || self.status == ConnectionStatus::Connecting
        {
            return Ok(());
        }

        self.status = ConnectionStatus::Connecting;

        let ws_url = match &self.ws_url {
            Some(url) => url.as_str(),
            None => {
                return Err(anyhow::anyhow!("无效的 WebSocket URL"));
            },
        };

        // 设置连接超时
        let connect_timeout = Duration::from_secs(10);

        match timeout(connect_timeout, connect_async(ws_url)).await {
            Ok(connect_result) => {
                match connect_result {
                    Ok((ws_stream, _)) => {
                        let (sink, stream) = ws_stream.split();

                        // 使用带同步检测的连接
                        let client_conn = Connection::new_with_sync_detection(
                            self.awareness.clone(),
                            ClientSink(sink),
                            ClientStream(stream),
                            self.sync_event_sender.clone(),
                        );

                        self.client_conn = Some(client_conn);
                        self.ws_reconnect_attempts = 0;

                        Ok(())
                    },
                    Err(e) => {
                        self.status = ConnectionStatus::Disconnected;
                        Err(anyhow::anyhow!("WebSocket 连接失败: {}", e))
                    },
                }
            },
            Err(_) => {
                self.status = ConnectionStatus::Disconnected;
                Err(anyhow::anyhow!("连接超时"))
            },
        }
    }

    /// 清理文档与 awareness 的订阅监听器
    fn clear_subscriptions(&mut self) {
        if !self.subscriptions.is_empty() {
            tracing::debug!(
                count = self.subscriptions.len(),
                "🧹 正在清理订阅监听器: {} 个",
                self.subscriptions.len()
            );
        }
        // 逐一 drop 以解除注册
        self.subscriptions.drain(..);
    }

    /// 分类连接错误
    fn classify_connection_error(
        &self,
        error: &anyhow::Error,
    ) -> ConnectionError {
        let error_str = error.to_string().to_lowercase();

        if error_str.contains("timeout") || error_str.contains("timed out") {
            ConnectionError::Timeout(10000)
        } else if error_str.contains("connection refused")
            || error_str.contains("failed to connect")
        {
            ConnectionError::ServerUnavailable(
                "服务端未启动或端口未开放".to_string(),
            )
        } else if error_str.contains("websocket") {
            ConnectionError::WebSocketError(error.to_string())
        } else {
            ConnectionError::NetworkError(error.to_string())
        }
    }
    fn update_status(
        &mut self,
        new_status: ConnectionStatus,
    ) {
        self.status = new_status.clone();

        // 发送状态变化事件
        if let Some(sender) = &self.sync_event_sender {
            let _ = sender.send(SyncEvent::ConnectionChanged(new_status));
        }
    }
    /// 检查服务端是否可用
    pub async fn check_server_availability(&self) -> bool {
        if let Some(ws_url) = &self.ws_url {
            let http_url = ws_url
                .as_str()
                .replace("ws://", "http://")
                .replace("wss://", "https://");

            // 尝试 HTTP 连接检查
            match tokio::time::timeout(
                Duration::from_secs(3),
                reqwest::get(&http_url),
            )
            .await
            {
                Ok(Ok(_)) => true,
                _ => false,
            }
        } else {
            false
        }
    }
    // 智能连接（先检查服务端可用性）
    pub async fn smart_connect(&mut self) -> anyhow::Result<()> {
        // 先检查服务端是否可用
        if !self.check_server_availability().await {
            self.status = ConnectionStatus::Failed(
                ConnectionError::ServerUnavailable("服务端未启动".to_string()),
            );
            return Err(anyhow::anyhow!("服务端未启动或不可访问"));
        }

        // 使用重试机制连接
        self.connect_with_retry(None).await?;
        self.setup_update_listeners().await;
        Ok(())
    }

    /// 设置统一的文档变更监听器
    /// 监听所有文档变更并发送事件通知
    async fn setup_update_listeners(&mut self) {
        // 延迟 100 毫秒，以避免与 yrs-warp 的初始事务发生竞争
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // 确保客户端连接存在
        let conn = match self.client_conn.as_ref() {
            Some(conn) => conn,
            None => {
                tracing::error!("尝试设置监听器时客户端连接不存在");
                return;
            }
        };

        // 1. 监听文档变更
        let doc_subscription = {
            let sink = conn.sink();
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
                                if let Some(binding) = sink_weak.upgrade() {
                                    let mut sink = binding.lock().await;
                                    if let Err(e) = sink.send(msg).await {
                                        tracing::debug!(
                                            "忽略发送错误（可能已断开）: {}",
                                            e
                                        );
                                    }
                                } else {
                                    tracing::debug!(
                                        "发送通道已释放（可能已断开），跳过文档更新发送"
                                    );
                                }
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
            // 再次安全获取连接（虽然上面已经检查过，但为了代码清晰性）
            let conn = match self.client_conn.as_ref() {
                Some(conn) => conn,
                None => {
                    tracing::error!("尝试设置 awareness 监听器时客户端连接不存在");
                    return;
                }
            };
            let sink = conn.sink();

            // 修复 on_update 签名以匹配 yrs v0.18.8
            let awareness_subscription = awareness_lock.on_update(move |event| {
                if let Some(awareness_update) = event.awareness_update() {
                    let sink_weak = sink.clone();
                    tokio::spawn(async move {
                        let msg: Vec<u8> =
                            Message::Awareness(awareness_update).encode_v1();
                        if let Some(binding) = sink_weak.upgrade() {
                            let mut sink = binding.lock().await;
                            if let Err(e) = sink.send(msg).await {
                                tracing::debug!(
                                    "忽略发送错误（可能已断开）: {}",
                                    e
                                );
                            }
                        } else {
                            tracing::debug!(
                                "发送通道已释放（可能已断开），跳过 Awareness 发送"
                            );
                        }
                    });
                }
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

        // 1) 先清理订阅监听器，防止回调在断连后继续触发
        self.clear_subscriptions();

        // 2) 优雅关闭连接（关闭 sink 以促使处理循环退出）
        if let Some(conn) = self.client_conn.take() {
            if let Err(e) = conn.close().await {
                tracing::debug!("关闭连接时出现错误（忽略）: {:?}", e);
            }
        }

        // 3) 更新状态并通知
        self.update_status(ConnectionStatus::Disconnected);
        tracing::info!("✅ WebSocket 连接已断开且监听器已清理");
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
        self.clear_subscriptions();
        tracing::debug!("🧹 WebsocketProvider 已清理（订阅监听器已释放）");
    }
}
