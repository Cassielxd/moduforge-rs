import * as Y from 'yjs';
import {
  CollaborationClientOptions,
  ConnectionStatus,
  ClientMessage,
  ServerMessage,
} from './types';

// Use a polyfill for WebSocket if it's not available (e.g., in Node.js environment)
const WebSocket = globalThis.WebSocket;

type EventMap = {
  'status': (status: ConnectionStatus) => void;
  'synced': (synced: boolean) => void;
  'error': (error: Error) => void;
};

export class CollaborationClient {
  public readonly doc: Y.Doc;
  private ws: WebSocket | null = null;
  private readonly url: string;
  private readonly room: string;
  private status: ConnectionStatus = ConnectionStatus.Disconnected;
  private synced: boolean = false;
  private readonly autoReconnect: boolean;
  private readonly reconnectDelay: number;
  private reconnectTimeoutId: any = null;
  private listeners: { [K in keyof EventMap]: EventMap[K][] } = {
    status: [],
    synced: [],
    error: [],
  };

  constructor(options: CollaborationClientOptions) {
    if (!WebSocket) {
        throw new Error(
            'WebSocket is not available in this environment. Please polyfill it.'
        );
    }

    this.doc = options.doc;
    this.url = options.url;
    this.room = options.room;
    this.autoReconnect = options.autoReconnect ?? true;
    this.reconnectDelay = options.reconnectDelay ?? 2000;

    this.doc.on('update', this.handleDocUpdate);
  }

  public connect(): void {
    if (this.ws || this.status === ConnectionStatus.Connecting) {
      return;
    }

    this.setStatus(ConnectionStatus.Connecting);
    if (this.reconnectTimeoutId) {
      clearTimeout(this.reconnectTimeoutId);
      this.reconnectTimeoutId = null;
    }

    this.ws = new WebSocket(this.url);
    this.ws.binaryType = 'arraybuffer';
    this.ws.onopen = this.handleOpen;
    this.ws.onmessage = this.handleMessage;
    this.ws.onclose = this.handleClose;
    this.ws.onerror = this.handleError;
  }

  public disconnect(): void {
    if (this.reconnectTimeoutId) {
      clearTimeout(this.reconnectTimeoutId);
      this.reconnectTimeoutId = null;
    }
    
    if (this.ws) {
        // Send leave message before closing
        this.sendMessage({ LeaveRoom: { room_id: this.room } });
        this.ws.close();
    }
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

  private setSynced(synced: boolean): void {
    if (this.synced !== synced) {
      this.synced = synced;
      this.emit('synced', this.synced);
    }
  }

  private sendMessage(message: ClientMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  private handleOpen = (): void => {
    this.setStatus(ConnectionStatus.Connected);
    this.sendMessage({ JoinRoom: { room_id: this.room } });

    const stateVector = Y.encodeStateVector(this.doc);
    this.sendMessage({
      YrsSyncRequest: {
        room_id: this.room,
        state_vector: Array.from(stateVector),
      },
    });
  };

  private handleMessage = (event: MessageEvent): void => {
    try {
        if (event.data instanceof ArrayBuffer) {
            // Yjs updates from the backend are expected to be binary
            Y.applyUpdate(this.doc, new Uint8Array(event.data), this);
            this.setSynced(true);
        } else if (typeof event.data === 'string') {
            const message: ServerMessage = JSON.parse(event.data);
            if ('Error' in message) {
                this.emit('error', new Error(message.Error.message));
            } else if ('Notification' in message) {
                // Handle notifications if needed
            } else if('YrsUpdate' in message) {
                // Handle JSON-wrapped binary update if backend sends it
                Y.applyUpdate(this.doc, new Uint8Array(message.YrsUpdate.update), this);
                this.setSynced(true);
            }
        }
    } catch (error) {
        this.emit('error', error as Error);
    }
  };

  private handleError = (event: Event): void => {
    const error = new Error('WebSocket error');
    this.emit('error', error);
  };

  private handleClose = (): void => {
    this.ws = null;
    this.setSynced(false);

    if (this.status !== ConnectionStatus.Disconnected && this.autoReconnect) {
      this.setStatus(ConnectionStatus.Reconnecting);
      this.reconnectTimeoutId = setTimeout(() => {
        this.connect();
      }, this.reconnectDelay);
    } else {
      this.setStatus(ConnectionStatus.Disconnected);
    }
  };

  private handleDocUpdate = (update: Uint8Array, origin: any): void => {
    if (origin !== this) {
      this.sendMessage({
        YrsUpdate: {
          room_id: this.room,
          update: Array.from(update),
        },
      });
    }
  };

  public destroy(): void {
    this.disconnect();
    this.doc.off('update', this.handleDocUpdate);
    // Clear all listeners
    this.listeners = { status: [], synced: [], error: [] };
  }
} 