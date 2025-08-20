<template>
  <div class="simple-header" :class="{ 'draggable': draggable, 'web-environment': !isTauri }" :data-tauri-drag-region="draggable">
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
import { webWindowController, isTauriEnvironment } from '../utils/webEnvironment.js'

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
  return isTauriEnvironment()
})

// 检测是否为子窗口
const isChildWindow = computed(() => {
  if (!isTauri.value) return false
  try {
    // 通过URL参数或窗口标识符来判断是否为子窗口
    const urlParams = new URLSearchParams(window.location.search)
    const isChild = urlParams.has('mode') || urlParams.has('formType') || 
                    window.location.pathname.includes('form-page')
    console.log('检测子窗口状态:', isChild, 'URL:', window.location.href)
    return isChild
  } catch (error) {
    console.warn('检测子窗口状态失败:', error)
    return false
  }
})

// 初始化窗口管理
const initWindowManager = async () => {
  if (!isTauri.value) {
    console.log('非Tauri环境，跳过窗口管理器初始化')
    return
  }

  try {
    const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow')
    currentWindow.value = getCurrentWebviewWindow()

    if (currentWindow.value) {
      console.log('窗口管理器初始化成功', {
        label: currentWindow.value.label,
        isChild: isChildWindow.value
      })
      
      // 检查当前窗口状态
      try {
        isMaximized.value = await currentWindow.value.isMaximized()
        console.log('当前窗口最大化状态:', isMaximized.value)
      } catch (stateError) {
        console.warn('获取窗口状态失败:', stateError)
        isMaximized.value = false
      }

      // 监听窗口状态变化事件
      try {
        const unlisten = await currentWindow.value.listen('tauri://resize', async () => {
          try {
            const newMaximizedState = await currentWindow.value.isMaximized()
            isMaximized.value = newMaximizedState
            emit('window-state-change', { maximized: newMaximizedState })
            console.log('窗口状态变化:', newMaximizedState)
          } catch (listenerError) {
            console.warn('监听器中获取窗口状态失败:', listenerError)
          }
        })

        return unlisten
      } catch (listenError) {
        console.warn('添加窗口状态监听器失败:', listenError)
      }
    } else {
      console.warn('无法获取当前窗口实例')
    }
  } catch (error) {
    console.error('初始化窗口管理器失败:', error)
    // 在非Tauri环境或出错时，设置默认状态
    isMaximized.value = false
  }
}

// 窗口控制方法
const handleMinimize = async () => {
  if (loading.value) {
    console.log('操作进行中，跳过最小化')
    return
  }

  // 先发出事件，让外部组件处理
  emit('minimize')
  
  // 如果是非 Tauri 环境，执行 Web 模拟
  if (!isTauri.value) {
    await webWindowController.minimize()
    return
  }

  // 在 Tauri 环境下，如果外部组件监听了事件，就不再执行内部逻辑
  // 这里简单地不执行任何操作，让外部处理
  console.log('已发出最小化事件，等待外部处理')
}

const handleMaximize = async () => {
  if (loading.value) {
    console.log('操作进行中，跳过最大化切换')
    return
  }

  // 先发出事件，让外部组件处理
  emit('maximize')

  // 如果是非 Tauri 环境，执行 Web 模拟
  if (!isTauri.value) {
    const result = await webWindowController.toggleMaximize()
    if (result.success) {
      isMaximized.value = result.isMaximized
    }
    return
  }

  // 在 Tauri 环境下，让外部处理
  console.log('已发出最大化事件，等待外部处理')
}

const handleClose = async () => {
  if (loading.value) {
    console.log('操作进行中，跳过关闭')
    return
  }

  // 先发出事件，让外部组件处理
  emit('close')

  // 如果是非 Tauri 环境，执行 Web 模拟
  if (!isTauri.value) {
    await webWindowController.close()
    return
  }

  // 在 Tauri 环境下，让外部处理
  console.log('已发出关闭事件，等待外部处理')
}

// 生命周期
let unlistenResize = null

onMounted(async () => {
  console.log('SimpleHeader 组件挂载', {
    title: props.title,
    showWindowControls: props.showWindowControls,
    isTauri: isTauri.value
  })
  
  if (isTauri.value) {
    unlistenResize = await initWindowManager()
  } else {
    // 非Tauri环境，同步Web控制器状态
    isMaximized.value = webWindowController.getMaximizedState()
    console.log('Web环境初始化，最大化状态:', isMaximized.value)
  }
})

onUnmounted(() => {
  if (unlistenResize) {
    unlistenResize()
  }
})
</script>

<!-- 样式已移动到全局CSS文件 src/styles/simple-header.css -->
