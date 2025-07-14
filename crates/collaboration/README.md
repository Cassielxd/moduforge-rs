# ModuForge 协作系统 (moduforge-collaboration)

`moduforge-collaboration` 是一个为 ModuForge 生态系统提供实时协作功能的 Rust crate。它基于 CRDT (无冲突复制数据类型) 技术，允许多个用户同时在同一个文档上工作，并实时同步所有更改。

## 🏗️ 架构概述

协作系统采用分层架构设计，每个组件都有明确的职责：

```
┌─────────────────────────────────────────────────────────────┐
│                    CollaborationServer                      │
│              (WebSocket 服务器 + 路由管理)                    │
├─────────────────────────────────────────────────────────────┤
│                    SyncService                              │
│              (业务逻辑 + 状态管理)                            │
├─────────────────────────────────────────────────────────────┤
│                    YrsManager                               │
│              (CRDT 文档管理)                                 │
├─────────────────────────────────────────────────────────────┤
│                    Mapper                                   │
│              (数据转换 + 步骤映射)                            │
└─────────────────────────────────────────────────────────────┘
```

## 🧩 核心组件

### 1. CollaborationServer
**文件**: `src/ws_server.rs`  
**职责**: WebSocket 服务器和 HTTP 路由管理

- **WebSocket 连接管理**: 处理客户端连接、断开和消息路由
- **房间存在性检查**: 严格的房间验证机制
- **HTTP 端点**: 提供房间状态查询和健康检查
- **错误处理**: 统一的错误响应格式

**关键特性**:
```rust
// 严格的房间存在性检查
if !server.sync_service().yrs_manager().room_exists(&room_id) {
    return Err(warp::reject::custom(RoomNotFoundError::new(room_id)));
}

// 自定义错误处理
async fn handle_rejection(err: Rejection) -> Result<impl Reply> {
    if let Some(room_error) = err.find::<RoomNotFoundError>() {
        return Ok(json!({
            "error": "ROOM_NOT_FOUND",
            "message": format!("房间 '{}' 不存在", room_error.room_id()),
            "code": 404
        }));
    }
    // ... 其他错误处理
}
```

### 2. SyncService
**文件**: `src/sync_service.rs`  
**职责**: 业务逻辑和状态管理

- **房间生命周期管理**: 创建、初始化、下线房间
- **事务处理**: 将 ModuForge 事务同步到 Yrs 文档
- **状态查询**: 提供房间状态和统计信息
- **数据同步**: Tree 到 Yrs 文档的双向转换

**房间状态枚举**:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RoomStatus {
    NotExists,    // 房间不存在
    Created,      // 房间已创建但未初始化数据
    Initialized,  // 房间已初始化并有数据
    Shutting,     // 房间正在下线中
    Offline,      // 房间已下线
}
```

### 3. YrsManager
**文件**: `src/yrs_manager.rs`  
**职责**: CRDT 文档管理

- **文档生命周期**: 创建、访问、清理 Yrs 文档
- **线程安全**: 使用 `DashMap` 和 `RwLock` 确保并发安全
- **资源管理**: 自动清理不活跃的房间
- **批量操作**: 支持批量房间管理

**核心方法**:
```rust
impl YrsManager {
    // 获取或创建房间的 Awareness 引用
    pub fn get_or_create_awareness(&self, room_id: &str) -> AwarenessRef;
    
    // 检查房间是否存在
    pub fn room_exists(&self, room_id: &str) -> bool;
    
    // 移除房间并清理资源
    pub async fn remove_room(&self, room_id: &str) -> Option<AwarenessRef>;
    
    // 强制清理房间资源
    pub async fn force_cleanup_room(&self, room_id: &str) -> bool;
}
```

### 4. Mapper
**文件**: `src/mapping.rs`  
**职责**: 数据转换和步骤映射

- **步骤转换器**: 将 ModuForge 步骤转换为 Yrs 操作
- **类型安全**: 使用 Trait 系统确保类型安全
- **可扩展性**: 支持自定义转换器注册
- **数据序列化**: Tree 和快照之间的转换

**转换器系统**:
```rust
pub trait StepConverter: Send + Sync {
    fn apply_to_yrs_txn(
        &self,
        step: &dyn Step,
        txn: &mut TransactionMut,
    ) -> Result<StepResult, Box<dyn std::error::Error>>;
    
