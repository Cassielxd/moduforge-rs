# ModuForge 协作客户端

协作客户端（`moduforge-collaboration-client`）提供了基于 WebSocket 和 Yrs (Yjs Rust 实现) 的实时协作功能，支持多用户同时编辑文档。

## 核心功能

- **WebSocket 连接管理**：自动重连、连接状态监控
- **CRDT 同步**：基于 Yrs 的冲突解决算法
- **意识系统（Awareness）**：用户状态和光标位置同步
- **步骤转换**：将编辑操作转换为可同步的步骤
- **类型安全**：静态分发的转换器系统

## 架构概览

```
┌──────────────────┐
│  WebSocket 层     │ ← ClientSink/ClientStream
├──────────────────┤
│  Connection 层    │ ← 连接管理和消息处理
├──────────────────┤
│  Provider 层      │ ← WebsocketProvider
├──────────────────┤
│  Yrs/CRDT 层     │ ← Awareness, Doc
├──────────────────┤
│  Mapping 层       │ ← Step 转换器
└──────────────────┘
```

## WebSocket 连接

### ClientSink 和 ClientStream

封装 WebSocket 的发送和接收流，处理二进制消息传输。

```rust
use mf_collab_client::client::{ClientSink, ClientStream};
use tokio_tungstenite::connect_async;
use futures_util::StreamExt;

// 建立 WebSocket 连接
let (ws_stream, _) = connect_async("ws://localhost:3000/room1").await?;
let (sink, stream) = ws_stream.split();

// 包装成客户端流
let client_sink = ClientSink(sink);
let client_stream = ClientStream(stream);
```

### Connection 管理

Connection 负责协议同步和消息处理。

```rust
use mf_collab_client::conn::Connection;
use yrs::sync::Awareness;
use std::sync::Arc;
use tokio::sync::RwLock;

// 创建 Awareness（用户意识系统）
let doc = yrs::Doc::new();
let awareness = Arc::new(RwLock::new(Awareness::new(doc)));

// 创建带同步检测的连接
let conn = Connection::new_with_sync_detection(
    awareness.clone(),
    client_sink,
    client_stream,
    Some(event_sender), // 同步事件发送器
);
```

## WebsocketProvider

高级 API，自动管理连接生命周期。

```rust
use mf_collab_client::provider::WebsocketProvider;
use mf_collab_client::types::ConnectionRetryConfig;

// 创建 Provider
let mut provider = WebsocketProvider::new(
    "ws://localhost:3000".to_string(),
    "price-calculation-room".to_string(), // 房间名
    awareness.clone(),
).await;

// 连接配置
let retry_config = ConnectionRetryConfig {
    max_attempts: 5,
    initial_delay_ms: 1000,
    backoff_multiplier: 2.0,
    max_delay_ms: 30000,
    connect_timeout_ms: 10000,
};

// 连接并自动重试
provider.connect_with_retry(Some(retry_config)).await?;
```

### 连接状态管理

```rust
use mf_collab_client::types::ConnectionStatus;

match provider.status {
    ConnectionStatus::Disconnected => {
        println!("未连接");
    },
    ConnectionStatus::Connecting => {
        println!("连接中...");
    },
    ConnectionStatus::Connected => {
        println!("已连接");
    },
    ConnectionStatus::Failed(error) => {
        println!("连接失败: {}", error);
    },
    ConnectionStatus::Retrying { attempt, max_attempts } => {
        println!("重试 {}/{}", attempt, max_attempts);
    },
}
```

## 同步事件系统

监听同步状态变化和数据更新。

```rust
use mf_collab_client::types::{SyncEvent, ProtocolSyncState};

// 创建事件通道
let (event_sender, mut event_receiver) = tokio::sync::broadcast::channel(100);

// 监听同步事件
tokio::spawn(async move {
    while let Ok(event) = event_receiver.recv().await {
        match event {
            SyncEvent::InitialSyncCompleted { has_data, elapsed_ms } => {
                println!("首次同步完成，耗时 {}ms", elapsed_ms);
                if has_data {
                    println!("接收到已有数据");
                } else {
                    println!("空房间，开始新文档");
                }
            },
            SyncEvent::DataReceived => {
                println!("收到数据更新");
            },
            SyncEvent::ConnectionChanged(status) => {
                println!("连接状态变化: {:?}", status);
            },
            SyncEvent::ConnectionFailed(error) => {
                eprintln!("连接失败: {}", error);
            },
            _ => {}
        }
    }
});
```

