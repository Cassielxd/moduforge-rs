import * as Y from 'yjs';
import { Awareness } from 'y-protocols/awareness';
import { 
  CollaborationClient, 
  ConnectionStatus, 
  CollaborationErrorHandler,
  ErrorType,
  CollaborationError
} from '../src/index.ts';

// 为日志条目定义一个接口，增强类型安全
interface UpdateInfo {
  timestamp: string;
  size: number;
  parsed: Record<string, any>;
  source: string;
}

class App {
  // 为类属性添加明确的类型
  private client: CollaborationClient | null;
  private readonly ydoc: Y.Doc;
  private readonly awareness: Awareness;
  private localUpdates: UpdateInfo[];
  private errorHandler: CollaborationErrorHandler;

  constructor() {
    this.client = null;
    this.ydoc = new Y.Doc();
    this.awareness = new Awareness(this.ydoc);
    this.localUpdates = [];
    
    // 初始化错误处理器
    this.errorHandler = new CollaborationErrorHandler({
      autoClose: true,
      autoCloseDelay: 8000,
      showRetryButton: true,
    });
    
    this.initUI();
    this.updateDataDisplay();
  }

  initUI(): void {
    // 使用非空断言 (!) 是一个好习惯，因为我们确定这些元素存在于 HTML 中
    document.getElementById('connect-btn')!.onclick = () => this.connect();
    document.getElementById('disconnect-btn')!.onclick = () => this.disconnect();
    document.getElementById('add-node-btn')!.onclick = () => this.addNode();
    document.getElementById('update-attr-btn')!.onclick = () => this.updateAttribute();
    document.getElementById('clear-data-btn')!.onclick = () => this.clearData();
    
    // 光标功能按钮
    document.getElementById('set-cursor-btn')!.onclick = () => this.setCursor();
    document.getElementById('simulate-typing-btn')!.onclick = () => this.simulateTyping();
    document.getElementById('move-cursor-btn')!.onclick = () => this.moveCursor();
    document.getElementById('show-data-diff-btn')!.onclick = () => this.showDataDifference();
    
    // 添加手动重试按钮
    const retryBtn = document.getElementById('retry-btn');
    if (retryBtn) {
      retryBtn.onclick = () => this.retry();
      retryBtn.style.display = 'none';
    }
    
    const requestSyncBtn = document.getElementById('request-sync-btn');
    if (requestSyncBtn) requestSyncBtn.style.display = 'none';
    const testUpdateBtn = document.getElementById('test-update-btn');
    if (testUpdateBtn) testUpdateBtn.style.display = 'none';
    
    this.ydoc.on('update', (update: Uint8Array, origin: any) => {
        // 这个监听器现在只负责更新"全量"视图
        this.updateDataDisplay();
    });

    // 使用 observe 方法来精确捕获增量变更
    this.ydoc.getMap('nodes').observe(yMapEvent => {
      this.logDetailedChanges(yMapEvent);
    });

    // 监听 awareness 变化（光标位置同步）
    this.awareness.on('change', () => {
      this.updateCursorDisplay();
    });

    this.updateConnectionUI(ConnectionStatus.Disconnected);
    
    // 监听网络状态变化
    this.setupNetworkMonitoring();
  }

  private setupNetworkMonitoring(): void {
    // 监听网络状态变化
    window.addEventListener('online', () => {
      this.log('🌐 网络已恢复');
      if (this.client && this.client.getStatus() === ConnectionStatus.Failed) {
        this.log('🔄 网络恢复后尝试重连');
        setTimeout(() => {
          this.client?.retry();
        }, 1000);
      }
    });

    window.addEventListener('offline', () => {
      this.log('📵 网络已断开');
      if (this.client) {
        this.errorHandler.showNotification(this.errorHandler['createNotification']({
          type: 'warning',
          title: '网络断开',
          message: '检测到网络连接中断，将在网络恢复后自动重连',
          closable: true,
          duration: 5000,
        }));
      }
    });
  }

