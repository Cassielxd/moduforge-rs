import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';
import {
  CollaborationClientOptions,
  ConnectionStatus,
  ErrorType,
  CollaborationError,
  RetryConfig,
  UserInfo,
  CursorPosition,
  AwarenessState,
} from './types';

type EventMap = {
  'status': (status: ConnectionStatus) => void;
  'synced': (synced: boolean) => void;
  'error': (error: CollaborationError) => void;
  'reconnectAttempt': (attempt: number, maxAttempts: number) => void;
  'connectionTimeout': () => void;
  'awarenessChange': (states: Map<number, AwarenessState>) => void;
  'cursorMove': (userId: string, cursor: CursorPosition) => void;
  'userJoin': (user: UserInfo) => void;
  'userLeave': (userId: string) => void;
};

export class CollaborationClient {
  public readonly doc: Y.Doc;
  private provider: WebsocketProvider | null = null;
  private readonly url: string;
  private readonly room: string;
  private status: ConnectionStatus = ConnectionStatus.Disconnected;
  private readonly awareness: any;

  // 配置选项
  private readonly autoReconnect: boolean;
  private readonly reconnectDelay: number;
  private readonly maxReconnectAttempts: number;
  private readonly connectionTimeout: number;

  // 重试状态
  private reconnectAttempts: number = 0;
  private reconnectTimeoutId: number | null = null;
  private connectionTimeoutId: number | null = null;
  private isDestroyed: boolean = false;

  private listeners: { [K in keyof EventMap]: EventMap[K][] } = {
    status: [],
    synced: [],
    error: [],
    reconnectAttempt: [],
    connectionTimeout: [],
    awarenessChange: [],
    cursorMove: [],
    userJoin: [],
    userLeave: [],
  };

  // 默认重试配置
  private readonly defaultRetryConfig: RetryConfig = {
    maxAttempts: 5,
    baseDelay: 1000,
    backoffMultiplier: 2,
    maxDelay: 30000,
  };

  constructor(options: CollaborationClientOptions) {
    this.doc = options.doc;
    this.url = options.url;
    this.room = options.room;
    this.awareness = options.awareness;

    // 设置配置选项
    this.autoReconnect = options.autoReconnect ?? true;
    this.reconnectDelay = options.reconnectDelay ?? 2000;
    this.maxReconnectAttempts = options.maxReconnectAttempts ?? 5;
    this.connectionTimeout = options.connectionTimeout ?? 10000;

    // 初始化用户信息
    if (options.user) {
      this.awareness.setLocalStateField('user', options.user);
      // 同时设置在线状态
      this.awareness.setLocalStateField('online', true);
    }
  }

  public connect(): void {
    if (this.isDestroyed) {
      this.emitError({
        type: ErrorType.CONNECTION_FAILED,
        message: '客户端已被销毁，无法连接',
        recoverable: false,
        suggestedAction: '请创建新的客户端实例',
      });
      return;
    }

    if (this.provider && this.status === ConnectionStatus.Connected) {
      return; // 已经连接
    }

    this.clearTimeouts();
    this.setStatus(ConnectionStatus.Connecting);

    try {
      // 设置连接超时
      this.connectionTimeoutId = window.setTimeout(() => {
        this.handleConnectionTimeout();
      }, this.connectionTimeout);

      // 先检查房间是否存在，然后再连接
      this.checkRoomExists()
        .then((exists) => {
          if (!exists) {
            this.handleRoomNotFound();
            return;
          }
          this.establishConnection();
        })
        .catch((error) => {
          // 如果预检查失败，仍然尝试连接（可能是网络问题或服务器不支持预检查）
          console.warn('房间预检查失败，直接尝试连接:', error);
          this.establishConnection();
        });

    } catch (error) {
      this.handleConnectionError(error);
    }
  }

  private async checkRoomExists(): Promise<boolean> {
    try {
      // 使用新的后端房间检查接口
      const baseUrl = this.url.replace('ws://', 'http://').replace('wss://', 'https://');
      const checkUrl = `${baseUrl}/room-check/${this.room}`;
      
      const response = await fetch(checkUrl, {
        method: 'GET',
        headers: {
          'Accept': 'application/json',
        },
        // 添加超时处理
        signal: AbortSignal.timeout(5000),
      });
      
      if (response.ok) {
        const data = await response.json();
        return data.exists === true;
      } else if (response.status === 404) {
        return false;
      } else {
        // 其他错误，假设房间存在，让 WebSocket 处理
        throw new Error(`房间检查失败: ${response.status}`);
      }
    } catch (error) {
      // 网络错误或超时，假设房间存在，让 WebSocket 连接处理错误
      console.warn('房间检查请求失败:', error);
      throw error;
    }
  }

  private establishConnection(): void {
    if (this.provider) {
      this.provider.destroy();
    }

    try {
      this.provider = new WebsocketProvider(this.url, this.room, this.doc, {
        awareness: this.awareness,
      });

      this.setupProviderListeners();
    } catch (error) {
      this.handleConnectionError(error);
    }
  }

