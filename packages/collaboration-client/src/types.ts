import * as Y from 'yjs';
import { Awareness } from 'y-protocols/awareness';

/**
 * 光标位置信息
 */
export interface CursorPosition {
  /**
   * 节点ID或文档路径
   */
  anchor: {
    nodeId?: string;
    path?: number[];
    offset: number;
  };
  
  /**
   * 选择范围的头部位置（如果有选中文本）
   */
  head?: {
    nodeId?: string;
    path?: number[];
    offset: number;
  };
  
  /**
   * 光标位置的时间戳
   */
  timestamp: number;
}

/**
 * 用户信息
 */
export interface UserInfo {
  /**
   * 用户唯一标识
   */
  id: string;
  
  /**
   * 用户显示名称
   */
  name: string;
  
  /**
   * 用户头像URL（可选）
   */
  avatar?: string;
  
  /**
   * 用户颜色（用于光标和选区显示）
   */
  color: string;
  
  /**
   * 用户是否在线
   */
  online: boolean;
  
  /**
   * 最后活动时间
   */
  lastSeen: number;
}

/**
 * 光标状态信息
 */
export interface CursorState {
  /**
   * 用户信息
   */
  user: UserInfo;
  
  /**
   * 当前光标位置
   */
  cursor?: CursorPosition;
  
  /**
   * 是否正在输入
   */
  typing?: boolean;
  
  /**
   * 输入状态的时间戳
   */
  typingTimestamp?: number;
}

/**
 * Awareness 状态接口
 */
export interface AwarenessState {
  /**
   * 用户信息
   */
  user?: UserInfo;
  
  /**
   * 光标状态
   */
  cursor?: CursorPosition;
  
  /**
   * 是否正在输入
   */
  typing?: boolean;
  
  /**
   * 输入状态时间戳
   */
  typingTimestamp?: number;
  
  /**
   * 自定义状态数据
   */
  [key: string]: any;
}

/**
 * 光标事件类型
 */
export enum CursorEventType {
  CURSOR_MOVE = 'cursor_move',
  SELECTION_CHANGE = 'selection_change', 
  TYPING_START = 'typing_start',
  TYPING_STOP = 'typing_stop',
  USER_JOIN = 'user_join',
  USER_LEAVE = 'user_leave',
}

/**
 * 光标事件数据
 */
export interface CursorEvent {
  type: CursorEventType;
  userId: string;
  user: UserInfo;
  cursor?: CursorPosition;
  timestamp: number;
}

/**
 * Configuration options for the CollaborationClient.
 */
export interface CollaborationClientOptions {
  /**
   * The WebSocket endpoint to connect to.
   * e.g., 'ws://localhost:8080'
   */
  url: string;

  /**
   * The room identifier.
   */
  room: string;

  /**
   * The Yjs document to be synced.
   */
  doc: Y.Doc;
  
  /**
   * The Yjs awareness instance for this client.
   */
  awareness: Awareness;

  /**
   * 当前用户信息
   */
  user?: UserInfo;

  /**
   * Optional. Controls whether the client should attempt to reconnect
   * automatically if the connection is lost.
   * @default true
   */
  autoReconnect?: boolean;

  /**
   * Optional. The delay in milliseconds before attempting to reconnect.
   * @default 2000
   */
  reconnectDelay?: number;

  /**
   * Optional. Maximum number of reconnection attempts.
   * @default 5
   */
  maxReconnectAttempts?: number;

  /**
   * Optional. Timeout for initial connection in milliseconds.
   * @default 10000
   */
  connectionTimeout?: number;

  /**
   * Optional. 光标同步配置
   */
  cursor?: {
    /**
     * 是否启用光标同步
     * @default true
     */
    enabled?: boolean;
    
    /**
     * 光标位置更新的防抖延迟（毫秒）
     * @default 100
     */
    debounceDelay?: number;
    
    /**
     * 输入状态的超时时间（毫秒）
     * @default 3000
     */
    typingTimeout?: number;
    
    /**
     * 是否显示其他用户的光标
     * @default true
     */
    showOtherCursors?: boolean;
  };
}

/**
 * Represents the connection status of the client.
 */
export enum ConnectionStatus {
  Disconnected = 'disconnected',
  Connecting = 'connecting',
  Connected = 'connected',
  Reconnecting = 'reconnecting',
  Failed = 'failed',
}

/**
 * Error types that can occur during collaboration.
 */
export enum ErrorType {
  // 连接相关错误
  CONNECTION_FAILED = 'CONNECTION_FAILED',
  CONNECTION_TIMEOUT = 'CONNECTION_TIMEOUT',
  WEBSOCKET_ERROR = 'WEBSOCKET_ERROR',
  
  // 房间相关错误
  ROOM_NOT_FOUND = 'ROOM_NOT_FOUND',
  ROOM_ACCESS_DENIED = 'ROOM_ACCESS_DENIED',
  ROOM_FULL = 'ROOM_FULL',
  
  // 同步相关错误
  SYNC_FAILED = 'SYNC_FAILED',
  DATA_CORRUPTION = 'DATA_CORRUPTION',
  
  // 网络相关错误
  NETWORK_ERROR = 'NETWORK_ERROR',
  SERVER_ERROR = 'SERVER_ERROR',
  
  // 未知错误
  UNKNOWN_ERROR = 'UNKNOWN_ERROR',
}

/**
 * Detailed error information.
 */
export interface CollaborationError {
  /**
   * The type of error that occurred.
   */
  type: ErrorType;

  /**
   * Human-readable error message.
   */
  message: string;

  /**
   * Additional error details from the server.
   */
  details?: {
    code?: number;
    room_id?: string;
    server_message?: string;
    timestamp?: string;
    [key: string]: any;
  };

  /**
   * Whether this error is recoverable (can retry).
   */
  recoverable: boolean;

  /**
   * Suggested action for the user or application.
   */
  suggestedAction?: string;

  /**
   * Original error object if available.
   */
  originalError?: Error | Event | any;
}

/**
 * Retry configuration for failed operations.
 */
export interface RetryConfig {
  /**
   * Maximum number of retry attempts.
   */
  maxAttempts: number;

  /**
   * Base delay between retries in milliseconds.
   */
  baseDelay: number;

  /**
   * Multiplier for exponential backoff.
   */
  backoffMultiplier: number;

  /**
   * Maximum delay between retries in milliseconds.
   */
  maxDelay: number;
} 