<template>
  <div class="app-header" :class="{ 'draggable': draggable }" :data-tauri-drag-region="draggable">
    <div class="header-content">
      <!-- 左侧内容区域 -->
      <div class="header-left">
        <slot name="left">
          <div class="logo" v-if="showLogo">
            <h2>{{ title }}</h2>
          </div>
        </slot>
      </div>

      <!-- 中间内容区域 -->
      <div class="header-center" :data-tauri-drag-region="draggable">
        <slot name="center">
          <!-- 可拖动区域 -->
        </slot>
      </div>

      <!-- 右侧内容区域 -->
      <div class="header-right">
        <slot name="right">
          <!-- 用户信息等其他内容 -->
        </slot>
        
        <!-- 窗口控制按钮 -->
        <div class="window-controls" v-if="showWindowControls">
          <button
            class="window-control-btn minimize-btn"
            @click="handleMinimize"
            :title="minimizeTitle"
            :disabled="loading"
          >
            <svg width="12" height="12" viewBox="0 0 12 12">
              <path d="M2 6h8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </button>
          
          <button
            class="window-control-btn maximize-btn"
            @click="handleMaximize"
            :title="maximizeTitle"
            :disabled="loading"
          >
            <svg width="12" height="12" viewBox="0 0 12 12">
              <path v-if="!isMaximized" d="M2 2h8v8H2z" stroke="currentColor" stroke-width="1.5" fill="none"/>
              <path v-else d="M2 3h6v6H2z M4 1h6v6" stroke="currentColor" stroke-width="1.5" fill="none"/>
            </svg>
          </button>
          
          <button
            class="window-control-btn close-btn"
            @click="handleClose"
            :title="closeTitle"
            :disabled="loading"
          >
            <svg width="12" height="12" viewBox="0 0 12 12">
              <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'

// Props
const props = defineProps({
  title: {
    type: String,
    default: '应用标题'
  },
  showLogo: {
    type: Boolean,
    default: true
  },
  showWindowControls: {
    type: Boolean,
    default: true
  },
  draggable: {
    type: Boolean,
    default: true
  },
  minimizeTitle: {
    type: String,
    default: '最小化'
  },
  maximizeTitle: {
    type: String,
    default: '最大化/还原'
  },
  closeTitle: {
    type: String,
    default: '关闭'
  },
  useChildWindow: {
    type: Boolean,
    default: false
  }
})

// Emits
const emit = defineEmits(['minimize', 'maximize', 'close', 'window-state-change'])

// State
const loading = ref(false)
const isMaximized = ref(false)
const currentWindow = ref(null)

// 检测是否在 Tauri 环境中
const isTauri = computed(() => {
  return typeof window !== 'undefined' && window.__TAURI__
})

// 初始化窗口管理
const initWindowManager = async () => {
  if (!isTauri.value) return

  try {
    const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow')
    currentWindow.value = getCurrentWebviewWindow()
    
    // 监听窗口状态变化
    if (currentWindow.value) {
      // 检查当前窗口状态
      const isMaximizedState = await currentWindow.value.isMaximized()
      isMaximized.value = isMaximizedState
      
      // 监听窗口状态变化事件
      const unlisten = await currentWindow.value.listen('tauri://resize', async () => {
        const maximized = await currentWindow.value.isMaximized()
        isMaximized.value = maximized
        emit('window-state-change', { maximized })
      })
      
      return unlisten
    }
  } catch (error) {
    console.error('初始化窗口管理器失败:', error)
  }
}

// 窗口控制方法
const handleMinimize = async () => {
  if (loading.value) return
  
  loading.value = true
  try {
    emit('minimize')
    
    if (isTauri.value && currentWindow.value) {
      if (props.useChildWindow) {
        // 子窗口直接最小化
        await currentWindow.value.minimize()
      } else {
        // 主窗口使用自定义逻辑，包括子窗口联动
        const { invoke } = await import('@tauri-apps/api/core')
        const windowLabel = currentWindow.value.label || 'main'
        await invoke('minimize_window_with_children', { windowId: windowLabel })
      }
    }
  } catch (error) {
    console.error('最小化窗口失败:', error)
  } finally {
    loading.value = false
  }
}

const handleMaximize = async () => {
  if (loading.value) return
  
  loading.value = true
  try {
    emit('maximize')
    
    if (isTauri.value && currentWindow.value) {
      if (isMaximized.value) {
        await currentWindow.value.unmaximize()
      } else {
        await currentWindow.value.maximize()
      }
      isMaximized.value = !isMaximized.value
    }
  } catch (error) {
    console.error('切换窗口最大化状态失败:', error)
  } finally {
    loading.value = false
  }
}

const handleClose = async () => {
  if (loading.value) return
  
  loading.value = true
  try {
    emit('close')
    
    if (isTauri.value && currentWindow.value) {
      if (props.useChildWindow) {
        // 子窗口直接关闭
        await currentWindow.value.close()
      } else {
        // 主窗口使用自定义逻辑，包括子窗口联动关闭
        const { invoke } = await import('@tauri-apps/api/core')
        const windowLabel = currentWindow.value.label || 'main'
        await invoke('close_window_with_children', { windowId: windowLabel })
      }
    }
  } catch (error) {
    console.error('关闭窗口失败:', error)
  } finally {
    loading.value = false
  }
}

// 生命周期
let unlistenResize = null

onMounted(async () => {
  unlistenResize = await initWindowManager()
})

onUnmounted(() => {
  if (unlistenResize) {
    unlistenResize()
  }
})
</script>

<style scoped>
.app-header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  padding: 0;
  height: 60px;
  position: relative;
  z-index: 1000;
}

.app-header.draggable {
  -webkit-app-region: drag;
}

.header-content {
  display: flex;
  align-items: center;
  height: 100%;
  max-width: 100%;
  padding: 0 20px;
  position: relative;
}

.header-left {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.header-center {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 0;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.logo h2 {
  margin: 0;
  color: white;
  font-weight: 600;
  font-size: 18px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: 12px;
}

.window-control-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.1);
  color: white;
  cursor: pointer;
  transition: all 0.2s ease;
  backdrop-filter: blur(10px);
}

.window-control-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: translateY(-1px);
}

.window-control-btn:active {
  transform: translateY(0);
}

.window-control-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.window-control-btn.close-btn:hover {
  background: rgba(239, 68, 68, 0.8);
}

.window-control-btn svg {
  width: 12px;
  height: 12px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .header-content {
    padding: 0 12px;
  }
  
  .logo h2 {
    font-size: 16px;
  }
  
  .window-control-btn {
    width: 28px;
    height: 28px;
  }
  
  .window-control-btn svg {
    width: 10px;
    height: 10px;
  }
}

/* 深色主题支持 */
@media (prefers-color-scheme: dark) {
  .app-header {
    background: linear-gradient(135deg, #2d3748 0%, #4a5568 100%);
    border-bottom-color: rgba(255, 255, 255, 0.05);
  }
}
</style>