  private setupProviderListeners(): void {
    if (!this.provider) return;

    this.provider.on('status', (event: { status: 'connected' | 'connecting' | 'disconnected' }) => {
      this.clearConnectionTimeout();

      switch (event.status) {
        case 'connected':
          this.reconnectAttempts = 0; // 重置重连计数
          this.setStatus(ConnectionStatus.Connected);
          break;
        case 'connecting':
          this.setStatus(ConnectionStatus.Connecting);
          break;
        case 'disconnected':
          this.handleDisconnection();
          break;
      }
    });

    this.provider.on('sync', (isSynced: boolean) => {
      this.emit('synced', isSynced);
      if (!isSynced && this.status === ConnectionStatus.Connected) {
        this.emitError({
          type: ErrorType.SYNC_FAILED,
          message: '数据同步失败',
          recoverable: true,
          suggestedAction: '请检查网络连接或刷新页面重试',
        });
      }
    });

    // 监听 awareness 变化（光标同步）
    this.awareness.on('change', ({ added, updated, removed }: { added: number[], updated: number[], removed: number[] }) => {
      const states = this.awareness.getStates();
      this.emit('awarenessChange', states);
      
      // 处理新加入的用户
      added.forEach((clientId: number) => {
        const state = states.get(clientId);
        if (state && state.user) {
          this.emit('userJoin', state.user);
        }
      });
      
      // 处理更新的用户（包括光标移动）
      updated.forEach((clientId: number) => {
        const state = states.get(clientId);
        if (state && state.cursor) {
          this.emit('cursorMove', state.user?.id || `client_${clientId}`, state.cursor);
        }
      });
      
      // 处理离开的用户
      removed.forEach((clientId: number) => {
        this.emit('userLeave', `client_${clientId}`);
      });
    });

    // 监听 WebSocket 错误
    if (this.provider.ws) {
      this.provider.ws.addEventListener('error', (event) => {
        this.handleWebSocketError(event);
      });

      this.provider.ws.addEventListener('close', (event) => {
        this.handleWebSocketClose(event);
      });
    }
  }

  private handleConnectionTimeout(): void {
    this.clearConnectionTimeout();
    this.emit('connectionTimeout');
    
    this.emitError({
      type: ErrorType.CONNECTION_TIMEOUT,
      message: `连接超时（${this.connectionTimeout}ms）`,
      recoverable: true,
      suggestedAction: '请检查网络连接和服务器状态',
    });

    this.setStatus(ConnectionStatus.Failed);
    this.attemptReconnect();
  }

  private handleRoomNotFound(): void {
    this.clearConnectionTimeout();
    
    this.emitError({
      type: ErrorType.ROOM_NOT_FOUND,
      message: `房间 "${this.room}" 不存在`,
      details: {
        room_id: this.room,
        code: 404,
      },
      recoverable: false,
      suggestedAction: '请检查房间ID是否正确，或联系管理员创建房间',
    });

    this.setStatus(ConnectionStatus.Failed);
  }

  private handleWebSocketError(event: Event): void {
    this.emitError({
      type: ErrorType.WEBSOCKET_ERROR,
      message: 'WebSocket 连接发生错误',
      recoverable: true,
      suggestedAction: '连接将自动重试',
      originalError: event,
    });
  }

  private handleWebSocketClose(event: CloseEvent): void {
    const { code, reason } = event;
    
    // 根据关闭代码确定错误类型
    let errorType = ErrorType.CONNECTION_FAILED;
    let message = '连接已关闭';
    let recoverable = true;
    let suggestedAction = '将尝试自动重连';

    switch (code) {
      case 1000: // 正常关闭
        return; // 不当作错误处理
      case 1001: // 端点离开
        message = '服务器端点离开';
        break;
      case 1002: // 协议错误
        errorType = ErrorType.WEBSOCKET_ERROR;
        message = 'WebSocket 协议错误';
        recoverable = false;
        suggestedAction = '请联系技术支持';
        break;
      case 1003: // 不支持的数据类型
        errorType = ErrorType.WEBSOCKET_ERROR;
        message = '服务器不支持的数据类型';
        recoverable = false;
        break;
      case 1006: // 异常关闭
        errorType = ErrorType.NETWORK_ERROR;
        message = '网络连接异常断开';
        break;
      case 1011: // 服务器错误
        errorType = ErrorType.SERVER_ERROR;
        message = '服务器内部错误';
        break;
      case 4000: // 自定义：房间不存在
        errorType = ErrorType.ROOM_NOT_FOUND;
        message = `房间 "${this.room}" 不存在`;
        recoverable = false;
        suggestedAction = '请检查房间ID或联系管理员';
        break;
      case 4001: // 自定义：房间已满
        errorType = ErrorType.ROOM_FULL;
        message = '房间已满，无法加入';
        recoverable = false;
        suggestedAction = '请稍后重试或联系管理员';
        break;
      case 4003: // 自定义：访问被拒绝
        errorType = ErrorType.ROOM_ACCESS_DENIED;
        message = '房间访问被拒绝';
        recoverable = false;
        suggestedAction = '请检查访问权限';
        break;
      default:
        message = reason || `连接关闭 (代码: ${code})`;
    }

    this.emitError({
      type: errorType,
      message,
      details: {
        code,
        reason,
        room_id: this.room,
      },
      recoverable,
      suggestedAction,
      originalError: event,
    });
  }

