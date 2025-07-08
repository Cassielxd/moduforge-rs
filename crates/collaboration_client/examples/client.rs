use std::sync::Arc;
use tokio::time::{self, Duration};
use yrs::{sync::Awareness, Doc, Map, Observable, Transact};
use yrs_warp::AwarenessRef;
use tracing_subscriber;
use serde_json;

use mf_collab_client::{provider::WebsocketProvider};

/// 客户端示例，连接到 `collaboration.rs` 中启动的测试服务器
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    // 从 `collaboration.rs` 测试中获取服务器详细信息
    let server_url = "ws://127.0.0.1:8080/collaboration"; // 确保端口与服务器测试匹配
    let room_name = "demo-room";

    // 1. 初始化客户端的文档和 awareness 状态
    let doc = Doc::new();
    let awareness: AwarenessRef =
        Arc::new(tokio::sync::RwLock::new(Awareness::new(doc)));

    let mut provider = WebsocketProvider::new(
        server_url.to_string(),
        room_name.to_string(),
        awareness.clone(),
    )
    .await;
   

    // 3. 连接到服务器
    provider.connect().await;
    {
        let nodes_map = awareness.read().await.doc().get_or_insert_map("nodes");
        provider.subscription(nodes_map.observe(move|txn, event|{
            for (key, change) in event.keys(txn) {
                match change {
                    yrs::types::EntryChange::Inserted(value) => {
                        println!("新增 key: {}, value: {:?}", key, value);
                    }
                    yrs::types::EntryChange::Removed(old_value) => {
                        println!("删除 key: {}, old value: {:?}", key, old_value);
                    }
                    yrs::types::EntryChange::Updated(old_value, new_value) => {
                        println!("更新 key: {}, old: {:?}, new: {:?}", key, old_value, new_value);
                    }
                }
            }
        }));
    }
    {
        let mut awareness_lock = awareness.write().await;
        provider.subscription(awareness_lock.on_update(move |event|{
            println!("awareness update: {:?}", event.awareness_update());
        }));
        awareness_lock.set_local_state(serde_json::json!({
            "user": {
                "id": "client-user-1",
                "name": "用户李兴栋",
                "color": "#FFEAA7",
                "online": true
            },
            "online": true
        }).to_string());
    }
    
    // 4. 事件循环
    loop {
        tokio::select! {
            // 定期添加本地更改以测试发送更新
            _ = time::sleep(Duration::from_secs(3)) => {
                if provider.is_connected() {/* 
                    let mut awareness_lock = awareness.write().await;
                    let doc = awareness_lock.doc_mut();
                    let nodes_map = doc.get_or_insert_map("nodes");
                    let mut txn = doc.transact_mut_with(doc.client_id().to_string());
                    // 生成新节点 ID
            let node_id = format!("client_node_{}", rand::random::<u64>());

            // 简单地插入一个文本值作为节点内容
            let node_content = format!("{{\"type\": \"DXGC\", \"id\": \"{}\", \"client\": \"rust_client\"}}", node_id);
            nodes_map.insert(&mut txn, node_id.as_str(), node_content.as_str());

            // 事务会在 drop 时自动提交
            drop(txn);
                    tracing::info!("📝 已发送本地文档更改。");
                */ }
            }
            // 处理 Ctrl-C 以优雅地断开连接
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("🔌 正在断开连接...");
                provider.disconnect().await;
                tracing::info!("✅ 已断开连接。");
                break;
            }
        }
    }

    Ok(())
}
