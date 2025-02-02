use log::{error, Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::collections::HashMap;
use valuation_model::{
    node_type::NodeSpec,
    schema::{Schema, SchemaSpec},
    types::ContentEnum,
};

fn main() {
    init().expect("");
    test();
}

fn test() {
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
    match Schema::compile(instance_spec) {
        Ok(schema) => {
            let node_type = schema.nodes.get("doc").unwrap(); //
            let node = node_type.create(None, None, vec![], None);
            dbg!(node);
        }
        Err(_) => todo!(),
    }
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
