use std::{env::current_dir, sync::Arc};

use moduforge_core::state::transaction::Transaction;
use moduforge_runtime::{
    cache::{cache::DocumentCache, l1::L1Cache, l2::L2Cache},
    event_handler::{create_delta_handler, create_snapshot_handler},
    node::Node,
    runtime::{Runtime, RuntimeOptions},
    types::{Content, Extensions},
};

#[tokio::main]
async fn main() {
    test().await;
    /*  let mut path = current_dir().unwrap();
    path = path.join("./data");
    let mut runtime = Runtime::create(RuntimeOptions {
        content: Content::None,
        extensions: get_base(),
        history_limit: Some(10),
        event_handlers: vec![],
        storage_path: None,
    })
    .await;
    runtime.start_event_loop();

    tokio::time::sleep(std::time::Duration::from_secs(100)).await;  */
}
#[allow(dead_code)]
async fn test() {

    let mut runtime = Runtime::create(RuntimeOptions {
        content: Content::None,
        extensions: get_base(),
        history_limit: Some(10),
        event_handlers: vec![],
        storage_path: None,
    })
    .await;
    runtime.start_event_loop();
    let binding = runtime.get_schema();
    let node_type = binding.nodes.get("DW").unwrap();

    for i in 1..1000 {
        let state = runtime.get_state();
        let mut tr: Transaction = Transaction::new(state);
        tr.add_node(
            state.doc().inner.root_id.to_string(),
            node_type.create(None, None, vec![], None),
        );
        let _ = runtime.dispatch(tr).await;
       
    }

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
