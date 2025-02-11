use moduforge_core::state::transaction::Transaction;
use moduforge_runtime::{
    cache::CacheKey,
    node::Node,
    runtime::{Runtime, RuntimeOptions},
    types::{Content, Extensions},
};

#[tokio::main]
async fn main() {
    //test().await;
    //test_from_snapshot().await;
    let mut runtime = Runtime::create(RuntimeOptions {
        content: Content::None,
        extensions: get_base(),
        history_limit: Some(10),
        event_handlers: vec![],
        storage_option: None,
    })
    .await;
    runtime.start_event_loop();
    let binding = runtime.get_schema();
    let node_type = binding.nodes.get("DW").unwrap();
    let state = runtime.get_state();
    dbg!(state.doc());
        let mut tr: Transaction = Transaction::new(state);
        tr.add_node(
            state.doc().inner.root_id.to_string(),
            node_type.create(None, None, vec![], None),
        );
    
        dbg!(tr.doc);
}
#[allow(dead_code)]
async fn test_from_snapshot() {
    let runtime = Runtime::create(RuntimeOptions {
        content: Content::None,
        extensions: get_base(),
        history_limit: Some(10),
        event_handlers: vec![],
        storage_option: None,
    })
    .await;
    runtime.start_event_loop();
    let data = runtime.get_snapshot(&CacheKey {
        doc_id: "7294968876259868672".to_string(),
        version: 900,
    });
    dbg!(data);
    tokio::time::sleep(std::time::Duration::from_secs(100)).await;
}
#[allow(dead_code)]
async fn test_create_snapshot() {
    let mut runtime = Runtime::create(RuntimeOptions {
        content: Content::None,
        extensions: get_base(),
        history_limit: Some(10),
        event_handlers: vec![],
        storage_option: None,
    })
    .await;
    runtime.start_event_loop();
    let binding = runtime.get_schema();
    let node_type = binding.nodes.get("DW").unwrap();

    for _i in 1..1000 {
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
