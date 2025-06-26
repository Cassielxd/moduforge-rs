import * as Y from 'yjs';

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
}

// --- Incoming and Outgoing WebSocket Messages ---

export type ClientMessage =
  | { JoinRoom: { room_id: string } }
  | { LeaveRoom: { room_id: string } }
  | { YrsUpdate: { room_id: string; update: number[] } }
  | { YrsSyncRequest: { room_id: string; state_vector: number[] } }
  | { StateSync: { room_id: string; operation: string; data: any; timestamp: number } }
  | { Ping: {} };

export type ServerMessage =
  | { YrsUpdate: { room_id: string, update: number[] } }
  | { Error: { message: string } }
  | { Notification: { message: string } }
  | { Pong: {} };

/**
 * Represents the connection status of the client.
 */
export enum ConnectionStatus {
  Disconnected = 'disconnected',
  Connecting = 'connecting',
  Connected = 'connected',
  Reconnecting = 'reconnecting',
} 