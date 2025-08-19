<template>
  <div class="window-manager-demo">
    <div class="demo-header">
      <h2>窗体管理演示</h2>
      <p>演示 Tauri 中类似 Electron 的父子窗体关系管理</p>
    </div>
    
    <div class="demo-content">
      <!-- 窗口信息显示 -->
      <div class="window-info">
        <h3>当前窗口信息</h3>
        <div class="info-item">
          <span>窗口标签:</span>
          <span>{{ windowInfo.label || '未知' }}</span>
        </div>
        <div class="info-item">
          <span>最小化状态:</span>
          <span :class="{ minimized: windowInfo.isMinimized }">
            {{ windowInfo.isMinimized ? '已最小化' : '正常' }}
          </span>
        </div>
        <div class="info-item">
          <span>子窗口数量:</span>
          <span>{{ windowInfo.children.length }}</span>
        </div>
        <div v-if="windowInfo.children.length > 0" class="children-list">
          <span>子窗口列表:</span>
          <ul>
            <li v-for="child in windowInfo.children" :key="child">
              {{ child }}
              <button @click="closeChild(child)" class="close-btn">关闭</button>
            </li>
          </ul>
        </div>
      </div>
      
      <!-- 窗口操作按钮 -->
      <div class="window-controls">
        <h3>窗口操作</h3>
        <div class="control-group">
          <button @click="minimizeWindow" :disabled="isMinimized">
            最小化窗口
          </button>
          <button @click="restoreWindow" :disabled="!isMinimized">
            恢复窗口
          </button>
          <button @click="closeWindow" class="danger">
            关闭窗口
          </button>
        </div>
      </div>
      
      <!-- 子窗口创建 -->
      <div class="child-window-controls">
        <h3>创建子窗口</h3>
        
        <div class="create-form">
          <div class="form-group">
            <label>窗口标题:</label>
            <input v-model="newWindow.title" placeholder="输入窗口标题" />
          </div>
          
          <div class="form-group">
            <label>窗口URL:</label>
            <input v-model="newWindow.url" placeholder="输入窗口URL" />
          </div>
          
          <div class="form-group">
            <label>窗口大小:</label>
            <div class="size-inputs">
              <input 
                v-model.number="newWindow.width" 
                type="number" 
                placeholder="宽度" 
                min="300"
              />
              <span>×</span>
              <input 
                v-model.number="newWindow.height" 
                type="number" 
                placeholder="高度" 
                min="200"
              />
            </div>
          </div>
          
          <div class="form-group">
            <label>
              <input v-model="newWindow.modal" type="checkbox" />
              模态窗口
            </label>
          </div>
          
          <div class="control-group">
            <button @click="createChildWindow">
              创建子窗口
            </button>
            <button @click="createModalDialog">
              创建模态对话框
            </button>
            <button @click="createToolWindow">
              创建工具窗口
            </button>
          </div>
        </div>
      </div>
      
      <!-- 预设窗口快速创建 -->
      <div class="preset-windows">
        <h3>预设窗口</h3>
        <div class="control-group">
          <button @click="createFormWindow">
            表单窗口
          </button>
          <button @click="createSettingsWindow">
            设置窗口
          </button>
          <button @click="createAboutDialog">
            关于对话框
          </button>
          <button @click="createHelpWindow">
            帮助窗口
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useWindowManager } from '../composables/useWindowManager.js'

// 使用窗体管理器
const {
  currentWindow,
  isMinimized,
  children,
  initWindowManager,
  createChildWindow: createChild,
  closeChildWindow,
  minimizeWindow: minimize,
  restoreWindow: restore,
  closeWindow: close,
  createModalDialog: createModal,
  createToolWindow: createTool,
  getWindowInfo
} = useWindowManager()

// 新窗口表单数据
const newWindow = ref({
  title: '新窗口',
  url: '/?window=child',
  width: 800,
  height: 600,
  modal: false
})

// 计算属性
const windowInfo = computed(() => getWindowInfo())

// 窗口操作方法
const minimizeWindow = async () => {
  try {
    await minimize()
  } catch (error) {
    console.error('最小化失败:', error)
  }
}

const restoreWindow = async () => {
  try {
    await restore()
  } catch (error) {
    console.error('恢复失败:', error)
  }
}

const closeWindow = async () => {
  if (confirm('确定要关闭窗口吗？这将关闭所有子窗口。')) {
    try {
      await close()
    } catch (error) {
      console.error('关闭失败:', error)
    }
  }
}

// 子窗口操作方法
const createChildWindow = async () => {
  try {
    const windowId = `child-${Date.now()}`
    await createChild({
      windowId,
      title: newWindow.value.title,
      url: newWindow.value.url,
      width: newWindow.value.width,
      height: newWindow.value.height,
      modal: newWindow.value.modal
    })
  } catch (error) {
    console.error('创建子窗口失败:', error)
  }
}

