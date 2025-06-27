import { CollaborationError, ErrorType } from './types';

export interface ErrorDisplayOptions {
  /**
   * 容器元素或选择器，用于显示错误信息
   */
  container?: HTMLElement | string;
  
  /**
   * 是否自动关闭错误提示
   */
  autoClose?: boolean;
  
  /**
   * 自动关闭延迟时间（毫秒）
   */
  autoCloseDelay?: number;
  
  /**
   * 是否显示重试按钮
   */
  showRetryButton?: boolean;
  
  /**
   * 自定义 CSS 类名
   */
  customClass?: string;
  
  /**
   * 是否使用原生 alert（用于调试）
   */
  useNativeAlert?: boolean;
}

export interface NotificationOptions {
  /**
   * 通知类型
   */
  type: 'error' | 'warning' | 'info' | 'success';
  
  /**
   * 通知标题
   */
  title: string;
  
  /**
   * 通知内容
   */
  message: string;
  
  /**
   * 是否可关闭
   */
  closable?: boolean;
  
  /**
   * 自动关闭时间（毫秒），0 表示不自动关闭
   */
  duration?: number;
  
  /**
   * 点击操作按钮的回调
   */
  onAction?: () => void;
  
  /**
   * 操作按钮文本
   */
  actionText?: string;
}

/**
 * 协作错误处理工具类
 */
export class CollaborationErrorHandler {
  private container: HTMLElement | null = null;
  private options: ErrorDisplayOptions;
  private activeNotifications: Set<HTMLElement> = new Set();

  constructor(options: ErrorDisplayOptions = {}) {
    this.options = {
      autoClose: true,
      autoCloseDelay: 5000,
      showRetryButton: true,
      useNativeAlert: false,
      ...options,
    };

    this.initContainer();
    this.injectStyles();
  }

