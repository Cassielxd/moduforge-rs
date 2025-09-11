<template>
  <div class="form-window">
    <!-- 公共窗口头部 -->
    <ModalWindowHeader
      :title="operateLabel || '表单'"
      :subtitle="getSubtitle()"
      :show-close="true"
      :is-maximized="isMaximized"
      @minimize="handleMinimize"
      @maximize="handleMaximize"
      @close="handleCancel"
    />

    <!-- 加载骨架屏 -->
    <div v-if="loading" class="loading-skeleton">
      <a-skeleton active :paragraph="{ rows: 8 }" />
    </div>

    <!-- 窗口内容 -->
    <div v-else class="window-content">
      <a-form
        ref="formRef"
        :model="formData"
        :rules="formRules"
        layout="vertical"
        class="form-container"
      >
        <!-- 基本信息 -->
        <a-card title="基本信息" :bordered="false" class="form-card">
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="名称" name="name">
                <a-input 
                  v-model:value="formData.name" 
                  placeholder="请输入名称"
                  :disabled="mode === 'view'"
                />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="类型" name="type">
                <a-select 
                  v-model:value="formData.type" 
                  placeholder="请选择类型"
                  :disabled="mode === 'view'"
                >
                  <a-select-option value="estimate">概算</a-select-option>
                  <a-select-option value="budget">预算</a-select-option>
                  <a-select-option value="contract">合同</a-select-option>
                </a-select>
              </a-form-item>
            </a-col>
          </a-row>
          
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="金额" name="amount">
                <a-input-number
                  v-model:value="formData.amount"
                  placeholder="请输入金额"
                  :min="0"
                  :precision="2"
                  style="width: 100%"
                  :disabled="mode === 'view'"
                />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="创建人" name="creator">
                <a-input 
                  v-model:value="formData.creator" 
                  placeholder="请输入创建人"
                  :disabled="mode === 'view'"
                />
              </a-form-item>
            </a-col>
          </a-row>

          <a-form-item label="描述" name="description">
            <a-textarea
              v-model:value="formData.description"
              placeholder="请输入描述"
              :rows="3"
              :disabled="mode === 'view'"
            />
          </a-form-item>
        </a-card>

        <!-- 时间信息 -->
        <a-card title="时间信息" :bordered="false" class="form-card">
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="开始日期" name="startDate">
                <a-date-picker
                  v-model:value="formData.startDate"
                  placeholder="请选择开始日期"
                  style="width: 100%"
                  :disabled="mode === 'view'"
                />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="结束日期" name="endDate">
                <a-date-picker
                  v-model:value="formData.endDate"
                  placeholder="请选择结束日期"
                  style="width: 100%"
                  :disabled="mode === 'view'"
                />
              </a-form-item>
            </a-col>
          </a-row>
        </a-card>
      </a-form>

      <!-- 底部操作按钮 -->
      <div class="form-actions">
        <a-space>
          <a-button @click="handleCancel">
            {{ mode === 'view' ? '关闭' : '取消' }}
          </a-button>
          <a-button v-if="mode !== 'view'" @click="handleReset">
            重置
          </a-button>
          <a-button 
            v-if="mode !== 'view'"
            type="primary" 
            :loading="submitting"
            @click="handleSubmit"
          >
            {{ mode === 'create' ? '创建' : '保存' }}
          </a-button>
        </a-space>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, defineProps, defineEmits, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import ModalWindowHeader from '../ModalWindowHeader.vue'
import { useWindowControls } from '../../composables/useWindowControls.js'

// Props
const props = defineProps({
  operateType: String,
  operateLabel: String,
  mode: {
    type: String,
    default: 'create',
    validator: value => ['create', 'edit', 'view'].includes(value)
  },
  data: {
    type: Object,
    default: () => ({})
  },
  parentWindow: String,
  appId: String
})

// Emits
const emit = defineEmits(['submit', 'cancel', 'close', 'update'])

// 使用窗口控制 composable
const { isMaximized, handleMinimize, handleMaximize, handleClose } = useWindowControls()

// 响应式数据
const loading = ref(true)
const submitting = ref(false)
const formRef = ref()

// 表单数据
const formData = ref({
  name: '',
  type: '',
  amount: 0,
  creator: '',
  description: '',
  startDate: null,
  endDate: null,
  ...props.data
})

// 表单验证规则
const formRules = {
  name: [
    { required: true, message: '请输入名称', trigger: 'blur' }
  ],
  type: [
    { required: true, message: '请选择类型', trigger: 'change' }
  ],
  amount: [
    { required: true, message: '请输入金额', trigger: 'blur' }
  ],
  creator: [
    { required: true, message: '请输入创建人', trigger: 'blur' }
  ]
}

// 计算属性
const getSubtitle = () => {
  const modeText = {
    create: '创建新记录',
    edit: '编辑记录信息',
    view: '查看记录详情'
  }
  return modeText[props.mode] || '表单操作'
}

// 方法
const handleCancel = async () => {
  await handleClose(emit)
}

const handleReset = () => {
  formRef.value?.resetFields()
  message.info('表单已重置')
}

const handleSubmit = async () => {
  try {
    await formRef.value?.validate()
    
    submitting.value = true
    
    // 模拟提交延迟
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    console.log('提交表单数据:', formData.value)
    
    // 发出提交事件
    emit('submit', {
      mode: props.mode,
      data: formData.value,
      timestamp: Date.now()
    })
    
    message.success(`${props.mode === 'create' ? '创建' : '保存'}成功`)
    
    // 延迟关闭窗口
    setTimeout(async () => {
      await handleClose(emit)
    }, 500)
  } catch (error) {
    console.error('表单验证失败:', error)
    message.error('请检查表单数据')
  } finally {
    submitting.value = false
  }
}

// 初始化
onMounted(async () => {
  // 模拟加载延迟 - 300ms
  await new Promise(resolve => setTimeout(resolve, 300))
  loading.value = false
})
</script>

<style scoped>
.form-window {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #f5f5f5;
}

.loading-skeleton {
  padding: 20px;
  background: white;
  height: calc(100vh - 60px);
  overflow: hidden;
}

.window-content {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
  background: #f5f5f5;
}

.form-container {
  max-width: 800px;
  margin: 0 auto;
}

.form-card {
  margin-bottom: 16px;
}

.form-actions {
  background: white;
  padding: 16px 20px;
  border-top: 1px solid #f0f0f0;
  text-align: right;
  margin-top: 20px;
}

/* 响应式调整 */
@media (max-width: 768px) {
  .window-content {
    padding: 16px;
  }
  
  .form-actions {
    text-align: center;
  }
}
</style>
