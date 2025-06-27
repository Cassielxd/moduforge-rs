use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use yrs::sync::Awareness;
use yrs::Doc;
use yrs_warp::AwarenessRef;

#[derive(Default)]
#[derive(Debug)]
pub struct YrsManager {
    awareness_refs: DashMap<String, AwarenessRef>,
}

impl YrsManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieves or creates an Awareness reference for a given room.
    ///
    /// If an awareness object for the room doesn't exist, it creates a new Yrs `Doc`,
    /// wraps it in an `Awareness` object, and stores it for future use.
    pub fn get_or_create_awareness(&self, room_id: &str) -> AwarenessRef {
        if let Some(awareness_ref) = self.awareness_refs.get(room_id) {
            return awareness_ref.clone();
        }

        let doc = Doc::new();
        let awareness = Awareness::new(doc);
        let awareness_ref = Arc::new(RwLock::new(awareness));
        self.awareness_refs
            .insert(room_id.to_string(), awareness_ref.clone());
        awareness_ref
    }

    /// Retrieves the awareness reference for a given room, if it exists.
    pub fn get_awareness_ref(&self, room_id: &str) -> Option<AwarenessRef> {
        self.awareness_refs.get(room_id).map(|r| r.value().clone())
    }

    /// 检查房间是否存在
    pub fn room_exists(&self, room_id: &str) -> bool {
        self.awareness_refs.contains_key(room_id)
    }

    /// 获取所有活跃房间的ID列表
    pub fn get_active_rooms(&self) -> Vec<String> {
        self.awareness_refs.iter().map(|entry| entry.key().clone()).collect()
    }

    /// 获取房间数量
    pub fn room_count(&self) -> usize {
        self.awareness_refs.len()
    }

    /// 移除房间并清理相关资源
    /// 这是房间下线的核心方法
    pub async fn remove_room(&self, room_id: &str) -> Option<AwarenessRef> {
        tracing::info!("Removing room '{}' from YrsManager", room_id);
        
        if let Some((_, awareness_ref)) = self.awareness_refs.remove(room_id) {
            tracing::info!("Room '{}' successfully removed from YrsManager", room_id);
            Some(awareness_ref)
        } else {
            tracing::warn!("Attempted to remove non-existent room: '{}'", room_id);
            None
        }
    }

    /// 强制清理房间资源（即使有客户端连接）
    /// 用于紧急情况下的房间清理
    pub async fn force_cleanup_room(&self, room_id: &str) -> bool {
        tracing::warn!("Force cleaning up room: '{}'", room_id);
        
        if let Some((_, awareness_ref)) = self.awareness_refs.remove(room_id) {
            // 尝试获取写锁并清理
            if let Ok(mut awareness) = awareness_ref.try_write() {
                // 清理 awareness 中的客户端状态
                awareness.clean_local_state();
                tracing::info!("Room '{}' force cleanup completed", room_id);
                true
            } else {
                tracing::error!("Failed to acquire write lock for room '{}' during force cleanup", room_id);
                false
            }
        } else {
            tracing::warn!("Room '{}' not found during force cleanup", room_id);
            false
        }
    }

    /// 批量清理多个房间
    pub async fn remove_rooms(&self, room_ids: &[String]) -> Vec<String> {
        let mut removed_rooms = Vec::new();
        
        for room_id in room_ids {
            if self.remove_room(room_id).await.is_some() {
                removed_rooms.push(room_id.clone());
            }
        }
        
        tracing::info!("Batch removed {} out of {} rooms", removed_rooms.len(), room_ids.len());
        removed_rooms
    }

    /// 清理所有房间（服务器关闭时使用）
    pub async fn shutdown_all_rooms(&self) {
        tracing::info!("Shutting down all {} rooms", self.awareness_refs.len());
        
        let all_rooms: Vec<String> = self.awareness_refs.iter()
            .map(|entry| entry.key().clone())
            .collect();
        
        for room_id in all_rooms {
            self.remove_room(&room_id).await;
        }
        
        tracing::info!("All rooms have been shut down");
    }
}
