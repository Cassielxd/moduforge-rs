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
      backdrop-filter: blur(2px);
    `
    
    document.body.appendChild(overlay)
  }

  // 启用窗口交互
  const enableWindowInteraction = () => {
    // 移除遮罩层
    const overlay = document.getElementById('modal-overlay')
    if (overlay) {
      overlay.remove()
    }
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
