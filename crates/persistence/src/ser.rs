//! 持久化负载的序列化辅助工具。
//!
//! 使用简单分帧：将各步骤的序列化字节以 `0x1E`（记录分隔符）连接。
//! 压缩为可选，建议在分帧之后进行；校验和在压缩（或加密）后的字节上计算。

use std::collections::HashMap;
use crc32fast::Hasher as Crc32;
use zstd::stream::encode_all;
use mf_state::Transaction;
use serde::{Serialize, Deserialize};

/// 可选的 zstd 压缩（level=1，优先速度）。
pub fn compress_if_needed(
    input: &[u8],
    enable: bool,
) -> anyhow::Result<Vec<u8>> {
    if !enable {
        return Ok(input.to_vec());
    }
    let out = encode_all(input, 1)?; // fast
    Ok(out)
}

/// CRC32 校验，提供轻量级的数据完整性保护。
pub fn checksum32(input: &[u8]) -> u32 {
    let mut h = Crc32::new();
    h.update(input);
    h.finalize()
}

/// 将步骤序列化结果以 RS(0x1E) 作为分隔拼接。
pub fn serialize_steps_concat(transaction: &Transaction) -> Vec<u8> {
    let mut payload: Vec<u8> = Vec::new();
    for step in transaction.steps.iter() {
        if let Some(s) = step.serialize() {
            payload.extend_from_slice(&s);
            payload.push(0x1E);
        }
    }
    payload
}

// 标准化的步骤帧格式，便于跨版本重放
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TypeWrapper {
    pub type_id: String,
    pub data: Vec<u8>,
}

// 快照序列化载体（与 StateSerialize 的字段对应）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SnapshotData {
    pub node_pool: Vec<u8>,
    pub state_fields: HashMap<String, Vec<u8>>,
}

pub fn frame_steps(transaction: &Transaction) -> Vec<TypeWrapper> {
    let mut frames: Vec<TypeWrapper> =
        Vec::with_capacity(transaction.steps.len());
    for step in transaction.steps.iter() {
        if let Some(data) = step.serialize() {
            frames.push(TypeWrapper { type_id: step.name(), data });
        }
    }
    frames
}

pub fn frame_invert_steps(transaction: &Transaction) -> Vec<TypeWrapper> {
    if transaction.invert_steps.is_empty() {
        return Vec::new();
    }
    let mut frames: Vec<TypeWrapper> =
        Vec::with_capacity(transaction.invert_steps.len());
    let mut invert_steps: Vec<_> =
        transaction.invert_steps.iter().cloned().collect();
    invert_steps.reverse();
    for step in invert_steps {
        if let Some(data) = step.serialize() {
            frames.push(TypeWrapper { type_id: step.name(), data });
        }
    }
    frames
}
