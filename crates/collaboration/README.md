# ModuForge 协作 Crate

`moduforge-collaboration` 是一个为 ModuForge 生态系统提供实时协作功能的 Rust crate。它允许多个用户同时在同一个文档上工作，并实时同步所有更改。

## 核心技术

为了确保高性能和高可靠性，协作服务器构建在一系列健壮且现代的技术之上：

-   **WebSocket**: 服务器使用 WebSocket (`tokio-tungstenite`) 在客户端和服务器之间进行持久、低延迟的双向通信。这对于实现实时协作至关重要。
-   **CRDTs (无冲突复制数据类型)**: 同步逻辑的核心是 `yrs`，它是流行的 Yjs CRDT 框架的 Rust 移植。CRDTs 允许本地优先的开发模式，并保证即使在多个用户并发编辑的情况下，文档状态最终也能无冲突地达成一致。
-   **Tokio**: 整个服务器构建在 Tokio 异步运行时之上，使其能够高效地处理大量并发连接。

## 架构

该 crate 由几个协同工作的关键组件构成，共同提供协作服务：

### `CollaborationServer`

这是服务器的主要入口点。其职责包括：
-   接受来自客户端的 WebSocket 连接请求。
-   管理每个客户端连接的生命周期，包括注册和清理。
-   将客户端组织到"房间"中，每个房间对应一个唯一的文档协作会话。
-   向房间内的所有客户端广播消息。

### `SyncService`

`SyncService` 充当业务逻辑层。它处理来自客户端的传入消息，并与其他组件协调。其处理的事务包括：
-   客户端加入或离开房间的请求。
-   处理 `Yrs` 更新消息，并将其应用到相应的文档上。
-   为新加入的客户端同步文档状态。

### `YrsManager`

该组件负责管理所有活动的 CRDT 文档 (`yrs::Doc`)。它：
-   维护一个从 `room_id` 到 `yrs::Doc` 的映射。
-   提供一种线程安全的方式来访问、创建和更新文档。
-   确保对给定文档的所有更改都得到正确处理。

## 房间管理和错误处理

### 🔒 **严格的房间存在性检查**

从安全性和资源管理角度考虑，系统采用严格的房间管理策略：

#### **房间必须预先初始化**
```rust
// ✅ 正确方式：预先初始化房间
let server = CollaborationServer::with_sync_service(yrs_manager, sync_service, 8080);
server.init_room_with_data("room-123", &tree).await?;
```

#### **客户端连接检查**
当客户端尝试连接到房间时：
- ✅ **房间存在** → 允许连接，正常进行 WebSocket 升级
- ❌ **房间不存在** → 返回 404 错误，拒绝连接

```rust
// WebSocket 连接: ws://localhost:8080/collaboration/room-123

// 如果 room-123 不存在，客户端会收到：
{
  "error": "ROOM_NOT_FOUND",
  "message": "房间 'room-123' 不存在",
  "room_id": "room-123",
  "code": 404
}
```

### 🎯 **房间状态管理**

#### **房间状态枚举**
```rust
pub enum RoomStatus {
    NotExists,    // 房间不存在
    Created,      // 房间已创建但未初始化数据
    Initialized,  // 房间已初始化并有数据
    Shutting,     // 房间正在下线中
    Offline,      // 房间已下线
}
```

#### **状态检查 API**
```rust
// 检查房间是否存在
let exists = yrs_manager.room_exists("room-id");

// 获取房间状态
let status = sync_service.get_room_status("room-id").await;

// 获取详细房间信息
let room_info = sync_service.get_room_info("room-id").await;
```

## 数据流

1.  **服务器启动** → 预初始化房间并同步现有 Tree 数据
2.  客户端请求连接到特定房间
3.  **房间存在性检查** → 验证房间是否已初始化
4.  **连接建立** → 仅当房间存在时才升级到 WebSocket
5.  客户端自动获得完整的文档状态（通过 Yrs 的增量同步机制）
6.  当客户端进行更改时，通过 `YrsMiddleware` 同步到 Yrs 文档
7.  所有更改实时广播给房间内的其他客户端

## 通信协议

客户端和服务器之间的通信是通过一个 JSON 序列化的枚举 `WsMessage` 来处理的。这定义了协作服务的 API。

关键消息类型包括：
-   `JoinRoom { room_id }`: 客户端请求加入房间。
-   `LeaveRoom { room_id }`: 客户端离开房间。
-   `YrsUpdate { room_id, update }`: 客户端发送文档更改（作为二进制的 `Yrs` 更新负载）。
-   `YrsSyncRequest { room_id, state_vector }`: 客户端请求它尚未拥有的最新更改。

