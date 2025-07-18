use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Failed(ConnectionError), // 新增失败状态
    Retrying { attempt: u32, max_attempts: u32 }, // 新增重试状态
}

pub struct WebsocketProviderOptions {
    pub connect: bool,
    pub resync_interval: Option<u64>,
    pub max_backoff_time: u64,
}

impl Default for WebsocketProviderOptions {
    fn default() -> Self {
        Self { connect: true, resync_interval: None, max_backoff_time: 2500 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSnapshot {
    pub room_id: String,
    pub root_id: String,
    pub nodes: HashMap<String, NodeData>,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub id: String,
    pub node_type: String,
    pub attrs: HashMap<String, serde_json::Value>,
    pub content: Vec<String>,
    pub marks: Vec<MarkData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkData {
    pub mark_type: String,
    pub attrs: HashMap<String, serde_json::Value>,
}

/// Step操作结果 - 用于记录操作信息并发送给前端
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub step_name: String,
    pub description: String,
    pub timestamp: u64,
    pub client_id: String,
}

/// 协议同步状态
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolSyncState {
    /// 未开始
    NotStarted,
    /// 已发送 SyncStep1
    Step1Sent,
    /// 已接收 SyncStep2 - 这就是首次同步完成的标志！
    Step2Received,
    /// 后续更新中
    Updating,
}

/// 同步事件
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// 协议同步状态变化
    ProtocolStateChanged(ProtocolSyncState),
    /// 首次同步完成（空房间也算）
    InitialSyncCompleted { has_data: bool, elapsed_ms: u64 },
    /// 收到数据更新
    DataReceived,
    /// 连接状态变化
    ConnectionChanged(ConnectionStatus),
    /// 连接失败
    ConnectionFailed(ConnectionError),
}

/// 同步事件回调
pub type SyncEventSender = broadcast::Sender<SyncEvent>;
pub type SyncEventReceiver = broadcast::Receiver<SyncEvent>;

/// 连接错误类型
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum ConnectionError {
    #[error("服务端未启动或无法连接: {0}")]
    ServerUnavailable(String),
    #[error("连接超时: {0}ms")]
    Timeout(u64),
    #[error("网络错误: {0}")]
    NetworkError(String),
    #[error("WebSocket 错误: {0}")]
    WebSocketError(String),
}

#[derive(Debug, Clone)]
pub struct ConnectionRetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for ConnectionRetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}
