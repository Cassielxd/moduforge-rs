use std::sync::Arc;
use yrs::{Doc, Transact, Update, StateVector, ReadTxn};
use yrs::updates::decoder::Decode;
use dashmap::DashMap;
use crate::{Result, TransmissionError};

pub struct YrsManager {
    /// 每个活跃的房间对应一个 Yrs 文档. Arc<Doc> 本身是线程安全的.
    rooms: DashMap<String, Arc<Doc>>,
}

impl YrsManager {
    pub fn new() -> Self {
        Self { rooms: DashMap::new() }
    }

    /// 获取或创建房间的 Yrs 文档
    pub fn get_or_create_doc(
        &self,
        room_id: &str,
    ) -> Arc<Doc> {
        self.rooms
            .entry(room_id.to_string())
            .or_insert_with(|| {
                let doc = Doc::new();
                // The document is initialized on-demand, no need to pre-populate structures
                // as they will be created by the first applied transaction or snapshot.
                Arc::new(doc)
            })
            .clone()
    }

    /// 获取房间的Yrs文档（用于直接操作）
    pub fn get_doc(
        &self,
        room_id: &str,
    ) -> Option<Arc<Doc>> {
        self.rooms.get(room_id).map(|entry| entry.value().clone())
    }

    /// 移除房间的Doc
    pub fn remove_doc(
        &self,
        room_id: &str,
    ) -> Option<(String, Arc<Doc>)> {
        self.rooms.remove(room_id)
    }

    /// 获取房间列表
    pub fn list_rooms(&self) -> Vec<String> {
        self.rooms.iter().map(|entry| entry.key().clone()).collect()
    }

    /// 获取房间的完整状态更新，用于新客户端加入
    pub fn get_full_state_update(
        &self,
        room_id: &str,
    ) -> Result<Vec<u8>> {
        let doc = self.get_doc(room_id).ok_or_else(|| {
            TransmissionError::RoomNotFound(room_id.to_string())
        })?;
        let txn = doc.transact();
        Ok(txn.encode_state_as_update_v1(&StateVector::default()))
    }

    /// 根据客户端的状态向量计算差异更新
    pub fn get_diff_update(
        &self,
        room_id: &str,
        state_vector: &[u8],
    ) -> Result<Vec<u8>> {
        let doc = self.get_doc(room_id).ok_or_else(|| {
            TransmissionError::RoomNotFound(room_id.to_string())
        })?;
        let sv = StateVector::decode_v1(state_vector).map_err(|e| {
            TransmissionError::YrsError(format!(
                "Failed to decode state vector: {}",
                e
            ))
        })?;
        let txn = doc.transact();
        Ok(txn.encode_diff_v1(&sv))
    }

    /// 应用外部更新到房间
    pub fn apply_update(
        &self,
        room_id: &str,
        update: &[u8],
    ) -> Result<()> {
        let doc = self.get_doc(room_id).ok_or_else(|| {
            TransmissionError::RoomNotFound(room_id.to_string())
        })?;
        let update = Update::decode_v1(update)
            .map_err(|e| TransmissionError::YrsError(e.to_string()))?;

        let mut txn = doc.transact_mut();
        txn.apply_update(update)?;

        Ok(())
    }
}

impl Default for YrsManager {
    fn default() -> Self {
        Self::new()
    }
}