  logDetailedChanges(event: Y.YMapEvent<any>): void {
    const changes: any[] = [];
    // 遍历所有发生变化的键
    event.keysChanged.forEach(key => {
        const change = event.changes.keys.get(key)!;
        changes.push({
            key,
            action: change.action,
            // oldValue 只有在需要时才保证存在，这里我们记录下来
            oldValue: change.oldValue, 
            // newValue 可以从当前文档状态中直接获取
            newValue: this.ydoc.getMap('nodes').get(key),
        });
    });

    const changeInfo = {
        // 通过 transaction.origin 判断变更来源是本地还是服务器
        source: event.transaction.origin ? 'server' : 'local',
        timestamp: new Date().toISOString(),
        changes,
    };
    
    const changeInfoString = JSON.stringify(changeInfo, null, 2);

    this.log(`🔄 Yjs Detailed Changes Observed`, {
        changeCount: changes.length
    });
    
    const patchesElement = document.getElementById('patches-data')!;
    patchesElement.textContent = changeInfoString;
  }

  updateDataDisplay(): void {
    // snapshot 部分保持不变
    const snapshotElement = document.getElementById('snapshot-data')!;
    const nodes = this.ydoc.getMap('nodes').toJSON();
    snapshotElement.textContent = JSON.stringify(nodes, null, 2);
    
    // patches 部分现在由 logDetailedChanges 直接更新，这里可以留空或设置默认值
    const patchesElement = document.getElementById('patches-data')!;
    if (!patchesElement.textContent?.startsWith('{')) {
        patchesElement.textContent = "No updates yet.";
    }
  }