const createModalDialog = async () => {
  try {
    const windowId = `modal-${Date.now()}`
    await createModal({
      windowId,
      title: '模态对话框',
      url: '/?window=modal',
      width: 500,
      height: 300
    })
  } catch (error) {
    console.error('创建模态对话框失败:', error)
  }
}

const createToolWindow = async () => {
  try {
    const windowId = `tool-${Date.now()}`
    await createTool({
      windowId,
      title: '工具窗口',
      url: '/?window=tool',
      width: 600,
      height: 400
    })
  } catch (error) {
    console.error('创建工具窗口失败:', error)
  }
}

const closeChild = async (childId) => {
  try {
    await closeChildWindow(childId)
  } catch (error) {
    console.error('关闭子窗口失败:', error)
  }
}

// 预设窗口创建方法
const createFormWindow = async () => {
  try {
    const windowId = `form-${Date.now()}`
    await createChild({
      windowId,
      title: '表单窗口',
      url: '/form?mode=create',
      width: 900,
      height: 700,
      modal: false
    })
  } catch (error) {
    console.error('创建表单窗口失败:', error)
  }
}

const createSettingsWindow = async () => {
  try {
    const windowId = `settings-${Date.now()}`
    await createChild({
      windowId,
      title: '设置',
      url: '/settings',
      width: 800,
      height: 600,
      modal: false
    })
  } catch (error) {
    console.error('创建设置窗口失败:', error)
  }
}

const createAboutDialog = async () => {
  try {
    const windowId = `about-${Date.now()}`
    await createModal({
      windowId,
      title: '关于',
      url: '/about',
      width: 400,
      height: 300
    })
  } catch (error) {
    console.error('创建关于对话框失败:', error)
  }
}

const createHelpWindow = async () => {
  try {
    const windowId = `help-${Date.now()}`
    await createChild({
      windowId,
      title: '帮助文档',
      url: '/help',
      width: 1000,
      height: 800,
      modal: false
    })
  } catch (error) {
    console.error('创建帮助窗口失败:', error)
  }
}

// 生命周期
let cleanup = null

onMounted(async () => {
  cleanup = await initWindowManager()
})

onUnmounted(() => {
  if (cleanup) {
    cleanup()
  }
})
</script>

<style scoped>
.window-manager-demo {
  padding: 20px;
  max-width: 800px;
  margin: 0 auto;
}

.demo-header {
  text-align: center;
  margin-bottom: 30px;
}

.demo-header h2 {
  color: #333;
  margin-bottom: 10px;
}

.demo-header p {
  color: #666;
  font-size: 14px;
}

.demo-content {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

.window-info,
.window-controls,
.child-window-controls,
.preset-windows {
  background: #f5f5f5;
  padding: 20px;
  border-radius: 8px;
  border: 1px solid #ddd;
}

.window-info h3,
.window-controls h3,
.child-window-controls h3,
.preset-windows h3 {
  margin: 0 0 15px 0;
  color: #333;
  font-size: 16px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  padding: 5px 0;
  border-bottom: 1px solid #eee;
}

.info-item span:first-child {
  font-weight: 500;
  color: #555;
}

.minimized {
  color: #f56565;
  font-weight: 500;
}

.children-list {
  margin-top: 10px;
}

.children-list ul {
  list-style: none;
  padding: 0;
  margin: 5px 0 0 0;
}

.children-list li {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 5px 10px;
  background: white;
  margin-bottom: 5px;
  border-radius: 4px;
  border: 1px solid #ddd;
}

.close-btn {
  background: #f56565;
  color: white;
  border: none;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 12px;
  cursor: pointer;
}

.close-btn:hover {
  background: #e53e3e;
}

.control-group {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.control-group button {
  padding: 8px 16px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  transition: all 0.2s;
}

.control-group button:hover:not(:disabled) {
  background: #f0f0f0;
  border-color: #bbb;
}

.control-group button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.control-group button.danger {
  background: #f56565;
  color: white;
  border-color: #f56565;
}

.control-group button.danger:hover:not(:disabled) {
  background: #e53e3e;
  border-color: #e53e3e;
}

.create-form {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.form-group label {
  font-weight: 500;
  color: #555;
  font-size: 14px;
}

.form-group input[type="text"],
.form-group input[type="number"] {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.size-inputs {
  display: flex;
  align-items: center;
  gap: 10px;
}

.size-inputs input {
  flex: 1;
}

.size-inputs span {
  color: #666;
  font-weight: 500;
}

.form-group input[type="checkbox"] {
  width: auto;
  margin-right: 8px;
}
</style>
