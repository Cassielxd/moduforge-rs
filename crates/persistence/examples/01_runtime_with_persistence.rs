// 运行：
//   cargo run -p moduforge-persistence --example 01_runtime_with_persistence
// 说明：
//   - 使用 schema/main.xml 加载节点定义
//   - 注入持久化中间件（事件 + 定期快照）
//   - 演示新增一个 dwgc 节点并分发事务

use std::sync::Arc;
use mf_core::{EditorOptionsBuilder, ForgeAsyncRuntime, ForgeResult};
use mf_persistence::api::{CommitMode, PersistOptions};
use mf_persistence::subscriber::SnapshotSubscriber;
use mf_persistence::sqlite::SqliteEventStore;
// runtime-only example: for export/import, see export_doc.rs and export_zip.rs

#[tokio::main]
async fn main() -> ForgeResult<()> {
    // 1) 构造事件存储（SQLite WAL）
    let store = SqliteEventStore::open(
        "./data/persistence_demo.sqlite",
        CommitMode::AsyncDurable { group_window_ms: 8 },
    )?;

    // 2) 配置持久化选项（定期快照）
    let persist_opts = PersistOptions {
        commit_mode: CommitMode::AsyncDurable { group_window_ms: 8 },
        snapshot_every_n_events: 1000,
        snapshot_every_bytes: 8 * 1024 * 1024,
        snapshot_every_ms: 5 * 60 * 1000,
        compression: true,
    };

    // 3) 注入持久化订阅者（由事件驱动持久化与快照）
    let subscriber = Arc::new(SnapshotSubscriber::new(
        store.clone(),
        persist_opts.clone(),
        "default_doc",
    ));

    let options =
        EditorOptionsBuilder::new().add_event_handler(subscriber).build();

    // 4) 从 XML schema 加载运行时（注意路径相对于工作目录，必要时调整为 "../../schema/main.xml"）
    let xml_path = "schema/main.xml";
    let mut editor =
        ForgeAsyncRuntime::from_xml_schema_path(xml_path, Some(options), None)
            .await?;

    // 5) 构造一个新增节点事务（dwgc）并分发
    let doc = editor.doc();
    let mut tr = editor.get_tr();
    let schema = &tr.schema;
    // 依赖 main.xml 中定义的 <node name="dwgc" ...>
    let factory = schema.factory();
    let dw_node = factory.create_tree("dwgc", None, None, vec![], None)?;
    tr.add_node(doc.root_id().clone(), vec![dw_node])?;
    editor.dispatch(tr).await?;
    // 睡眠五秒，等待持久化与快照
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    let state = editor.get_state();
    println!("事务已分发并持久化，当前版本: {}", state.version);

    // 如需导出/导入演示，请运行：
    // cargo run -p moduforge-persistence --example export_doc
    // cargo run -p moduforge-persistence --example export_zip
    Ok(())
}
