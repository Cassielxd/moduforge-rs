# ModuForge Collaboration Client

现代化的协作客户端，支持实时数据同步和完整的错误处理机制。

## 🌟 特性

- **实时协作** - 基于 Yjs CRDT 的无冲突数据同步
- **智能错误处理** - 详细的错误分类和用户友好的提示
- **自动重连机制** - 网络断开时的智能重连策略
- **TypeScript 支持** - 完整的类型定义
- **可定制 UI** - 灵活的错误提醒界面

## 📦 安装

```bash
npm install collaboration-client
```

## 🚀 快速开始

### 基本使用

```typescript
import * as Y from 'yjs';
import { Awareness } from 'y-protocols/awareness';
import { 
  CollaborationClient, 
  CollaborationErrorHandler,
  ConnectionStatus 
} from 'collaboration-client';

// 创建 Yjs 文档和 Awareness
const doc = new Y.Doc();
const awareness = new Awareness(doc);

// 创建错误处理器
const errorHandler = new CollaborationErrorHandler({
  autoClose: true,
  autoCloseDelay: 5000,
  showRetryButton: true,
});

// 创建协作客户端
const client = new CollaborationClient({
  url: 'ws://localhost:8080/collaboration',
  room: 'my-room',
  doc,
  awareness,
  autoReconnect: true,
  maxReconnectAttempts: 5,
  connectionTimeout: 10000,
});

// 监听事件
client.on('status', (status) => {
  console.log('连接状态:', status);
});

client.on('error', (error) => {
  console.error('协作错误:', error);
  errorHandler.showError(error, () => client.retry());
});

client.on('reconnectAttempt', (attempt, maxAttempts) => {
  errorHandler.showReconnectStatus(attempt, maxAttempts);
});

// 连接
client.connect();
```

### 高级错误处理

```typescript
import { ErrorType, CollaborationError } from 'collaboration-client';

// 自定义错误处理
client.on('error', (error: CollaborationError) => {
  switch (error.type) {
    case ErrorType.ROOM_NOT_FOUND:
      // 房间不存在 - 显示创建房间指引
      showRoomCreationHelp(error.details?.room_id);
      break;
      
    case ErrorType.CONNECTION_TIMEOUT:
      // 连接超时 - 检查网络状态
      checkNetworkStatus();
      break;
      
    case ErrorType.SYNC_FAILED:
      // 同步失败 - 建议刷新页面
      if (confirm('数据同步失败，是否刷新页面？')) {
        window.location.reload();
      }
      break;
      
    default:
      // 通用错误处理
      errorHandler.showError(error, () => client.retry());
  }
});

// 连接状态处理
client.on('status', (status: ConnectionStatus) => {
  switch (status) {
    case ConnectionStatus.Connected:
      errorHandler.showConnectionSuccess();
      break;
      
    case ConnectionStatus.Failed:
      // 显示重试按钮
      showRetryButton();
      break;
      
    case ConnectionStatus.Reconnecting:
      // 显示重连状态
      showReconnectingIndicator();
      break;
  }
});
```

## 🎨 错误处理器配置

### CollaborationErrorHandler 选项

```typescript
const errorHandler = new CollaborationErrorHandler({
  // 容器选择器或元素
  container: '#error-container', // 或 document.getElementById('container')
  
  // 自动关闭配置
  autoClose: true,
  autoCloseDelay: 5000,
  
  // 显示重试按钮
  showRetryButton: true,
  
  // 自定义样式类
  customClass: 'my-custom-error',
  
  // 使用原生 alert（调试模式）
  useNativeAlert: false,
});
```

### 错误类型

```typescript
enum ErrorType {
  // 连接相关
  CONNECTION_FAILED = 'CONNECTION_FAILED',
  CONNECTION_TIMEOUT = 'CONNECTION_TIMEOUT',
  WEBSOCKET_ERROR = 'WEBSOCKET_ERROR',
  
  // 房间相关
  ROOM_NOT_FOUND = 'ROOM_NOT_FOUND',
  ROOM_ACCESS_DENIED = 'ROOM_ACCESS_DENIED',
  ROOM_FULL = 'ROOM_FULL',
  
  // 同步相关
  SYNC_FAILED = 'SYNC_FAILED',
  DATA_CORRUPTION = 'DATA_CORRUPTION',
  
  // 网络相关
  NETWORK_ERROR = 'NETWORK_ERROR',
  SERVER_ERROR = 'SERVER_ERROR',
  
  // 未知错误
  UNKNOWN_ERROR = 'UNKNOWN_ERROR',
}
```

