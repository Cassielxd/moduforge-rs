import { ref, readonly, onMounted, onUnmounted } from 'vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

/**
 * 窗口间数据交换
 * 用于父子窗口之间的数据通信
 */
export function useWindowDataExchange(options = {}) {
  const {
    windowId = null,
    isChildWindow = false
  } = options

  const currentWindow = ref(null)
  const receivedData = ref({})
  const eventHandlers = ref(new Map())
  
  // 事件监听器清理函数
  let unlisteners = []

  /**
   * 初始化数据交换
   */
  const initialize = async () => {
    try {
      currentWindow.value = getCurrentWebviewWindow()
      
      // 设置通用数据交换监听
      const unlistenDataExchange = await currentWindow.value.listen('data-exchange', (event) => {
        console.log('收到数据交换事件:', event.payload)
        
        const { type, data, from, to } = event.payload
        
        // 如果指定了接收窗口且不是当前窗口，则忽略
        if (to && to !== currentWindow.value.label) {
          return
        }
        
        // 更新接收到的数据
        receivedData.value = {
          ...receivedData.value,
          [type]: data,
          lastUpdate: Date.now(),
          from
        }
        
        // 触发对应的事件处理器
        if (eventHandlers.value.has(type)) {
          const handler = eventHandlers.value.get(type)
          handler(data, from)
        }
      })

      unlisteners.push(unlistenDataExchange)

      console.log(`数据交换初始化完成: ${currentWindow.value.label}`)
    } catch (error) {
      console.error('初始化窗口数据交换失败:', error)
    }
  }

  /**
   * 向指定窗口发送数据
   */
  const sendDataToWindow = async (targetWindowId, type, data) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      
      await invoke('send_window_message', {
        targetWindowId,
        payload: {
          type,
          data,
          from: currentWindow.value?.label,
          to: targetWindowId,
          timestamp: Date.now()
        }
      })

      console.log(`数据已发送到窗口 ${targetWindowId}:`, { type, data })
    } catch (error) {
      console.error('发送数据失败:', error)
      throw error
    }
  }

  /**
   * 向父窗口发送数据
   */
  const sendDataToParent = async (type, data) => {
    try {
      if (!isChildWindow) {
        console.warn('当前不是子窗口，无法向父窗口发送数据')
        return
      }

      // 通过后端获取父窗口ID
      const { invoke } = await import('@tauri-apps/api/core')
      const parentWindowId = await invoke('get_parent_window', {
        windowId: currentWindow.value?.label
      })

      if (parentWindowId) {
        await sendDataToWindow(parentWindowId, type, data)
      } else {
        console.warn('未找到父窗口')
      }
    } catch (error) {
      console.error('向父窗口发送数据失败:', error)
      throw error
    }
  }

  /**
   * 向所有子窗口广播数据
   */
  const broadcastToChildren = async (type, data) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      
      await invoke('broadcast_to_children', {
        windowId: currentWindow.value?.label,
        payload: {
          type,
          data,
          from: currentWindow.value?.label,
          timestamp: Date.now()
        }
      })

      console.log('数据已广播到所有子窗口:', { type, data })
    } catch (error) {
      console.error('广播数据失败:', error)
      throw error
    }
  }

  /**
   * 注册数据类型的事件处理器
   */
  const onDataReceived = (type, handler) => {
    eventHandlers.value.set(type, handler)
    
    // 返回取消注册的函数
    return () => {
      eventHandlers.value.delete(type)
    }
  }

  /**
   * 注册表单数据更新处理器
   */
  const onFormDataUpdate = (handler) => {
    return onDataReceived('form-update', handler)
  }

  /**
   * 注册表单提交处理器
   */
  const onFormSubmit = (handler) => {
    return onDataReceived('form-submit', handler)
  }

  /**
   * 注册数据刷新请求处理器
   */
  const onDataRefreshRequest = (handler) => {
    return onDataReceived('refresh-request', handler)
  }

  /**
   * 发送表单数据更新
   */
  const sendFormUpdate = async (formData, targetWindow = null) => {
    if (isChildWindow && !targetWindow) {
      await sendDataToParent('form-update', formData)
    } else if (targetWindow) {
      await sendDataToWindow(targetWindow, 'form-update', formData)
    } else {
      await broadcastToChildren('form-update', formData)
    }
  }

  /**
   * 发送表单提交事件
   */
  const sendFormSubmit = async (formData, targetWindow = null) => {
    if (isChildWindow && !targetWindow) {
      await sendDataToParent('form-submit', formData)
    } else if (targetWindow) {
      await sendDataToWindow(targetWindow, 'form-submit', formData)
    } else {
      await broadcastToChildren('form-submit', formData)
    }
  }

  /**
   * 请求数据刷新
   */
  const requestDataRefresh = async (targetWindow = null) => {
    const refreshData = {
      requestId: `refresh-${Date.now()}`,
      windowId: currentWindow.value?.label
    }

    if (isChildWindow && !targetWindow) {
      await sendDataToParent('refresh-request', refreshData)
    } else if (targetWindow) {
      await sendDataToWindow(targetWindow, 'refresh-request', refreshData)
    } else {
      await broadcastToChildren('refresh-request', refreshData)
    }
  }

  /**
   * 获取接收到的数据
   */
  const getReceivedData = (type = null) => {
    if (type) {
      return receivedData.value[type] || null
    }
    return receivedData.value
  }

  /**
   * 清理资源
   */
  const cleanup = () => {
    unlisteners.forEach(unlisten => {
      try {
        unlisten()
      } catch (error) {
        console.warn('清理数据交换监听器失败:', error)
      }
    })
    unlisteners = []
    eventHandlers.value.clear()
  }

  // 生命周期管理
  onMounted(async () => {
    await initialize()
  })

  onUnmounted(() => {
    cleanup()
  })

  return {
    // 状态
    currentWindow: readonly(currentWindow),
    receivedData: readonly(receivedData),

    // 基础方法
    initialize,
    sendDataToWindow,
    sendDataToParent,
    broadcastToChildren,
    onDataReceived,

    // 便捷方法
    onFormDataUpdate,
    onFormSubmit,
    onDataRefreshRequest,
    sendFormUpdate,
    sendFormSubmit,
    requestDataRefresh,

    // 工具方法
    getReceivedData,
    cleanup
  }
}

/**
 * 子窗口数据交换
 */
export function useChildWindowDataExchange() {
  return useWindowDataExchange({ isChildWindow: true })
}

/**
 * 父窗口数据交换
 */
export function useParentWindowDataExchange() {
  return useWindowDataExchange({ isChildWindow: false })
}