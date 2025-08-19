<template>
  <div class="simple-header" :class="{ 'draggable': draggable }" :data-tauri-drag-region="draggable">
    <div class="header-content">
      <!-- 左侧标题 -->
      <div class="header-left">
        <slot name="left">
          <h3 class="title">{{ title }}</h3>
        </slot>
      </div>

      <!-- 中间可拖动区域 -->
      <div class="header-center" :data-tauri-drag-region="draggable">
        <slot name="center"></slot>
      </div>

      <!-- 右侧窗口控制 -->
      <div class="header-right">
        <slot name="right"></slot>

        <!-- 窗口控制按钮 -->
        <div class="window-controls" v-if="showWindowControls">
          <button
            class="control-btn minimize-btn"
            @click="handleMinimize"
            :title="minimizeTitle"
            :disabled="loading"
          >
            <svg width="12" height="12" viewBox="0 0 12 12">
              <path d="M2 6h8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </button>

          <button
            class="control-btn maximize-btn"
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
            class="control-btn close-btn"
            @click="handleClose"
            :title="closeTitle"
            :disabled="loading"
          >
            <svg width="12" height="12" viewBox="0 0 12 12">
              <path d="M2 2l8 8M10 2l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
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
    default: '应用'
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

    if (currentWindow.value) {
      // 检查当前窗口状态
      isMaximized.value = await currentWindow.value.isMaximized()

      // 监听窗口状态变化事件
      const unlisten = await currentWindow.value.listen('tauri://resize', async () => {
        isMaximized.value = await currentWindow.value.isMaximized()
        emit('window-state-change', { maximized: isMaximized.value })
      })

      return unlisten
    }
  } catch (error) {
    console.error('初始化窗口管理器失败:', error)
  }
}

// 窗口控制方法
const handleMinimize = async () => {
  if (loading.value || !isTauri.value || !currentWindow.value) return

  loading.value = true
  try {
    emit('minimize')
    await currentWindow.value.minimize()
  } catch (error) {
    console.error('最小化窗口失败:', error)
  } finally {
    loading.value = false
  }
}

const handleMaximize = async () => {
  if (loading.value || !isTauri.value || !currentWindow.value) return

  loading.value = true
  try {
    emit('maximize')

    if (isMaximized.value) {
      await currentWindow.value.unmaximize()
    } else {
      await currentWindow.value.maximize()
    }
  } catch (error) {
    console.error('切换窗口最大化状态失败:', error)
  } finally {
    loading.value = false
  }
}

const handleClose = async () => {
  if (loading.value || !isTauri.value || !currentWindow.value) return

  loading.value = true
  try {
    emit('close')
    await currentWindow.value.close()
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
.simple-header {
  background: #fff;
  border-bottom: 1px solid #e8e8e8;
  padding: 0;
  height: 48px;
  position: relative;
  z-index: 1000;
}

.simple-header.draggable {
  -webkit-app-region: drag;
}

.header-content {
  display: flex;
  align-items: center;
  height: 100%;
  padding: 0 16px;
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
  gap: 8px;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.title {
  margin: 0;
  color: #262626;
  font-weight: 500;
  font-size: 16px;
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: 12px;
}

.control-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.1);
  color: #8c8c8c;
  cursor: pointer;
  transition: all 0.2s ease;
  backdrop-filter: blur(10px);
}

.control-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  color: #262626;
  transform: translateY(-1px);
}

.control-btn:active {
  transform: translateY(0);
}

.control-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.control-btn.close-btn:hover {
  background: rgba(239, 68, 68, 0.8);
  color: white;
}

.control-btn svg {
  width: 12px;
  height: 12px;
}

/* 深色主题支持 */
@media (prefers-color-scheme: dark) {
  .simple-header {
    background: #141414;
    border-bottom-color: #303030;
  }
  
  .title {
    color: #fff;
  }
  
  .control-btn {
    color: #8c8c8c;
  }
  
  .control-btn:hover {
    background: #262626;
    color: #fff;
  }
  
  .control-btn:active {
    background: #1890ff;
  }
}
</style>
