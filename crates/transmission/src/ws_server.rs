use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use dashmap::DashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use crate::{Result, TransmissionError, YrsManager, ClientInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsMessage {
    /// 客户端加入房间
    JoinRoom { room_id: String },
    /// 客户端离开房间
    LeaveRoom { room_id: String },
    /// Yrs update 数据 (二进制)
    YrsUpdate { room_id: String, update: Vec<u8> },
    /// JSON格式的状态同步消息
    StateSync { 
        room_id: String, 
        operation: String,
        data: serde_json::Value,
        timestamp: u64 
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
    ToRoom { room_id: String, message: String, exclude_client: Option<String> },
    /// 向房间广播二进制消息
    ToRoomBinary { room_id: String, data: Vec<u8>, exclude_client: Option<String> },
    /// 向特定客户端发送消息
    ToClient { client_id: String, message: String },
    /// 向特定客户端发送二进制消息
    ToClientBinary { client_id: String, data: Vec<u8> },
}

/// 客户端连接信息（扩展版）
#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub info: ClientInfo,
    pub sender: mpsc::UnboundedSender<Message>,
}

pub struct WebSocketServer {
    yrs_manager: Arc<YrsManager>,
    /// 客户端连接信息和发送通道
    clients: Arc<DashMap<String, ClientConnection>>,
    /// 房间到客户端的映射
    room_clients: Arc<DashMap<String, Vec<String>>>,
    /// 广播消息通道
    broadcast_tx: mpsc::UnboundedSender<BroadcastMessage>,
    broadcast_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<BroadcastMessage>>>,
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
    pub async fn start(&self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        tracing::info!("WebSocket 服务器启动在: {}", addr);

        // 启动广播处理任务
        self.start_broadcast_handler().await;

        while let Ok((stream, addr)) = listener.accept().await {
            let clients = self.clients.clone();
            let room_clients = self.room_clients.clone();
            let yrs_manager = self.yrs_manager.clone();
            let broadcast_tx = self.broadcast_tx.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(
                    stream, 
                    addr, 
                    clients, 
                    room_clients, 
                    yrs_manager,
                    broadcast_tx
                ).await {
                    tracing::error!("处理连接失败: {}", e);
                }
            });
        }

        Ok(())
    }

    /// 启动广播消息处理器
    async fn start_broadcast_handler(&self) {
        let clients = self.clients.clone();
        let room_clients = self.room_clients.clone();
        let broadcast_rx = self.broadcast_rx.clone();
        
        tokio::spawn(async move {
            let mut rx = broadcast_rx.lock().await;
            
            while let Some(broadcast_msg) = rx.recv().await {
                Self::handle_broadcast_message(broadcast_msg, &clients, &room_clients).await;
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
                if let Some(client_list) = room_clients.get(&room_id) {
                    for client_id in client_list.iter() {
                        if let Some(ref exclude) = exclude_client {
                            if client_id == exclude {
                                continue;
                            }
                        }
                        
                        if let Some(client) = clients.get(client_id) {
                            let _ = client.sender.send(Message::Text(message.clone()));
                        }
                    }
                }
            }
            BroadcastMessage::ToRoomBinary { room_id, data, exclude_client } => {
                if let Some(client_list) = room_clients.get(&room_id) {
                    for client_id in client_list.iter() {
                        if let Some(ref exclude) = exclude_client {
                            if client_id == exclude {
                                continue;
                            }
                        }
                        
                        if let Some(client) = clients.get(client_id) {
                            let _ = client.sender.send(Message::Binary(data.clone()));
                        }
                    }
                }
            }
            BroadcastMessage::ToClient { client_id, message } => {
                if let Some(client) = clients.get(&client_id) {
                    let _ = client.sender.send(Message::Text(message));
                }
            }
            BroadcastMessage::ToClientBinary { client_id, data } => {
                if let Some(client) = clients.get(&client_id) {
                    let _ = client.sender.send(Message::Binary(data));
                }
            }
        }
    }

    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        clients: Arc<DashMap<String, ClientConnection>>,
        room_clients: Arc<DashMap<String, Vec<String>>>,
        yrs_manager: Arc<YrsManager>,
        broadcast_tx: mpsc::UnboundedSender<BroadcastMessage>,
    ) -> Result<()> {
        let ws_stream = accept_async(stream).await
            .map_err(|e| TransmissionError::WebSocketError(e.to_string()))?;
        
        let client_id = Uuid::new_v4().to_string();
        tracing::info!("新客户端连接: {} ({})", client_id, addr);

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let (msg_tx, mut msg_rx) = mpsc::unbounded_channel();

        // 存储客户端连接信息
        let client_info = ClientInfo {
            id: client_id.clone(),
            room_id: String::new(),
            connected_at: std::time::SystemTime::now(),
        };
        
        let client_connection = ClientConnection {
            info: client_info,
            sender: msg_tx,
        };
        
        clients.insert(client_id.clone(), client_connection);

        let client_id_clone = client_id.clone();
        let clients_clone = clients.clone();
        let room_clients_clone = room_clients.clone();
        
        // 启动发送任务
        let send_task = tokio::spawn(async move {
            while let Some(message) = msg_rx.recv().await {
                if ws_sender.send(message).await.is_err() {
                    break;
                }
            }
        });

        // 处理接收消息循环
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        match Self::handle_message(
                            &client_id,
                            ws_msg,
                            &clients,
                            &room_clients,
                            &yrs_manager,
                            &broadcast_tx,
                        ).await {
                            Ok(_) => {},
                            Err(e) => {
                                let error_msg = WsMessage::Error { 
                                    message: e.to_string() 
                                };
                                if let Ok(json) = serde_json::to_string(&error_msg) {
                                    let _ = broadcast_tx.send(BroadcastMessage::ToClient {
                                        client_id: client_id.clone(),
                                        message: json,
                                    });
                                }
                            }
                        }
                    }
                }
                Ok(Message::Binary(data)) => {
                    // 处理二进制数据（Yrs update）
                    if let Some(client_info) = clients.get(&client_id) {
                        let room_id = client_info.info.room_id.clone();
                        if !room_id.is_empty() {
                            // 应用 update 到 Yrs 文档
                            if let Err(e) = yrs_manager.apply_update(&room_id, &data) {
                                tracing::error!("应用 Yrs update 失败: {}", e);
                            } else {
                                // 广播给房间内其他客户端
                                let _ = broadcast_tx.send(BroadcastMessage::ToRoomBinary {
                                    room_id,
                                    data,
                                    exclude_client: Some(client_id.clone()),
                                });
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("客户端 {} 断开连接", client_id);
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket 错误: {}", e);
                    break;
                }
                _ => {}
            }
        }

        // 清理连接
        send_task.abort();
        Self::cleanup_client(&client_id_clone, &clients_clone, &room_clients_clone).await;

        Ok(())
    }

    async fn handle_message(
        client_id: &str,
        message: WsMessage,
        clients: &DashMap<String, ClientConnection>,
        room_clients: &DashMap<String, Vec<String>>,
        yrs_manager: &YrsManager,
        broadcast_tx: &mpsc::UnboundedSender<BroadcastMessage>,
    ) -> Result<()> {
        match message {
            WsMessage::JoinRoom { room_id } => {
                // 更新客户端信息
                if let Some(mut client) = clients.get_mut(client_id) {
                    client.info.room_id = room_id.clone();
                }

                // 添加到房间客户端列表
                room_clients.entry(room_id.clone())
                    .or_insert_with(Vec::new)
                    .push(client_id.to_string());

                // 发送全量更新
                let state_update = yrs_manager.get_full_state_update(&room_id)?;
                let _ = broadcast_tx.send(BroadcastMessage::ToClientBinary {
                    client_id: client_id.to_string(),
                    data: state_update,
                });

                tracing::info!("客户端 {} 加入房间 {}", client_id, room_id);
                Ok(())
            }
            WsMessage::LeaveRoom { room_id } => {
                // 从房间移除客户端
                if let Some(mut client_list) = room_clients.get_mut(&room_id) {
                    client_list.retain(|id| id != client_id);
                }

                // 更新客户端信息
                if let Some(mut client) = clients.get_mut(client_id) {
                    client.info.room_id.clear();
                }

                tracing::info!("客户端 {} 离开房间 {}", client_id, room_id);
                Ok(())
            }
            WsMessage::YrsUpdate { room_id, update } => {
                // 应用 update 到 Yrs 文档
                yrs_manager.apply_update(&room_id, &update)?;
                
                // 广播给房间内其他客户端
                let _ = broadcast_tx.send(BroadcastMessage::ToRoomBinary {
                    room_id,
                    data: update,
                    exclude_client: Some(client_id.to_string()),
                });
                
                Ok(())
            }
            WsMessage::StateSync { room_id: _, operation: _, data: _, timestamp: _ } => {
                // 处理 JSON 格式的状态同步消息
                // 这里需要根据实际需求实现状态同步逻辑
                Ok(())
            }
            WsMessage::Ping => {
                let pong = WsMessage::Pong;
                let json = serde_json::to_string(&pong)?;
                let _ = broadcast_tx.send(BroadcastMessage::ToClient {
                    client_id: client_id.to_string(),
                    message: json,
                });
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn cleanup_client(
        client_id: &str,
        clients: &DashMap<String, ClientConnection>,
        room_clients: &DashMap<String, Vec<String>>,
    ) {
        // 从所有房间移除客户端
        if let Some((_, client)) = clients.remove(client_id) {
            if !client.info.room_id.is_empty() {
                if let Some(mut client_list) = room_clients.get_mut(&client.info.room_id) {
                    client_list.retain(|id| id != client_id);
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
    pub fn broadcast_to_room(&self, room_id: &str, message: String, exclude_client: Option<String>) -> Result<()> {
        self.broadcast_tx.send(BroadcastMessage::ToRoom {
            room_id: room_id.to_string(),
            message,
            exclude_client,
        }).map_err(|e| TransmissionError::WebSocketError(e.to_string()))?;
        Ok(())
    }

    /// 🚀 主动推送二进制数据到房间
    pub fn broadcast_binary_to_room(&self, room_id: &str, data: Vec<u8>, exclude_client: Option<String>) -> Result<()> {
        self.broadcast_tx.send(BroadcastMessage::ToRoomBinary {
            room_id: room_id.to_string(),
            data,
            exclude_client,
        }).map_err(|e| TransmissionError::WebSocketError(e.to_string()))?;
        Ok(())
    }

    /// 🚀 主动推送消息到特定客户端
    pub fn send_to_client(&self, client_id: &str, message: String) -> Result<()> {
        self.broadcast_tx.send(BroadcastMessage::ToClient {
            client_id: client_id.to_string(),
            message,
        }).map_err(|e| TransmissionError::WebSocketError(e.to_string()))?;
        Ok(())
    }

    /// 🚀 主动推送二进制数据到特定客户端
    pub fn send_binary_to_client(&self, client_id: &str, data: Vec<u8>) -> Result<()> {
        self.broadcast_tx.send(BroadcastMessage::ToClientBinary {
            client_id: client_id.to_string(),
            data,
        }).map_err(|e| TransmissionError::WebSocketError(e.to_string()))?;
        Ok(())
    }
} 