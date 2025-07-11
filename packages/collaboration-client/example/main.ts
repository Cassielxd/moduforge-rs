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

  /**
   * 自定义 JSON 序列化函数，处理 BigInt 类型
   * @param obj 要序列化的对象
   * @param space 缩进空格数
   * @param bigIntAsNumber 是否将 BigInt 转换为普通数字（可能丢失精度）
   */
  private safeStringify(obj: any, space?: number, bigIntAsNumber: boolean = false): string {
    try {
      return JSON.stringify(obj, (key, value) => {
        if (typeof value === 'bigint') {
          if (bigIntAsNumber) {
            // 转换为普通数字（可能丢失精度）
            const num = Number(value);
            if (Number.isSafeInteger(num)) {
              return num;
            } else {
              // 如果超出安全整数范围，保持 BigInt 格式
              return value.toString() + 'n';
            }
          } else {
            // 保持 BigInt 格式
            return value.toString() + 'n';
          }
        }
        // 处理字符串形式的 BigInt（来自 Yjs）
        if (typeof value === 'string' && value.endsWith('n') && /^-?\d+n$/.test(value)) {
          if (bigIntAsNumber) {
            const num = Number(value.slice(0, -1));
            if (Number.isSafeInteger(num)) {
              return num;
            }
          }
          // 保持原样或转换为 BigInt 对象
          return value;
        }
        return value;
      }, space);
    } catch (error) {
      console.error('JSON 序列化失败:', error);
      return JSON.stringify({
        error: '序列化失败',
        message: error instanceof Error ? error.message : '未知错误',
        originalData: String(obj)
      }, null, 2);
    }
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
    
    // 深度更新测试按钮
    const deepUpdateBtn = document.getElementById('deep-update-btn');
    if (deepUpdateBtn) {
      deepUpdateBtn.onclick = () => this.testDeepUpdates();
    }
    
    // 刷新快照按钮
    const refreshSnapshotBtn = document.getElementById('refresh-snapshot-btn');
    if (refreshSnapshotBtn) {
      refreshSnapshotBtn.onclick = () => this.refreshSnapshot();
    }
    
    // 请求重新同步按钮
    const requestSyncBtn = document.getElementById('request-sync-btn');
    if (requestSyncBtn) {
      requestSyncBtn.onclick = () => this.requestSync();
    }
    
    // 测试本地更新按钮
    const testUpdateBtn = document.getElementById('test-update-btn');
    if (testUpdateBtn) {
      testUpdateBtn.onclick = () => this.testLocalUpdate();
    }
    
    // BigInt 测试按钮
    const bigIntTestBtn = document.getElementById('bigint-test-btn');
    if (bigIntTestBtn) {
      bigIntTestBtn.onclick = () => this.testBigIntDisplay();
    }
    
    // BigInt 数学运算按钮
    const bigIntMathBtn = document.getElementById('bigint-math-btn');
    if (bigIntMathBtn) {
      bigIntMathBtn.onclick = () => this.testBigIntMath();
    }
    
    // BigInt 比较运算按钮
    const bigIntCompareBtn = document.getElementById('bigint-compare-btn');
    if (bigIntCompareBtn) {
      bigIntCompareBtn.onclick = () => this.testBigIntComparison();
    }
    
    // BigInt 计算器按钮
    const bigIntCalculatorBtn = document.getElementById('bigint-calculator-btn');
    if (bigIntCalculatorBtn) {
      bigIntCalculatorBtn.onclick = () => this.testBigIntCalculator();
    }
    
    // 添加手动重试按钮
    const retryBtn = document.getElementById('retry-btn');
    if (retryBtn) {
      retryBtn.onclick = () => this.retry();
      retryBtn.style.display = 'none';
    }
    
    this.ydoc.on('update', (update: Uint8Array, origin: any) => {
        // 这个监听器现在只负责更新"全量"视图
        this.updateDataDisplay();
    });

    // 使用 observeDeep 方法进行深度更新监听，捕获嵌套对象的变化
    this.ydoc.getMap('nodes').observeDeep((events: Y.YEvent<any>[]) => {
      this.logDeepChanges(events);
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
    
    const changeInfoString = this.safeStringify(changeInfo, 2);

    this.log(`🔄 Yjs Detailed Changes Observed`, {
        changeCount: changes.length
    });
    
    const patchesElement = document.getElementById('patches-data')!;
    patchesElement.textContent = changeInfoString;
  }

  logDeepChanges(events: Y.YEvent<any>[]): void {
    if (events.length === 0) return;

    const patchesElement = document.getElementById('patches-data')!;
    const changes: any[] = [];

    events.forEach(event => {
      const path = this.getEventPath(event);
      let changeInfo: any = {
        path,
        timestamp: new Date().toISOString(),
        source: event.transaction.origin ? 'server' : 'local',
      };

      if (event instanceof Y.YMapEvent) {
        changeInfo.type = 'YMapEvent';
        changeInfo.changes = Array.from(event.changes.keys.entries()).map(([key, change]) => ({
          key,
          action: change.action,
          oldValue: change.oldValue,
          newValue: event.target.get(key),
        }));
      } else if (event instanceof Y.YArrayEvent) {
        changeInfo.type = 'YArrayEvent';
        changeInfo.changes = event.changes.delta.map(delta => ({
          retain: delta.retain,
          delete: delta.delete,
          insert: delta.insert,
        }));
      } else if (event instanceof Y.YTextEvent) {
        changeInfo.type = 'YTextEvent';
        changeInfo.changes = event.changes.delta.map(delta => ({
          retain: delta.retain,
          delete: delta.delete,
          insert: delta.insert,
        }));
      }

      changes.push(changeInfo);
    });

    const updateInfo = {
      timestamp: new Date().toISOString(),
      eventCount: events.length,
      changes,
      summary: {
        localChanges: changes.filter(c => c.source === 'local').length,
        serverChanges: changes.filter(c => c.source === 'server').length,
        totalPaths: new Set(changes.map(c => c.path)).size,
      }
    };

    patchesElement.textContent = this.safeStringify(updateInfo, 2);
    
    // 自动刷新数据快照以反映最新状态
    this.updateDataDisplay();
  }

  private getEventPath(event: Y.YEvent<any>): string {
    const path: string[] = [];
    let current: any = event.target;
    
    // 向上遍历事件目标，构建路径
    while (current) {
      if (current instanceof Y.Map) {
        // 对于 Map，我们需要找到它的键名
        const parent = current.parent;
        if (parent instanceof Y.Map) {
          // 在父 Map 中查找当前 Map 的键
          for (const [key, value] of parent.entries()) {
            if (value === current) {
              path.unshift(key);
              break;
            }
          }
        }
      } else if (current instanceof Y.Array) {
        // 对于 Array，我们需要找到它的索引
        const parent = current.parent;
        if (parent instanceof Y.Map) {
          for (const [key, value] of parent.entries()) {
            if (value === current) {
              path.unshift(key);
              break;
            }
          }
        }
      } else if (current instanceof Y.Text) {
        // 对于 Text，类似处理
        const parent = current.parent;
        if (parent instanceof Y.Map) {
          for (const [key, value] of parent.entries()) {
            if (value === current) {
              path.unshift(key);
              break;
            }
          }
        }
      }
      
      current = current.parent;
    }
    
    return path.join('.');
  }

  updateDataDisplay(): void {
    // 获取完整的文档快照，包括所有嵌套结构
    const snapshotElement = document.getElementById('snapshot-data')!;
    const nodes = this.ydoc.getMap('nodes');
    
    // 创建完整的快照数据，包括所有嵌套的 Yjs 对象
    const fullSnapshot = this.createFullSnapshot(nodes);
    
    // 可以选择将 BigInt 显示为普通数字（如果数值在安全范围内）
    // 设置为 true 会将 "0n" 显示为 0，但可能丢失大整数的精度
    const showBigIntAsNumber = false; // 可以根据需要调整
    
    snapshotElement.textContent = this.safeStringify(fullSnapshot, 2, showBigIntAsNumber);
    
    // patches 部分现在由 logDeepChanges 直接更新，这里可以留空或设置默认值
    const patchesElement = document.getElementById('patches-data')!;
    if (!patchesElement.textContent?.startsWith('{')) {
        patchesElement.textContent = "等待深度更新事件...\n\n说明：\n- 使用 observeDeep 监听嵌套对象变化\n- 支持 Map、Array、Text 等所有 Yjs 数据类型\n- 提供详细的变更路径和类型信息\n- BigInt 值显示为 '0n' 格式是正常的";
    }
  }

  private createFullSnapshot(nodesMap: Y.Map<any>): any {
    const snapshot: any = {};
    
    // 遍历所有节点
    for (const [nodeId, nodeValue] of nodesMap.entries()) {
      if (nodeValue instanceof Y.Map) {
        snapshot[nodeId] = this.convertYMapToObject(nodeValue);
      } else {
        snapshot[nodeId] = nodeValue;
      }
    }
    
    return {
      timestamp: new Date().toISOString(),
      nodeCount: Object.keys(snapshot).length,
      nodes: snapshot,
      meta: this.getMetaInfo()
    };
  }

  private convertYMapToObject(yMap: Y.Map<any>): any {
    const result: any = {};
    
    for (const [key, value] of yMap.entries()) {
      if (value instanceof Y.Map) {
        result[key] = this.convertYMapToObject(value);
      } else if (value instanceof Y.Array) {
        result[key] = this.convertYArrayToArray(value);
      } else if (value instanceof Y.Text) {
        result[key] = {
          type: 'YText',
          content: value.toString(),
          length: value.length
        };
      } else {
        result[key] = value;
      }
    }
    
    return result;
  }

  private convertYArrayToArray(yArray: Y.Array<any>): any[] {
    const result: any[] = [];
    
    for (let i = 0; i < yArray.length; i++) {
      const value = yArray.get(i);
      if (value instanceof Y.Map) {
        result.push(this.convertYMapToObject(value));
      } else if (value instanceof Y.Array) {
        result.push(this.convertYArrayToArray(value));
      } else if (value instanceof Y.Text) {
        result.push({
          type: 'YText',
          content: value.toString(),
          length: value.length
        });
      } else {
        result.push(value);
      }
    }
    
    return result;
  }

  private getMetaInfo(): any {
    return {
      docId: this.ydoc.guid,
      clientId: this.ydoc.clientID,
      isSynced: this.client?.getStatus() === 'connected' || false,
      connectionStatus: this.client?.getStatus() || 'disconnected',
      awarenessStates: this.awareness.getStates().size,
      lastUpdate: new Date().toISOString()
    };
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
    
    // 设置基本属性
    newNodeMap.set('type', 'text-client');
    newNodeMap.set('createdAt', new Date().toISOString());
    
    // 创建嵌套的属性结构
    const attrsMap = new Y.Map();
    attrsMap.set('title', '新节点');
    attrsMap.set('description', '这是一个测试节点');
    
    // 创建嵌套的数组结构
    const tagsArray = new Y.Array();
    tagsArray.push(['tag1', 'tag2', 'tag3']);
    attrsMap.set('tags', tagsArray);
    
    // 创建嵌套的 Map 结构
    const metadataMap = new Y.Map();
    metadataMap.set('version', '1.0.0');
    metadataMap.set('author', 'test-user');
    attrsMap.set('metadata', metadataMap);
    
    // 创建文本内容
    const contentText = new Y.Text();
    contentText.insert(0, '这是节点的内容');
    attrsMap.set('content', contentText);
    
    newNodeMap.set('attrs', attrsMap);
    nodesMap.set(nodeId, newNodeMap);
    this.log(`✅ 添加节点: ${nodeId}`);
    
    // 更新数据快照以显示新节点
    this.updateDataDisplay();
  }

  updateAttribute(): void {
    if (!this.client || this.client.getStatus() !== ConnectionStatus.Connected) {
      this.log('❌ 请先连接到房间');
      return;
    }

    const nodesMap = this.client.doc.getMap('nodes');
    const nodeIds = Array.from(nodesMap.keys());
    
    if (nodeIds.length === 0) {
      this.log('❌ 没有可更新的节点，请先添加节点');
      return;
    }

    // 随机选择一个节点进行更新
    const randomNodeId = nodeIds[Math.floor(Math.random() * nodeIds.length)];
    const nodeMap = nodesMap.get(randomNodeId) as Y.Map<any>;
    
    if (nodeMap) {
      // 更新基本属性
      nodeMap.set('updatedAt', new Date().toISOString());
      nodeMap.set('version', (nodeMap.get('version') || 0) + 1);
      
      // 更新或创建嵌套属性
      let attrsMap = nodeMap.get('attrs') as Y.Map<any>;
      if (!attrsMap) {
        attrsMap = new Y.Map();
        nodeMap.set('attrs', attrsMap);
      }
      
      // 更新嵌套属性
      attrsMap.set('title', `更新后的标题 ${Date.now()}`);
      attrsMap.set('description', `更新后的描述 ${Date.now()}`);
      
      // 更新嵌套数组
      let tagsArray = attrsMap.get('tags') as Y.Array<any>;
      if (!tagsArray) {
        tagsArray = new Y.Array();
        attrsMap.set('tags', tagsArray);
      }
      tagsArray.push([`tag_${Date.now()}`]);
      
      // 更新嵌套 Map
      let metadataMap = attrsMap.get('metadata') as Y.Map<any>;
      if (!metadataMap) {
        metadataMap = new Y.Map();
        attrsMap.set('metadata', metadataMap);
      }
      metadataMap.set('lastModified', new Date().toISOString());
      metadataMap.set('modifiedBy', 'test-user');
      
      this.log(`✅ 更新节点属性: ${randomNodeId}`);
      
      // 更新数据快照以显示更新后的节点
      this.updateDataDisplay();
    } else {
      this.log(`❌ 节点不存在: ${randomNodeId}`);
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
      cursorsElement.textContent = this.safeStringify(displayData, 2);
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

      console.log("📊 数据存储对比:", this.safeStringify(comparison, 2));
      this.log("📊 数据存储对比已输出到控制台，打开开发者工具查看详细信息");
      
      // 也可以在页面上显示
      alert(`📊 数据存储对比：
      
📄 Yjs Doc (持久化): ${Object.keys(docData).length} 个节点
🖱️ Awareness (临时): ${Object.keys(awarenessData).length} 个用户在线

详细信息请查看浏览器控制台。`);
    }, 500);
  }

  testDeepUpdates(): void {
    if (!this.client || this.client.getStatus() !== ConnectionStatus.Connected) {
      this.log('❌ 请先连接到房间');
      return;
    }

    this.log('🧪 开始深度更新测试...');

    // 创建一个复杂的测试节点
    const nodesMap = this.client.doc.getMap('nodes');
    const testNodeId = 'deep_test_' + Date.now();
    const testNodeMap = new Y.Map();

    // 基本属性
    testNodeMap.set('type', 'deep-test');
    testNodeMap.set('createdAt', new Date().toISOString());

    // 创建复杂的嵌套结构
    const complexAttrs = new Y.Map();
    
    // 嵌套 Map
    const nestedMap = new Y.Map();
    nestedMap.set('level1', 'value1');
    nestedMap.set('level2', 'value2');
    nestedMap.set('bigIntValue', 0); // 这会被转换为 BigInt
    nestedMap.set('largeNumber', 9223372036854775807); // 大整数，会显示为 BigInt
    complexAttrs.set('nestedMap', nestedMap);

    // 嵌套数组
    const nestedArray = new Y.Array();
    nestedArray.push(['item1', 'item2']);
    complexAttrs.set('nestedArray', nestedArray);

    // 文本内容
    const textContent = new Y.Text();
    textContent.insert(0, '初始文本内容');
    complexAttrs.set('textContent', textContent);

    testNodeMap.set('complexAttrs', complexAttrs);
    nodesMap.set(testNodeId, testNodeMap);

    this.log(`✅ 创建测试节点: ${testNodeId}`);

    // 延迟执行深度更新操作
    setTimeout(() => {
      this.log('🔄 执行深度更新操作...');
      
      const testNode = nodesMap.get(testNodeId) as Y.Map<any>;
      if (testNode) {
        const complexAttrs = testNode.get('complexAttrs') as Y.Map<any>;
        
        // 更新嵌套 Map
        const nestedMap = complexAttrs.get('nestedMap') as Y.Map<any>;
        nestedMap.set('level3', 'value3');
        nestedMap.set('level1', 'updated_value1');
        
        // 更新嵌套数组
        const nestedArray = complexAttrs.get('nestedArray') as Y.Array<any>;
        nestedArray.push(['item3', 'item4']);
        nestedArray.delete(0, 1); // 删除第一个元素
        
        // 更新文本内容
        const textContent = complexAttrs.get('textContent') as Y.Text;
        textContent.insert(textContent.length, ' - 追加内容');
        textContent.delete(0, 4); // 删除前4个字符
        
        // 添加新的嵌套结构
        const newNestedMap = new Y.Map();
        newNestedMap.set('newKey', 'newValue');
        complexAttrs.set('newNestedMap', newNestedMap);
        
        this.log('✅ 深度更新操作完成，观察 patches 区域的变化');
      }
    }, 1000);

    // 再次延迟执行更多操作
    setTimeout(() => {
      this.log('🔄 执行第二轮深度更新操作...');
      
      const testNode = nodesMap.get(testNodeId) as Y.Map<any>;
      if (testNode) {
        const complexAttrs = testNode.get('complexAttrs') as Y.Map<any>;
        
        // 删除一些属性
        complexAttrs.delete('nestedArray');
        
        // 修改新添加的嵌套 Map
        const newNestedMap = complexAttrs.get('newNestedMap') as Y.Map<any>;
        if (newNestedMap) {
          newNestedMap.set('anotherKey', 'anotherValue');
        }
        
        this.log('✅ 第二轮深度更新操作完成');
      }
    }, 3000);
  }

  refreshSnapshot(): void {
    this.log('🔄 手动刷新数据快照...');
    this.updateDataDisplay();
    this.log('✅ 数据快照已刷新');
  }

  requestSync(): void {
    if (!this.client || this.client.getStatus() !== ConnectionStatus.Connected) {
      this.log('❌ 请先连接到房间');
      return;
    }

    this.log('🔄 请求重新同步...');
    
    // 强制刷新数据快照
    this.updateDataDisplay();
    
    // 记录当前状态
    const currentState = {
      docId: this.ydoc.guid,
      clientId: this.ydoc.clientID,
      nodeCount: this.ydoc.getMap('nodes').size,
      awarenessStates: this.awareness.getStates().size,
      timestamp: new Date().toISOString()
    };
    
    this.log('📊 当前同步状态:', currentState);
    this.log('✅ 重新同步请求完成');
  }

  testLocalUpdate(): void {
    this.log('🧪 开始测试本地更新...');
    
    // 创建一个本地测试节点
    const nodesMap = this.ydoc.getMap('nodes');
    const testNodeId = 'local_test_' + Date.now();
    const testNodeMap = new Y.Map();
    
    testNodeMap.set('type', 'local-test');
    testNodeMap.set('createdAt', new Date().toISOString());
    testNodeMap.set('description', '这是一个本地测试节点');
    
    nodesMap.set(testNodeId, testNodeMap);
    
    this.log(`✅ 创建本地测试节点: ${testNodeId}`);
    this.log('📝 注意：这个更新不会同步到服务器，仅用于测试本地 Yjs 功能');
    
    // 更新数据快照
    this.updateDataDisplay();
  }

  testBigIntDisplay(): void {
    this.log('🧪 开始测试 BigInt 显示格式...');
    
    const nodesMap = this.ydoc.getMap('nodes');
    const testNodeId = 'bigint_test_' + Date.now();
    const testNodeMap = new Y.Map();
    
    // 创建包含各种数字类型的测试数据
    testNodeMap.set('type', 'bigint-test');
    testNodeMap.set('smallNumber', 42); // 普通数字
    testNodeMap.set('zeroNumber', 0); // 零值
    testNodeMap.set('largeNumber', 9223372036854775807); // 大整数
    testNodeMap.set('negativeNumber', -123); // 负数
    
    // 创建嵌套结构
    const nestedMap = new Y.Map();
    nestedMap.set('nestedSmall', 100);
    nestedMap.set('nestedLarge', 18446744073709551615); // 更大的整数
    testNodeMap.set('nestedNumbers', nestedMap);
    
    nodesMap.set(testNodeId, testNodeMap);
    
    this.log(`✅ 创建 BigInt 测试节点: ${testNodeId}`);
    this.log('📊 观察数据快照中的数字显示格式：');
    this.log('   - 小数字可能显示为普通数字');
    this.log('   - 大数字会显示为 "数字n" 格式（如 "9223372036854775807n"）');
    this.log('   - 这是 JavaScript BigInt 的标准表示');
    
    // 更新数据快照
    this.updateDataDisplay();
  }

  testBigIntMath(): void {
    this.log('🧮 开始测试 BigInt 数学运算...');
    
    const nodesMap = this.ydoc.getMap('nodes');
    const testNodeId = 'bigint_math_' + Date.now();
    const testNodeMap = new Y.Map();
    
    // 创建 BigInt 值进行数学运算
    const a = BigInt(1000000000000000000); // 1 quintillion
    const b = BigInt(500000000000000000);  // 500 quadrillion
    
    // 执行各种数学运算
    const addition = a + b;        // 加法
    const subtraction = a - b;     // 减法
    const multiplication = a * b;  // 乘法
    const division = a / b;        // 除法（整数除法）
    const remainder = a % b;       // 取余
    const power = a ** BigInt(2);  // 幂运算
    
    // 存储运算结果
    testNodeMap.set('type', 'bigint-math');
    testNodeMap.set('operandA', a.toString() + 'n');
    testNodeMap.set('operandB', b.toString() + 'n');
    testNodeMap.set('addition', addition.toString() + 'n');
    testNodeMap.set('subtraction', subtraction.toString() + 'n');
    testNodeMap.set('multiplication', multiplication.toString() + 'n');
    testNodeMap.set('division', division.toString() + 'n');
    testNodeMap.set('remainder', remainder.toString() + 'n');
    testNodeMap.set('power', power.toString() + 'n');
    
    // 创建运算历史
    const operationsArray = new Y.Array();
    const operations = [
      `${a}n + ${b}n = ${addition}n`,
      `${a}n - ${b}n = ${subtraction}n`,
      `${a}n * ${b}n = ${multiplication}n`,
      `${a}n / ${b}n = ${division}n`,
      `${a}n % ${b}n = ${remainder}n`,
      `${a}n ** 2 = ${power}n`
    ];
    operations.forEach(op => operationsArray.push([op]));
    testNodeMap.set('operations', operationsArray);
    
    nodesMap.set(testNodeId, testNodeMap);
    
    this.log(`✅ 创建 BigInt 数学运算测试节点: ${testNodeId}`);
    this.log('🧮 BigInt 数学运算结果：');
    this.log(`   加法: ${a}n + ${b}n = ${addition}n`);
    this.log(`   减法: ${a}n - ${b}n = ${subtraction}n`);
    this.log(`   乘法: ${a}n * ${b}n = ${multiplication}n`);
    this.log(`   除法: ${a}n / ${b}n = ${division}n`);
    this.log(`   取余: ${a}n % ${b}n = ${remainder}n`);
    this.log(`   幂运算: ${a}n ** 2 = ${power}n`);
    
    // 更新数据快照
    this.updateDataDisplay();
  }

  testBigIntComparison(): void {
    this.log('🔍 开始测试 BigInt 比较运算...');
    
    const nodesMap = this.ydoc.getMap('nodes');
    const testNodeId = 'bigint_compare_' + Date.now();
    const testNodeMap = new Y.Map();
    
    // 创建 BigInt 值进行比较
    const small = BigInt(100);
    const medium = BigInt(1000);
    const large = BigInt(10000);
    
    // 执行比较运算
    const comparisons = {
      'small < medium': small < medium,
      'medium > small': medium > small,
      'large >= medium': large >= medium,
      'small <= medium': small <= medium,
      'small === small': small === small,
      'small !== medium': small !== medium
    };
    
    // 存储比较结果
    testNodeMap.set('type', 'bigint-comparison');
    testNodeMap.set('small', small.toString() + 'n');
    testNodeMap.set('medium', medium.toString() + 'n');
    testNodeMap.set('large', large.toString() + 'n');
    
    const comparisonArray = new Y.Array();
    Object.entries(comparisons).forEach(([operation, result]) => {
      comparisonArray.push([`${operation}: ${result}`]);
    });
    testNodeMap.set('comparisons', comparisonArray);
    
    nodesMap.set(testNodeId, testNodeMap);
    
    this.log(`✅ 创建 BigInt 比较运算测试节点: ${testNodeId}`);
    this.log('🔍 BigInt 比较运算结果：');
    Object.entries(comparisons).forEach(([operation, result]) => {
      this.log(`   ${operation}: ${result}`);
    });
    
    // 更新数据快照
    this.updateDataDisplay();
  }

  testBigIntCalculator(): void {
    this.log('🧮 开始测试 BigInt 计算器...');
    
    const nodesMap = this.ydoc.getMap('nodes');
    const testNodeId = 'bigint_calculator_' + Date.now();
    const testNodeMap = new Y.Map();
    
    // 模拟金融计算场景
    const principal = BigInt(1000000000000000000); // 1 quintillion (基础金额)
    const rate = BigInt(5); // 5% 年利率
    const years = BigInt(10); // 10年
    
    // 计算复利
    const rateMultiplier = BigInt(100) + rate; // 105
    const finalAmount = principal * (rateMultiplier ** years) / (BigInt(100) ** years);
    
    // 计算利息
    const interest = finalAmount - principal;
    
    // 计算月供（简化计算）
    const monthlyRate = rate * BigInt(12) / BigInt(100); // 月利率
    const totalMonths = years * BigInt(12); // 总月数
    
    // 存储计算结果
    testNodeMap.set('type', 'bigint-calculator');
    testNodeMap.set('principal', principal.toString() + 'n');
    testNodeMap.set('annualRate', rate.toString() + '%');
    testNodeMap.set('years', years.toString() + 'n');
    testNodeMap.set('finalAmount', finalAmount.toString() + 'n');
    testNodeMap.set('interest', interest.toString() + 'n');
    testNodeMap.set('monthlyRate', monthlyRate.toString() + '%');
    testNodeMap.set('totalMonths', totalMonths.toString() + 'n');
    
    // 创建计算历史
    const calculationArray = new Y.Array();
    const calculations = [
      `本金: ${principal}n`,
      `年利率: ${rate}%`,
      `投资年限: ${years}年`,
      `最终金额: ${finalAmount}n`,
      `利息收入: ${interest}n`,
      `月利率: ${monthlyRate}%`,
      `总月数: ${totalMonths}n`
    ];
    calculations.forEach(calc => calculationArray.push([calc]));
    testNodeMap.set('calculations', calculationArray);
    
    nodesMap.set(testNodeId, testNodeMap);
    
    this.log(`✅ 创建 BigInt 计算器测试节点: ${testNodeId}`);
    this.log('🧮 BigInt 金融计算示例：');
    this.log(`   本金: ${principal}n`);
    this.log(`   年利率: ${rate}%`);
    this.log(`   投资年限: ${years}年`);
    this.log(`   最终金额: ${finalAmount}n`);
    this.log(`   利息收入: ${interest}n`);
    this.log(`   月利率: ${monthlyRate}%`);
    this.log(`   总月数: ${totalMonths}n`);
    
    // 更新数据快照
    this.updateDataDisplay();
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
    (document.getElementById('request-sync-btn') as HTMLButtonElement).disabled = !isConnected;
    (document.getElementById('test-update-btn') as HTMLButtonElement).disabled = false; // 本地测试不需要连接
    (document.getElementById('room-input') as HTMLInputElement).disabled = isConnected || isConnecting;
    
    // 光标功能按钮状态
    (document.getElementById('set-cursor-btn') as HTMLButtonElement).disabled = !isConnected;
    (document.getElementById('simulate-typing-btn') as HTMLButtonElement).disabled = !isConnected;
    (document.getElementById('move-cursor-btn') as HTMLButtonElement).disabled = !isConnected;
    (document.getElementById('show-data-diff-btn') as HTMLButtonElement).disabled = !isConnected;
    
    // 深度更新测试按钮状态
    const deepUpdateBtn = document.getElementById('deep-update-btn') as HTMLButtonElement;
    if (deepUpdateBtn) {
      deepUpdateBtn.disabled = !isConnected;
    }
    
    // 刷新快照按钮状态
    const refreshSnapshotBtn = document.getElementById('refresh-snapshot-btn') as HTMLButtonElement;
    if (refreshSnapshotBtn) {
      refreshSnapshotBtn.disabled = !isConnected;
    }
    
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