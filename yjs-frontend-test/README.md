# ModuForge Yjs 前端测试客户端

这是一个用于测试 ModuForge-RS 后端与前端 Yjs 协作同步功能的测试客户端。

## 功能特性

- 🔌 **WebSocket 连接**: 连接到 Rust 后端 WebSocket 服务器
- 📄 **实时同步**: 通过 Yjs 实现多客户端实时协作
- 📝 **增量更新**: 显示增量变更和完整数据快照
- 🏠 **房间管理**: 支持多房间隔离
- 📋 **事件日志**: 实时显示连接状态和同步事件
- 🔄 **数据操作**: 测试节点添加、属性更新等操作

## 快速开始

### 1. 安装依赖

```bash
npm install
```

### 2. 启动开发服务器

```bash
npm run dev
```

服务器将在 http://localhost:3000 启动。

### 3. 启动 Rust 后端

确保 ModuForge-RS 的 transmission 服务已启动：

```bash
cd ../crates/transmission
cargo run --bin main
```

后端将在 ws://localhost:8080 启动 WebSocket 服务。

### 4. 测试协作

1. 在浏览器中打开 http://localhost:3000
2. 输入房间ID（如 "demo-room"）
3. 点击"连接房间"
4. 在不同标签页或浏览器中重复步骤，测试多客户端同步

## 界面说明

### 状态指示器
- **WebSocket**: 显示与后端的连接状态
- **Yjs 同步**: 显示 Yjs 文档同步状态
- **客户端数**: 当前房间的客户端数量
- **版本**: 当前数据版本号

### 控制面板
- **连接房间**: 连接到指定房间ID
- **断开连接**: 断开当前连接
- **添加节点**: 向 Yjs 文档添加测试节点
- **更新属性**: 更新 Yjs 文档属性
- **清空数据**: 清除本地 Yjs 文档数据
- **请求重新同步**: 向服务器请求完整同步

### 数据显示
- **当前数据快照**: 显示服务器端的完整数据快照
- **增量更新 (Patches)**: 显示增量变更记录
- **事件日志**: 显示连接和同步事件日志

## 技术架构

### 前端技术栈
- **Yjs**: CRDT 协作编辑库
- **WebSocket**: 实时通信
- **Vite**: 构建工具
- **原生 JavaScript**: 简单易懂的实现

### 消息协议

#### 客户端 → 服务器
```json
// 加入房间
{ "JoinRoom": { "room_id": "demo-room" } }

// 离开房间
{ "LeaveRoom": { "room_id": "demo-room" } }

// Yjs 更新
{ "YrsUpdate": { "room_id": "demo-room", "update": [1,2,3,...] } }

// 请求重新同步
{ "RequestResync": { "room_id": "demo-room", "from_version": null } }

// 心跳
{ "Ping": {} }
```

#### 服务器 → 客户端
```json
// 房间快照
{ "RoomSnapshot": { "tree": {...}, "version": 1 } }

// 房间增量更新
{ "RoomPatches": { "patches": [...], "version": 2 } }

// 同步完成
{ "SyncComplete": { "version": 2 } }

// 错误消息
{ "Error": { "message": "错误信息" } }

// 心跳响应
{ "Pong": {} }
```

### Yjs 数据结构

```javascript
// Y.Array: 存储节点列表
ydoc.getArray('nodes')

// Y.Map: 存储属性键值对
ydoc.getMap('attributes')
```

## 调试功能

- 应用实例通过 `window.app` 暴露到全局作用域
- 浏览器控制台显示详细的调试信息
- 实时事件日志显示连接和同步状态

## 构建部署

```bash
# 构建生产版本
npm run build

# 预览生产构建
npm run preview
```

## 注意事项

1. 确保 Rust 后端服务已启动
2. 检查 WebSocket 连接地址 (默认 ws://localhost:8080)
3. 多客户端测试时使用相同的房间ID
4. 观察事件日志了解同步状态

## 故障排除

### 连接失败
- 检查 Rust 后端是否启动
- 确认端口 8080 未被占用
- 检查防火墙设置

### 同步异常
- 查看浏览器控制台错误信息
- 检查事件日志中的错误消息
- 尝试断开重连或重新同步

### 性能问题
- 避免频繁的大量数据操作
- 监控内存使用情况
- 检查网络延迟 