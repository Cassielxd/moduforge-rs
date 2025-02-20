use std::sync::Arc;

use async_trait::async_trait;
use moduforge_core::{
    state::transaction::{Command, Transaction},
    transform::transform::TransformError,
};
use moduforge_runtime::{
    cache::CacheKey,
    node::Node,
    runtime::Editor,
    types::{EditorOptions, Extensions},
};

#[tokio::main]
async fn main() {
    test_create_snapshot().await;
}
#[allow(dead_code)]
async fn test_from_snapshot() {
    let runtime = Editor::create(EditorOptions::default().set_extensions(get_base())).await;
    let data = runtime.get_snapshot(&CacheKey {
        doc_id: "7297544091158446080".to_string(),
        version: 900,
        time: 1,
    });
    dbg!(data);
    tokio::time::sleep(std::time::Duration::from_secs(100)).await;
}
#[derive(Clone, Default, Debug)]
struct MyCommand;
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
#[allow(dead_code)]
async fn test_create_snapshot() {
    let mut runtime = Editor::create(EditorOptions::default().set_extensions(get_base())).await;
    let mut tr: Transaction = runtime.get_tr();
    tr.transaction(MyCommand::new()).await;
    let _ = runtime.dispatch(tr).await;

    tokio::time::sleep(std::time::Duration::from_secs(100)).await;
}

fn get_base() -> Vec<Extensions> {
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
