use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use dashmap::DashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use crate::{Result, TransmissionError, YrsManager, ClientInfo, SyncService};
use anyhow;
use yrs::updates::decoder::Decode;
use moduforge_core::runtime::ForgeRuntime;
use yrs::Transact;
use yrs::ReadTxn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsMessage {
    /// 客户端加入房间
    JoinRoom { room_id: String },
    /// 客户端离开房间
    LeaveRoom { room_id: String },
    /// Yrs update 数据 (二进制)
    YrsUpdate { room_id: String, update: Vec<u8> },
    /// 客户端请求同步
    YrsSyncRequest { room_id: String, state_vector: Vec<u8> },
    /// JSON格式的状态同步消息
    StateSync {
        room_id: String,
        operation: String,
        data: serde_json::Value,
        timestamp: u64,
    },
    /// 心跳
    Ping,
    /// 心跳响应
    Pong,
    /// 错误消息
    Error { message: String },
    /// 服务器通知消息
    Notification { message: String },
}

/// 内部广播消息
#[derive(Debug, Clone)]
pub enum BroadcastMessage {
    /// 向房间广播文本消息
    ToRoom {
        room_id: String,
        message: Arc<String>,
        exclude_client: Option<String>,
    },
    /// 向房间广播二进制消息
    ToRoomBinary {
        room_id: String,
        data: Arc<Vec<u8>>,
        exclude_client: Option<String>,
    },
    /// 向特定客户端发送消息
    ToClient { client_id: String, message: Arc<String> },
    /// 向特定客户端发送二进制消息
    ToClientBinary { client_id: String, data: Arc<Vec<u8>> },
}

/// A message to be sent to a single client's websocket.
/// Using Arc to avoid cloning the underlying data for every client in a broadcast.
#[derive(Debug, Clone)]
pub enum OutgoingMessage {
    Text(Arc<String>),
    Binary(Arc<Vec<u8>>),
    // We can add other types like Close, etc. if needed
}

impl From<OutgoingMessage> for Message {
    fn from(msg: OutgoingMessage) -> Self {
        match msg {
            OutgoingMessage::Text(text) => Message::Text((*text).clone()),
            OutgoingMessage::Binary(data) => Message::Binary((*data).clone()),
        }
    }
}

/// 客户端连接信息（扩展版）
#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub info: ClientInfo,
    pub sender: mpsc::UnboundedSender<OutgoingMessage>,
}

pub struct WebSocketServer {
    yrs_manager: Arc<YrsManager>,
    /// 客户端连接信息和发送通道
    clients: Arc<DashMap<String, ClientConnection>>,
    /// 房间到客户端的映射
    room_clients: Arc<DashMap<String, Vec<String>>>,
    /// 广播消息通道
    broadcast_tx: mpsc::UnboundedSender<BroadcastMessage>,
    broadcast_rx:
        Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<BroadcastMessage>>>,
}

impl WebSocketServer {
    pub fn new(yrs_manager: Arc<YrsManager>) -> Self {
        let (broadcast_tx, broadcast_rx) = mpsc::unbounded_channel();

        Self {
            yrs_manager,
            clients: Arc::new(DashMap::new()),
            room_clients: Arc::new(DashMap::new()),
            broadcast_tx,
            broadcast_rx: Arc::new(tokio::sync::Mutex::new(broadcast_rx)),
        }
    }

    /// 启动 WebSocket 服务器
    pub async fn start(
        self: Arc<Self>,
        addr: SocketAddr,
        sync_service: Arc<SyncService>,
        runtime: Arc<tokio::sync::Mutex<moduforge_core::runtime::ForgeRuntime>>,
    ) -> Result<()> {
        tokio::spawn(async move {
            
        let listener = TcpListener::bind(addr).await.unwrap();
        tracing::info!("WebSocket 服务器启动在: {}", addr);

        // 启动广播处理任务
        self.clone().start_broadcast_handler().await;

        while let Ok((stream, addr)) = listener.accept().await {
            let server = self.clone();
            let sync_service = sync_service.clone();
            let runtime = runtime.clone();

            tokio::spawn(async move {
                if let Err(e) = server
                    .handle_connection(stream, addr, sync_service, runtime)
                    .await
                {
                    tracing::error!("处理连接失败: {}", e);
                }
            });
        }
        });

        Ok(())
    }

