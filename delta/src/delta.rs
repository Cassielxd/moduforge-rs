use std::sync::Arc;

use bincode::{
    Decode, Encode,
    error::{DecodeError, EncodeError},
};
use moduforge_core::{
    state::{state::State, transaction::Transaction},
    transform::{ConcreteStep, transform::Transform},
};
use serde::{Deserialize, Serialize};

use super::{from_binary, to_binary};
#[derive(Debug, Clone, Serialize, Deserialize, Decode, Encode)]
pub struct TransactionDelta {
    parent_version: u64,
    tr_id: u64,
    pub steps: Vec<ConcreteStep>,
}
/// 将Transaction转换为TransactionDelta
pub fn to_delta(
    tr: &Transaction,
    base_version: u64,
) -> TransactionDelta {
    let steps = tr.steps.iter().map(Transaction::as_concrete).collect();
    TransactionDelta { parent_version: base_version, tr_id: tr.id, steps }
}

/// 从增量记录重建事务
pub fn apply_delta(
    state: &State,
    delta: TransactionDelta,
) -> Transaction {
    let mut tr = Transaction::new(state);
    tr.id = delta.tr_id;
    for s in delta.steps.into_iter() {
        let _ = tr.step(Arc::new(s));
    }
    tr
}

pub async fn apply_state_delta(
    state: &State,
    delta: TransactionDelta,
) -> State {
    if delta.parent_version != state.version {
        return state.clone();
    }
    let tr = apply_delta(state, delta);
    match state.apply(tr).await {
        Ok(result) => result.state,
        Err(_) => state.clone(),
    }
}

// 从一个快照数据创建一个TransactionDelta
pub fn create_tr_from_snapshot(snapshot_data: &Vec<u8>) -> Result<TransactionDelta, DecodeError> {
    let f = from_binary::<TransactionDelta>(snapshot_data)?;
    Ok(f)
}
// 创建 一个事务快照
pub fn create_tr_snapshot(tr_data: TransactionDelta) -> Result<Vec<u8>, EncodeError> {
    to_binary::<TransactionDelta>(tr_data)
}
