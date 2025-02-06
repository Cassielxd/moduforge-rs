use std::{collections::HashMap, sync::Arc};

use moduforge_core::{
    model::node_pool::{NodePool, NodePoolInner},
    state::{plugin::PluginState, state::{Configuration, State}},
};
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct FullSnapshot {
     pub inner: Arc<NodePoolInner>,              // 节点池完整数据
    pub state_fields: HashMap<String, PluginState>, // State的字段序列化 */ 
     pub version: u64,                           // 版本号用于兼容性 */
}
// 根据快照快复节点池和State
pub fn create_state_from_snapshot(
    config: Configuration,
    snapshot_data: Vec<u8>,
) -> serde_json::Result<State> {
   let f :FullSnapshot = serde_json::from_slice(&snapshot_data)?;
    let mut config = config;
     config.doc = Some(Arc::new(NodePool {
        inner: f.inner.clone(),
    }));
    let mut state = State::new(config);
    Ok(state)
}

//根据状态创建快照
pub fn create_full_snapshot(state: &State) -> serde_json::Result<Vec<u8>> {
    let snapshot = FullSnapshot {
        inner: state.node_pool.inner.clone(),
         state_fields: state
            .fields_instances.clone(),
         version: state.version, // 根据实际情况调整
    };
    serde_json::to_vec(&snapshot)
}