    /// 启动广播消息处理器
    async fn start_broadcast_handler(self: Arc<Self>) {
        let clients = self.clients.clone();
        let room_clients = self.room_clients.clone();
        let broadcast_rx = self.broadcast_rx.clone();

        tokio::spawn(async move {
            let mut rx = broadcast_rx.lock().await;

            while let Some(broadcast_msg) = rx.recv().await {
                Self::handle_broadcast_message(
                    broadcast_msg,
                    &clients,
                    &room_clients,
                )
                .await;
            }
        });
    }

    /// 处理广播消息
    async fn handle_broadcast_message(
        message: BroadcastMessage,
        clients: &DashMap<String, ClientConnection>,
        room_clients: &DashMap<String, Vec<String>>,
    ) {
        match message {
            BroadcastMessage::ToRoom { room_id, message, exclude_client } => {
                let outgoing_msg = OutgoingMessage::Text(message);
                if let Some(client_list) = room_clients.get(&room_id) {
                    for client_id in client_list.iter() {
                        if let Some(ref exclude) = exclude_client {
                            if client_id == exclude {
                                continue;
                            }
                        }

                        if let Some(client) = clients.get(client_id) {
                            let _ = client.sender.send(outgoing_msg.clone());
                        }
                    }
                }
            },
            BroadcastMessage::ToRoomBinary {
                room_id,
                data,
                exclude_client,
            } => {
                let outgoing_msg = OutgoingMessage::Binary(data);
                if let Some(client_list) = room_clients.get(&room_id) {
                    for client_id in client_list.iter() {
                        if let Some(ref exclude) = exclude_client {
                            if client_id == exclude {
                                continue;
                            }
                        }

                        if let Some(client) = clients.get(client_id) {
                            let _ = client.sender.send(outgoing_msg.clone());
                        }
                    }
                }
            },
            BroadcastMessage::ToClient { client_id, message } => {
                if let Some(client) = clients.get(&client_id) {
                    let _ = client.sender.send(OutgoingMessage::Text(message));
                }
            },
            BroadcastMessage::ToClientBinary { client_id, data } => {
                if let Some(client) = clients.get(&client_id) {
                    let _ = client.sender.send(OutgoingMessage::Binary(data));
                }
            },
        }
    }

