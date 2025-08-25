# 协作编辑器架构说明

## 概述

本项目实现了一个基于 yrs (Yjs for Rust) 的协作编辑器，支持实时同步和深度监听功能。

## 核心组件

### 1. CollabEditor (协作编辑器)

主要的协作编辑器类，继承自 `EditorTrait`，提供以下功能：

- **基础编辑功能**: 通过内部的 `ForgeAsyncRuntime` 提供状态管理、撤销/重做等功能
- **协作同步**: 通过 `CollabSyncManager` 管理与远程服务器的连接和同步
- **全局同步管理**: 集成 `GlobalSyncManager` 实现统一的事件处理

### 2. CollabSyncManager (协作同步管理器)

负责管理与 yrs 服务器的连接：

- **WebSocket 连接**: 通过 `WebsocketProvider` 连接到协作服务器
- **文档意识**: 管理 yrs 文档的 `Awareness` 状态
- **数据同步**: 提供本地数据到远程的同步功能

### 3. GlobalSyncManager (全局同步管理器)

统一管理所有编辑器实例的同步事件：

- **事件通道**: 使用 tokio 的 `mpsc` 通道处理同步事件
- **深度监听**: 监听 yrs 文档的深度变化
- **事件分发**: 将远程变化转换为本地事务并应用

## 同步机制

### 事件类型

```rust
pub enum SyncEventType {
    /// 来自 yrs 的远程变化（通用）
    YrsChange(Vec<DeepEvent>),
    /// 来自本地的事务
    LocalTransaction(Transaction),
    /// 节点添加事件
    NodesAdded(HashMap<String, Any>),
    /// 节点更新事件
    NodesUpdated(HashMap<String, (Any, Any)>), // (old_value, new_value)
    /// 节点删除事件
    NodesRemoved(HashMap<String, Any>),
    /// 节点深度变化事件
    NodesDeepChange(Vec<DeepEvent>),
}
```

### 同步流程

1. **本地变化 → 远程同步**:
   - 本地事务通过 `CollabStateField` 插件自动同步到 yrs
   - 使用 `Utils::apply_transaction_to_yrs` 方法

2. **远程变化 → 本地应用**:
   - yrs 深度监听器捕获远程变化
   - 通过事件通道发送到全局同步管理器
   - 转换为本地事务并应用到编辑器

## 实现状态

### 已完成功能

- ✅ 基础协作编辑器结构
- ✅ WebSocket 连接管理
- ✅ 本地到远程的同步
- ✅ yrs 深度监听器设置
- ✅ 事件通道架构
- ✅ 全局同步管理器

### 待实现功能

- ⏳ **yrs 事件到本地事务的转换**: 这是最关键的部分，需要实现：
  - 解析 yrs `DeepEvent` 事件
  - 提取节点变化信息（添加、删除、更新）
  - 创建对应的 `Step`（AddNodeStep、RemoveNodeStep、AttrStep 等）
  - 组装成 `Transaction` 并应用到本地编辑器

- ⏳ **冲突解决**: 处理并发编辑时的冲突
- ⏳ **性能优化**: 批量处理事件，减少频繁的状态更新

## 关键技术点

### 1. 深度监听

使用 yrs 的 `observe_deep` 方法监听文档的所有变化：

```rust
let _subscription = doc.observe_deep(move |_txn, events| {
    // 处理深度变化事件
    sender.send(SyncEventType::YrsChange(events.to_vec()))
});
```

### 2. 事件驱动架构

通过 tokio 的异步通道实现事件驱动：

```rust
let (sender, receiver) = mpsc::unbounded_channel();
// 发送事件
sender.send(SyncEventType::YrsChange(events))?;
// 接收和处理事件
while let Some(event) = receiver.recv().await {
    handle_sync_event(event).await?;
}
```

### 3. 插件集成

通过 `CollabStateField` 插件自动处理本地事务的同步：

```rust
async fn apply(&self, tr: &Transaction, ...) -> Arc<dyn Resource> {
    Utils::apply_transaction_to_yrs(self.awareness.clone(), tr).await;
    value
}
```

## 使用方式

### 创建协作编辑器

```rust
let options = CollabEditorOptions::new(
    "ws://127.0.0.1:8080/collaboration".to_string(),
    "room_name".to_string()
);
let editor = CollabEditor::create(options).await?;
```

### 执行编辑操作

```rust
let command = InsertChildCommand { data: add_request };
editor.command(Arc::new(command)).await?;
```

## 下一步工作

1. **实现 yrs 事件转换**: 这是最重要的功能，需要参考 `Utils::apply_yrs_to_tree` 的逆向过程
2. **测试协作功能**: 创建多个编辑器实例测试实时同步
3. **性能优化**: 优化事件处理和状态更新的性能
4. **错误处理**: 完善网络断开、重连等异常情况的处理

## 相关文件

- `src/core/collab_editor.rs`: 主要的协作编辑器实现
- `src/core/sync_manager.rs`: 全局同步管理器
- `src/plugins/collab.rs`: 协作插件，处理本地事务同步
- `src/initialize/editor.rs`: 编辑器初始化配置
