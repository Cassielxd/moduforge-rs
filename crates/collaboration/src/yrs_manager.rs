use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use yrs::sync::Awareness;
use yrs::Doc;
use yrs_warp::AwarenessRef;

#[derive(Default, Debug)]
pub struct YrsManager {
    awareness_refs: DashMap<String, AwarenessRef>,
}

impl YrsManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取或创建房间的 Awareness 引用
    ///
    /// 如果房间的 awareness 对象不存在，则创建一个新的 Yrs `Doc`，
    /// 将其包装在 `Awareness` 对象中，并存储供未来使用。
    pub fn get_or_create_awareness(
        &self,
        room_id: &str,
    ) -> AwarenessRef {
        if let Some(awareness_ref) = self.awareness_refs.get(room_id) {
            return awareness_ref.clone();
        }

        let doc: Doc = Doc::new();
        let awareness = Awareness::new(doc);
        let awareness_ref = Arc::new(RwLock::new(awareness));
        self.awareness_refs.insert(room_id.to_string(), awareness_ref.clone());
        awareness_ref
    }

    /// 获取给定房间的 awareness 引用，如果存在的话
    pub fn get_awareness_ref(
        &self,
        room_id: &str,
    ) -> Option<AwarenessRef> {
        self.awareness_refs.get(room_id).map(|r| r.value().clone())
    }

    /// 检查房间是否存在
    pub fn room_exists(
        &self,
        room_id: &str,
    ) -> bool {
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
    pub async fn remove_room(
        &self,
        room_id: &str,
    ) -> Option<AwarenessRef> {
        tracing::info!("🔄 移除房间: '{}'", room_id);

        if let Some((_, awareness_ref)) = self.awareness_refs.remove(room_id) {
            tracing::info!("🔄 房间 '{}' 成功 removed", room_id);
            Some(awareness_ref)
        } else {
            tracing::warn!("🔄 尝试移除不存在的房间: '{}'", room_id);
            None
        }
    }

    /// 强制清理房间资源（即使有客户端连接）
    /// 用于紧急情况下的房间清理
    pub async fn force_cleanup_room(
        &self,
        room_id: &str,
    ) -> bool {
        tracing::warn!("🔄 强制清理房间: '{}'", room_id);

        if let Some((_, awareness_ref)) = self.awareness_refs.remove(room_id) {
            // 尝试获取写锁并清理
            if let Ok(mut awareness) = awareness_ref.try_write() {
                // 清理 awareness 中的客户端状态
                awareness.clean_local_state();
                tracing::info!("房间 '{}' 强制清理完成", room_id);
                true
            } else {
                tracing::error!("🔄 获取写锁失败");
                false
            }
        } else {
            tracing::warn!("🔄 房间 '{}' 不存在", room_id);
            false
        }
    }

    /// 批量清理多个房间
    pub async fn remove_rooms(
        &self,
        room_ids: &[String],
    ) -> Vec<String> {
        let mut removed_rooms = Vec::new();

        for room_id in room_ids {
            if self.remove_room(room_id).await.is_some() {
                removed_rooms.push(room_id.clone());
            }
        }

        tracing::info!(
            "🔄 批量移除 {} 个房间 out of {} rooms",
            removed_rooms.len(),
            room_ids.len()
        );
        removed_rooms
    }

    /// 清理所有房间（服务器关闭时使用）
    pub async fn shutdown_all_rooms(&self) {
        tracing::info!("🔄 关闭所有 {} 个房间", self.awareness_refs.len());

        let all_rooms: Vec<String> = self
            .awareness_refs
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        for room_id in all_rooms {
            self.remove_room(&room_id).await;
        }

        tracing::info!("🔄 所有房间已关闭");
    }
}
