import * as Y from 'yjs';
import { CollaborationClient, ConnectionStatus } from '../src/index.ts';

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
  private localUpdates: UpdateInfo[];

  constructor() {
    this.client = null;
    this.ydoc = new Y.Doc();
    this.localUpdates = [];
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

    this.updateConnectionUI(ConnectionStatus.Disconnected);
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
        source: event.transaction.origin === this.client ? 'server' : 'local',
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
      alert('Please enter a room ID');
      return;
    }

    if (this.client) {
      this.client.destroy();
    }

    this.client = new CollaborationClient({
      url: 'ws://localhost:8080',
      room: roomId,
      doc: this.ydoc,
    });

    this.client.on('status', (status: ConnectionStatus) => {
      this.log(`Connection status changed: ${status}`);
      this.updateConnectionUI(status);
    });

    this.client.on('synced', (synced: boolean) => {
      this.log(`Sync status changed: ${synced ? 'Synced' : 'Not Synced'}`);
      this.updateYjsStatus(synced);
    });
    
    this.client.on('error', (error: Error) => {
        this.log(`An error occurred: ${error.message}`, error);
    });

    this.client.connect();
  }

  disconnect(): void {
    if (this.client) {
      this.client.disconnect();
      this.client = null;
    }
    this.localUpdates = [];
    this.updateConnectionUI(ConnectionStatus.Disconnected);
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
    }
  }

  clearData(): void {
    if (!this.client || !this.client.doc) return;
    this.client.doc.getMap('nodes').clear();
  }

  updateConnectionUI(status: ConnectionStatus): void {
    const isConnected = status === ConnectionStatus.Connected;
    const isConnecting = status === ConnectionStatus.Connecting || status === ConnectionStatus.Reconnecting;

    (document.getElementById('connect-btn') as HTMLButtonElement).disabled = isConnected || isConnecting;
    (document.getElementById('disconnect-btn') as HTMLButtonElement).disabled = !isConnected;
    (document.getElementById('add-node-btn') as HTMLButtonElement).disabled = !isConnected;
    (document.getElementById('update-attr-btn') as HTMLButtonElement).disabled = !isConnected;
    (document.getElementById('clear-data-btn') as HTMLButtonElement).disabled = !isConnected;
    (document.getElementById('room-input') as HTMLInputElement).disabled = isConnected || isConnecting;
    
    const indicator = document.getElementById('ws-status')!;
    if (isConnected) {
        indicator.classList.add('connected');
        indicator.title = 'Connected';
    } else {
        indicator.classList.remove('connected');
        indicator.title = status;
    }
  }

  updateYjsStatus(synced: boolean): void {
    const indicator = document.getElementById('yjs-status')!;
    if (synced) {
      indicator.classList.add('connected');
    } else {
      indicator.classList.remove('connected');
    }
  }


  log(message: string, data: any = null): void {
    const timestamp = new Date().toLocaleTimeString();
    const logElement = document.getElementById('log-data')!;
    const logEntry = document.createElement('div');
    const dataString = data ? ` - ${JSON.stringify(data)}` : '';
    logEntry.textContent = `[${timestamp}] ${message}${dataString}`;
    
    if (data) console.log(`[LOG] ${message}`, data);
    else console.log(`[LOG] ${message}`);

    if (logElement.firstChild) {
      logElement.insertBefore(logEntry, logElement.firstChild);
    } else {
      logElement.appendChild(logEntry);
    }
  }
}

// 声明全局变量以供调试
declare global {
    interface Window { app: App; }
}

window.app = new App();
console.log('ModuForge Yjs test client initialized (TypeScript).');
console.log('You can access the app instance via `window.app`.');