  private handleConnectionError(error: any): void {
    this.clearConnectionTimeout();
    
    this.emitError({
      type: ErrorType.CONNECTION_FAILED,
      message: '连接失败',
      recoverable: true,
      suggestedAction: '将尝试自动重连',
      originalError: error,
    });

    this.setStatus(ConnectionStatus.Failed);
    this.attemptReconnect();
  }

  private handleDisconnection(): void {
    if (this.status === ConnectionStatus.Connected) {
      this.setStatus(ConnectionStatus.Disconnected);
      this.attemptReconnect();
    }
  }

  private attemptReconnect(): void {
    if (!this.autoReconnect || this.isDestroyed) {
      return;
    }

    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      this.emitError({
        type: ErrorType.CONNECTION_FAILED,
        message: `重连失败，已达到最大尝试次数 (${this.maxReconnectAttempts})`,
        recoverable: false,
        suggestedAction: '请手动刷新页面或检查网络连接',
      });
      this.setStatus(ConnectionStatus.Failed);
      return;
    }

    this.reconnectAttempts++;
    this.setStatus(ConnectionStatus.Reconnecting);
    this.emit('reconnectAttempt', this.reconnectAttempts, this.maxReconnectAttempts);

    // 计算重连延迟（指数退避）
    const delay = Math.min(
      this.reconnectDelay * Math.pow(this.defaultRetryConfig.backoffMultiplier, this.reconnectAttempts - 1),
      this.defaultRetryConfig.maxDelay
    );

    this.reconnectTimeoutId = window.setTimeout(() => {
      if (!this.isDestroyed) {
        this.connect();
      }
    }, delay);
  }

  private clearTimeouts(): void {
    this.clearConnectionTimeout();
    this.clearReconnectTimeout();
  }

  private clearConnectionTimeout(): void {
    if (this.connectionTimeoutId) {
      clearTimeout(this.connectionTimeoutId);
      this.connectionTimeoutId = null;
    }
  }

  private clearReconnectTimeout(): void {
    if (this.reconnectTimeoutId) {
      clearTimeout(this.reconnectTimeoutId);
      this.reconnectTimeoutId = null;
    }
  }

  private emitError(error: Omit<CollaborationError, 'details'> & { details?: any }): void {
    const fullError: CollaborationError = {
      ...error,
      details: {
        timestamp: new Date().toISOString(),
        room_id: this.room,
        ...error.details,
      },
    };
    
    console.error('[CollaborationClient] Error:', fullError);
    this.emit('error', fullError);
  }

  public disconnect(): void {
    this.clearTimeouts();
    this.reconnectAttempts = 0;
    
    // 清理 awareness 状态
    this.awareness.setLocalStateField('online', false);
    this.awareness.setLocalStateField('cursor', null);
    this.awareness.setLocalStateField('typing', false);
    
    if (this.provider) {
      this.provider.disconnect();
    }
    
    this.setStatus(ConnectionStatus.Disconnected);
  }

  public destroy(): void {
    this.isDestroyed = true;
    this.clearTimeouts();
    
    if (this.provider) {
      this.provider.destroy();
      this.provider = null;
    }
    
    this.listeners = { 
      status: [], 
      synced: [], 
      error: [], 
      reconnectAttempt: [], 
      connectionTimeout: [],
      awarenessChange: [],
      cursorMove: [],
      userJoin: [],
      userLeave: [],
    };
    this.setStatus(ConnectionStatus.Disconnected);
  }

  // 手动重试连接
  public retry(): void {
    if (this.status === ConnectionStatus.Failed || this.status === ConnectionStatus.Disconnected) {
      this.reconnectAttempts = 0; // 重置计数
      this.connect();
    }
  }

  // 获取当前状态
  public getStatus(): ConnectionStatus {
    return this.status;
  }

  // 获取重连信息
  public getReconnectInfo(): { attempts: number; maxAttempts: number } {
    return {
      attempts: this.reconnectAttempts,
      maxAttempts: this.maxReconnectAttempts,
    };
  }

  public on<K extends keyof EventMap>(event: K, listener: EventMap[K]): void {
    this.listeners[event].push(listener);
  }

  public off<K extends keyof EventMap>(event: K, listener: EventMap[K]): void {
    const eventListeners = this.listeners[event];
    this.listeners[event] = eventListeners.filter(l => l !== listener) as any;
  }

  private emit<K extends keyof EventMap>(event: K, ...args: Parameters<EventMap[K]>): void {
    this.listeners[event].forEach(listener => (listener as any)(...args));
  }

  private setStatus(status: ConnectionStatus): void {
    if (this.status !== status) {
      this.status = status;
      this.emit('status', this.status);
    }
  }
} 