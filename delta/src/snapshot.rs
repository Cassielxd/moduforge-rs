use std::{collections::HashMap, sync::Arc};

use moduforge_core::{
    model::node_pool::{NodePool},
    state::{plugin::PluginState, state::{Configuration, State}},
};
use serde::{Deserialize, Serialize};


#[derive(Deserialize,Serialize, PartialEq, Debug)]
pub struct FullSnapshot {
     pub node_pool: Arc<NodePool>,              // 节点池完整数据
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
     config.doc = Some(f.node_pool.clone());
    let mut state = State::new(config);
    for (key,p_state) in f.state_fields.into_iter(){
       let _ = state.set_field(&key, p_state);
    }
    Ok(state)
} 

//根据状态创建快照
 pub fn create_full_snapshot(state: &State) -> serde_json::Result<Vec<u8>> {
    let snapshot = FullSnapshot {
        node_pool: state.node_pool.clone(),
         state_fields: state
            .fields_instances.clone(),
         version: state.version, // 根据实际情况调整
    };
    serde_json::to_vec(&snapshot)
}
