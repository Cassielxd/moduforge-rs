use async_trait::async_trait;
use log::{error, Level, LevelFilter, Metadata, Record, SetLoggerError};
use moduforge_core::{
    model::{
        attrs,
        node_type::NodeSpec,
        schema::{AttributeSpec, Schema, SchemaSpec},
    },
    state::{
        plugin::{Plugin, PluginKey, PluginState},
        state::{State, StateConfig},
        transaction::Transaction,
    },
};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
#[tokio::main]
async fn main() {
    let _ = test1().await;
}
async fn test1() -> Result<(), Box<dyn std::error::Error>> {
    init().expect("");
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
    let mut state = State::create(StateConfig {
        schema: Some(Arc::new(schema)),
        doc: None,
        stored_marks: None,
        plugins: Some(vec![Arc::new(PluginImpl::new())]),
    })
    .await?;
    dbg!(state.doc());
    let mut tr: Transaction = Transaction::new(&state);
    let mut values: im::HashMap<String, String> = im::HashMap::new();
    values.insert("name".to_string(), "李兴栋".to_string());
    tr.set_node_attribute(state.doc().inner.root_id.to_string(), values);
    let state = state.apply(&mut tr).await?;
    dbg!(state.doc());
    Ok(())
}
struct SimpleLogger;
impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
    fn flush(&self) {}
}
static LOGGER: SimpleLogger = SimpleLogger;
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}
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
