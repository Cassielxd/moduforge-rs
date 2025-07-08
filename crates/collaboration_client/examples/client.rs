use std::sync::Arc;
use tokio::time::{self, Duration};
use yrs::{sync::Awareness, Doc, Map, Observable, Transact};
use yrs_warp::AwarenessRef;
use tracing_subscriber;
use serde_json;

use mf_collab_client::{provider::WebsocketProvider, types::ProviderEvent};

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
   
    // 2. 订阅 provider 事件
    let mut rx = provider.subscribe();

    // 3. 连接到服务器
    provider.connect().await;
    let nodes_map = awareness.read().await.doc().get_or_insert_map("nodes");
    nodes_map.observe(move|txn, event|{
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
    });
    // 4. 事件循环
    loop {
        tokio::select! {
            // 监听 provider 事件
            Ok(event) = rx.recv() => {
                match event {
                    ProviderEvent::Status(status) => {
                        if status == "connected" {
                            // 设置本地 awareness 状态
                            let mut awareness_lock = awareness.write().await;

                            // 生成符合格式的用户信息
                            let user_info = serde_json::json!({
                                "user": {
                                    "id": "client-user-1",
                                    "name": "用户李兴栋",
                                    "color": "#FFEAA7",
                                    "online": true
                                },
                                "online": true
                            });

                            awareness_lock.set_local_state(user_info.to_string());
                            tracing::info!("👤 已设置本地 awareness 状态。");
                        }
                    }
                    // 根据您之前的修改，这里处理 SyncMessage
                    ProviderEvent::SyncMessage { step, data_length, preview } => {
                        tracing::info!(
                            "📄 收到同步消息 (步骤: {}, 大小: {}, 预览: '{}')",
                            step,
                            data_length,
                            preview
                        );
                    }
                    ProviderEvent::AwarenessMessage { data_length, preview } => {
                        tracing::info!(
                            "👥 收到 awareness 更新 (长度: {}, 预览: {})",
                            data_length,
                            preview
                        );
                        let awareness_lock = awareness.read().await;
                        let states: Vec<_> = awareness_lock.clients().iter().map(|(id, state)| (id, state.clone())).collect();
                        tracing::info!("📊 当前 awareness 状态: {:?}", states);
                    }
                    ProviderEvent::ConnectionClose => {
                        tracing::info!("❌ 连接已关闭。");
                        break;
                    }
                    ProviderEvent::ConnectionError(err) => {
                        tracing::error!("❌ 连接错误: {}", err);
                        break;
                    }
                    _ => {}
                }
            }
            // 定期添加本地更改以测试发送更新
            _ = time::sleep(Duration::from_secs(3)) => {
                if provider.is_connected() {
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
                }
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
