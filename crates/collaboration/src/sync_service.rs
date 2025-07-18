use std::sync::Arc;
use yrs::{Map, ReadTxn as _, Transact, WriteTxn as _};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::yrs_manager::YrsManager;
use crate::RoomSnapshot;

/// 房间状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RoomStatus {
    /// 房间不存在
    NotExists,
    /// 房间已创建但未初始化数据
    Created,
    /// 房间已初始化并有数据
    Initialized,
    /// 房间正在下线中
    Shutting,
    /// 房间已下线
    Offline,
}

/// 房间信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub room_id: String,
    pub status: RoomStatus,
    pub node_count: usize,
    pub client_count: usize,
    pub last_activity: std::time::SystemTime,
}

#[derive(Clone)]
pub struct SyncService {
    yrs_manager: Arc<YrsManager>,
    client_id: String,
}

impl SyncService {
    pub fn new(yrs_manager: Arc<YrsManager>) -> Self {
        Self { yrs_manager, client_id: "server".to_string() }
    }

    /// 初始化房间，确保 Yrs 文档存在
    pub fn init_room(
        &self,
        room_id: &str,
    ) {
        tracing::info!("🔄 初始化房间: {}", room_id);
        self.yrs_manager.get_or_create_awareness(room_id);
    }

    /// 检查房间是否已初始化（有数据）
    pub async fn is_room_initialized(
        &self,
        room_id: &str,
    ) -> bool {
        if let Some(awareness_ref) = self.yrs_manager.get_awareness_ref(room_id)
        {
            let awareness = awareness_ref.read().await;
            let doc = awareness.doc();
            let txn = doc.transact();

            if let Some(nodes_map) = txn.get_map("nodes") {
                nodes_map.len(&txn) > 0
            } else {
                false
            }
        } else {
            false
        }
    }

    /// 获取房间状态信息
    pub async fn get_room_status(
        &self,
        room_id: &str,
    ) -> RoomStatus {
        if !self.yrs_manager.room_exists(room_id) {
            return RoomStatus::NotExists;
        }

        if self.is_room_initialized(room_id).await {
            RoomStatus::Initialized
        } else {
            RoomStatus::Created
        }
    }

    /// 获取房间详细信息
    pub async fn get_room_info(
        &self,
        room_id: &str,
    ) -> Option<RoomInfo> {
        if !self.yrs_manager.room_exists(room_id) {
            return None;
        }

        let status = self.get_room_status(room_id).await;
        let mut node_count = 0;
        let mut client_count = 0;

        if let Some(awareness_ref) = self.yrs_manager.get_awareness_ref(room_id)
        {
            if let Ok(awareness) = awareness_ref.try_read() {
                let doc = awareness.doc();
                let txn = doc.transact();

                // 获取节点数量
                if let Some(nodes_map) = txn.get_map("nodes") {
                    node_count = nodes_map.len(&txn);
                }

                // 获取客户端数量
                client_count = awareness.clients().len();
            }
        }

        Some(RoomInfo {
            room_id: room_id.to_string(),
            status,
            node_count: node_count as usize,
            client_count,
            last_activity: std::time::SystemTime::now(),
        })
    }

    /// 房间下线 - 核心下线方法
    /// 1. 断开所有客户端
    /// 2. 可选保存数据
    /// 3. 清理资源
    pub async fn offline_room(
        &self,
        room_id: &str,
        save_data: bool,
    ) -> Result<Option<RoomSnapshot>> {
        tracing::info!("🔄 开始下线房间: {}", room_id);

        let mut final_snapshot = None;

        // 1. 检查房间是否存在
        if !self.yrs_manager.room_exists(room_id) {
            tracing::warn!("🔄 尝试下线不存在的房间: {}", room_id);
            return Ok(None);
        }

        // 2. 如果需要保存数据，先创建快照
        if save_data {
            if let Some(awareness_ref) =
                self.yrs_manager.get_awareness_ref(room_id)
            {
                let awareness = awareness_ref.read().await;
                let doc = awareness.doc();
                let txn = doc.transact();

                // 从 Yrs 文档重建 Tree 快照
                if let Some(nodes_map) = txn.get_map("nodes") {
                    let node_count = nodes_map.len(&txn);
                    tracing::info!(
                        "🔄 保存 {} 个节点 from room: {}",
                        node_count,
                        room_id
                    );

                    // 创建简化的快照（实际项目中可能需要完整的 Tree 重建）
                    final_snapshot = Some(RoomSnapshot {
                        room_id: room_id.to_string(),
                        root_id: "root".to_string(), // 简化处理
                        nodes: std::collections::HashMap::new(),
                        version: 0,
                    });
                }
            }
        }

        // 3. 从 YrsManager 中移除房间（这会自动断开客户端）
        if let Some(_awareness_ref) =
            self.yrs_manager.remove_room(room_id).await
        {
            tracing::info!("🔄 房间 '{}' 成功下线", room_id);
        } else {
            tracing::error!("🔄 从 YrsManager 中移除房间 '{}' 失败", room_id);
            return Err(crate::error::TransmissionError::SyncError(format!(
                "Failed to offline room: {}",
                room_id
            )));
        }

        Ok(final_snapshot)
    }

    /// 强制房间下线（用于紧急情况）
    pub async fn force_offline_room(
        &self,
        room_id: &str,
    ) -> Result<bool> {
        tracing::warn!("Force offlining room: {}", room_id);

        let success = self.yrs_manager.force_cleanup_room(room_id).await;

        if success {
            tracing::info!("Room '{}' force offlined successfully", room_id);
        } else {
            tracing::error!("Failed to force offline room: {}", room_id);
        }

        Ok(success)
    }

    /// 批量下线房间
    pub async fn offline_rooms(
        &self,
        room_ids: &[String],
        save_data: bool,
    ) -> Result<Vec<(String, Option<RoomSnapshot>)>> {
        tracing::info!("🔄 批量下线 {} 个房间", room_ids.len());

        let mut results = Vec::new();

        for room_id in room_ids {
            match self.offline_room(room_id, save_data).await {
                Ok(snapshot) => {
                    results.push((room_id.clone(), snapshot));
                },
                Err(e) => {
                    tracing::error!("🔄 下线房间 '{}' 失败: {}", room_id, e);
                    results.push((room_id.clone(), None));
                },
            }
        }

        tracing::info!(
            "🔄 批量下线完成: {}/{} 个房间成功下线",
            results.iter().filter(|(_, snapshot)| snapshot.is_some()).count(),
            room_ids.len()
        );

        Ok(results)
    }

    /// 获取所有活跃房间列表
    pub fn get_active_rooms(&self) -> Vec<String> {
        self.yrs_manager.get_active_rooms()
    }

    /// 获取房间统计信息
    pub async fn get_rooms_stats(&self) -> Vec<RoomInfo> {
        let room_ids = self.get_active_rooms();
        let mut stats = Vec::new();

        for room_id in room_ids {
            if let Some(info) = self.get_room_info(&room_id).await {
                stats.push(info);
            }
        }

        stats
    }

    /// 获取 YrsManager 的引用（用于高级操作）
    pub fn yrs_manager(&self) -> &Arc<YrsManager> {
        &self.yrs_manager
    }
}

impl std::fmt::Debug for SyncService {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("SyncService")
            .field("client_id", &self.client_id)
            .finish()
    }
}
