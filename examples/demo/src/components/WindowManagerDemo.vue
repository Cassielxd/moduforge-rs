<template>
  <div class="window-manager-demo">
    <div class="demo-header">
      <h2>通用窗体管理演示</h2>
      <p>演示工作台和子应用之间的窗体管理功能</p>
      <div class="current-info">
        <span>当前环境: <strong>工作台主应用</strong></span>
        <span>打开窗口数: <strong>{{ universalWindowManager.windowCount.value }}</strong></span>
      </div>
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
      
      <!-- 通用窗体管理演示 -->
      <div class="universal-window-controls">
        <h3>通用窗体管理功能</h3>
        <div class="control-group">
          <button @click="openEstimateDemo">打开概算演示</button>
          <button @click="openTableTest">打开表格测试</button>
          <button @click="openDataViewer">打开数据查看器</button>
          <button @click="openRoughEstimate">打开概算子应用</button>
          <button @click="openSettings">打开设置（模态）</button>
          <button @click="closeAllUniversalWindows" class="danger">关闭所有通用窗口</button>
        </div>
        
        <div v-if="universalWindowManager.hasOpenWindows.value" class="universal-window-list">
          <h4>通用窗体管理器打开的窗口：</h4>
          <ul>
            <li v-for="window in universalWindowManager.openWindowList.value" :key="window.id">
              <span>{{ window.title }} ({{ window.type }})</span>
              <button @click="closeUniversalWindow(window.id)" class="close-btn">关闭</button>
            </li>
          </ul>
        </div>
      </div>

      <!-- 子窗口创建 -->
      <div class="child-window-controls">
        <h3>底层窗口管理（原生创建）</h3>
        
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
import { useMainAppWindowManager, useMainWindowManagement } from '@cost-app/shared-components'

// 使用底层窗体管理器
const {
  currentWindow,
  windowLabel,
  isMinimized,
  children,
  windowInfo,
  initializeWindow,
  createChildWindow: createChild,
  closeChildWindow,
  minimizeWindow: minimize,
  restoreWindow: restore,
  closeWindow: close,
  createModalDialog: createModal,
  createToolWindow: createTool
} = useMainWindowManagement()

// 使用通用窗体管理器 - 新的高级管理器
const universalWindowManager = useMainAppWindowManager()

// 新窗口表单数据
const newWindow = ref({
  title: '新窗口',
  url: '/?window=child',
  width: 800,
  height: 600,
  modal: false
})

// windowInfo 已从 useMainWindowManagement 中获取

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

// 通用窗体管理方法
const openEstimateDemo = async () => {
  try {
    // 直接打开概算子应用
    await universalWindowManager.quick.roughEstimate()
    console.log('概算管理窗口已打开')
  } catch (error) {
    console.error('打开概算管理失败:', error)
  }
}

const openTableTest = async () => {
  try {
    await universalWindowManager.quick.tableTest()
    console.log('表格测试窗口已打开')
  } catch (error) {
    console.error('打开表格测试失败:', error)
  }
}

const openDataViewer = async () => {
  try {
    await universalWindowManager.quick.dataViewer()
    console.log('数据查看器窗口已打开')
  } catch (error) {
    console.error('打开数据查看器失败:', error)
  }
}

const openRoughEstimate = async () => {
  try {
    await universalWindowManager.quick.roughEstimate()
    console.log('概算子应用已打开')
  } catch (error) {
    console.error('打开概算子应用失败:', error)
  }
}

const openSettings = async () => {
  try {
    await universalWindowManager.quick.settings()
    console.log('设置窗口已打开')
  } catch (error) {
    console.error('打开设置失败:', error)
  }
}

const closeAllUniversalWindows = async () => {
  try {
    await universalWindowManager.closeAllWindows()
    console.log('所有通用窗口已关闭')
  } catch (error) {
    console.error('关闭窗口失败:', error)
  }
}

const closeUniversalWindow = async (windowId) => {
  try {
    await universalWindowManager.closeWindow(windowId)
    console.log('窗口已关闭:', windowId)
  } catch (error) {
    console.error('关闭窗口失败:', error)
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
  await initializeWindow()
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

.current-info {
  display: flex;
  gap: 20px;
  justify-content: center;
  margin-top: 10px;
  font-size: 14px;
  color: #666;
}

.demo-content {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

.window-info,
.window-controls,
.child-window-controls,
.preset-windows,
.universal-window-controls {
  background: #f5f5f5;
  padding: 20px;
  border-radius: 8px;
  border: 1px solid #ddd;
}

.window-info h3,
.window-controls h3,
.child-window-controls h3,
.preset-windows h3,
.universal-window-controls h3 {
  margin: 0 0 15px 0;
  color: #333;
  font-size: 16px;
}

.universal-window-controls {
  background: #e6f7ff;
  border-color: #91d5ff;
}

.universal-window-controls h3 {
  color: #1890ff;
}

.universal-window-list {
  margin-top: 15px;
  padding-top: 15px;
  border-top: 1px solid #91d5ff;
}

.universal-window-list h4 {
  margin: 0 0 10px 0;
  color: #1890ff;
  font-size: 14px;
}

.universal-window-list ul {
  list-style: none;
  padding: 0;
  margin: 0;
}

.universal-window-list li {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #f0f8ff;
  margin-bottom: 5px;
  border-radius: 4px;
  border: 1px solid #d1e9ff;
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
