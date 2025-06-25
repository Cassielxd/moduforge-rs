use std::sync::Arc;
use moduforge_model::tree::Tree;
use moduforge_state::{State, Transaction};
use yrs::{ReadTxn, Transact};
use crate::{
    Result, 
    YrsManager, 
    WebSocketServer, 
    RoomSnapshot,
    mapping::Mapper,
    TransmissionError,
};
use crate::ws_server::WsMessage;
use yrs::updates::decoder::Decode;

#[derive(Clone)]
pub struct SyncService {
    yrs_manager: Arc<YrsManager>,
    ws_server: Arc<WebSocketServer>,
}

impl SyncService {
    pub fn new(yrs_manager: Arc<YrsManager>, ws_server: Arc<WebSocketServer>) -> Self {
        Self {
            yrs_manager,
            ws_server,
        }
    }

    /// 将 ModuForge 的 Tree 初始化为房间快照
    pub fn init_room_from_tree(&self, room_id: &str, _tree: &Tree) -> Result<()> {
        tracing::info!("初始化房间 {} 从 Tree", room_id);
        
        // 创建房间（空的 Yrs 文档）
        let _doc = self.yrs_manager.get_or_create_doc(room_id);
        
        // 这里不存储全量数据到 Yrs，只初始化空文档
        // 前端需要全量数据时通过 get_room_snapshot 获取
        
        Ok(())
    }

    /// 获取房间的完整快照（用于前端首次加载）
    pub fn get_room_snapshot(&self, room_id: &str, tree: &Tree) -> RoomSnapshot {
        tracing::debug!("获取房间 {} 的快照", room_id);
        Mapper::tree_to_snapshot(tree, room_id.to_string())
    }

    /// 🚀 内部方法：处理事务变更并以二进制格式批量推送到前端
    pub async fn handle_transaction_applied(
        &self,
        room_id: &str,
        transactions: &[Transaction],
        _new_state: &State,
        client_id: Option<String>,
    ) -> Result<()> {
        if transactions.is_empty() {
            return Ok(());
        }

        tracing::info!("检测到 {} 个事务变更，开始批量同步到房间 {}", transactions.len(), room_id);
        
        let client_id = client_id.unwrap_or_else(|| "server".to_string());
        let doc = self.yrs_manager.get_or_create_doc(room_id);
        let registry = Mapper::global_registry();

        // The update is generated from the transaction, so we need to capture it.
        let update: Vec<u8>; 
        { // Scoped to confine the transaction
            let mut txn = doc.transact_mut();
            for (i, transaction) in transactions.iter().enumerate() {
                tracing::debug!("处理事务 {}/{}: {} 个操作", i + 1, transactions.len(), transaction.steps.len());
                for (j, step) in transaction.steps.iter().enumerate() {
                    let type_name = std::any::type_name_of_val(step.as_ref());
                    tracing::debug!("  - 操作 {}/{}: {}", j + 1, transaction.steps.len(), type_name);
                    
                    if let Some(converter) = registry.find_converter(step.as_ref()) {
                        if let Err(e) = converter.apply_to_yrs_txn(step.as_ref(), &mut txn, &client_id) {
                            tracing::error!("应用Step到Yrs事务失败: {}", e);
                        }
                    } else {
                        tracing::warn!("未找到Step转换器: {}", type_name);
                    }
                }
            }
            // At the end of the transaction, encode the changes made *within this transaction*.
            update = txn.encode_update_v1();
        } // Yrs transaction commits here

        // 🚀 主动推送 update 给连接的客户端
        if !update.is_empty() {
            self.ws_server.broadcast_binary_to_room(room_id, update, Some(client_id))?;
        }

        // 发送通知消息
        /* let change_msg = format!("应用了 {} 个事务，总共 {} 个操作", 
            transactions.len(), 
            transactions.iter().map(|t| t.steps.len()).sum::<usize>()
        );
        
        if let Err(e) = self.notify_room_change(room_id, change_msg) {
            tracing::error!("发送变更通知失败: {}", e);
        } */

        Ok(())
    }


    /// 移除房间
    pub fn remove_room(&self, room_id: &str) {
        tracing::info!("移除房间: {}", room_id);
        if self.yrs_manager.remove_doc(room_id).is_some() {
            tracing::info!("房间 {} 的 Yrs Doc 已成功移除", room_id);
        } else {
            tracing::warn!("尝试移除房间 {} 的 Yrs Doc，但未找到", room_id);
        }
    }

    /// 获取服务状态
    pub fn get_status(&self) -> SyncServiceStatus {
        SyncServiceStatus {
            client_count: self.ws_server.client_count(),
            room_count: self.ws_server.room_count(),
            rooms: self.yrs_manager.list_rooms(),
        }
    }

