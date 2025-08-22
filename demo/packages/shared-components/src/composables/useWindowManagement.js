import { ref, computed, onMounted, onUnmounted, nextTick, readonly } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

/**
 * 完整的窗口管理系统
 * 支持父子窗口关系、模态/非模态窗口、联动操作
 */
export function useWindowManagement(options = {}) {
  const {
    isChildWindow = false,
    autoInitialize = true
  } = options

  // 状态管理
  const currentWindow = ref(null)
  const windowLabel = ref('')
  const isMaximized = ref(false)
  const isMinimized = ref(false)
  const isModal = ref(false)
  const parentWindowId = ref(null)
  const childWindows = ref([])
  const windowReady = ref(false)

  // 事件监听器存储
  let eventUnlisteners = []

  // 检测是否在 Tauri 环境中
  const isTauri = computed(() => {
    return typeof window !== 'undefined' && window.__TAURI__
  })

  // 窗口信息
  const windowInfo = computed(() => ({
    label: windowLabel.value,
    isMaximized: isMaximized.value,
    isMinimized: isMinimized.value,
    isModal: isModal.value,
    parentWindowId: parentWindowId.value,
    childWindows: childWindows.value,
    isReady: windowReady.value
  }))

  /**
   * 初始化窗口管理
   */
  const initializeWindow = async () => {
    if (!isTauri.value) {
      console.warn('不在 Tauri 环境中，跳过窗口管理初始化')
      return
    }

    try {
      currentWindow.value = getCurrentWebviewWindow()
      windowLabel.value = currentWindow.value.label
      
      console.log(`初始化窗口管理: ${windowLabel.value}`)

      // 获取初始状态
      isMaximized.value = await currentWindow.value.isMaximized()
      
      // 如果是子窗口，获取父窗口信息
      if (isChildWindow) {
        try {
          const { invoke } = await import('@tauri-apps/api/core')
          const parentId = await invoke('get_parent_window', {
            windowId: windowLabel.value
          })
          parentWindowId.value = parentId
          console.log(`父窗口ID: ${parentId}`)
        } catch (error) {
          console.error('获取父窗口ID失败:', error)
        }
      }
      
      // 设置事件监听
      await setupEventListeners()
      
      // 如果是子窗口，设置模态监听
      if (isChildWindow) {
        await setupModalListeners()
      }

      windowReady.value = true
      console.log(`窗口管理初始化完成: ${windowLabel.value}`)
    } catch (error) {
      console.error('初始化窗口管理失败:', error)
    }
  }

  /**
   * 设置事件监听器
   */
  const setupEventListeners = async () => {
    if (!currentWindow.value) return

    try {
      // 窗口大小变化监听
      const unlistenResize = await currentWindow.value.listen('tauri://resize', async () => {
        const maximized = await currentWindow.value.isMaximized()
        isMaximized.value = maximized
      })

      // 窗口最小化监听
      const unlistenMinimize = await currentWindow.value.listen('tauri://minimize', () => {
        isMinimized.value = true
        console.log(`窗口已最小化: ${windowLabel.value}`)
      })

      // 窗口恢复监听
      const unlistenRestore = await currentWindow.value.listen('tauri://focus', () => {
        if (isMinimized.value) {
          isMinimized.value = false
          console.log(`窗口已恢复: ${windowLabel.value}`)
        }
      })

      // 窗口关闭监听
      const unlistenClose = await currentWindow.value.listen('tauri://close-requested', (event) => {
        console.log(`窗口准备关闭: ${windowLabel.value}`)
        handleWindowClose(event)
      })

      eventUnlisteners.push(unlistenResize, unlistenMinimize, unlistenRestore, unlistenClose)
    } catch (error) {
      console.error('设置事件监听器失败:', error)
    }
  }

  /**
   * 设置模态窗口监听器（用于子窗口）
   */
  const setupModalListeners = async () => {
    if (!currentWindow.value) return

    try {
      // 监听父窗口禁用事件
      const unlistenDisabled = await currentWindow.value.listen('window-disabled', () => {
        console.log(`父窗口已禁用: ${parentWindowId.value}`)
      })

      // 监听父窗口启用事件
      const unlistenEnabled = await currentWindow.value.listen('window-enabled', () => {
        console.log(`父窗口已启用: ${parentWindowId.value}`)
      })

      eventUnlisteners.push(unlistenDisabled, unlistenEnabled)
    } catch (error) {
      console.error('设置模态监听器失败:', error)
    }
  }

  /**
   * 处理窗口关闭事件
   */
  const handleWindowClose = async (event) => {
    try {
      if (!isChildWindow) {
        // 主窗口关闭时，使用后端统一处理
        await invoke('close_window_with_children', { 
          windowId: windowLabel.value 
        })
      } else {
        // 子窗口关闭时，通知父窗口
        if (parentWindowId.value) {
          await invoke('close_child_window', {
            windowId: windowLabel.value,
            parentWindow: parentWindowId.value
          })
        }
      }
    } catch (error) {
      console.error('处理窗口关闭失败:', error)
    }
  }

  /**
   * 创建子窗口
   */
  const createChildWindow = async (options) => {
    try {
      const {
        windowId,
        title,
        url,
        modal = false,
        width = 800,
        height = 600,
        ...otherOptions
      } = options

      console.log(`创建子窗口: ${windowId}`, { modal, parent: windowLabel.value })

      await invoke('create_child_window', {
        windowId,
        title,
        url,
        modal,
        width,
        height,
        parentWindow: windowLabel.value,
        ...otherOptions
      })

      // 添加到子窗口列表
      if (!childWindows.value.includes(windowId)) {
        childWindows.value.push(windowId)
      }

      console.log(`子窗口创建成功: ${windowId}`)
      return windowId
    } catch (error) {
      console.error('创建子窗口失败:', error)
      throw error
    }
  }

  /**
   * 创建模态对话框
   */
  const createModalDialog = async (options) => {
    return await createChildWindow({
      ...options,
      modal: true,
      width: options.width || 600,
      height: options.height || 400
    })
  }

  /**
   * 创建非模态工具窗口
   */
  const createToolWindow = async (options) => {
    return await createChildWindow({
      ...options,
      modal: false,
      width: options.width || 800,
      height: options.height || 600
    })
  }

  /**
   * 关闭子窗口
   */
  const closeChildWindow = async (windowId) => {
    try {
      await invoke('close_child_window', {
        windowId,
        parentWindow: windowLabel.value
      })

      // 从子窗口列表中移除
      childWindows.value = childWindows.value.filter(id => id !== windowId)

      console.log(`子窗口已关闭: ${windowId}`)
    } catch (error) {
      console.error('关闭子窗口失败:', error)
      throw error
    }
  }

  /**
   * 最小化当前窗口及其子窗口
   */
  const minimizeWindow = async () => {
    try {
      if (!currentWindow.value) {
        throw new Error('当前窗口对象未初始化')
      }

      if (isChildWindow) {
        // 子窗口直接最小化
        await currentWindow.value.minimize()
      } else {
        // 主窗口使用后端统一处理
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('minimize_window_with_children', { 
          windowId: windowLabel.value 
        })
      }
      
      isMinimized.value = true
      console.log(`窗口已最小化: ${windowLabel.value}`)
    } catch (error) {
      console.error('最小化窗口失败:', error)
      throw error
    }
  }

  /**
   * 恢复当前窗口及其子窗口
   */
  const restoreWindow = async () => {
    try {
      if (isChildWindow) {
        // 子窗口直接恢复
        await currentWindow.value.unminimize()
      } else {
        // 主窗口使用后端统一处理
        await invoke('restore_window_with_children', { 
          windowId: windowLabel.value 
        })
      }

      isMinimized.value = false
      console.log(`窗口已恢复: ${windowLabel.value}`)
    } catch (error) {
      console.error('恢复窗口失败:', error)
      throw error
    }
  }

  /**
   * 最大化/还原窗口
   */
  const toggleMaximize = async () => {
    try {
      if (!currentWindow.value) {
        throw new Error('当前窗口对象未初始化')
      }

      if (isMaximized.value) {
        await currentWindow.value.unmaximize()
      } else {
        await currentWindow.value.maximize()
      }
      
      isMaximized.value = !isMaximized.value
      console.log(`窗口最大化状态: ${isMaximized.value ? '最大化' : '还原'}`)
    } catch (error) {
      console.error('切换窗口最大化状态失败:', error)
      throw error
    }
  }

  /**
   * 关闭当前窗口
   */
  const closeWindow = async () => {
    try {
      if (!currentWindow.value) {
        throw new Error('当前窗口对象未初始化')
      }

      if (isChildWindow) {
        await currentWindow.value.close()
      } else {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('close_window_with_children', { 
          windowId: windowLabel.value 
        })
      }
      
      console.log(`窗口已关闭: ${windowLabel.value}`)
    } catch (error) {
      console.error('关闭窗口失败:', error)
      throw error
    }
  }

  /**
   * 设置窗口焦点
   */
  const focusWindow = async () => {
    try {
      await currentWindow.value.setFocus()
    } catch (error) {
      console.error('设置窗口焦点失败:', error)
    }
  }

  /**
   * 显示窗口
   */
  const showWindow = async () => {
    try {
      await currentWindow.value.show()
    } catch (error) {
      console.error('显示窗口失败:', error)
    }
  }

  /**
   * 隐藏窗口
   */
  const hideWindow = async () => {
    try {
      await currentWindow.value.hide()
    } catch (error) {
      console.error('隐藏窗口失败:', error)
    }
  }

  /**
   * 清理资源
   */
  const cleanup = () => {
    eventUnlisteners.forEach(unlisten => {
      try {
        unlisten()
      } catch (error) {
        console.warn('清理事件监听器失败:', error)
      }
    })
    eventUnlisteners = []
  }

  // 生命周期管理
  if (autoInitialize) {
    onMounted(async () => {
      await nextTick()
      await initializeWindow()
    })
  }

  onUnmounted(() => {
    cleanup()
  })

  // 返回 API
  return {
    // 状态
    currentWindow: readonly(currentWindow),
    windowLabel: readonly(windowLabel),
    isMaximized: readonly(isMaximized),
    isMinimized: readonly(isMinimized),
    isModal: readonly(isModal),
    parentWindowId: readonly(parentWindowId),
    childWindows: readonly(childWindows),
    windowReady: readonly(windowReady),
    windowInfo,
    isTauri,

    // 方法
    initializeWindow,
    createChildWindow,
    createModalDialog,
    createToolWindow,
    closeChildWindow,
    minimizeWindow,
    restoreWindow,
    toggleMaximize,
    closeWindow,
    focusWindow,
    showWindow,
    hideWindow,
    cleanup
  }
}

/**
 * 子窗口专用的窗口管理
 */
export function useChildWindowManagement() {
  return useWindowManagement({ 
    isChildWindow: true,
    autoInitialize: true
  })
}

/**
 * 主窗口专用的窗口管理
 */
export function useMainWindowManagement() {
  return useWindowManagement({ 
    isChildWindow: false,
    autoInitialize: true
  })
}