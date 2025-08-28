<template>
  <div class="modal-window-header" data-tauri-drag-region>
    <div class="header-content">
      <!-- 窗口标题 -->
      <div class="window-title">
        <h3>{{ title }}</h3>
      </div>
      
      <!-- 右侧内容和窗口控制 -->
      <div class="header-right">
        <slot name="right"></slot>
        
        <!-- 窗口控制按钮 -->
        <div class="window-controls">
          <button
            class="control-btn minimize-btn"
            @click="$emit('minimize')"
            title="最小化"
            :disabled="loading"
          >
            <svg width="14" height="14" viewBox="0 0 14 14">
              <path d="M3 7h8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </button>
          
          <button
            class="control-btn maximize-btn"
            @click="$emit('maximize')"
            :title="isMaximized ? '还原' : '最大化'"
            :disabled="loading"
          >
            <svg width="14" height="14" viewBox="0 0 14 14">
              <path v-if="!isMaximized" d="M3 3h8v8H3z" stroke="currentColor" stroke-width="1.5" fill="none"/>
              <path v-else d="M3 4h6v6H3z M5 2h6v6" stroke="currentColor" stroke-width="1.5" fill="none"/>
            </svg>
          </button>
          
          <button
            class="control-btn close-btn"
            @click="$emit('close')"
            title="关闭"
            :disabled="loading"
          >
            <svg width="14" height="14" viewBox="0 0 14 14">
              <path d="M4 4l6 6M10 4l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

// Props
const props = defineProps({
  title: {
    type: String,
    default: '窗口标题'
  },
  isMaximized: {
    type: Boolean,
    default: false
  }
})

// Emits
const emit = defineEmits(['minimize', 'maximize', 'close'])

// 状态
const loading = ref(false)
</script>

<style scoped>
.modal-window-header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  height: 48px;
  position: relative;
  z-index: 1000;
  -webkit-app-region: drag;
  user-select: none;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  padding: 0 16px;
  position: relative;
}

.window-title {
  flex: 1;
  display: flex;
  align-items: center;
  min-width: 0;
}

.window-title h3 {
  margin: 0;
  color: white;
  font-weight: 600;
  font-size: 14px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 6px;
}

.control-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.15);
  color: white;
  cursor: pointer;
  transition: all 0.2s ease;
  backdrop-filter: blur(10px);
  position: relative;
  overflow: hidden;
}

.control-btn::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0);
  transition: background 0.2s ease;
  z-index: -1;
}

.control-btn:hover::before {
  background: rgba(255, 255, 255, 0.1);
}

.control-btn:hover {
  background: rgba(255, 255, 255, 0.25);
  transform: translateY(-0.5px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.control-btn:active {
  transform: translateY(0);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.control-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.control-btn:disabled::before {
  background: transparent;
}

.close-btn:hover {
  background: rgba(239, 68, 68, 0.9);
  transform: translateY(-0.5px);
}

.close-btn:hover::before {
  background: rgba(255, 255, 255, 0.1);
}

.control-btn svg {
  width: 14px;
  height: 14px;
  pointer-events: none;
}

/* 响应式设计 */
@media (max-width: 600px) {
  .header-content {
    padding: 0 12px;
  }
  
  .window-title h3 {
    font-size: 13px;
  }
  
  .control-btn {
    width: 26px;
    height: 26px;
  }
  
  .control-btn svg {
    width: 12px;
    height: 12px;
  }
}

/* 针对不同系统的优化 */
@media (prefers-color-scheme: dark) {
  .modal-window-header {
    background: linear-gradient(135deg, #2d3748 0%, #4a5568 100%);
    border-bottom-color: rgba(255, 255, 255, 0.05);
  }
}

/* Windows 风格调整 */
@media screen and (-ms-high-contrast: active), (-ms-high-contrast: none) {
  .control-btn {
    backdrop-filter: none;
  }
}
</style>