    /// 获取 YrsManager 的引用（用于高级操作）
    pub fn yrs_manager(&self) -> &Arc<YrsManager> {
        &self.yrs_manager
    }

    /// 获取 WebSocketServer 的引用（用于高级操作）
    pub fn ws_server(&self) -> &Arc<WebSocketServer> {
        &self.ws_server
    }

    /// 🚀 主动推送消息到房间的所有客户端
    pub fn broadcast_message_to_room(&self, room_id: &str, message: String) -> Result<()> {
        self.ws_server.broadcast_to_room(room_id, message, None)
    }

    /// 🚀 主动推送二进制数据到房间的所有客户端
    pub fn broadcast_data_to_room(&self, room_id: &str, data: Vec<u8>) -> Result<()> {
        self.ws_server.broadcast_binary_to_room(room_id, data, None)
    }

    /// 🚀 主动推送消息到特定客户端
    pub fn send_message_to_client(&self, client_id: &str, message: String) -> Result<()> {
        self.ws_server.send_to_client(client_id, message)
    }

    /// 🚀 主动推送二进制数据到特定客户端
    pub fn send_data_to_client(&self, client_id: &str, data: Vec<u8>) -> Result<()> {
        self.ws_server.send_binary_to_client(client_id, data)
    }

    /// 🚀 主动推送 ModuForge 变更通知到房间
    pub fn notify_room_change(&self, room_id: &str, change_description: String) -> Result<()> {
        let notification = WsMessage::Notification { 
            message: change_description
        };
        
        let json = serde_json::to_string(&notification)?;
        self.broadcast_message_to_room(room_id, json)
    }

    /// 🚀 为新加入的客户端处理同步请求
    /// 1. 从权威State生成一个临时的Doc快照
    /// 2. 将当前房间的增量更新应用到快照上
    /// 3. 根据客户端的状态向量计算并返回最终的diff
    pub async fn handle_sync_request(
        &self,
        room_id: &str,
        tree: &Tree,
        client_state_vector: &[u8],
    ) -> Result<Vec<u8>> {
        // A: 从权威State生成一个临时的Doc快照
        let snapshot_doc = Mapper::tree_to_yrs_doc(tree);

        // B: 获取当前房间的在线增量Doc
        if let Some(online_doc) = self.yrs_manager.get_doc(room_id) {
            // C: 将在线Doc的更新应用到快照上
            let online_update = online_doc.transact().encode_state_as_update_v1(&Default::default());
            let mut snapshot_txn = snapshot_doc.transact_mut();
            let update_decoded = yrs::Update::decode_v1(&online_update)
                .map_err(|e| TransmissionError::YrsError(format!("Failed to decode online doc update: {}", e)))?;
            snapshot_txn.apply_update(update_decoded)?;
        }

        // D: 根据客户端的状态向量计算并返回最终的diff
        let client_sv = yrs::StateVector::decode_v1(client_state_vector)
            .map_err(|e| TransmissionError::YrsError(e.to_string()))?;
        
        let final_diff = snapshot_doc.transact().encode_diff_v1(&client_sv);
        
        Ok(final_diff)
    }

    /// 🚀 主动推送 JSON 格式的状态同步消息到房间
    pub fn broadcast_json_sync_to_room(&self, room_id: &str, operation: &str, data: serde_json::Value) -> Result<()> {
        let sync_message = WsMessage::StateSync {
            room_id: room_id.to_string(),
            operation: operation.to_string(),
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };
        
        let json = serde_json::to_string(&sync_message)?;
        self.broadcast_message_to_room(room_id, json)
    }
}

// 添加 Debug trait 实现
impl std::fmt::Debug for SyncService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncService")
            .field("yrs_manager", &"YrsManager")
            .field("ws_server", &"WebSocketServer")
            .finish()
    }
}

impl Default for SyncService {
    fn default() -> Self {
        let yrs_manager = Arc::new(YrsManager::new());
        let ws_server = Arc::new(WebSocketServer::new(yrs_manager.clone()));
        Self::new(yrs_manager, ws_server)
    }
}

#[derive(Debug, Clone)]
pub struct SyncServiceStatus {
    pub client_count: usize,
    pub room_count: usize,
    pub rooms: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use moduforge_model::{node::Node, attrs::Attrs};
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_sync_service_creation() {
        let service = SyncService::default();
        let status = service.get_status();
        
        assert_eq!(status.client_count, 0);
        assert_eq!(status.room_count, 0);
        assert!(status.rooms.is_empty());
    }
} 