  connect(): void {
    const roomId = (document.getElementById('room-input') as HTMLInputElement).value.trim();
    if (!roomId) {
      this.errorHandler.showNotification(this.errorHandler['createNotification']({
        type: 'warning',
        title: '输入错误',
        message: '请输入房间ID',
        closable: true,
        duration: 3000,
      }));
      return;
    }

    if (this.client) {
      this.client.destroy();
    }

    // 生成随机用户信息
    const userColors = ['#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4', '#FFEAA7', '#DDA0DD', '#98D8C8'];
    const userName = `用户${Math.floor(Math.random() * 1000)}`;
    const userColor = userColors[Math.floor(Math.random() * userColors.length)];

    this.client = new CollaborationClient({
      url: 'ws://localhost:8080/collaboration',
      room: roomId,
      doc: this.ydoc,
      awareness: this.awareness,
      user: {
        id: `user_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        name: userName,
        color: userColor,
        online: true,
        lastSeen: Date.now(),
      },
      autoReconnect: true,
      maxReconnectAttempts: 5,
      connectionTimeout: 10000,
      cursor: {
        enabled: true,
        debounceDelay: 100,
        typingTimeout: 3000,
        showOtherCursors: true,
      },
    });

    // 设置事件监听器
    this.setupClientListeners();
    
    this.client.connect();
    this.log(`🔗 尝试连接到房间: ${roomId}`);
  }

  private setupClientListeners(): void {
    if (!this.client) return;

    // 连接状态变化
    this.client.on('status', (status: ConnectionStatus) => {
      this.log(`📡 连接状态: ${status}`);
      this.updateConnectionUI(status);
      
      if (status === ConnectionStatus.Connected) {
        this.errorHandler.showConnectionSuccess();
      }
    });

    // 同步状态变化
    this.client.on('synced', (synced: boolean) => {
      this.log(`🔄 同步状态: ${synced ? '已同步' : '未同步'}`);
      this.updateYjsStatus(synced);
    });

    // 错误处理
    this.client.on('error', (error: CollaborationError) => {
      this.log(`❌ 协作错误: ${error.type} - ${error.message}`);
      
      this.errorHandler.showError(error, () => {
        this.retry();
      });
      
      // 根据错误类型进行特殊处理
      this.handleSpecificError(error);
    });

    // 重连尝试
    this.client.on('reconnectAttempt', (attempt: number, maxAttempts: number) => {
      this.log(`🔄 重连尝试: ${attempt}/${maxAttempts}`);
      this.errorHandler.showReconnectStatus(attempt, maxAttempts);
    });

    // 连接超时
    this.client.on('connectionTimeout', () => {
      this.log(`⏰ 连接超时`);
    });

    // 光标和用户事件监听
    this.client.on('awarenessChange', (states) => {
      this.log(`👥 Awareness 状态变化: ${states.size} 个客户端`);
      this.updateCursorDisplay();
    });

    this.client.on('userJoin', (user) => {
      this.log(`👋 用户加入: ${user.name} (${user.id})`);
    });

    this.client.on('userLeave', (userId) => {
      this.log(`👋 用户离开: ${userId}`);
    });

    this.client.on('cursorMove', (userId, cursor) => {
      this.log(`🖱️ 用户 ${userId} 移动光标到: 节点 ${cursor.anchor.nodeId}, 偏移 ${cursor.anchor.offset}`);
    });
  }

  private handleSpecificError(error: CollaborationError): void {
    switch (error.type) {
      case ErrorType.ROOM_NOT_FOUND:
        // 房间不存在时，显示创建房间的建议
        setTimeout(() => {
          const createRoomNotification = this.errorHandler['createNotification']({
            type: 'info',
            title: '房间不存在',
            message: '是否需要了解如何创建房间？',
            closable: true,
            duration: 0,
            onAction: () => {
              this.showRoomCreationHelp();
            },
            actionText: '查看帮助',
          });
          this.errorHandler.showNotification(createRoomNotification);
        }, 1000);
        break;

      case ErrorType.CONNECTION_FAILED:
        // 连接失败时，显示服务器状态检查建议
        if (error.details?.attempts >= 3) {
          setTimeout(() => {
            const serverCheckNotification = this.errorHandler['createNotification']({
              type: 'warning',
              title: '多次连接失败',
              message: '可能是服务器离线或网络问题',
              closable: true,
              duration: 0,
              onAction: () => {
                this.checkServerStatus();
              },
              actionText: '检查服务器',
            });
            this.errorHandler.showNotification(serverCheckNotification);
          }, 2000);
        }
        break;

      case ErrorType.SYNC_FAILED:
        // 同步失败时，建议刷新页面
        setTimeout(() => {
          const refreshNotification = this.errorHandler['createNotification']({
            type: 'warning',
            title: '数据同步异常',
            message: '建议刷新页面以获取最新数据',
            closable: true,
            duration: 0,
            onAction: () => {
              window.location.reload();
            },
            actionText: '刷新页面',
          });
          this.errorHandler.showNotification(refreshNotification);
        }, 1500);
        break;
    }
  }

  private showRoomCreationHelp(): void {
    const helpContent = `
      <div style="font-size: 13px; line-height: 1.6;">
        <p><strong>如何创建房间？</strong></p>
        <ol style="margin: 8px 0; padding-left: 20px;">
          <li>联系服务器管理员</li>
          <li>确保房间ID正确</li>
          <li>检查是否有访问权限</li>
        </ol>
        <p style="color: #718096;">
          房间需要在服务器端预先创建和初始化才能使用。
        </p>
      </div>
    `;
    
    const helpNotification = this.errorHandler['createNotification']({
      type: 'info',
      title: '房间创建帮助',
      message: '',
      closable: true,
      duration: 0,
    });
    
    const messageEl = helpNotification.querySelector('.collaboration-notification-message')!;
    messageEl.innerHTML = helpContent;
    
    this.errorHandler.showNotification(helpNotification);
  }

  private async checkServerStatus(): Promise<void> {
    try {
      // 使用正确的健康检查接口
      const response = await fetch('http://localhost:8080/health', {
        method: 'GET',
        headers: {
          'Accept': 'application/json',
        },
        signal: AbortSignal.timeout(5000),
      });
      
      if (response.ok) {
        const healthData = await response.json();
        const statusNotification = this.errorHandler['createNotification']({
          type: 'success',
          title: '服务器状态正常',
          message: `服务器响应正常，活跃房间: ${healthData.statistics?.active_rooms || 0} 个`,
          closable: true,
          duration: 5000,
        });
        this.errorHandler.showNotification(statusNotification);
      } else {
        throw new Error(`服务器响应异常: ${response.status}`);
      }
    } catch (error) {
      let errorMessage = '无法连接到服务器';
      
      if (error instanceof Error) {
        if (error.name === 'AbortError') {
          errorMessage = '健康检查请求超时';
        } else if (error.message.includes('fetch')) {
          errorMessage = '网络连接失败，请检查服务器是否运行';
        }
      }
      
      const errorNotification = this.errorHandler['createNotification']({
        type: 'error',
        title: '服务器状态检查失败',
        message: errorMessage,
        closable: true,
        duration: 8000,
      });
      this.errorHandler.showNotification(errorNotification);
    }
  }

  disconnect(): void {
    if (this.client) {
      this.client.disconnect();
      this.client = null;
    }
    this.localUpdates = [];
    this.errorHandler.clearAll();
    this.updateConnectionUI(ConnectionStatus.Disconnected);
    this.log(`🔌 已断开连接`);
  }

  retry(): void {
    if (this.client) {
      this.log(`🔄 手动重试连接`);
      this.client.retry();
    } else {
      this.connect();
    }
  }

  addNode(): void {
    if (!this.client || !this.client.doc) return;
    const nodesMap = this.client.doc.getMap('nodes');
    const nodeId = 'node_' + Date.now();
    const newNodeMap = new Y.Map();
    newNodeMap.set('type', 'text-client');
    const attrsMap = new Y.Map();
    attrsMap.set('createdAt', new Date().toISOString());
    newNodeMap.set('attrs', attrsMap);
    nodesMap.set(nodeId, newNodeMap);
    this.log(`➕ 添加节点: ${nodeId}`);
  }

  updateAttribute(): void {
    if (!this.client || !this.client.doc) return;
    const nodesMap = this.client.doc.getMap('nodes');
    const nodeKeys = Array.from(nodesMap.keys());
    if (nodeKeys.length === 0) return;
    const lastNodeKey = nodeKeys[nodeKeys.length - 1];
    const nodeToUpdate = nodesMap.get(lastNodeKey) as Y.Map<any>;
    if (nodeToUpdate) {
      const attrsMap = nodeToUpdate.get('attrs') as Y.Map<any>;
      attrsMap.set('updatedAt', new Date().toISOString());
      this.log(`✏️ 更新节点属性: ${lastNodeKey}`);
    }
  }

  clearData(): void {
    if (!this.client || !this.client.doc) return;
    this.client.doc.getMap('nodes').clear();
    this.log(`🗑️ 清空数据`);
  }

  // 光标功能方法
  setCursor(): void {
    if (!this.client || this.client.getStatus() !== ConnectionStatus.Connected) {
      this.log('❌ 请先连接到房间');
      return;
    }

    // 模拟设置光标位置
    const randomNodeId = `node_${Math.floor(Math.random() * 10)}`;
    const randomOffset = Math.floor(Math.random() * 100);
    
    const cursorPosition = {
      anchor: {
        nodeId: randomNodeId,
        offset: randomOffset,
      },
      timestamp: Date.now(),
    };

    // 通过 awareness 设置光标位置
    this.awareness.setLocalStateField('cursor', cursorPosition);
    this.log(`🖱️ 设置光标位置: 节点 ${randomNodeId}, 偏移 ${randomOffset}`);
    
    this.updateCursorDisplay();
  }

  simulateTyping(): void {
    if (!this.client || this.client.getStatus() !== ConnectionStatus.Connected) {
      this.log('❌ 请先连接到房间');
      return;
    }

    // 模拟开始输入
    this.awareness.setLocalStateField('typing', true);
    this.awareness.setLocalStateField('typingTimestamp', Date.now());
    this.log('⌨️ 开始输入...');
    
    // 3秒后停止输入
    setTimeout(() => {
      this.awareness.setLocalStateField('typing', false);
      this.log('⌨️ 停止输入');
      this.updateCursorDisplay();
    }, 3000);
    
    this.updateCursorDisplay();
  }

  moveCursor(): void {
    if (!this.client || this.client.getStatus() !== ConnectionStatus.Connected) {
      this.log('❌ 请先连接到房间');
      return;
    }

    // 模拟光标移动
    const randomNodeId = `node_${Math.floor(Math.random() * 20)}`;
    const randomOffset = Math.floor(Math.random() * 200);
    
    const cursorPosition = {
      anchor: {
        nodeId: randomNodeId,
        offset: randomOffset,
      },
      timestamp: Date.now(),
    };

    this.awareness.setLocalStateField('cursor', cursorPosition);
    this.log(`🖱️ 移动光标到: 节点 ${randomNodeId}, 偏移 ${randomOffset}`);
    
    this.updateCursorDisplay();
  }

  updateCursorDisplay(): void {
    const cursorsData: any = {};
    
    // 获取所有用户的状态（仅来自 Awareness，不影响 Doc）
    this.awareness.getStates().forEach((state, clientId) => {
      if (state) {
        cursorsData[clientId] = {
          user: state.user || { name: `用户${clientId}`, color: '#666' },
          cursor: state.cursor,
          typing: state.typing,
          typingTimestamp: state.typingTimestamp,
          online: state.online,
        };
      }
    });

    // 更新光标显示区域
    const cursorsElement = document.getElementById('cursors-data');
    if (cursorsElement) {
      const displayData = {
        awareness_states: cursorsData,
        total_users: Object.keys(cursorsData).length,
        note: "⚠️ 这些数据仅存在于 Awareness 中，不会保存到 Doc"
      };
      cursorsElement.textContent = JSON.stringify(displayData, null, 2);
    }
  }

  showDataDifference(): void {
    if (!this.client || this.client.getStatus() !== ConnectionStatus.Connected) {
      this.log('❌ 请先连接到房间');
      return;
    }

    // 添加一些测试数据到 Doc
    this.addNode();
    this.setCursor();
    this.simulateTyping();

    // 显示对比信息
    setTimeout(() => {
      const docData = this.ydoc.getMap('nodes').toJSON();
      const awarenessData: any = {};
      
      this.awareness.getStates().forEach((state, clientId) => {
        if (state) {
          awarenessData[clientId] = state;
        }
      });

      const comparison = {
        "🗄️ Yjs Doc 数据 (持久化)": {
          "说明": "这些数据会被保存到服务器，刷新页面仍然存在",
          "数据": docData,
          "特点": ["持久化", "版本控制", "CRDT 同步", "可撤销/重做"]
        },
        "👥 Awareness 数据 (临时)": {
          "说明": "这些数据仅在用户在线时存在，用户离线立即消失",
          "数据": awarenessData,
          "特点": ["临时存储", "实时同步", "用户状态", "光标位置"]
        },
        "🔄 总结": {
          "Doc 节点数": Object.keys(docData).length,
          "在线用户数": Object.keys(awarenessData).length,
          "关键区别": "Doc 数据永久保存，Awareness 数据临时存在"
        }
      };

      console.log("📊 数据存储对比:", comparison);
      this.log("📊 数据存储对比已输出到控制台，打开开发者工具查看详细信息");
      
      // 也可以在页面上显示
      alert(`📊 数据存储对比：
      
📄 Yjs Doc (持久化): ${Object.keys(docData).length} 个节点
🖱️ Awareness (临时): ${Object.keys(awarenessData).length} 个用户在线

详细信息请查看浏览器控制台。`);
    }, 500);
  }

  updateConnectionUI(status: ConnectionStatus): void {
    const isConnected = status === ConnectionStatus.Connected;
    const isConnecting = status === ConnectionStatus.Connecting || status === ConnectionStatus.Reconnecting;
    const isFailed = status === ConnectionStatus.Failed;

    (document.getElementById('connect-btn') as HTMLButtonElement).disabled = isConnected || isConnecting;
    (document.getElementById('disconnect-btn') as HTMLButtonElement).disabled = !isConnected && !isConnecting;
    (document.getElementById('add-node-btn') as HTMLButtonElement).disabled = !isConnected;
    (document.getElementById('update-attr-btn') as HTMLButtonElement).disabled = !isConnected;
    (document.getElementById('clear-data-btn') as HTMLButtonElement).disabled = !isConnected;
    (document.getElementById('room-input') as HTMLInputElement).disabled = isConnected || isConnecting;
    
    // 光标功能按钮状态
    (document.getElementById('set-cursor-btn') as HTMLButtonElement).disabled = !isConnected;
    (document.getElementById('simulate-typing-btn') as HTMLButtonElement).disabled = !isConnected;
    (document.getElementById('move-cursor-btn') as HTMLButtonElement).disabled = !isConnected;
    (document.getElementById('show-data-diff-btn') as HTMLButtonElement).disabled = !isConnected;
    
    // 显示/隐藏重试按钮
    const retryBtn = document.getElementById('retry-btn') as HTMLButtonElement;
    if (retryBtn) {
      retryBtn.style.display = isFailed ? 'inline-block' : 'none';
      retryBtn.disabled = isConnecting;
    }
    
    const indicator = document.getElementById('ws-status')!;
    indicator.className = 'status-indicator';
    
    switch (status) {
      case ConnectionStatus.Connected:
        indicator.classList.add('connected');
        indicator.title = '已连接';
        break;
      case ConnectionStatus.Connecting:
        indicator.classList.add('connecting');
        indicator.title = '正在连接...';
        break;
      case ConnectionStatus.Reconnecting:
        indicator.classList.add('reconnecting');
        indicator.title = '正在重连...';
        break;
      case ConnectionStatus.Failed:
        indicator.classList.add('failed');
        indicator.title = '连接失败';
        break;
      default:
        indicator.classList.add('disconnected');
        indicator.title = '未连接';
    }
  }

  updateYjsStatus(synced: boolean): void {
    const indicator = document.getElementById('yjs-status')!;
    if (synced) {
      indicator.classList.add('connected');
      indicator.title = '已同步';
    } else {
      indicator.classList.remove('connected');
      indicator.title = '未同步';
    }
  }

  log(message: string, data?: any): void {
    const timestamp = new Date().toLocaleTimeString();
    console.log(`[${timestamp}] ${message}`, data || '');
    
    // 也可以显示在页面上
    const logElement = document.getElementById('log-output');
    if (logElement) {
      const logEntry = document.createElement('div');
      logEntry.style.cssText = 'font-size: 12px; color: #666; margin-bottom: 4px;';
      logEntry.textContent = `[${timestamp}] ${message}`;
      logElement.appendChild(logEntry);
      logElement.scrollTop = logElement.scrollHeight;
    }
  }
}

// 扩展窗口对象，使其在控制台中可用
declare global {
  interface Window { app: App; }
}

// 页面加载完成后启动应用
document.addEventListener('DOMContentLoaded', () => {
  window.app = new App();
  console.log('🚀 协作客户端示例已启动 - 使用 window.app 访问应用实例');
});