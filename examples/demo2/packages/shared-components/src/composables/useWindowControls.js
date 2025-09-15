import { ref, onMounted } from 'vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

/**
 * 窗口控制 composable
 * 提供统一的窗口最小化、最大化、关闭功能
 */
export function useWindowControls() {
  const currentWindow = ref(null)
  const isMaximized = ref(false)

  // 获取当前窗口引用
  const initWindow = async () => {
    try {
      currentWindow.value = getCurrentWebviewWindow()
      console.log('获取窗口引用成功:', currentWindow.value?.label)
    } catch (error) {
      console.error('获取窗口引用失败:', error)
    }
  }

  // 最小化窗口
  const handleMinimize = async () => {
    try {
      if (!currentWindow.value) {
        await initWindow()
      }
      await currentWindow.value.minimize()
    } catch (error) {
      console.error('最小化窗口失败:', error)
    }
  }

  // 最大化/还原窗口
  const handleMaximize = async () => {
    try {
      if (!currentWindow.value) {
        await initWindow()
      }
      
      if (isMaximized.value) {
        await currentWindow.value.unmaximize()
        isMaximized.value = false
      } else {
        await currentWindow.value.maximize()
        isMaximized.value = true
      }
    } catch (error) {
      console.error('最大化/还原窗口失败:', error)
    }
  }

  // 关闭窗口
  const handleClose = async (emit) => {
    try {
      // 先发出取消事件（如果有emit函数）
      if (emit && typeof emit === 'function') {
        emit('cancel')
      }
      
      // 然后关闭当前窗口
      if (!currentWindow.value) {
        await initWindow()
      }
      await currentWindow.value.close()
    } catch (error) {
      console.error('关闭窗口失败:', error)
    }
  }

  // 自动初始化
  onMounted(async () => {
    await initWindow()
  })

  return {
    currentWindow,
    isMaximized,
    handleMinimize,
    handleMaximize,
    handleClose,
    initWindow
  }
}