    async fn handle_connection(
        self: Arc<Self>,
        stream: TcpStream,
        addr: SocketAddr,
        sync_service: Arc<SyncService>,
        runtime: Arc<tokio::sync::Mutex<moduforge_core::runtime::ForgeRuntime>>,
    ) -> Result<()> {
        let ws_stream = accept_async(stream).await.map_err(|e| {
            TransmissionError::Other(anyhow::anyhow!("WebSocket升级失败: {}", e))
        })?;

        let client_id = Uuid::new_v4().to_string();
        let (tx, mut rx) = mpsc::unbounded_channel::<OutgoingMessage>();

        let client_info = ClientInfo {
            id: client_id.clone(),
            room_id: String::new(),
            connected_at: std::time::SystemTime::now(),
        };

        let client_connection = ClientConnection {
            info: client_info,
            sender: tx,
        };

        self.clients.insert(client_id.clone(), client_connection);

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // 处理接收到的消息
        let server = self.clone();
        let sync_service_clone = sync_service.clone();
        let runtime_clone = runtime.clone();
        let client_id_clone = client_id.clone();

        let receive_task = tokio::spawn(async move {
            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(ws_message) = serde_json::from_str::<WsMessage>(&text) {
                            if let Err(e) = server
                                .handle_message(&client_id_clone, ws_message, &sync_service_clone, &runtime_clone)
                                .await
                            {
                                tracing::error!("处理消息失败: {}", e);
                            }
                        }
                    },
                    Ok(Message::Binary(data)) => {
                        // 处理二进制消息（Yrs updates）
                        tracing::debug!("收到二进制消息，长度: {}", data.len());
                    },
                    Ok(Message::Close(_)) => {
                        tracing::info!("客户端 {} 主动断开连接", client_id_clone);
                        break;
                    },
                    Err(e) => {
                        tracing::error!("WebSocket错误: {}", e);
                        break;
                    },
                    _ => {}
                }
            }
        });

        // 处理发送的消息
        let send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = ws_sender.send(msg.into()).await {
                    tracing::error!("发送消息失败: {}", e);
                    break;
                }
            }
        });

        // 等待任一任务完成
        tokio::select! {
            _ = receive_task => {
                tracing::info!("接收任务完成");
            }
            _ = send_task => {
                tracing::info!("发送任务完成");
            }
        }

        // 清理客户端连接
        self.cleanup_client(&client_id).await;

        Ok(())
    }

    async fn handle_message(
        &self,
        client_id: &str,
        message: WsMessage,
        sync_service: &Arc<SyncService>,
        runtime: &Arc<tokio::sync::Mutex<moduforge_core::runtime::ForgeRuntime>>,
    ) -> Result<()> {
        match message {
            WsMessage::JoinRoom { room_id } => {
                if let Some(mut client) = self.clients.get_mut(client_id) {
                    client.info.room_id = room_id.clone();
                }
                self.room_clients
                    .entry(room_id.clone())
                    .or_default()
                    .push(client_id.to_string());
                let _doc = self.yrs_manager.get_or_create_doc(&room_id);
                tracing::info!("客户端 {} 加入房间 {}", client_id, room_id);
                Ok(())
            },
            WsMessage::LeaveRoom { room_id } => {
                if let Some(mut client_list) =
                    self.room_clients.get_mut(&room_id)
                {
                    client_list.retain(|id| id != client_id);
                }
                if let Some(mut client) = self.clients.get_mut(client_id) {
                    client.info.room_id = String::new();
                }
                tracing::info!("客户端 {} 离开房间 {}", client_id, room_id);
                Ok(())
            },
            WsMessage::YrsUpdate { room_id, update } => {
                let doc =
                    self.yrs_manager.get_doc(&room_id).ok_or_else(|| {
                        TransmissionError::RoomNotFound(room_id.clone())
                    })?;
                if let Err(e) = apply_update_to_doc(&doc, &update) {
                    tracing::error!("应用更新失败: {}", e);
                } else {
                    self.broadcast_tx
                        .send(BroadcastMessage::ToRoomBinary {
                            room_id,
                            data: Arc::new(update),
                            exclude_client: Some(client_id.to_string()),
                        })
                        .map_err(|e| {
                            TransmissionError::Other(anyhow::anyhow!(
                                e.to_string()
                            ))
                        })?;
                }
                Ok(())
            },
            WsMessage::YrsSyncRequest { room_id, state_vector } => {
                tracing::debug!(
                    "收到来自 {} 的同步请求，房间: {}",
                    client_id,
                    room_id
                );

                let tree = runtime.lock().await.doc().get_inner().clone();
                let diff_update = sync_service
                    .handle_sync_request(&room_id, &tree, &state_vector)
                    .await?;

                if !diff_update.is_empty() {
                    self.broadcast_tx
                        .send(BroadcastMessage::ToClientBinary {
                            client_id: client_id.to_string(),
                            data: Arc::new(diff_update),
                        })
                        .map_err(|e| {
                            TransmissionError::Other(anyhow::anyhow!(
                                e.to_string()
                            ))
                        })?;
                }

                Ok(())
            },
            WsMessage::Ping => {
                let pong = WsMessage::Pong;
                let json = serde_json::to_string(&pong)?;
                self.broadcast_tx
                    .send(BroadcastMessage::ToClient {
                        client_id: client_id.to_string(),
                        message: Arc::new(json),
                    })
                    .map_err(|e| {
                        TransmissionError::Other(anyhow::anyhow!(e.to_string()))
                    })?;
                Ok(())
            },
            _ => Ok(()),
        }
    }

    async fn cleanup_client(
        &self,
        client_id: &str,
    ) {
        if let Some((_, client)) = self.clients.remove(client_id) {
            tracing::info!("客户端断开连接: {}", client_id);
            let room_id = client.info.room_id;

            if !room_id.is_empty() {
                let should_remove_doc = if let Some(mut room) =
                    self.room_clients.get_mut(&room_id)
                {
                    room.retain(|id| id != client_id);
                    room.is_empty()
                } else {
                    false
                };

                if should_remove_doc {
                    if self.room_clients.remove(&room_id).is_some() {
                        self.yrs_manager.remove_doc(&room_id);
                        tracing::info!("房间 {} 已空，移除Yrs Doc", room_id);
                    }
                }
            }
        }
    }

    /// 获取在线客户端数量
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// 获取房间数量
    pub fn room_count(&self) -> usize {
        self.room_clients.len()
    }

    /// 🚀 主动推送消息到房间
    pub fn broadcast_to_room(
        &self,
        room_id: &str,
        message: String,
        exclude_client: Option<String>,
    ) -> Result<()> {
        self.broadcast_tx
            .send(BroadcastMessage::ToRoom {
                room_id: room_id.to_string(),
                message: Arc::new(message),
                exclude_client,
            })
            .map_err(|e| {
                TransmissionError::Other(anyhow::anyhow!(e.to_string()))
            })
    }

    /// 🚀 主动推送二进制数据到房间
    pub fn broadcast_binary_to_room(
        &self,
        room_id: &str,
        data: Vec<u8>,
        exclude_client: Option<String>,
    ) -> Result<()> {
        self.broadcast_tx
            .send(BroadcastMessage::ToRoomBinary {
                room_id: room_id.to_string(),
                data: Arc::new(data),
                exclude_client,
            })
            .map_err(|e| {
                TransmissionError::Other(anyhow::anyhow!(e.to_string()))
            })
    }

    /// 🚀 主动推送消息到特定客户端
    pub fn send_to_client(
        &self,
        client_id: &str,
        message: String,
    ) -> Result<()> {
        self.broadcast_tx
            .send(BroadcastMessage::ToClient {
                client_id: client_id.to_string(),
                message: Arc::new(message),
            })
            .map_err(|e| {
                TransmissionError::Other(anyhow::anyhow!(e.to_string()))
            })
    }

    /// 🚀 主动推送二进制数据到特定客户端
    pub fn send_binary_to_client(
        &self,
        client_id: &str,
        data: Vec<u8>,
    ) -> Result<()> {
        self.broadcast_tx
            .send(BroadcastMessage::ToClientBinary {
                client_id: client_id.to_string(),
                data: Arc::new(data),
            })
            .map_err(|e| {
                TransmissionError::Other(anyhow::anyhow!(e.to_string()))
            })
    }
}

// Helper functions extracted for clarity
fn apply_update_to_doc(
    doc: &Arc<yrs::Doc>,
    update: &[u8],
) -> crate::Result<()> {
    let u = yrs::Update::decode_v1(update)?;
    let mut txn = doc.transact_mut();
    txn.apply_update(u)?;
    Ok(())
}

#[allow(dead_code)]
fn get_diff_update_from_doc(
    doc: &Arc<yrs::Doc>,
    sv: &[u8],
) -> std::result::Result<Vec<u8>, yrs::encoding::read::Error> {
    let state_vector = yrs::StateVector::decode_v1(sv)?;
    let txn = doc.transact();
    Ok(txn.encode_diff_v1(&state_vector))
}