## Step 转换系统

### 静态分发的转换器

使用泛型和静态分发提高性能。

```rust
use mf_collab_client::mapping_v2::{TypedStepConverter, StepResult};
use mf_model::step::Step;
use mf_state::State;
use yrs::TransactionMut;

// 定义价格计算步骤转换器
struct PriceStepConverter;

impl TypedStepConverter for PriceStepConverter {
    type Step = PriceCalculationStep;

    fn can_handle(&self, step: &Step) -> bool {
        step.step_type() == "price_calculation"
    }

    fn convert(
        &self,
        step: Self::Step,
        state: &State,
        txn: &mut TransactionMut,
    ) -> Result<StepResult, String> {
        // 获取价格数据
        let price_map = txn.get_or_insert_map("prices");

        // 应用价格更新
        match step {
            PriceCalculationStep::UpdatePrice { item_id, price } => {
                price_map.insert(txn, &item_id, price);
            },
            PriceCalculationStep::ApplyDiscount { discount_rate } => {
                // 应用折扣逻辑
                apply_discount_to_all(price_map, txn, discount_rate);
            },
        }

        Ok(StepResult {
            step_id: uuid::Uuid::new_v4().to_string(),
            step_name: "价格更新".to_string(),
            description: format!("更新价格: {}", step.description()),
            timestamp: chrono::Utc::now().timestamp() as u64,
            client_id: state.client_id().to_string(),
        })
    }
}
```

### 注册转换器

```rust
use mf_collab_client::mapping_v2::ConverterRegistry;

// 全局注册转换器
ConverterRegistry::register(PriceStepConverter);
ConverterRegistry::register(MaterialStepConverter);
ConverterRegistry::register(LaborStepConverter);

// 使用转换器
let step = PriceCalculationStep::UpdatePrice {
    item_id: "material_001".to_string(),
    price: 150.50,
};

let result = ConverterRegistry::convert(step, &state, &mut txn)?;
println!("步骤已应用: {}", result.description);
```

## 实际应用示例

### Price-RS 协作编辑

```rust
use mf_collab_client::{WebsocketProvider, AwarenessRef};
use yrs::{Doc, TransactionMut, Map};
use std::sync::Arc;
use tokio::sync::RwLock;

// 1. 初始化文档和意识系统
let doc = Doc::new();
let awareness: AwarenessRef = Arc::new(RwLock::new(
    yrs::sync::Awareness::new(doc.clone())
));

// 2. 创建并连接 Provider
let mut provider = WebsocketProvider::new(
    "ws://collaboration-server:3000".to_string(),
    "project_12345".to_string(), // 项目 ID 作为房间名
    awareness.clone(),
).await;

provider.connect_with_retry(None).await?;

// 3. 设置用户信息
{
    let mut awareness = awareness.write().await;
    let local_state = serde_json::json!({
        "user": {
            "id": "user_001",
            "name": "张工程师",
            "color": "#4A90E2",
            "role": "造价工程师"
        },
        "cursor": null
    });
    awareness.set_local_state(local_state);
}

// 4. 监听文档变化
let doc_clone = doc.clone();
let _sub = doc.observe_update_v1(move |_, event| {
    println!("文档更新: {:?}", event.update.len());

    // 处理更新
    let txn = doc_clone.transact();
    if let Some(prices) = txn.get_map("prices") {
        for (key, value) in prices.iter(&txn) {
            println!("价格项 {}: {:?}", key, value);
        }
    }
});

// 5. 应用本地更改
{
    let mut txn = doc.transact_mut();

    // 获取或创建价格表
    let prices = txn.get_or_insert_map("prices");

    // 添加工程项目
    prices.insert(&mut txn, "concrete_c30", yrs::Any::from(json!({
        "name": "C30混凝土",
        "unit": "m³",
        "unit_price": 450.00,
        "quantity": 1000,
        "total": 450000.00
    })));

    prices.insert(&mut txn, "steel_hrb400", yrs::Any::from(json!({
        "name": "HRB400钢筋",
        "unit": "吨",
        "unit_price": 4500.00,
        "quantity": 150,
        "total": 675000.00
    })));

    // 添加汇总信息
    let summary = txn.get_or_insert_map("summary");
    summary.insert(&mut txn, "subtotal", 1125000.00);
    summary.insert(&mut txn, "tax_rate", 0.09);
    summary.insert(&mut txn, "tax", 101250.00);
    summary.insert(&mut txn, "total", 1226250.00);
}

// 6. 监听其他用户的光标位置
let awareness_clone = awareness.clone();
tokio::spawn(async move {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let awareness = awareness_clone.read().await;
        let states = awareness.clients();

        for (client_id, state) in states {
            if *client_id != awareness.doc().client_id() {
                if let Some(user_state) = state.get("user") {
                    println!("用户 {} 在线: {}",
                        client_id,
                        user_state.get("name").unwrap_or(&json!("未知"))
                    );
                }
            }
        }
    }
});
```

