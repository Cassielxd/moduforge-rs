<template>
  <div class="form-window">
    <!-- 使用专用的模态窗口头部 -->
    <ModalWindowHeader
      :title="windowTitle"
      :is-maximized="false"
      @minimize="onMinimize"
      @maximize="onMaximize"
      @close="onClose"
    >
      <template #right>
        <div class="header-actions">
          <span class="window-type">{{ isModal ? '模态' : '非模态' }}</span>
        </div>
      </template>
    </ModalWindowHeader>

    <!-- 主内容区域 -->
    <div class="form-content">
      <a-card :title="`${formType} - ${mode}模式`" :bordered="false">
        <div class="form-info">
          <a-descriptions :column="2" size="small">
            <a-descriptions-item label="窗口类型">
              {{ isModal ? '模态窗口' : '非模态窗口' }}
            </a-descriptions-item>
            <a-descriptions-item label="父窗口">
              {{ displayParentWindow }}
            </a-descriptions-item>
            <a-descriptions-item label="当前窗口">
              {{ displayCurrentWindow }}
            </a-descriptions-item>
            <a-descriptions-item label="操作模式">
              {{ mode }}
            </a-descriptions-item>
          </a-descriptions>
        </div>

        <!-- 表单区域 -->
        <a-divider />
        <a-form
          ref="formRef"
          :model="formData"
          :label-col="{ span: 6 }"
          :wrapper-col="{ span: 18 }"
        >
          <a-form-item label="项目名称" name="name" :rules="[{ required: true, message: '请输入项目名称' }]">
            <a-input 
              v-model:value="formData.name" 
              placeholder="请输入项目名称"
              :disabled="mode === 'view'"
            />
          </a-form-item>

          <a-form-item label="项目类型" name="type">
            <a-select 
              v-model:value="formData.type" 
              placeholder="请选择项目类型"
              :disabled="mode === 'view'"
            >
              <a-select-option value="building">建筑工程</a-select-option>
              <a-select-option value="infrastructure">基础设施</a-select-option>
              <a-select-option value="landscape">景观工程</a-select-option>
              <a-select-option value="renovation">装修改造</a-select-option>
            </a-select>
          </a-form-item>

          <a-form-item label="概算金额" name="amount">
            <a-input-number 
              v-model:value="formData.amount" 
              :min="0"
              :formatter="value => `￥ ${value}`.replace(/\B(?=(\d{3})+(?!\d))/g, ',')"
              :parser="value => value.replace(/￥\s?|(,*)/g, '')"
              style="width: 100%"
              placeholder="请输入概算金额"
              :disabled="mode === 'view'"
            />
          </a-form-item>

          <a-form-item label="负责人" name="creator">
            <a-input 
              v-model:value="formData.creator" 
              placeholder="请输入负责人姓名"
              :disabled="mode === 'view'"
            />
          </a-form-item>

          <a-form-item label="项目描述" name="description">
            <a-textarea 
              v-model:value="formData.description" 
              :rows="3"
              placeholder="请输入项目描述"
              :disabled="mode === 'view'"
            />
          </a-form-item>

          <a-form-item label="开始时间" name="startDate">
            <a-date-picker 
              v-model:value="formData.startDate" 
              style="width: 100%"
              :disabled="mode === 'view'"
            />
          </a-form-item>

          <a-form-item label="结束时间" name="endDate">
            <a-date-picker 
              v-model:value="formData.endDate" 
              style="width: 100%"
              :disabled="mode === 'view'"
            />
          </a-form-item>
        </a-form>

        <!-- 操作按钮区域 -->
        <a-divider />
        <div class="form-actions">
          <a-space>
            <a-button v-if="mode !== 'view'" @click="onReset">重置</a-button>
            <a-button @click="onCancel">{{ mode === 'view' ? '关闭' : '取消' }}</a-button>
            <a-button v-if="mode !== 'view'" type="primary" @click="onSubmit" :loading="submitting">
              {{ mode === 'create' ? '创建' : '更新' }}
            </a-button>
            
            <!-- 测试按钮：演示父子窗口交互 -->
            <a-button v-if="mode !== 'view'" @click="openTestModal" type="dashed">
              打开测试模态窗口
            </a-button>
            <a-button v-if="mode !== 'view'" @click="openTestTool" type="dashed">
              打开测试工具窗口
            </a-button>
          </a-space>
        </div>

        <!-- 调试信息 -->
        <a-divider />
        <a-collapse>
          <a-collapse-panel key="1" header="调试信息">
            <pre>{{ JSON.stringify(windowInfo, null, 2) }}</pre>
          </a-collapse-panel>
        </a-collapse>
      </a-card>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { message } from 'ant-design-vue'
import { useChildWindowManagement } from '../composables/useSimpleWindowManagement.js'
import { useChildWindowDataExchange } from '../composables/useWindowDataExchange.js'
import AppHeader from '../layouts/AppHeader.vue'

// Props
const props = defineProps({
  mode: {
    type: String,
    default: 'create', // create, edit, view
    validator: value => ['create', 'edit', 'view'].includes(value)
  },
  formType: {
    type: String,
    default: 'general'
  },
  initialData: {
    type: Object,
    default: () => ({})
  },
  isModal: {
    type: Boolean,
    default: false
  },
  parentWindow: {
    type: String,
    default: ''
  }
})

// 使用窗口管理
const windowManagement = useChildWindowManagement()
const { 
  windowInfo, 
  createModalDialog, 
  createToolWindow,
  minimizeWindow,
  toggleMaximize,
  closeWindow
} = windowManagement

// 使用数据交换
const dataExchange = useChildWindowDataExchange()
const {
  sendFormUpdate,
  sendFormSubmit,
  onDataRefreshRequest
} = dataExchange

