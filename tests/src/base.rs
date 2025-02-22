use std::sync::Arc;

use async_trait::async_trait;
use moduforge_core::{
    state::transaction::{Command, Transaction},
    transform::transform::TransformError,
};
use moduforge_runtime::{node::Node, types::Extensions};

pub fn get_base() -> Vec<Extensions> {
    let mut extensions = vec![];

    let mut top_node = Node::default();
    top_node
        .set_top_node()
        .set_name("doc")
        .set_content("DW+")
        .set_desc("顶级节点")
        .set_attr("name", None);
    extensions.push(Extensions::N(top_node));
    let mut dw = Node::default();
    dw.set_name("DW").set_desc("页面");
    extensions.push(Extensions::N(dw));
    extensions
}

#[derive(Clone, Default, Debug)]
pub struct MyCommand;
impl MyCommand {
    pub fn new() -> Arc<MyCommand> {
        Arc::new(MyCommand)
    }
}

#[async_trait]
impl Command for MyCommand {
    fn name(&self) -> String {
        "cassie".to_string()
    }
    async fn execute(&self, tr: &mut Transaction) -> Result<(), TransformError> {
        for _i in 1..1000 {
            tr.add_node(
                tr.doc().inner.root_id.to_string(),
                tr.schema
                    .nodes
                    .get("DW")
                    .unwrap()
                    .create(None, None, vec![], None),
            );
        }
        Ok(())
    }
}
