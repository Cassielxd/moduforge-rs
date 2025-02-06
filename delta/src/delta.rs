
use moduforge_core::{
    state::{state::State, transaction::Transaction},
    transform::{transform::Transform, ConcreteStep},
};
#[derive(Debug, Clone)]
pub struct TransactionDelta {
    parent_version: u64,
    timestamp: u64,
    pub steps: Vec<ConcreteStep>,
}
/// 将Transaction转换为TransactionDelta
pub fn to_delta(
    tr: &Transaction,
    base_version: u64,
) -> TransactionDelta {
    let steps = tr
        .steps
        .iter()
        .map(|s| Transaction::as_concrete(s))
        .collect();
    TransactionDelta {
        parent_version: base_version,
        timestamp: tr.time,
        steps,
    }
}

/// 从增量记录重建事务
pub fn apply_delta(state: &State, delta: TransactionDelta) -> Transaction {
    let mut tr = Transaction::new(state);
    tr.time = delta.timestamp;
    for s in delta.steps.into_iter() {
        match s {
            ConcreteStep::UpdateAttrs(attr_step) => {
                let _ = tr.step(Box::new(attr_step));
            }
        }
    }
    tr
}

pub async fn apply_state_delta(state: &State, delta: TransactionDelta) -> State {
    if delta.parent_version != state.version {
        return state.clone();
    }
    let mut tr = apply_delta(state, delta);
    match state.apply_transaction(&mut tr).await {
        Ok(result) => result.state,
        Err(_) => state.clone(),
    }
}
