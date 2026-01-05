# moduforge-collaboration 文档

`moduforge-collaboration` 基于 Yrs (CRDT) 提供实时协作能力。

## 概述

Collaboration 层提供多人实时协作、房间管理和断线恢复。

## 核心功能

- **CRDT 协作**：基于 Yrs 的无冲突复制
- **WebSocket 服务**：基于 Warp 的实时通信
- **房间管理**：多房间隔离
- **Awareness**：用户状态感知
- **断线恢复**：自动重连和同步

## 服务端

```rust
use mf_collaboration::CollaborationServer;

// 创建服务器
let server = CollaborationServer::new(config)?;

// 启动服务
server.start().await?;
```

## 客户端

```rust
use mf_collaboration_client::CollaborationClient;

// 连接服务器
let client = CollaborationClient::connect("ws://localhost:8080")?;

// 加入房间
client.join_room("room1").await?;

// 应用本地变更
client.apply_transaction(tr).await?;

// 接收远程变更
let updates = client.receive_updates().await?;
```

## 架构

```
客户端1 ←→ WebSocket ←→ 服务器 ←→ WebSocket ←→ 客户端2
             ↓                         ↓
          本地事务                  本地事务
             ↓                         ↓
           CRDT合并                 CRDT合并
```

## 下一步

- 查看 [协作编辑器示例](../examples/collaborative.md)
