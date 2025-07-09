use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
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
