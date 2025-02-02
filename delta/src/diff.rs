use std::sync::Arc;

use bincode::{Decode, Encode};
use moduforge_core::model::{node::Node, types::NodeId};
use serde::{Deserialize, Serialize};

// 修改 diff.rs 添加序列化支持
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum NodeDiff {
    Add(#[bincode(with_serde)] NodeId, usize),
    Remove(#[bincode(with_serde)] NodeId, usize),
    Update(
        #[bincode(with_serde)] NodeId,
        #[bincode(with_serde)] Arc<Node>,
    ),
}