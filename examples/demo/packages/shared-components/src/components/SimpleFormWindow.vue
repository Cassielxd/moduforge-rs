<template>
  <div class="simple-form-window">
    <!-- 使用模态窗口头部组件 -->
    <ModalWindowHeader
      :title="title"
      :is-maximized="windowInfo.isMaximized"
      @minimize="handleMinimize"
      @maximize="handleMaximize"
      @close="handleClose"
    />

    <!-- 内容区域 -->
    <div class="window-content">
      <div class="info-panel">
        <h3>窗口信息</h3>
        <p>模式: {{ mode }}</p>
        <p>类型: {{ formType }}</p>
        <p>模态: {{ isModal ? '是' : '否' }}</p>
        <p>父窗口: {{ parentWindow || '未指定' }}</p>
        <p>当前窗口: {{ currentWindowLabel }}</p>
        <p>窗口就绪: {{ windowReady ? '是' : '否' }}</p>
        <p>所属应用: {{ appId || '未指定' }}</p>
        <p>应用端口: {{ appPort || '未指定' }}</p>
        <!-- <p>当前URL: {{ window.location.href }}</p> -->
      </div>

      <div class="form-panel">
        <h3>简单表单</h3>
        <div class="form-group">
          <label>项目名称:</label>
          <input v-model="formData.name" placeholder="输入项目名称">
        </div>
        <div class="form-group">
          <label>金额:</label>
          <input v-model="formData.amount" type="number" placeholder="输入金额">
        </div>
        <div class="form-actions">
          <button @click="handleSave" :disabled="saving">{{ saving ? '保存中...' : '保存' }}</button>
          <button @click="handleClose">关闭</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import ModalWindowHeader from './ModalWindowHeader.vue'

// Props
const props = defineProps({
  mode: { type: String, default: 'create' },
  formType: { type: String, default: 'test' },
  isModal: { type: Boolean, default: false },
  parentWindow: { type: String, default: '' },
  appId: { type: String, default: '' },
  appPort: { type: String, default: '' },
  initialData: { type: Object, default: () => ({}) }
})

// 状态
const currentWindow = ref(null)
const windowInfo = ref({
  label: '',
  isMaximized: false,
  isReady: false
})
const saving = ref(false)
const formData = ref({
  name: props.initialData?.name || '',
  amount: props.initialData?.amount || 0
})

// 计算属性
const title = computed(() => `${props.mode} - ${props.formType}`)
const currentWindowLabel = computed(() => windowInfo.value.label || '未知')
const windowReady = computed(() => windowInfo.value.isReady)

// 初始化
onMounted(async () => {
  console.log('SimpleFormWindow mounted with props:', props)
  console.log('当前页面URL:', window.location.href)
  console.log('所属应用:', props.appId, '端口:', props.appPort)
  
  try {
    currentWindow.value = getCurrentWebviewWindow()
    windowInfo.value.label = currentWindow.value?.label || '未知'
    windowInfo.value.isMaximized = await currentWindow.value?.isMaximized() || false
    windowInfo.value.isReady = true
    
    console.log('窗口初始化成功:', {
      label: windowInfo.value.label,
      isMaximized: windowInfo.value.isMaximized,
      appId: props.appId,
      appPort: props.appPort
    })
    
    // 监听窗口状态变化
    if (currentWindow.value) {
      const unlisten = await currentWindow.value.listen('tauri://resize', async () => {
        windowInfo.value.isMaximized = await currentWindow.value.isMaximized()
      })
    }
  } catch (error) {
    console.error('窗口初始化失败:', error)
  }
})

// 窗口控制方法
const handleMinimize = async () => {
  console.log('最小化窗口')
  try {
    if (currentWindow.value) {
      await currentWindow.value.minimize()
      console.log('最小化成功')
    } else {
      console.error('窗口对象未初始化')
    }
  } catch (error) {
    console.error('最小化失败:', error)
  }
}

const handleMaximize = async () => {
  console.log('切换最大化')
  try {
    if (currentWindow.value) {
      if (windowInfo.value.isMaximized) {
        await currentWindow.value.unmaximize()
        windowInfo.value.isMaximized = false
        console.log('还原窗口')
      } else {
        await currentWindow.value.maximize()
        windowInfo.value.isMaximized = true
        console.log('最大化窗口')
      }
    } else {
      console.error('窗口对象未初始化')
    }
  } catch (error) {
    console.error('最大化操作失败:', error)
  }
}

const handleClose = async () => {
  console.log('关闭窗口')
  try {
    if (currentWindow.value) {
      await currentWindow.value.close()
      console.log('窗口已关闭')
    } else {
      console.error('窗口对象未初始化')
    }
  } catch (error) {
    console.error('关闭失败:', error)
  }
}

const handleSave = async () => {
  console.log('保存数据:', formData.value)
  saving.value = true
  
  try {
    // 模拟保存
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    // 这里可以发送数据到父窗口
    console.log('数据保存成功')
    
    // 延迟关闭
    setTimeout(() => {
      handleClose()
    }, 500)
  } catch (error) {
    console.error('保存失败:', error)
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.simple-form-window {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: white;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

/* ModalWindowHeader的样式通过全局CSS文件提供 */


.window-content {
  flex: 1;
  display: flex;
  padding: 20px;
  gap: 20px;
  overflow-y: auto;
}

.info-panel,
.form-panel {
  flex: 1;
  background: #f8f9fa;
  padding: 16px;
  border-radius: 8px;
  border: 1px solid #e9ecef;
}

.info-panel h3,
.form-panel h3 {
  margin: 0 0 16px 0;
  color: #495057;
  font-size: 16px;
  font-weight: 600;
}

.info-panel p {
  margin: 8px 0;
  color: #6c757d;
  font-size: 14px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 4px;
  color: #495057;
  font-size: 14px;
  font-weight: 500;
}

.form-group input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ced4da;
  border-radius: 4px;
  font-size: 14px;
}

.form-group input:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 2px rgba(102, 126, 234, 0.2);
}

.form-actions {
  display: flex;
  gap: 8px;
  margin-top: 20px;
}

.form-actions button {
  padding: 8px 16px;
  border: 1px solid #ced4da;
  border-radius: 4px;
  background: white;
  color: #495057;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.form-actions button:first-child {
  background: #667eea;
  color: white;
  border-color: #667eea;
}

.form-actions button:hover {
  opacity: 0.8;
}

.form-actions button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>