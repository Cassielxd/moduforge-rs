use async_trait::async_trait;
use moduforge_core::{
    model::{
        node_type::NodeSpec,
        schema::{AttributeSpec, Schema, SchemaSpec},
    },
    state::{
        plugin::{Plugin, PluginKey},
        state::{State, StateConfig},
        transaction::Transaction,
    },
};
use moduforge_delta::snapshot::{create_full_snapshot, create_state_from_snapshot};

use std::{collections::HashMap, sync::Arc};
use tokio::fs;

#[derive(Clone, Debug)]
struct PluginImpl {
    key: PluginKey,
}
impl PluginImpl {
    pub fn new() -> Self {
        PluginImpl {
            key: PluginKey::new(Some("plugin"), Some("plugin")),
        }
    }
}
#[async_trait]
impl Plugin for PluginImpl {
    fn key(&self) -> &PluginKey {
        return &self.key;
    }
    async fn filter_transaction(&self, _tr: &Transaction, _state: &State) -> bool {
        true
    }
}
async fn get_base() -> Result<State, Box<dyn std::error::Error>> {
    let mut nodes = HashMap::new();
    let mut attrs = HashMap::new();
    attrs.insert(
        "name".to_string(),
        AttributeSpec {
            default: Some("string".to_string()),
            validate: None,
        },
    );
    nodes.insert(
        "doc".to_string(),
        NodeSpec {
            content: Some("DW+".to_string()),
            marks: None,
            group: None,
            desc: Some("顶级节点".to_string()),
            attrs: Some(attrs),
        },
    );
    nodes.insert(
        "DW".to_string(),
        NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("页面".to_string()),
            attrs: None,
        },
    );
    let marks = HashMap::new();

    let instance_spec = SchemaSpec {
        nodes,
        marks,
        top_node: Some("doc".to_string()),
    };
    let schema = Schema::compile(instance_spec)?;
    let state = State::create(StateConfig {
        schema: Some(Arc::new(schema)),
        doc: None,
        stored_marks: None,
        plugins: Some(vec![Arc::new(PluginImpl::new())]),
    })
    .await?;
    Ok(state)
}
async fn from_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = get_base().await?;
    let snapshot_data = fs::read("./snapshot_v1.bin").await.unwrap();
    state = create_state_from_snapshot(state.config.clone(), snapshot_data)?;
    dbg!(state);
    Ok(())
}
async fn create_all_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = get_base().await?;
    state = state.apply(&mut Transaction::new(&state)).await?;
    let full_data = create_full_snapshot(&state)?;
    fs::write("./snapshot_v1.bin", full_data).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    create_all_snapshot().await.unwrap();
    from_snapshot().await.unwrap();
}
