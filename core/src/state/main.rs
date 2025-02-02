use async_trait::async_trait;
use log::{error, Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::{collections::HashMap, sync::Arc};
use valuation_model::{
    node_type::NodeSpec,
    schema::{Schema, SchemaSpec},
};
use valuation_state::{
    plugin::{Plugin, PluginSpec, PluginState, PluginTrTrait, StateField},
    state::{State, StateConfig},
    transaction::Transaction,
};
#[tokio::main]
async fn main() {
    let _ = test1().await;
}
async fn test1() -> Result<(), Box<dyn std::error::Error>> {
    init().expect("");
    let mut nodes = HashMap::new();
    nodes.insert(
        "doc".to_string(),
        NodeSpec {
            content: Some("DW+".to_string()),
            marks: None,
            group: None,
            desc: Some("顶级节点".to_string()),
            attrs: None,
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
        plugins: Some(vec![get_plugin()]),
    })
    .await?;

    state = state.apply(&mut Transaction::new(&state)).await?;
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
struct PState {}
#[async_trait]
impl StateField for PState {
    async fn init(&self, config: &StateConfig, instance: Option<&State>) -> PluginState {
        return Arc::new(1);
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Option<&PluginState>,
        old_state: Option<&State>,
        new_state: Option<&State>,
    ) -> PluginState {
        match value {
            Some(v) => {
                if let Some(count) = v.downcast_ref::<i32>() {
                    Arc::new(count + 1)
                } else {
                    error!("Unexpected type in PluginState");
                    Arc::new(1)
                }
            }
            None => Arc::new(1),
        }
    }
}
#[derive(Clone, Debug)]
struct PluginTr {}
#[async_trait]
impl PluginTrTrait for PluginTr {
    async fn append_transaction<'a>(
        &self,
        tr: &'a mut Transaction,
        old_state: &State,
        new_state: &State,
    ) -> Option<&'a mut Transaction> {
        println!("asdasdasdasdas");
        tr.set_meta("aaa", Box::new("aaa".to_string()));
        return Some(tr);
    }
}
fn get_plugin() -> Plugin {
    let plugin = Plugin::new(PluginSpec {
        state: Some(Arc::new(PState {})),
        key: None,
        filter_transaction: None,
        append_transaction: Some(Arc::new(PluginTr {})),
    });
    plugin
}
