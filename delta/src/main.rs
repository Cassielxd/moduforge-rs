use async_trait::async_trait;
use im::Vector;
use moduforge_core::model::node::Node;
use moduforge_core::model::node_pool::NodePoolInner;
use moduforge_core::{
    model::{
        attrs::Attrs,
        mark::Mark,
        node_type::NodeSpec,
        schema::{AttributeSpec, Schema, SchemaSpec},
        types::NodeId,
    },
    state::{
        plugin::{Plugin, PluginState, },
        state::{State, StateConfig},
        transaction::Transaction,
    },
};
use moduforge_delta::snapshot::{create_full_snapshot, create_state_from_snapshot};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc};
use tokio::fs;

async fn from_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = get_base().await?;
    let snapshot_data = fs::read("./snapshot_v1.bin").await.unwrap();
    state = create_state_from_snapshot(state.config.clone(), snapshot_data)?;
    dbg!(state.doc());
    Ok(())
}

async fn get_base() -> Result<State, Box<dyn std::error::Error>> {
    let mut nodes = HashMap::new();
    let mut attrs = HashMap::new();
    attrs.insert(
        "name".to_string(),
        AttributeSpec {
            default: Some(json!("string")),
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
    let mut state = State::create(StateConfig {
        schema: Some(Arc::new(schema)),
        doc: None,
        stored_marks: None,
        plugins: Some(vec![]),
    })
    .await?;
    Ok(state)
}

/* async fn create_tr_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let state = get_base().await?;
    let mut tr: Transaction = Transaction::new(&state);
    let mut values: im::HashMap<String, serde_json::Value> = im::HashMap::new();
    values.insert("name".to_string(), json!("李兴栋"));
    tr.set_node_attribute(state.doc().inner.root_id.to_string(), values);
    let tr_delta = to_delta(&tr, state.version)?;
    let tr_data = to_binary(tr_delta)?;
    fs::write("snapshot_tr_v1.bin", tr_data).await?;
    Ok(())
} */
async fn create_all_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = get_base().await?;
    state = state.apply(&mut Transaction::new(&state)).await?;
    let full_data = create_full_snapshot(&state)?;
    fs::write("./snapshot_v1.bin", full_data).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    from_snapshot().await.unwrap();
}
