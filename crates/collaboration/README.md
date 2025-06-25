# ModuForge 协作 Crate

`moduforge-collaboration` 是一个为 ModuForge 生态系统提供实时协作功能的 Rust crate。它允许多个用户同时在同一个文档上工作，并实时同步所有更改。

## 核心技术

为了确保高性能和高可靠性，协作服务器构建在一系列健壮且现代的技术之上：

-   **WebSocket**: 服务器使用 WebSocket (`tokio-tungstenite`) 在客户端和服务器之间进行持久、低延迟的双向通信。这对于实现实时协作至关重要。
-   **CRDTs (无冲突复制数据类型)**: 同步逻辑的核心是 `yrs`，它是流行的 Yjs CRDT 框架的 Rust 移植。CRDTs 允许本地优先的开发模式，并保证即使在多个用户并发编辑的情况下，文档状态最终也能无冲突地达成一致。
-   **Tokio**: 整个服务器构建在 Tokio 异步运行时之上，使其能够高效地处理大量并发连接。

## 架构

该 crate 由几个协同工作的关键组件构成，共同提供协作服务：

### `WebSocketServer`

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

### 数据流

1.  客户端与 `WebSocketServer` 建立 WebSocket 连接。
2.  客户端发送 `JoinRoom` 消息以加入特定的协作会话。
3.  `WebSocketServer` 将消息传递给 `SyncService`。
4.  `SyncService` 与 `YrsManager` 交互，加载或创建该房间的文档，并将初始文档状态发送回客户端。
5.  当客户端进行更改时，它会向服务器发送 `YrsUpdate` 消息。
6.  `SyncService` 通过 `YrsManager` 将此更新应用到文档中。
7.  `WebSocketServer` 随后将此更新广播给同一房间中的所有其他客户端。

## 通信协议

客户端和服务器之间的通信是通过一个 JSON 序列化的枚举 `WsMessage` 来处理的。这定义了协作服务的 API。

关键消息类型包括：
-   `JoinRoom { room_id }`: 客户端请求加入房间。
-   `LeaveRoom { room_id }`: 客户端离开房间。
-   `YrsUpdate { room_id, update }`: 客户端发送文档更改（作为二进制的 `Yrs` 更新负载）。
-   `YrsSyncRequest { room_id, state_vector }`: 客户端请求它尚未拥有的最新更改。

## 如何运行

要启动协作服务器，您需要实例化主要组件并运行 `WebSocketServer`。`main.rs` 文件提供了一个可运行的示例。

以下是如何启动服务器的简化示例：

```rust,ignore
use moduforge_collaboration::{WebSocketServer, YrsManager, SyncService};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化 YrsManager
    let yrs_manager = Arc::new(YrsManager::new());

    // 2. 初始化 SyncService
    let sync_service = Arc::new(SyncService::new(
        yrs_manager.clone(),
        // ... 其他所需依赖
    ));

    // 3. 初始化 WebSocketServer
    let server = Arc::new(WebSocketServer::new(yrs_manager));

    // 4. 定义服务器地址
    let addr = "127.0.0.1:8080".parse().unwrap();

    // 5. 启动服务器
    println!("正在启动协作服务器于 {}...", addr);
    server.start(addr, sync_service, /* ForgeRuntime */).await?;

    Ok(())
} 