    fn name(&self) -> &'static str;
    fn supports(&self, step: &dyn Step) -> bool;
}

// 内置转换器
pub struct NodeStepConverter;    // 节点操作
pub struct AttrStepConverter;    // 属性操作
pub struct MarkStepConverter;    // 标记操作
```

### 5. YrsMiddleware
**文件**: `src/middleware.rs`  
**职责**: 中间件集成

- **事务拦截**: 拦截 ModuForge 事务并同步到 Yrs
- **自动同步**: 无需手动调用，自动处理状态变更
- **错误处理**: 优雅处理同步失败

```rust
#[async_trait]
impl Middleware for YrsMiddleware {
    async fn after_dispatch(
        &self,
        _state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> ForgeResult<Option<Transaction>> {
        // 自动同步事务到 Yrs 文档
        self.sync_service
            .handle_transaction_applied(transactions, &self.room_id)
            .await?;
        Ok(None)
    }
}
```

## 🔧 技术栈

### 核心依赖
```toml
[dependencies]
# 异步运行时
tokio = { workspace = true }
async-trait = { workspace = true }

# WebSocket 和 HTTP
warp = "0.3.7"
yrs-warp = "0.8.0"

# CRDT 引擎
yrs = "0.18.2"

# 并发和同步
parking_lot = { workspace = true }
dashmap = { workspace = true }

# 序列化
serde = { workspace = true }
serde_json = { workspace = true }

# 日志和监控
tracing = "0.1"
tracing-subscriber = "0.3"

# ModuForge 生态系统
moduforge-model = { version = "0.4.10", path = "../model" }
moduforge-state = { version = "0.4.10", path = "../state" }
moduforge-transform = { version = "0.4.10", path = "../transform" }
moduforge-core = { version = "0.4.10", path = "../core" }
```

### 核心技术
- **CRDTs**: 基于 Yrs (Yjs Rust 移植) 的无冲突复制数据类型
- **WebSocket**: 使用 Warp 框架的高性能 WebSocket 服务器
- **异步编程**: 基于 Tokio 的异步运行时
- **类型安全**: 完整的 Rust 类型系统保证

## 🚀 快速开始

### 基本使用

```rust
use mf_collab::{CollaborationServer, YrsManager, SyncService};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化核心组件
    let yrs_manager = Arc::new(YrsManager::new());
    let sync_service = Arc::new(SyncService::new(yrs_manager.clone()));
    
    // 2. 创建协作服务器
    let server = CollaborationServer::with_sync_service(
        yrs_manager,
        sync_service.clone(),
        8080
    );

    // 3. 预初始化房间（关键步骤）
    let rooms_to_initialize = ["room1", "room2", "project-main"];
    
    for room_id in &rooms_to_initialize {
        if let Some(existing_tree) = load_room_data(room_id).await? {
            server.init_room_with_data(room_id, &existing_tree).await?;
            println!("✅ 房间 '{}' 已初始化", room_id);
        }
    }

    // 4. 启动服务器
    println!("🚀 协作服务器启动于 127.0.0.1:8080");
    server.start().await;

    Ok(())
}
```

### 与 ModuForge 运行时集成

```rust
use mf_core::{ForgeRuntime, RuntimeOptions};
use mf_collab::YrsMiddleware;

async fn setup_collaborative_runtime(
    sync_service: Arc<SyncService>,
    room_id: String,
) -> ForgeRuntime {
    let mut options = RuntimeOptions::default();
    
    // 添加 Yrs 中间件
    let yrs_middleware = YrsMiddleware {
        sync_service: sync_service.clone(),
        room_id: room_id.clone(),
    };
    options.add_middleware(yrs_middleware);
    
    // 创建运行时
    ForgeRuntime::new(options).await
}
```

## 🔒 安全特性

### 严格的房间管理
- **预初始化要求**: 只有预初始化的房间才能接受客户端连接
- **存在性验证**: 每个连接请求都验证房间存在性
- **资源隔离**: 每个房间独立管理，避免资源泄露

### 错误处理
```rust
// 房间不存在时的错误响应
{
    "error": "ROOM_NOT_FOUND",
    "message": "房间 'room-123' 不存在",
    "room_id": "room-123",
    "code": 404
}
```

## 📊 监控和管理

### 房间状态查询
```rust
// 获取房间状态
let status = sync_service.get_room_status("room-id").await;

