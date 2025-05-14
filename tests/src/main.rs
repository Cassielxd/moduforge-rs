use std::sync::Arc;
use moduforge_state::init_logging;
use moduforge_core::{async_runtime::AsyncEditor, types::EditorOptionsBuilder};
use moduforge_test::{
    base::get_base,
    commands::MyCommand1,
    middleware::{Middleware1, Middleware2},
};

fn main() {
    init_logging("debug", None).unwrap();
    let options = EditorOptionsBuilder::new()
        .extensions(get_base())
        .add_middleware(Middleware1)
        .add_middleware(Middleware2)
        .build();
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let mut runtime = AsyncEditor::create(options).await.unwrap();
        let before_doc = runtime.doc();
        dbg!(before_doc.size());
        let _ = runtime.command(Arc::new(MyCommand1)).await;
        let after_doc = runtime.doc();
        dbg!(after_doc.size());
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    });
}