### 错误对象结构

```typescript
interface CollaborationError {
  type: ErrorType;
  message: string;
  details?: {
    code?: number;
    room_id?: string;
    server_message?: string;
    timestamp?: string;
    [key: string]: any;
  };
  recoverable: boolean;
  suggestedAction?: string;
  originalError?: Error | Event | any;
}
```

## 🔧 API 参考

### CollaborationClient

#### 构造函数选项

```typescript
interface CollaborationClientOptions {
  url: string;                    // WebSocket 服务器地址
  room: string;                   // 房间ID
  doc: Y.Doc;                     // Yjs 文档
  awareness: Awareness;           // Awareness 实例
  autoReconnect?: boolean;        // 自动重连（默认: true）
  reconnectDelay?: number;        // 重连延迟（默认: 2000ms）
  maxReconnectAttempts?: number;  // 最大重连次数（默认: 5）
  connectionTimeout?: number;     // 连接超时（默认: 10000ms）
}
```

#### 方法

- `connect()` - 连接到服务器
- `disconnect()` - 断开连接
- `retry()` - 手动重试连接
- `destroy()` - 销毁客户端
- `getStatus()` - 获取当前连接状态
- `getReconnectInfo()` - 获取重连信息

#### 事件

- `status` - 连接状态变化
- `synced` - 同步状态变化
- `error` - 错误发生
- `reconnectAttempt` - 重连尝试
- `connectionTimeout` - 连接超时

### CollaborationErrorHandler

#### 方法

- `showError(error, onRetry?)` - 显示错误信息
- `showReconnectStatus(attempt, maxAttempts)` - 显示重连状态
- `showConnectionSuccess()` - 显示连接成功
- `showNotification(notification)` - 显示自定义通知
- `clearAll()` - 清除所有通知
- `destroy()` - 销毁错误处理器

## 🎯 最佳实践

### 1. 错误分类处理

```typescript
// 根据错误类型进行不同处理
const handleCollaborationError = (error: CollaborationError) => {
  if (!error.recoverable) {
    // 不可恢复的错误 - 提供明确指引
    switch (error.type) {
      case ErrorType.ROOM_NOT_FOUND:
        showRoomCreationGuide();
        break;
      case ErrorType.ROOM_ACCESS_DENIED:
        showPermissionHelp();
        break;
    }
  } else {
    // 可恢复的错误 - 提供重试选项
    errorHandler.showError(error, () => client.retry());
  }
};
```

### 2. 网络状态监听

```typescript
// 监听网络状态变化
window.addEventListener('online', () => {
  if (client.getStatus() === ConnectionStatus.Failed) {
    client.retry();
  }
});

window.addEventListener('offline', () => {
  errorHandler.showNotification(errorHandler['createNotification']({
    type: 'warning',
    title: '网络连接断开',
    message: '请检查网络连接',
    closable: true,
  }));
});
```

### 3. 房间状态预检查

```typescript
// 连接前检查房间状态
const connectToRoom = async (roomId: string) => {
  try {
    // 使用后端房间检查接口
    const response = await fetch(`http://localhost:8080/collaboration/room-check/${roomId}`);
    
    if (response.ok) {
      const data = await response.json();
      if (data.exists) {
        // 房间存在，开始连接
        client.connect();
      } else {
        throw new Error('房间不存在');
      }
    } else {
      throw new Error(`房间检查失败: ${response.status}`);
    }
  } catch (error) {
    errorHandler.showError({
      type: ErrorType.ROOM_NOT_FOUND,
      message: `房间 "${roomId}" 不存在`,
      recoverable: false,
      suggestedAction: '请检查房间ID或联系管理员',
    });
  }
};

// 获取详细房间状态
const getRoomStatus = async (roomId: string) => {
  try {
    const response = await fetch(`http://localhost:8080/collaboration/rooms/${roomId}/status`);
    
    if (response.ok) {
      const status = await response.json();
      console.log('房间状态:', status);
      return status;
    } else {
      console.log('房间不存在或获取失败');
      return null;
    }
  } catch (error) {
    console.error('获取房间状态失败:', error);
    return null;
  }
};
```

### 4. 数据完整性检查

```typescript
// 定期检查数据同步状态
const checkSyncHealth = () => {
  if (!client.doc || client.getStatus() !== ConnectionStatus.Connected) {
    return;
  }
  
  // 检查文档是否有未同步的更改
  const hasLocalChanges = /* 检查逻辑 */;
  if (hasLocalChanges) {
    errorHandler.showNotification(errorHandler['createNotification']({
      type: 'warning',
      title: '数据同步延迟',
      message: '存在未同步的更改',
      actionText: '强制同步',
      onAction: () => client.retry(),
    }));
  }
};