### 协作会话管理

```rust
// 创建协作会话
struct CollaborationSession {
    provider: WebsocketProvider,
    awareness: AwarenessRef,
    doc: Doc,
    room_id: String,
}

impl CollaborationSession {
    pub async fn create(
        server_url: &str,
        room_id: &str,
        user_info: UserInfo,
    ) -> Result<Self> {
        let doc = Doc::new();
        let awareness = Arc::new(RwLock::new(Awareness::new(doc.clone())));

        // 设置用户信息
        {
            let mut awareness_guard = awareness.write().await;
            awareness_guard.set_local_state(json!({
                "user": user_info,
                "timestamp": chrono::Utc::now().timestamp()
            }));
        }

        // 创建 Provider
        let mut provider = WebsocketProvider::new(
            server_url.to_string(),
            room_id.to_string(),
            awareness.clone(),
        ).await;

        // 连接
        provider.connect_with_retry(None).await?;

        Ok(Self {
            provider,
            awareness,
            doc,
            room_id: room_id.to_string(),
        })
    }

    pub async fn apply_price_update(
        &self,
        item_id: &str,
        price: f64,
    ) -> Result<()> {
        let mut txn = self.doc.transact_mut();
        let prices = txn.get_or_insert_map("prices");

        // 更新价格
        if let Some(mut item) = prices.get(&txn, item_id) {
            if let Some(obj) = item.to_json(&txn).as_object_mut() {
                obj.insert("unit_price".to_string(), json!(price));
                obj.insert("updated_at".to_string(),
                    json!(chrono::Utc::now().to_rfc3339()));
                prices.insert(&mut txn, item_id, yrs::Any::from(json!(obj)));
            }
        }

        Ok(())
    }

    pub async fn get_online_users(&self) -> Vec<UserInfo> {
        let awareness = self.awareness.read().await;
        let mut users = Vec::new();

        for (client_id, state) in awareness.clients() {
            if let Some(user) = state.get("user") {
                users.push(UserInfo {
                    client_id: *client_id,
                    data: user.clone(),
                });
            }
        }

        users
    }
}

// 使用示例
let session = CollaborationSession::create(
    "ws://localhost:3000",
    "project_abc",
    UserInfo {
        id: "user_001".to_string(),
        name: "张工".to_string(),
        role: "造价工程师".to_string(),
    },
).await?;

// 更新价格
session.apply_price_update("material_001", 155.50).await?;

// 获取在线用户
let users = session.get_online_users().await;
for user in users {
    println!("在线用户: {:?}", user);
}
```

## 错误处理

### 连接错误类型

