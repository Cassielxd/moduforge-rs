//! 启动恢复：参考 price-storage 的做法（快照 + 事务重放）
use std::sync::Arc;

use crate::api::{EventStore};
use crate::ser::{SnapshotData, TypeWrapper};
use crate::step_factory::StepFactoryRegistry;

/// 从存储恢复状态：加载最新快照，重放其后的事件
pub async fn recover_state<E: EventStore + 'static>(
    store: &E,
    doc_id: &str,
    configuration: &mf_state::Configuration,
    step_factory: &StepFactoryRegistry,
    batch: u32,
) -> anyhow::Result<Arc<mf_state::State>> {
    // 1) 快照
    let mut state = if let Some(snap) = store.latest_snapshot(doc_id).await? {
        let bytes = zstd::decode_all(std::io::Cursor::new(snap.state_blob))?;
        let snap_data: SnapshotData = serde_json::from_slice(&bytes)?;
        let ser = mf_state::state::StateSerialize {
            node_pool: snap_data.node_pool,
            state_fields: snap_data.state_fields,
        };
        Arc::new(mf_state::State::deserialize(&ser, configuration).await?)
    } else {
        // 当无快照时，从配置生成空状态
        Arc::new(mf_state::State::new(Arc::new(configuration.clone()))?)
    };

    // 2) 事件重放
    let mut from_lsn =
        store.latest_snapshot(doc_id).await?.map(|s| s.upto_lsn).unwrap_or(0);
    loop {
        let evs = store.load_since(doc_id, from_lsn, batch).await?;
        if evs.is_empty() {
            break;
        }
        for ev in evs {
            let payload = zstd::decode_all(std::io::Cursor::new(ev.payload))?;
            let frames: Vec<TypeWrapper> = serde_json::from_slice(&payload)?;
            let mut tr = mf_state::Transaction::new(&state);
            for f in frames {
                tr.step(step_factory.create(&f.type_id, &f.data))?;
            }
            tr.commit();
            state = state.apply(tr).await?.state;
            from_lsn = ev.lsn;
        }
    }
    Ok(state)
}
