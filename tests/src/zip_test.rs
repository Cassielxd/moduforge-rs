/* use std::env::current_dir;

use moduforge_core::state::transaction::Transaction;
use moduforge_runtime::{runtime::Editor, traits::EditorCore, types::EditorOptions};

use crate::{base::get_base, commands::MyCommand};

pub async fn export_zip() {
    let mut runtime = Editor::create(EditorOptions::default().set_extensions(get_base())).await.unwrap();
    let mut tr: Transaction = runtime.get_tr();
    tr.transaction(MyCommand::new()).await;
    let _ = runtime.dispatch(tr).await;
    runtime.export_zip(current_dir().unwrap().join("test.zip").as_path()).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(50)).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    #[test]
    async fn test_export_zip() {
        export_zip().await;
    }
}
 */