// 每30秒检查一次
setInterval(checkSyncHealth, 30000);
```

## 🔍 调试

### 启用详细日志

```typescript
// 启用详细的错误日志
client.on('error', (error) => {
  console.group('Collaboration Error');
  console.error('Type:', error.type);
  console.error('Message:', error.message);
  console.error('Details:', error.details);
  console.error('Recoverable:', error.recoverable);
  console.error('Original:', error.originalError);
  console.groupEnd();
});
```

### 性能监控

```typescript
// 监控连接性能
let connectionStartTime: number;

client.on('status', (status) => {
  switch (status) {
    case ConnectionStatus.Connecting:
      connectionStartTime = Date.now();
      break;
    case ConnectionStatus.Connected:
      const connectTime = Date.now() - connectionStartTime;
      console.log(`连接耗时: ${connectTime}ms`);
      break;
  }
});
```

## 🐛 故障排除

### 常见问题

#### 1. 房间不存在错误

**问题**: 收到 `ROOM_NOT_FOUND` 错误
**解决**: 
- 确保房间在服务器端已创建
- 检查房间ID拼写
- 联系服务器管理员

#### 2. 连接超时

**问题**: 连接总是超时
**解决**:
- 检查服务器是否运行
- 确认 WebSocket 端口开放
- 增加 `connectionTimeout` 值

#### 3. 频繁重连

**问题**: 连接不稳定，频繁重连
**解决**:
- 检查网络稳定性
- 调整 `reconnectDelay` 和 `maxReconnectAttempts`
- 监控服务器负载

#### 4. 数据同步失败

**问题**: 数据不同步
**解决**:
- 检查 Yjs 文档结构
- 确认 Awareness 配置正确
- 重新连接或刷新页面

### 错误码对照表

| 错误码 | 含义 | 建议操作 |
|--------|------|----------|
| 1006 | 网络异常断开 | 检查网络连接 |
| 4000 | 房间不存在 | 检查房间ID或联系管理员 |
| 4001 | 房间已满 | 等待或联系管理员扩容 |
| 4003 | 访问被拒绝 | 检查权限或联系管理员 |

## 📄 许可证

ISC License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📞 支持

如有问题，请通过以下方式联系：

- GitHub Issues: [创建 Issue](https://github.com/your-repo/issues)
- 邮箱: support@moduforge.com
- 文档: [查看完整文档](https://docs.moduforge.com) 

## 🌐 后端 HTTP 接口

协作服务器提供了以下 HTTP 接口用于房间管理和状态检查：

### 房间检查接口

**GET** `/collaboration/room-check/{room_id}`

检查指定房间是否存在。

**响应示例:**
```json
// 房间存在
{
  "exists": true,
  "room_id": "my-room",
  "status": "available",
  "info": {
    "room_id": "my-room",
    "status": "Initialized",
    "node_count": 5,
    "client_count": 2,
    "last_activity": "2024-01-15T10:30:00Z"
  }
}

// 房间不存在
{
  "exists": false,
  "room_id": "my-room",
  "status": "not_found",
  "message": "房间 'my-room' 不存在"
}
```

### 健康检查接口

**GET** `/health`

获取服务器健康状态和统计信息。

**响应示例:**
```json
{
  "status": "healthy",
  "timestamp": 1642248600,
  "service": "ModuForge Collaboration Server",
  "version": "0.1.0",
  "statistics": {
    "active_rooms": 3,
    "total_rooms": 5,
    "rooms": ["room1", "room2", "room3"]
  }
}
```

### 房间状态接口

**GET** `/collaboration/rooms/{room_id}/status`

获取指定房间的详细状态信息。

**响应示例:**
```json
// 房间存在
{
  "room_id": "my-room",
  "status": "Initialized",
  "node_count": 10,
  "client_count": 3,
  "last_activity": 1642248600,
  "available": true
}

// 房间不存在
{
  "room_id": "my-room",
  "status": "not_found",
  "available": false,
  "message": "房间 'my-room' 不存在"
}
```

### CORS 支持

所有 HTTP 接口都支持跨域请求，允许前端应用从不同域名访问。 