## 如何运行

要启动协作服务器，您需要实例化主要组件并运行 `CollaborationServer`。

以下是如何启动服务器的完整示例：

```rust,ignore
use moduforge_collaboration::{CollaborationServer, YrsManager, SyncService};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化 YrsManager
    let yrs_manager = Arc::new(YrsManager::new());

    // 2. 初始化 SyncService
    let sync_service = Arc::new(SyncService::new(yrs_manager.clone()));

    // 3. 初始化 CollaborationServer
    let server = CollaborationServer::with_sync_service(
        yrs_manager,
        sync_service.clone(),
        8080
    );

    // 4. 关键步骤：使用现有数据预初始化房间
    let rooms_to_initialize = ["room1", "room2", "project-main"];
    
    for room_id in &rooms_to_initialize {
        // 从存储加载或创建初始 Tree
        if let Some(existing_tree) = load_room_data(room_id).await? {
            server.init_room_with_data(room_id, &existing_tree).await?;
            println!("✅ 房间 '{}' 已初始化", room_id);
        } else {
            println!("⚠️ 房间 '{}' 无初始数据，跳过初始化", room_id);
        }
    }

    // 5. 启动服务器
    println!("正在启动协作服务器于 127.0.0.1:8080...");
    println!("🔒 只有预初始化的房间才能接受客户端连接");
    server.start().await;

    Ok(())
}

async fn load_room_data(room_id: &str) -> anyhow::Result<Option<Tree>> {
    // 这里实现从数据库、文件等加载 Tree 的逻辑
    // 返回 Some(tree) 如果有现有数据，否则返回 None
    Ok(None)
}
```

### 错误处理示例

#### **客户端连接错误处理**
```javascript
// 前端 JavaScript 示例
const ws = new WebSocket('ws://localhost:8080/collaboration/non-existent-room');

ws.onerror = function(error) {
    console.error('WebSocket 连接失败:', error);
    // 服务器会返回 404 状态码，表示房间不存在
};

// 或者使用 fetch 检查房间状态
async function checkRoomExists(roomId) {
    try {
        const response = await fetch(`http://localhost:8080/collaboration/${roomId}`);
        if (response.status === 404) {
            const error = await response.json();
            console.log('房间不存在:', error.message);
            return false;
        }
        return true;
    } catch (error) {
        console.error('检查房间状态失败:', error);
        return false;
    }
}
```

### 高级使用场景

#### **动态房间管理**
```rust
// 运行时创建新房间
pub async fn create_room_on_demand(
    server: &CollaborationServer,
    room_id: &str,
    initial_tree: &Tree
) -> Result<bool> {
    // 检查房间是否已存在
    if server.sync_service().yrs_manager().room_exists(room_id) {
        return Ok(false); // 房间已存在
    }
    
    // 初始化新房间
    server.init_room_with_data(room_id, initial_tree).await?;
    Ok(true)
}
```

#### **房间生命周期管理**
```rust
// 完整的房间生命周期
async fn room_lifecycle_example() -> Result<()> {
    let server = setup_server().await;
    let room_id = "example-room";
    
    // 1. 检查房间状态
    let initial_status = server.sync_service().get_room_status(room_id).await;
    assert_eq!(initial_status, RoomStatus::NotExists);
    
    // 2. 初始化房间
    let tree = create_initial_tree();
    server.init_room_with_data(room_id, &tree).await?;
    
    // 3. 验证房间已初始化
    let status = server.sync_service().get_room_status(room_id).await;
    assert_eq!(status, RoomStatus::Initialized);
    
    // 4. 客户端现在可以连接
    // WebSocket 连接: ws://localhost:8080/collaboration/example-room
    
    // 5. 房间下线
    server.offline_room(room_id, true).await?;
    
    // 6. 验证房间已下线
    let final_status = server.sync_service().get_room_status(room_id).await;
    assert_eq!(final_status, RoomStatus::NotExists);
    
    Ok(())
}
```

## 最佳实践

1. **🔒 严格房间管理** - 只有预初始化的房间才能接受连接
2. **📊 状态监控** - 定期检查房间状态和连接数
3. **⚡ 预初始化** - 在服务器启动时预初始化常用房间
4. **🧹 生命周期管理** - 及时下线不活跃的房间释放资源
5. **🚨 错误处理** - 客户端应优雅处理房间不存在的情况
6. **📝 日志记录** - 详细记录房间操作以便调试和监控

这种设计确保了协作功能的高安全性、可控性和优秀的用户体验。