// 获取房间详细信息
let room_info = sync_service.get_room_info("room-id").await;
// RoomInfo {
//     room_id: "room-id",
//     status: RoomStatus::Initialized,
//     node_count: 42,
//     client_count: 3,
//     last_activity: SystemTime { ... }
// }
```

### 批量操作
```rust
// 下线空房间
let empty_rooms = server.offline_empty_rooms(true).await?;

// 下线不活跃房间
let inactive_rooms = server.offline_inactive_rooms(
    Duration::from_secs(3600), // 1小时无活动
    true
).await?;

// 条件下线
let rooms_to_offline = server.offline_rooms_by_condition(
    |room_info| room_info.client_count == 0,
    true
).await?;
```

## 🌐 WebSocket API

### 连接端点
```
WebSocket: ws://localhost:8080/collaboration/{room_id}
HTTP 状态检查: GET /collaboration/{room_id}
健康检查: GET /health
```

### 消息格式
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsMessage {
    JoinRoom { room_id: String },
    LeaveRoom { room_id: String },
    YrsUpdate { room_id: String, update: Vec<u8> },
    YrsSyncRequest { room_id: String, state_vector: Vec<u8> },
}
```

## 🧪 测试

项目包含完整的测试套件，覆盖核心功能：

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_collaboration
cargo test test_room_offline
cargo test test_conditional_offline
```

### 测试覆盖范围
- ✅ 基本协作功能
- ✅ 房间生命周期管理
- ✅ 错误处理和边界情况
- ✅ HTTP 端点功能
- ✅ 房间存在性检查
- ✅ 批量操作功能

## 🔧 配置选项

### 服务器配置
```rust
// 自定义端口
let server = CollaborationServer::with_sync_service(
    yrs_manager,
    sync_service,
    9000 // 自定义端口
);

// 自定义错误处理
server.set_error_handler(custom_error_handler);
```

### 中间件配置
```rust
// 自定义中间件栈
let mut middleware_stack = MiddlewareStack::new();
middleware_stack.add(YrsMiddleware::new(sync_service, room_id));
middleware_stack.add(LoggingMiddleware::new());
```

## 📈 性能优化

### 内存管理
- **智能清理**: 自动清理不活跃的房间
- **批量操作**: 支持批量房间管理减少锁竞争
- **资源池**: 复用 Yrs 文档对象

### 并发处理
- **异步 I/O**: 基于 Tokio 的高性能异步处理
- **锁优化**: 使用 `RwLock` 和 `DashMap` 优化并发访问
- **连接池**: 高效的 WebSocket 连接管理

## 🚨 错误处理

### 错误类型
```rust
#[derive(Error, Debug)]
pub enum TransmissionError {
    #[error("Yrs 操作错误: {0}")]
    YrsError(String),
    
    #[error("WebSocket 错误: {0}")]
    WebSocketError(String),
    
    #[error("房间不存在: {0}")]
    RoomNotFound(String),
    
    #[error("同步错误: {0}")]
    SyncError(String),
    
    #[error("其他错误: {0}")]
    Other(#[from] anyhow::Error),
}
```

### 错误恢复策略
- **自动重连**: 客户端自动重连机制
- **状态恢复**: 从快照恢复房间状态
- **优雅降级**: 部分功能失效时的降级处理

## 🔮 未来规划

### 计划功能
- [ ] 持久化存储支持
- [ ] 分布式部署
- [ ] 实时性能监控
- [ ] 插件系统扩展
- [ ] 移动端优化

### 性能目标
- [ ] 支持 1000+ 并发连接
- [ ] 毫秒级同步延迟
- [ ] 内存使用优化
- [ ] 网络带宽优化

## 📚 相关文档

- [ModuForge 核心文档](../core/README.md)
- [状态管理文档](../state/README.md)
- [数据模型文档](../model/README.md)
- [转换系统文档](../transform/README.md)

## 🤝 贡献指南

欢迎贡献代码！请确保：

1. 遵循 Rust 编码规范
2. 添加适当的测试
3. 更新相关文档
4. 通过所有 CI 检查

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](../../LICENSE) 文件。