// 窗口信息的计算属性
const displayParentWindow = computed(() => {
  return props.parentWindow || windowInfo.value.parentWindowId || '未知'
})

const displayCurrentWindow = computed(() => {
  return windowInfo.value.label || '未知'
})

// 表单数据
const formRef = ref()
const submitting = ref(false)
const formData = ref({
  name: '',
  type: '',
  amount: 0,
  creator: '',
  description: '',
  startDate: null,
  endDate: null,
  ...props.initialData
})

// 计算属性
const windowTitle = computed(() => {
  const modeText = {
    create: '新建',
    edit: '编辑', 
    view: '查看'
  }
  return `${modeText[props.mode]}${props.formType === 'estimate' ? '概算' : '表单'}`
})

// 方法
const onMinimize = async () => {
  console.log('FormWindow: 最小化窗口')
  try {
    await minimizeWindow()
    console.log('FormWindow: 最小化成功')
  } catch (error) {
    console.error('FormWindow: 最小化失败:', error)
    message.error('最小化失败')
  }
}

const onMaximize = async () => {
  console.log('FormWindow: 切换最大化状态')
  try {
    await toggleMaximize()
    console.log('FormWindow: 最大化切换成功')
  } catch (error) {
    console.error('FormWindow: 窗口操作失败:', error)
    message.error('窗口操作失败')
  }
}

const onClose = async () => {
  console.log('FormWindow: 关闭窗口')
  try {
    await closeWindow()
    console.log('FormWindow: 窗口关闭成功')
  } catch (error) {
    console.error('FormWindow: 关闭失败:', error)
    message.error('关闭失败')
  }
}

const onReset = () => {
  formRef.value?.resetFields()
  message.info('表单已重置')
}

const onCancel = async () => {
  try {
    await closeWindow()
  } catch (error) {
    message.error('关闭窗口失败')
  }
}

const onSubmit = async () => {
  try {
    await formRef.value?.validate()
    
    submitting.value = true
    
    // 模拟提交延迟
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    // 这里可以添加实际的数据提交逻辑
    console.log('提交表单数据:', formData.value)
    
    // 向父窗口发送表单提交事件
    try {
      await sendFormSubmit({
        action: props.mode,
        formType: props.formType,
        data: formData.value,
        timestamp: Date.now()
      })
      console.log('表单数据已发送到父窗口')
    } catch (error) {
      console.error('发送表单数据失败:', error)
    }
    
    message.success(`${props.mode === 'create' ? '创建' : '更新'}成功`)
    
    // 延迟关闭窗口，给父窗口处理时间
    setTimeout(async () => {
      await closeWindow()
    }, 500)
  } catch (error) {
    console.error('表单验证失败:', error)
    message.error('请检查表单数据')
  } finally {
    submitting.value = false
  }
}

// 测试父子窗口交互
const openTestModal = async () => {
  try {
    await createModalDialog({
      windowId: `test-modal-${Date.now()}`,
      title: '测试模态对话框',
      url: '/form-window?mode=create&formType=test&modal=true',
      width: 500,
      height: 300
    })
    message.info('测试模态窗口已打开')
  } catch (error) {
    message.error('打开模态窗口失败')
  }
}

const openTestTool = async () => {
  try {
    await createToolWindow({
      windowId: `test-tool-${Date.now()}`,
      title: '测试工具窗口',
      url: '/form-window?mode=view&formType=tool&modal=false',
      width: 600,
      height: 400
    })
    message.info('测试工具窗口已打开')
  } catch (error) {
    message.error('打开工具窗口失败')
  }
}

// 监听路由参数变化（如果有的话）
watch(() => props.initialData, (newData) => {
  Object.assign(formData.value, newData)
}, { deep: true })

onMounted(() => {
  console.log('表单窗口组件已挂载', {
    mode: props.mode,
    formType: props.formType,
    isModal: props.isModal,
    parentWindow: props.parentWindow
  })
})
</script>

<style scoped>
/* 确保AppHeader组件的样式正确应用 */
:deep(.app-header) {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%) !important;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  padding: 0;
  height: 60px;
  position: relative;
  z-index: 1000;
}

:deep(.app-header .header-content) {
  display: flex;
  align-items: center;
  height: 100%;
  max-width: 100%;
  padding: 0 20px;
  position: relative;
}

:deep(.app-header .header-left) {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

:deep(.app-header .header-center) {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 0;
}

:deep(.app-header .header-right) {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

:deep(.app-header .logo h2) {
  margin: 0;
  color: white;
  font-weight: 600;
  font-size: 18px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

:deep(.app-header .window-controls) {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: 12px;
}

:deep(.app-header .window-control-btn) {
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

:deep(.app-header .window-control-btn:hover) {
  background: rgba(255, 255, 255, 0.2);
  transform: translateY(-1px);
}

:deep(.app-header .window-control-btn:active) {
  transform: translateY(0);
}

:deep(.app-header .window-control-btn:disabled) {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

:deep(.app-header .window-control-btn.close-btn:hover) {
  background: rgba(239, 68, 68, 0.8);
}

:deep(.app-header .window-control-btn svg) {
  width: 12px;
  height: 12px;
}

/* 确保拖拽区域生效 */
:deep(.app-header.draggable) {
  -webkit-app-region: drag;
}

.form-window {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #f0f2f5;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.window-type {
  padding: 2px 8px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 4px;
  font-size: 12px;
  color: white;
}

.form-content {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
}

.form-info {
  margin-bottom: 16px;
}

.form-actions {
  text-align: right;
}

/* 响应式调整 */
@media (max-width: 768px) {
  .form-content {
    padding: 16px;
  }
  
  .form-actions {
    text-align: center;
  }
}
</style>