use bincode::{error::EncodeError, Decode, Encode};
use moduforge_core::{
    model::node_pool::NodePool,
    state::{
        plugin::PluginState,
        state::{Configuration, State},
    },
};
use std::{collections::HashMap, io, sync::Arc};

use crate::{from_binary, to_binary};

#[derive(Decode, Encode, PartialEq, Debug)]
pub struct FullSnapshot {
    pub node_pool: Arc<NodePool>,                   // 节点池完整数据
    pub state_fields: HashMap<String, PluginState>, // State的字段序列化 */
    pub version: u64,                               // 版本号用于兼容性 */
}

// 根据快照快复节点池和State
pub fn create_state_from_snapshot(
    config: Configuration,
    snapshot_data: Vec<u8>,
) -> Result<State, Box<dyn std::error::Error>> {
    let f = from_binary::<FullSnapshot>(&snapshot_data)?;
    let mut config = config;
    config.doc = Some(f.node_pool.clone());
    let mut state = State::new(config);
    for (key, p_state) in f.state_fields.into_iter() {
        let _ = state.set_field(&key, p_state);
    }
    Ok(state)
}

//根据状态创建快照
pub fn create_full_snapshot(state: &State) -> Result<Vec<u8>, EncodeError> {
    let snapshot = FullSnapshot {
        node_pool: state.node_pool.clone(),
        state_fields: state.fields_instances.clone(),
        version: state.version, // 根据实际情况调整
    };
    to_binary::<FullSnapshot>(snapshot)
}
