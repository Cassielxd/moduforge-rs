use moduforge_core::state::transaction::Transaction;
use moduforge_runtime::{cache::CacheKey, runtime::Editor, types::EditorOptions};

use crate::{base::get_base, commands::MyCommand};

#[allow(dead_code)]
async fn from_snapshot() {
    let runtime = Editor::create(EditorOptions::default().set_extensions(get_base())).await;
    let data = runtime.get_snapshot(&CacheKey { doc_id: "7297544091158446080".to_string(), version: 900, time: 1 });
    dbg!(data);
    tokio::time::sleep(std::time::Duration::from_secs(100)).await;
}
#[allow(dead_code)]
async fn create_snapshot() {
    let mut runtime = Editor::create(EditorOptions::default().set_extensions(get_base())).await;
    let mut tr: Transaction = runtime.get_tr();
    tr.transaction(MyCommand::new()).await;
    let _ = runtime.dispatch(tr).await;

    tokio::time::sleep(std::time::Duration::from_secs(100)).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    #[test]
    async fn test_create_snapshot() {
        create_snapshot().await;
    }

    #[test]
    #[should_panic]
    async fn test_from_snapshot() {
        from_snapshot().await;
    }
}
