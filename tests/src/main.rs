use std::sync::Arc;
use moduforge_core::{init_logging, state::transaction::Transaction};
use moduforge_runtime::{async_runtime::Editor, traits::EditorCore, types::EditorOptions};
use moduforge_test::{base::get_base, commands::MyCommand1};

#[tokio::main]
async fn main() {
    init_logging("debug", None).unwrap();
    let mut runtime = Editor::create(EditorOptions::default().set_extensions(get_base())).await.unwrap();
    let mut tr: Transaction = runtime.get_tr();
    tr.transaction(Arc::new(MyCommand1)).await;
    tr.set_meta("add_node", true);
    let before_doc = runtime.doc();
    let _ = runtime.dispatch(tr).await;
    let after_doc = runtime.doc();
    dbg!(before_doc);
    dbg!(after_doc);
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
}


