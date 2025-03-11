use super::{from_binary, to_binary};
use bincode::{Decode, Encode, error::EncodeError};
use im::HashMap as ImHashMap;
use moduforge_core::{
    model::node_pool::NodePool,
    state::state::{Configuration, State},
};
use std::{collections::HashMap, sync::Arc};

#[derive(Decode, Encode, PartialEq, Debug)]
pub struct FullSnapshot {
    pub node_pool: Arc<NodePool>,               // 节点池完整数据
    pub state_fields: HashMap<String, Vec<u8>>, // State的字段序列化 */
    pub version: u64,                           // 版本号用于兼容性 */
}

// 根据快照快复节点池和State
pub fn create_state_from_snapshot(
    config: Configuration,
    snapshot_data: Vec<u8>,
) -> Result<State, Box<dyn std::error::Error>> {
    let f = from_binary::<FullSnapshot>(&snapshot_data)?;
    let mut config = config;
    config.doc = Some(f.node_pool.clone());
    let mut state = State::new(Arc::new(config));
    let mut map_instances = ImHashMap::new();
    for plugin in state.plugins() {
        if let Some(state_field) = &plugin.spec.state {
            if let Some(value) = f.state_fields.get(&plugin.key) {
                if let Some(p_state) = state_field.deserialize(value) {
                    let key = plugin.key.clone();
                    map_instances.insert(key, p_state);
                }
            }
        }
    }
    state.fields_instances = map_instances;
    Ok(state)
}

//根据状态创建快照
pub fn create_full_snapshot(state: &State) -> Result<Vec<u8>, EncodeError> {
    let mut state_fields: HashMap<String, Vec<u8>> = HashMap::new();
    for plugin in state.plugins() {
        if let Some(state_field) = &plugin.spec.state {
            if let Some(value) = state.get_field(&plugin.key) {
                if let Some(json) = state_field.serialize(value) {
                    state_fields.insert(plugin.key.clone(), json);
                }
            };
        }
    }
    let snapshot = FullSnapshot {
        node_pool: state.node_pool.clone(),
        state_fields,
        version: state.version, // 根据实际情况调整
    };
    to_binary::<FullSnapshot>(snapshot)
}
