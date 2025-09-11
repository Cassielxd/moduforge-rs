import { ref, readonly, onMounted } from 'vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { invoke } from '@tauri-apps/api/core'

/**
 * 简化的窗口管理 Composable
 * 基于概算应用的成功实现，提供稳定可靠的窗口管理功能
 */
export function useSimpleWindowManagement(options = {}) {
  const {
    isMainWindow = false,  // 是否为主窗口（影响关闭和最小化行为）
    autoInit = true        // 是否自动初始化
  } = options

  // 状态
  const currentWindow = ref(null)
  const isMaximized = ref(false)
  const isReady = ref(false)
  const windowLabel = ref('')

  // 初始化窗口管理
  const initializeWindow = async () => {
    try {
      console.log('初始化窗口管理...')
      
      currentWindow.value = getCurrentWebviewWindow()
      if (!currentWindow.value) {
        throw new Error('无法获取窗口对象')
      }

      windowLabel.value = currentWindow.value.label || 'unknown'
      isMaximized.value = await currentWindow.value.isMaximized() || false
      
      console.log('窗口初始化成功:', {
        label: windowLabel.value,
        isMaximized: isMaximized.value
      })

      // 监听窗口状态变化
      const unlisten = await currentWindow.value.listen('tauri://resize', async () => {
        try {
          isMaximized.value = await currentWindow.value.isMaximized()
        } catch (error) {
          console.warn('更新窗口状态失败:', error)
        }
      })

      isReady.value = true
      return unlisten
    } catch (error) {
      console.error('窗口初始化失败:', error)
      throw error
    }
  }

  // 最小化窗口
  const minimizeWindow = async () => {
    try {
      if (!currentWindow.value) {
        throw new Error('窗口未初始化')
      }

      if (isMainWindow) {
        // 主窗口：使用后端命令处理子窗口联动
        await invoke('minimize_window_with_children', {
          windowId: windowLabel.value
        })
      } else {
        // 子窗口：直接最小化
        await currentWindow.value.minimize()
      }

      console.log('窗口已最小化')
    } catch (error) {
      console.error('最小化失败:', error)
      throw error
    }
  }

  // 最大化/还原窗口
  const toggleMaximize = async () => {
    try {
      if (!currentWindow.value) {
        throw new Error('窗口未初始化')
      }

      if (isMaximized.value) {
        await currentWindow.value.unmaximize()
        console.log('窗口已还原')
      } else {
        await currentWindow.value.maximize()
        console.log('窗口已最大化')
      }

      // 更新状态
      isMaximized.value = await currentWindow.value.isMaximized()
    } catch (error) {
      console.error('切换最大化状态失败:', error)
      throw error
    }
  }

  // 关闭窗口
  const closeWindow = async () => {
    try {
      if (!currentWindow.value) {
        throw new Error('窗口未初始化')
      }

      if (isMainWindow) {
        // 主窗口：使用后端命令处理子窗口联动
        await invoke('close_window_with_children', {
          windowId: windowLabel.value
        })
      } else {
        // 子窗口：直接关闭
        await currentWindow.value.close()
      }

      console.log('窗口已关闭')
    } catch (error) {
      console.error('关闭窗口失败:', error)
      throw error
    }
  }

  // 显示窗口
  const showWindow = async () => {
    try {
      if (currentWindow.value) {
        await currentWindow.value.show()
        await currentWindow.value.setFocus()
      }
    } catch (error) {
      console.error('显示窗口失败:', error)
      throw error
    }
  }

  // 隐藏窗口
  const hideWindow = async () => {
    try {
      if (currentWindow.value) {
        await currentWindow.value.hide()
      }
    } catch (error) {
      console.error('隐藏窗口失败:', error)
      throw error
    }
  }

  // 自动初始化
  if (autoInit) {
    onMounted(async () => {
      await initializeWindow()
    })
  }

  return {
    // 状态
    currentWindow: readonly(currentWindow),
    isMaximized: readonly(isMaximized),
    isReady: readonly(isReady),
    windowLabel: readonly(windowLabel),

    // 方法
    initializeWindow,
    minimizeWindow,
    toggleMaximize,
    closeWindow,
    showWindow,
    hideWindow
  }
}

/**
 * 主窗口管理 (包含子窗口联动)
 */
export function useMainWindowManagement(options = {}) {
  return useSimpleWindowManagement({
    isMainWindow: true,
    ...options
  })
}

/**
 * 子窗口管理
 */
export function useChildWindowManagement(options = {}) {
  return useSimpleWindowManagement({
    isMainWindow: false,
    ...options
  })
}