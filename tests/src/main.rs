use std::{env::current_dir, sync::Arc};

use moduforge_core::state::transaction::Transaction;
use moduforge_runtime::{runtime::Editor, types::EditorOptions};
use moduforge_test::{base::get_base, commands::MyCommand};

#[tokio::main]
async fn main() {
    let mut runtime = Editor::create(EditorOptions::default().set_extensions(get_base())).await;
    let mut tr: Transaction = runtime.get_tr();
    tr.transaction(MyCommand::new()).await;
    tr.set_meta("add_node", true);
    let _ = runtime.dispatch(tr).await;

    tokio::time::sleep(std::time::Duration::from_secs(100)).await;
}
