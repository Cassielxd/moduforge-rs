import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

/**
 * 窗体管理组合式函数
 * 提供完整的父子窗体关系管理功能
 */
export function useWindowManager() {
  const currentWindow = ref(null)
  const isMinimized = ref(false)
  const children = ref([])
  
  // 初始化窗体管理器
  const initWindowManager = async () => {
    try {
      currentWindow.value = getCurrentWebviewWindow()
      
      // 监听窗口最小化事件
      const unlistenMinimize = await currentWindow.value.listen('tauri://minimize', () => {
        console.log('窗口最小化')
        isMinimized.value = true
        handleWindowMinimize()
      })
      
      // 监听窗口恢复事件
      const unlistenRestore = await currentWindow.value.listen('tauri://focus', () => {
        console.log('窗口恢复焦点')
        if (isMinimized.value) {
          isMinimized.value = false
          handleWindowRestore()
        }
      })
      
      // 监听窗口关闭事件
      const unlistenClose = await currentWindow.value.listen('tauri://close-requested', () => {
        console.log('窗口即将关闭')
        handleWindowClose()
      })
      
      return () => {
        unlistenMinimize()
        unlistenRestore()
        unlistenClose()
      }
    } catch (error) {
      console.error('初始化窗体管理器失败:', error)
    }
  }
  
  // 处理窗口最小化
  const handleWindowMinimize = async () => {
    try {
      const windowLabel = currentWindow.value?.label || 'main'
      await invoke('minimize_window_with_children', { windowId: windowLabel })
    } catch (error) {
      console.error('处理窗口最小化失败:', error)
    }
  }
  
  // 处理窗口恢复
  const handleWindowRestore = async () => {
    try {
      const windowLabel = currentWindow.value?.label || 'main'
      await invoke('restore_window_with_children', { windowId: windowLabel })
    } catch (error) {
      console.error('处理窗口恢复失败:', error)
    }
  }
  
  // 处理窗口关闭
  const handleWindowClose = async () => {
    try {
      const windowLabel = currentWindow.value?.label || 'main'
      await invoke('close_window_with_children', { windowId: windowLabel })
    } catch (error) {
      console.error('处理窗口关闭失败:', error)
    }
  }
  
  // 创建子窗口
  const createChildWindow = async (options) => {
    try {
      const {
        windowId,
        title,
        url,
        modal = false,
        width = 800,
        height = 600,
        parentWindow = null
      } = options
      
      await invoke('create_child_window', {
        windowId,
        title,
        url,
        modal,
        width,
        height,
        parentWindow: parentWindow || currentWindow.value?.label
      })
      
      // 添加到子窗口列表
      if (!children.value.includes(windowId)) {
        children.value.push(windowId)
      }
      
      console.log('子窗口创建成功:', windowId)
      return windowId
    } catch (error) {
      console.error('创建子窗口失败:', error)
      throw error
    }
  }
  
  // 关闭子窗口
  const closeChildWindow = async (windowId, parentWindow = null) => {
    try {
      await invoke('close_child_window', {
        windowId,
        parentWindow: parentWindow || currentWindow.value?.label
      })
      
      // 从子窗口列表中移除
      children.value = children.value.filter(id => id !== windowId)
      
      console.log('子窗口关闭成功:', windowId)
    } catch (error) {
      console.error('关闭子窗口失败:', error)
      throw error
    }
  }
  
  // 最小化当前窗口及其子窗口
  const minimizeWindow = async () => {
    try {
      const windowLabel = currentWindow.value?.label || 'main'
      await invoke('minimize_window_with_children', { windowId: windowLabel })
      isMinimized.value = true
    } catch (error) {
      console.error('最小化窗口失败:', error)
      throw error
    }
  }
  
  // 恢复当前窗口及其子窗口
  const restoreWindow = async () => {
    try {
      const windowLabel = currentWindow.value?.label || 'main'
      await invoke('restore_window_with_children', { windowId: windowLabel })
      isMinimized.value = false
    } catch (error) {
      console.error('恢复窗口失败:', error)
      throw error
    }
  }
  
  // 关闭当前窗口及其所有子窗口
  const closeWindow = async () => {
    try {
      const windowLabel = currentWindow.value?.label || 'main'
      await invoke('close_window_with_children', { windowId: windowLabel })
    } catch (error) {
      console.error('关闭窗口失败:', error)
      throw error
    }
  }
  
  // 创建模态对话框
  const createModalDialog = async (options) => {
    const modalOptions = {
      ...options,
      modal: true,
      width: options.width || 600,
      height: options.height || 400
    }
    
    return await createChildWindow(modalOptions)
  }
  
  // 创建工具窗口
  const createToolWindow = async (options) => {
    const toolOptions = {
      ...options,
      modal: false,
      width: options.width || 800,
      height: options.height || 600
    }
    
    return await createChildWindow(toolOptions)
  }
  
  // 获取当前窗口信息
  const getWindowInfo = () => {
    return {
      label: currentWindow.value?.label,
      isMinimized: isMinimized.value,
      children: children.value
    }
  }
  
  return {
    // 状态
    currentWindow,
    isMinimized,
    children,
    
    // 方法
    initWindowManager,
    createChildWindow,
    closeChildWindow,
    minimizeWindow,
    restoreWindow,
    closeWindow,
    createModalDialog,
    createToolWindow,
    getWindowInfo
  }
}

/**
 * 窗体管理器的简化版本，用于子窗口
 */
export function useChildWindowManager() {
  const currentWindow = ref(null)
  
  const initChildWindow = async () => {
    try {
      currentWindow.value = getCurrentWebviewWindow()
    } catch (error) {
      console.error('初始化子窗口失败:', error)
    }
  }
  
  const closeCurrentWindow = async () => {
    try {
      if (currentWindow.value) {
        await currentWindow.value.close()
      }
    } catch (error) {
      console.error('关闭当前窗口失败:', error)
    }
  }
  
  const minimizeCurrentWindow = async () => {
    try {
      if (currentWindow.value) {
        await currentWindow.value.minimize()
      }
    } catch (error) {
      console.error('最小化当前窗口失败:', error)
    }
  }
  
  return {
    currentWindow,
    initChildWindow,
    closeCurrentWindow,
    minimizeCurrentWindow
  }
}
