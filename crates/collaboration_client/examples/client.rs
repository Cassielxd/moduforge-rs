use std::sync::Arc;
use tokio::time::{self, Duration};
use yrs::{sync::Awareness, DeepObservable, Doc, Map, Observable, Transact};
use mf_collab_client::AwarenessRef;

use mf_collab_client::{provider::WebsocketProvider, types::SyncEvent};

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
    let client_id = doc.client_id();
    let awareness: AwarenessRef =
        Arc::new(tokio::sync::RwLock::new(Awareness::new(doc)));

    // 🔍 生成唯一的用户 ID
    let unique_user_id =
        format!("client-user-{}-{}", chrono::Utc::now().timestamp(), client_id);

    tracing::info!("🆔 Yrs Client ID: {}", client_id);
    tracing::info!("👤 User ID: {}", unique_user_id);

    let mut provider = WebsocketProvider::new(
        server_url.to_string(),
        room_name.to_string(),
        awareness.clone(),
    )
    .await;

    // 订阅同步事件
    if let Some(mut receiver) = provider.subscribe_sync_events() {
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                match event {
                    SyncEvent::InitialSyncCompleted {
                        has_data,
                        elapsed_ms,
                    } => {
                        if has_data {
                            println!(
                                "🎉 同步完成，房间有数据！耗时: {elapsed_ms}ms"
                            );
                        } else {
                            println!(
                                "📭 同步完成，空房间！耗时: {elapsed_ms}ms"
                            );
                        }
                    },
                    SyncEvent::ProtocolStateChanged(state) => {
                        println!("📡 协议状态: {state:?}");
                    },
                    SyncEvent::DataReceived => {
                        println!("📥 收到数据更新");
                    },
                    SyncEvent::ConnectionFailed(error) => {
                        println!("🔌 监听: {error:?}");
                    },
                    SyncEvent::ConnectionChanged(status) => {
                        println!("🔌 连接状态: {status:?}");
                    },
                }
            }
        });
    }

    // 3. 连接到服务器
    provider.connect().await;
    {
        let nodes_map = awareness.read().await.doc().get_or_insert_map("nodes");
        // 订阅 nodes变更 浅
        provider.subscription(nodes_map.observe(move |txn, event| {
            for (key, change) in event.keys(txn) {
                match change {
                    yrs::types::EntryChange::Inserted(value) => {
                        println!("新增 key: {key}, value: {value:?}");
                    },
                    yrs::types::EntryChange::Removed(old_value) => {
                        println!(
                            "删除 key: {key}, old value: {old_value:?}"
                        );
                    },
                    yrs::types::EntryChange::Updated(old_value, new_value) => {
                        println!(
                            "更新 key: {key}, old: {old_value:?}, new: {new_value:?}"
                        );
                    },
                }
            }
        }));
        provider.subscription(nodes_map.observe_deep(move |_txn, events| {
            for event in events.iter() {
                match event {
                    yrs::types::Event::Array(_array_event) => {
                        // 更新了 标记数组 需要转换成 step
                    },
                    yrs::types::Event::Map(_map_event) => {
                        // 更新了 节点属性 需要转换成 step 或者 添加节点
                    },
                    _ => {},
                }
            }
        }));
    }
    {
        let client_id_ref = client_id;
        let mut awareness_lock = awareness.write().await;
        provider.subscription(awareness_lock.on_update(move |event| {
            println!("📡 awareness update: {:?}", event.awareness_state());
            let states = event.awareness_state();
            for client_id in states.all_clients() {
                let meta: &yrs::sync::awareness::MetaClientState =
                    states.get_meta(client_id).unwrap();
                if client_id == client_id_ref {
                    // 本地客户端
                    println!(
                        "🏠 本地客户端 {}: clock={}, last_updated={:?}",
                        client_id, meta.clock, meta.last_updated
                    );
                } else {
                    // 远程客户端
                    println!(
                        "🌐 远程客户端 {}: clock={}, last_updated={:?}",
                        client_id, meta.clock, meta.last_updated
                    );
                }
            }
        }));

        // 🎯 使用唯一的用户 ID 避免状态覆盖
        awareness_lock.set_local_state(
            serde_json::json!({
                "user": {
                    "id": unique_user_id,
                    "name": "用户李兴栋",
                    "color": "#FFEAA7",
                    "online": true,
                    "client_id": client_id,
                    "timestamp": chrono::Utc::now().timestamp()
                },
                "online": true
            })
            .to_string(),
        );

        tracing::info!("✅ 设置 awareness 状态完成");
    }

    // 4. 事件循环
    let mut counter = 0; // 🔄 添加计数器确保状态变化

    loop {
        tokio::select! {

            // 定期添加本地更改以测试发送更新
            _ = time::sleep(Duration::from_secs(3)) => {
                if provider.is_connected() {
                    counter += 1; // 🔄 递增计数器

                    let mut awareness_lock = awareness.write().await;
                    awareness_lock.set_local_state(serde_json::json!({
                        "user": {
                            "id": unique_user_id,
                            "name": "用户李兴栋",
                            "color": "#FFEAA7",
                            "online": true,
                            "client_id": client_id,
                            "timestamp": chrono::Utc::now().timestamp(),
                            "heartbeat": counter  // 🔄 添加变化的字段
                        },
                        "online": true,
                        "last_activity": chrono::Utc::now().timestamp()  // 🔄 另一个变化字段
                    }).to_string());

                    let doc = awareness_lock.doc_mut();
                    let nodes_map = doc.get_or_insert_map("nodes");
                    let mut txn = doc.transact_mut_with(doc.client_id().to_string());
                    // 生成新节点 ID
            let node_id = uuid::Uuid::new_v4().to_string();

            // 简单地插入一个文本值作为节点内容
            let node_content = format!("{{\"type\": \"DXGC\", \"id\": \"{node_id}\", \"client\": \"rust_client\"}}");
            nodes_map.insert(&mut txn, node_id.as_str(), node_content.as_str());

            // 事务会在 drop 时自动提交
            drop(txn);
                    tracing::info!("📝 已发送本地文档更改，heartbeat: {}", counter);
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
