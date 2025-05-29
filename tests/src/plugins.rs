use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use moduforge_macros_derive::PState;
use moduforge_state::{
    debug,
    error::StateResult,
    plugin::{PluginTrait, StateField},
    resource::Resource,
    state::{State, StateConfig},
    transaction::Transaction,
};
use moduforge_macros::{impl_plugin, impl_state_field};
async fn p1_append(
    _trs: &[Transaction],
    _: &State,
    _new_state: &State,
) -> StateResult<Option<Transaction>> {
    Ok(None)
}

// P1Plugin 是一个插件，用于在调度前后打印消息。用于案例测试
impl_plugin!(P1Plugin, p1_append);

#[derive(Debug, PState)]
pub struct P1State1 {
    pub map: HashMap<String, String>,
}

async fn p1_init(
    _config: &StateConfig,
    _instance: Option<&State>,
) -> Arc<dyn Resource> {
    let map: HashMap<String, String> =
        HashMap::from([("k".to_string(), "v".to_string())]);
    Arc::new(P1State1 { map })
}

async fn p1_apply(
    tr: &Transaction,
    value: Arc<dyn Resource>,
    _old_state: &State,
    _new_state: &State,
) -> Arc<dyn Resource> {
    let _ = value.downcast_arc::<P1State1>().unwrap();
    debug!("P1Plugin apply{}", tr.steps.len());
    value
}

impl_state_field!(P1StateField, p1_init, p1_apply);

async fn p2_append(
    trs: &[Transaction],
    _: &State,
    _: &State,
) -> StateResult<Option<Transaction>> {
    let tr = trs.last().unwrap().clone();
    let size = tr.doc.size();
    debug!("P2Plugin开始节点个数：{}", size);

    Ok(None)
}

// P2Plugin 是一个插件，用于在调度前后打印消息。用于案例测试
impl_plugin!(P2Plugin, p2_append);