  private initContainer(): void {
    if (this.options.container) {
      if (typeof this.options.container === 'string') {
        this.container = document.querySelector(this.options.container);
      } else {
        this.container = this.options.container;
      }
    }

    if (!this.container) {
      // 创建默认容器
      this.container = document.createElement('div');
      this.container.id = 'collaboration-error-container';
      this.container.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        z-index: 10000;
        max-width: 400px;
      `;
      document.body.appendChild(this.container);
    }
  }

  private injectStyles(): void {
    if (document.getElementById('collaboration-error-styles')) {
      return; // 样式已注入
    }

    const styles = document.createElement('style');
    styles.id = 'collaboration-error-styles';
    styles.textContent = `
      .collaboration-notification {
        background: white;
        border-radius: 8px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
        margin-bottom: 12px;
        padding: 16px;
        border-left: 4px solid #ddd;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
        font-size: 14px;
        line-height: 1.4;
        animation: slideIn 0.3s ease-out;
      }

      .collaboration-notification.error {
        border-left-color: #f56565;
        background: #fef5f5;
      }

      .collaboration-notification.warning {
        border-left-color: #ed8936;
        background: #fffaf0;
      }

      .collaboration-notification.info {
        border-left-color: #4299e1;
        background: #f0f8ff;
      }

      .collaboration-notification.success {
        border-left-color: #48bb78;
        background: #f0fff4;
      }

      .collaboration-notification-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 8px;
      }

      .collaboration-notification-title {
        font-weight: 600;
        margin: 0;
        color: #2d3748;
      }

      .collaboration-notification-close {
        background: none;
        border: none;
        font-size: 18px;
        cursor: pointer;
        color: #718096;
        padding: 0;
        margin-left: 12px;
      }

      .collaboration-notification-close:hover {
        color: #2d3748;
      }

      .collaboration-notification-message {
        color: #4a5568;
        margin-bottom: 12px;
      }

      .collaboration-notification-details {
        background: rgba(0, 0, 0, 0.05);
        border-radius: 4px;
        padding: 8px;
        font-size: 12px;
        color: #718096;
        margin-bottom: 12px;
        font-family: 'Monaco', 'Menlo', monospace;
      }

      .collaboration-notification-actions {
        display: flex;
        gap: 8px;
        align-items: center;
      }

      .collaboration-notification-button {
        background: #4299e1;
        color: white;
        border: none;
        border-radius: 4px;
        padding: 6px 12px;
        font-size: 12px;
        cursor: pointer;
        font-weight: 500;
      }

      .collaboration-notification-button:hover {
        background: #3182ce;
      }

      .collaboration-notification-button.secondary {
        background: #e2e8f0;
        color: #4a5568;
      }

      .collaboration-notification-button.secondary:hover {
        background: #cbd5e0;
      }

      .collaboration-reconnect-info {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 12px;
        color: #718096;
      }

      .collaboration-reconnect-spinner {
        width: 12px;
        height: 12px;
        border: 2px solid #e2e8f0;
        border-top-color: #4299e1;
        border-radius: 50%;
        animation: spin 1s linear infinite;
      }

      @keyframes slideIn {
        from {
          transform: translateX(100%);
          opacity: 0;
        }
        to {
          transform: translateX(0);
          opacity: 1;
        }
      }

      @keyframes slideOut {
        from {
          transform: translateX(0);
          opacity: 1;
        }
        to {
          transform: translateX(100%);
          opacity: 0;
        }
      }

      @keyframes spin {
        to {
          transform: rotate(360deg);
        }
      }
    `;
    document.head.appendChild(styles);
  }

  /**
   * 显示协作错误
   */
  public showError(error: CollaborationError, onRetry?: () => void): void {
    if (this.options.useNativeAlert) {
      alert(`${error.message}\n\n${error.suggestedAction || ''}`);
      return;
    }

    const notification = this.createNotification({
      type: 'error',
      title: this.getErrorTitle(error.type),
      message: error.message,
      closable: true,
      duration: error.recoverable ? this.options.autoCloseDelay : 0,
      onAction: error.recoverable && this.options.showRetryButton ? onRetry : undefined,
      actionText: error.recoverable ? '重试' : undefined,
    });

    // 添加详细信息
    if (error.details && Object.keys(error.details).length > 0) {
      const details = document.createElement('div');
      details.className = 'collaboration-notification-details';
      details.textContent = JSON.stringify(error.details, null, 2);
      
      const message = notification.querySelector('.collaboration-notification-message')!;
      message.insertAdjacentElement('afterend', details);
    }

    // 添加建议操作
    if (error.suggestedAction) {
      const suggestion = document.createElement('div');
      suggestion.style.cssText = 'font-size: 12px; color: #718096; margin-top: 8px;';
      suggestion.textContent = `💡 ${error.suggestedAction}`;
      
      const message = notification.querySelector('.collaboration-notification-message')!;
      message.insertAdjacentElement('afterend', suggestion);
    }

    this.showNotification(notification);
  }

  /**
   * 显示重连状态
   */
  public showReconnectStatus(attempt: number, maxAttempts: number): void {
    const existingReconnect = this.container?.querySelector('.collaboration-reconnect-notification');
    if (existingReconnect) {
      existingReconnect.remove();
    }

    const notification = this.createNotification({
      type: 'info',
      title: '正在重新连接...',
      message: `尝试重连 ${attempt}/${maxAttempts}`,
      closable: false,
      duration: 0,
    });

    notification.classList.add('collaboration-reconnect-notification');

    // 添加重连进度
    const progress = document.createElement('div');
    progress.className = 'collaboration-reconnect-info';
    progress.innerHTML = `
      <div class="collaboration-reconnect-spinner"></div>
      <span>重连进度: ${attempt}/${maxAttempts}</span>
    `;

    const message = notification.querySelector('.collaboration-notification-message')!;
    message.insertAdjacentElement('afterend', progress);

    this.showNotification(notification);
  }

  /**
   * 显示连接成功消息
   */
  public showConnectionSuccess(): void {
    // 移除重连通知
    const reconnectNotification = this.container?.querySelector('.collaboration-reconnect-notification');
    if (reconnectNotification) {
      this.removeNotification(reconnectNotification as HTMLElement);
    }

    const notification = this.createNotification({
      type: 'success',
      title: '连接成功',
      message: '已成功连接到协作服务器',
      closable: true,
      duration: 3000,
    });

    this.showNotification(notification);
  }

  /**
   * 显示一般通知
   */
  public showNotification(notification: HTMLElement): void {
    if (!this.container) return;

    this.container.appendChild(notification);
    this.activeNotifications.add(notification);

    // 自动关闭
    const duration = parseInt(notification.dataset.duration || '0');
    if (duration > 0) {
      setTimeout(() => {
        this.removeNotification(notification);
      }, duration);
    }
  }

  /**
   * 创建通知元素
   */
  private createNotification(options: NotificationOptions): HTMLElement {
    const notification = document.createElement('div');
    notification.className = `collaboration-notification ${options.type} ${this.options.customClass || ''}`;
    notification.dataset.duration = String(options.duration || 0);

    const header = document.createElement('div');
    header.className = 'collaboration-notification-header';

    const title = document.createElement('h4');
    title.className = 'collaboration-notification-title';
    title.textContent = options.title;
    header.appendChild(title);

    if (options.closable !== false) {
      const closeBtn = document.createElement('button');
      closeBtn.className = 'collaboration-notification-close';
      closeBtn.innerHTML = '×';
      closeBtn.onclick = () => this.removeNotification(notification);
      header.appendChild(closeBtn);
    }

    const message = document.createElement('div');
    message.className = 'collaboration-notification-message';
    message.textContent = options.message;

    notification.appendChild(header);
    notification.appendChild(message);

    // 添加操作按钮
    if (options.onAction && options.actionText) {
      const actions = document.createElement('div');
      actions.className = 'collaboration-notification-actions';

      const actionBtn = document.createElement('button');
      actionBtn.className = 'collaboration-notification-button';
      actionBtn.textContent = options.actionText;
      actionBtn.onclick = () => {
        options.onAction!();
        this.removeNotification(notification);
      };

      actions.appendChild(actionBtn);
      notification.appendChild(actions);
    }

    return notification;
  }

  /**
   * 移除通知
   */
  public removeNotification(notification: HTMLElement): void {
    if (!this.activeNotifications.has(notification)) return;

    notification.style.animation = 'slideOut 0.3s ease-in forwards';
    setTimeout(() => {
      if (notification.parentNode) {
        notification.parentNode.removeChild(notification);
      }
      this.activeNotifications.delete(notification);
    }, 300);
  }

  /**
   * 清除所有通知
   */
  public clearAll(): void {
    this.activeNotifications.forEach(notification => {
      this.removeNotification(notification);
    });
  }

  /**
   * 获取错误类型对应的标题
   */
  private getErrorTitle(errorType: ErrorType): string {
    const titles: Record<ErrorType, string> = {
      [ErrorType.CONNECTION_FAILED]: '连接失败',
      [ErrorType.CONNECTION_TIMEOUT]: '连接超时',
      [ErrorType.WEBSOCKET_ERROR]: 'WebSocket 错误',
      [ErrorType.ROOM_NOT_FOUND]: '房间不存在',
      [ErrorType.ROOM_ACCESS_DENIED]: '访问被拒绝',
      [ErrorType.ROOM_FULL]: '房间已满',
      [ErrorType.SYNC_FAILED]: '同步失败',
      [ErrorType.DATA_CORRUPTION]: '数据损坏',
      [ErrorType.NETWORK_ERROR]: '网络错误',
      [ErrorType.SERVER_ERROR]: '服务器错误',
      [ErrorType.UNKNOWN_ERROR]: '未知错误',
    };

    return titles[errorType] || '连接错误';
  }

  /**
   * 销毁错误处理器
   */
  public destroy(): void {
    this.clearAll();
    
    if (this.container && this.container.id === 'collaboration-error-container') {
      this.container.remove();
    }

    const styles = document.getElementById('collaboration-error-styles');
    if (styles) {
      styles.remove();
    }
  }
} 