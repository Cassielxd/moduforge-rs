import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

/**
 * 窗口模态状态管理组合式函数
 * 用于处理模态窗口的禁用/启用状态
 */
export function useWindowModal() {
  const isWindowDisabled = ref(false)
  const currentWindow = ref(null)
  let unlistenDisabled = null
  let unlistenEnabled = null

  // 初始化窗口监听
  const initWindowListeners = async () => {
    try {
      currentWindow.value = getCurrentWebviewWindow()
      
      // 监听窗口禁用事件
      unlistenDisabled = await currentWindow.value.listen('window-disabled', (event) => {
        console.log('窗口被禁用:', event.payload)
        isWindowDisabled.value = true
        disableWindowInteraction()
      })
      
      // 监听窗口启用事件
      unlistenEnabled = await currentWindow.value.listen('window-enabled', (event) => {
        console.log('窗口被启用:', event.payload)
        isWindowDisabled.value = false
        enableWindowInteraction()
      })
      
    } catch (error) {
      console.error('初始化窗口监听失败:', error)
    }
  }

  // 禁用窗口交互
  const disableWindowInteraction = () => {
    // 添加遮罩层
    const overlay = document.createElement('div')
    overlay.id = 'modal-overlay'
    overlay.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background-color: rgba(0, 0, 0, 0.3);
      z-index: 9999;
      cursor: not-allowed;
      backdrop-filter: blur(2px);
    `
    
    // 添加提示信息
    const message = document.createElement('div')
    message.style.cssText = `
      position: absolute;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      background: white;
      padding: 20px 30px;
      border-radius: 8px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
      font-size: 14px;
      color: #666;
      text-align: center;
      max-width: 300px;
    `
    message.innerHTML = `
      <div style="margin-bottom: 8px; font-size: 16px; color: #1890ff;">🔒 窗口已锁定</div>
      <div>请先关闭模态窗口才能继续操作</div>
    `
    
    overlay.appendChild(message)
    document.body.appendChild(overlay)
    
    // 禁用所有交互元素
    disableInteractiveElements()
  }

  // 启用窗口交互
  const enableWindowInteraction = () => {
    // 移除遮罩层
    const overlay = document.getElementById('modal-overlay')
    if (overlay) {
      overlay.remove()
    }
    
    // 启用所有交互元素
    enableInteractiveElements()
  }

  // 禁用交互元素
  const disableInteractiveElements = () => {
    const elements = document.querySelectorAll('button, input, select, textarea, a, [tabindex]')
    elements.forEach(element => {
      if (!element.hasAttribute('data-modal-disabled')) {
        element.setAttribute('data-modal-disabled', 'true')
        element.setAttribute('tabindex', '-1')
        element.style.pointerEvents = 'none'
        element.style.opacity = '0.6'
      }
    })
  }

  // 启用交互元素
  const enableInteractiveElements = () => {
    const elements = document.querySelectorAll('[data-modal-disabled]')
    elements.forEach(element => {
      element.removeAttribute('data-modal-disabled')
      element.removeAttribute('tabindex')
      element.style.pointerEvents = ''
      element.style.opacity = ''
    })
  }

  // 手动设置窗口状态
  const setWindowDisabled = (disabled) => {
    isWindowDisabled.value = disabled
    if (disabled) {
      disableWindowInteraction()
    } else {
      enableWindowInteraction()
    }
  }

  // 清理监听器
  const cleanup = () => {
    if (unlistenDisabled) {
      unlistenDisabled()
      unlistenDisabled = null
    }
    if (unlistenEnabled) {
      unlistenEnabled()
      unlistenEnabled = null
    }
    
    // 清理遮罩层
    const overlay = document.getElementById('modal-overlay')
    if (overlay) {
      overlay.remove()
    }
    
    // 恢复所有元素状态
    enableInteractiveElements()
  }

  // 组件挂载时初始化
  onMounted(() => {
    initWindowListeners()
  })

  // 组件卸载时清理
  onUnmounted(() => {
    cleanup()
  })

  return {
    isWindowDisabled,
    currentWindow,
    setWindowDisabled,
    cleanup,
    initWindowListeners
  }
}

/**
 * 简化版本的窗口模态管理
 * 只提供基本的禁用/启用功能
 */
export function useSimpleWindowModal() {
  const isDisabled = ref(false)

  const disable = () => {
    isDisabled.value = true
    document.body.style.pointerEvents = 'none'
    document.body.style.opacity = '0.6'
  }

  const enable = () => {
    isDisabled.value = false
    document.body.style.pointerEvents = ''
    document.body.style.opacity = ''
  }

  return {
    isDisabled,
    disable,
    enable
  }
}
