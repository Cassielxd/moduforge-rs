use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub id: String,
    pub room_id: String,
    pub connected_at: std::time::SystemTime,
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