```rust
use mf_collab_client::types::ConnectionError;

match error {
    ConnectionError::ServerUnavailable(msg) => {
        eprintln!("服务器不可用: {}", msg);
        // 尝试备用服务器
    },
    ConnectionError::Timeout(ms) => {
        eprintln!("连接超时: {}ms", ms);
        // 增加超时时间重试
    },
    ConnectionError::NetworkError(msg) => {
        eprintln!("网络错误: {}", msg);
        // 检查网络连接
    },
    ConnectionError::WebSocketError(msg) => {
        eprintln!("WebSocket错误: {}", msg);
        // 重新建立 WebSocket 连接
    },
    ConnectionError::ProtocolError(msg) => {
        eprintln!("协议错误: {}", msg);
        // 检查版本兼容性
    },
}
```

### 重连策略

```rust
// 自定义重连策略
let retry_config = ConnectionRetryConfig {
    max_attempts: 10,           // 最大重试次数
    initial_delay_ms: 500,       // 初始延迟
    backoff_multiplier: 1.5,     // 退避系数
    max_delay_ms: 60000,         // 最大延迟
    connect_timeout_ms: 5000,    // 连接超时
};

// 应用重连策略
loop {
    match provider.connect_with_retry(Some(retry_config.clone())).await {
        Ok(_) => {
            println!("连接成功");
            break;
        },
        Err(e) => {
            eprintln!("连接失败: {}", e);

            // 可以选择切换服务器
            if let Some(backup_server) = get_backup_server() {
                provider.server_url = backup_server;
                continue;
            }

            break;
        }
    }
}
```

## 性能优化

### 批量更新

```rust
// 批量更新以减少同步次数
let mut txn = doc.transact_mut();

// 收集所有更改
let updates = vec![
    ("item1", 100.0),
    ("item2", 200.0),
    ("item3", 300.0),
    // ... 更多项目
];

// 一次性应用所有更改
let prices = txn.get_or_insert_map("prices");
for (id, price) in updates {
    prices.insert(&mut txn, id, price);
}

// 事务结束时自动同步
drop(txn);
```

### 增量同步

```rust
// 只同步变化的部分
doc.observe_update_v1(move |_, event| {
    // event.update 只包含增量更新
    let update_size = event.update.len();

    if update_size > 1000 {
        // 大更新，可能需要优化
        tracing::warn!("Large update: {} bytes", update_size);
    }
});
```

### 内存管理

```rust
// 定期清理历史记录
tokio::spawn(async move {
    let mut interval = tokio::time::interval(
        tokio::time::Duration::from_secs(300) // 5分钟
    );

    loop {
        interval.tick().await;

        // 清理旧的 awareness 状态
        let mut awareness = awareness.write().await;
        awareness.clean_local_state();

        // 可选：压缩文档历史
        // doc.gc();
    }
});
```

## 最佳实践

### 1. 连接管理

- 使用连接池管理多个房间
- 实现自动重连机制
- 监控连接状态并及时通知用户

### 2. 数据同步

- 批量处理更新以提高效率
- 使用事务确保原子性
- 实现冲突解决策略

### 3. 用户体验

- 显示在线用户列表
- 实时显示其他用户的光标
- 提供离线编辑支持

### 4. 安全性

- 使用 WSS (WebSocket Secure) 连接
- 实现身份验证和授权
- 加密敏感数据

## 配置选项

```rust
use mf_collab_client::types::WebsocketProviderOptions;

let options = WebsocketProviderOptions {
    connect: true,              // 立即连接
    resync_interval: Some(30000), // 30秒重新同步
    max_backoff_time: 5000,     // 最大退避时间
};

// 应用配置
let provider = WebsocketProvider::with_options(
    server_url,
    room_name,
    awareness,
    options,
).await;
```

## 调试和监控

```rust
// 启用详细日志
tracing_subscriber::fmt()
    .with_env_filter("mf_collab_client=debug")
    .init();

// 监控同步性能
doc.observe_update_v1(move |_, event| {
    let metrics = SyncMetrics {
        update_size: event.update.len(),
        timestamp: chrono::Utc::now(),
        client_id: doc.client_id(),
    };

    // 发送到监控系统
    send_metrics(metrics);
});
```

## 版本兼容性

- 需要 Rust 1.70+
- 依赖 Yrs 0.16+
- 支持 WebSocket 协议 v13
- 兼容 Yjs 13.x 协议