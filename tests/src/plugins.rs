use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use moduforge_state::{
    debug,
    plugin::{PluginState, PluginTrait, StateField},
    state::{State, StateConfig},
    transaction::Transaction,
};
use moduforge_macros::{impl_plugin, impl_state_field};
async fn p1_append(
    _trs: &[Transaction],
    _: &State,
    _new_state: &State,
) -> Option<Transaction> {
    None
}

// P1Plugin 是一个插件，用于在调度前后打印消息。用于案例测试
impl_plugin!(P1Plugin, p1_append);

async fn p1_init(
    _config: &StateConfig,
    _instance: Option<&State>,
) -> PluginState {
    let map: HashMap<String, String> =
        HashMap::from([("k".to_string(), "v".to_string())]);
    Arc::new(map)
}

async fn p1_apply(
    tr: &Transaction,
    value: PluginState,
    _old_state: &State,
    _new_state: &State,
) -> PluginState {
    debug!("P1Plugin apply{}", tr.steps.len());
    value
}

impl_state_field!(P1State, p1_init, p1_apply);

async fn p2_append(
    trs: &[Transaction],
    _: &State,
    _: &State,
) -> Option<Transaction> {
    let tr = trs.last().unwrap().clone();
    let size = tr.doc.size();
    debug!("P2Plugin开始节点个数：{}", size);

    None
}

// P2Plugin 是一个插件，用于在调度前后打印消息。用于案例测试
impl_plugin!(P2Plugin, p2_append);
