use std::sync::Arc;
use moduforge_state::{init_logging, transaction::Transaction};
use moduforge_core::{async_runtime::AsyncEditor, types::EditorOptionsBuilder};
use moduforge_test::{
    base::get_base,
    commands::MyCommand1,
    middleware::{Middleware1, Middleware2},
};

#[tokio::main]
async fn main() {
    init_logging("debug", None).unwrap();
    let options = EditorOptionsBuilder::new()
        .extensions(get_base())
        .add_middleware(Middleware1)
        .add_middleware(Middleware2)
        .build();
    let mut runtime = AsyncEditor::create(options).await.unwrap();
    let mut tr: Transaction = runtime.get_tr();
    tr.transaction(Arc::new(MyCommand1)).await;
    tr.set_meta("add_node", true);
    let before_doc = runtime.doc();
    let _ = runtime.dispatch_flow(tr).await;
    let after_doc = runtime.doc();
    dbg!(before_doc);
    dbg!(after_doc);
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
}
