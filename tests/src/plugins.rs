use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use moduforge_core::state::{
    plugin::{PluginState, PluginTrait, StateField},
    state::{State, StateConfig},
    transaction::Transaction,
};

#[derive(Debug)]
pub struct P1State;
#[async_trait]
impl StateField for P1State {
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: Option<&State>,
    ) -> PluginState {
        let map: HashMap<String, String> = HashMap::from([("k".to_string(), "v".to_string())]);
        Arc::new(map)
    }

    async fn apply(
        &self,
        _tr: &Transaction,
        value: PluginState,
        _old_state: &State,
        _new_state: &State,
    ) -> PluginState {
        value
    }
}
#[derive(Debug)]
pub struct P1Plugin {}
#[async_trait]
impl PluginTrait for P1Plugin {
    async fn append_transaction(
        &self,
        tr: &Transaction,
        _old_state: &State,
        _new_state: &State,
    ) -> Option<Transaction> {
        let mut tr = tr.clone();
        let ok: Option<&bool> = tr.get_meta("add_node");
        println!("开始节点个数：{}", tr.doc.size());
        if let Some(ok) = ok {
            if *ok {
                tr.add_node(
                    tr.doc().inner.root_id.to_string(),
                    tr.schema.nodes.get("DW").unwrap().create(None, None, vec![], None),
                );
                println!("节点个数：{}", tr.doc.size());
                return Some(tr);
            }
        }

        None
    }
    async fn filter_transaction(
        &self,
        _tr: &Transaction,
        _state: &State,
    ) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct P2Plugin {}
#[async_trait]
impl PluginTrait for P2Plugin {
    async fn append_transaction(
        &self,
        tr: &Transaction,
        _: &State,
        _: &State,
    ) -> Option<Transaction> {
        let mut tr = tr.clone();
        let size = tr.doc.size();
        println!("P2Plugin开始节点个数：{}", tr.doc.size());
        if size < 10 {
            tr.add_node(
                tr.doc().inner.root_id.to_string(),
                tr.schema.nodes.get("DW").unwrap().create(None, None, vec![], None),
            );
            println!("P2Plugin节点个数：{}", tr.doc.size());
            return Some(tr);
        }

        None
    }
    async fn filter_transaction(
        &self,
        _: &Transaction,
        _: &State,
    ) -> bool {
        true
    }
}
