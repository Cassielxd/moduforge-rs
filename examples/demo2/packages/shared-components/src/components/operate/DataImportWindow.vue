<template>
  <div class="data-import-window">
    <!-- 公共窗口头部 -->
    <ModalWindowHeader
      :title="operateLabel || '数据导入'"
      :subtitle="'支持 Excel、CSV 格式文件导入'"
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
      <a-steps :current="currentStep" class="import-steps">
        <a-step title="选择文件" description="上传数据文件" />
        <a-step title="数据预览" description="预览导入数据" />
        <a-step title="导入设置" description="配置导入选项" />
        <a-step title="完成导入" description="执行数据导入" />
      </a-steps>

      <!-- 步骤1: 文件上传 -->
      <div v-if="currentStep === 0" class="step-content">
        <a-upload-dragger
          v-model:file-list="fileList"
          :before-upload="beforeUpload"
          :remove="handleRemove"
          accept=".xlsx,.xls,.csv"
          :multiple="false"
        >
          <p class="ant-upload-drag-icon">
            <inbox-outlined />
          </p>
          <p class="ant-upload-text">点击或拖拽文件到此区域上传</p>
          <p class="ant-upload-hint">
            支持 Excel (.xlsx, .xls) 和 CSV (.csv) 格式文件
          </p>
        </a-upload-dragger>

        <div class="file-info" v-if="selectedFile">
          <a-descriptions title="文件信息" bordered size="small">
            <a-descriptions-item label="文件名">{{ selectedFile.name }}</a-descriptions-item>
            <a-descriptions-item label="文件大小">{{ formatFileSize(selectedFile.size) }}</a-descriptions-item>
            <a-descriptions-item label="文件类型">{{ selectedFile.type || '未知' }}</a-descriptions-item>
          </a-descriptions>
        </div>
      </div>

      <!-- 步骤2: 数据预览 -->
      <div v-if="currentStep === 1" class="step-content">
        <div class="preview-header">
          <a-space>
            <span>预览数据 (前 {{ Math.min(previewData.length, 10) }} 行)</span>
            <a-tag color="blue">总计 {{ previewData.length }} 行</a-tag>
          </a-space>
        </div>

        <a-table
          :columns="previewColumns"
          :data-source="previewData.slice(0, 10)"
          :pagination="false"
          size="small"
          bordered
          :scroll="{ x: 800 }"
        />
      </div>

      <!-- 步骤3: 导入设置 -->
      <div v-if="currentStep === 2" class="step-content">
        <a-form layout="vertical">
          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="导入模式">
                <a-radio-group v-model:value="importSettings.mode">
                  <a-radio value="append">追加数据</a-radio>
                  <a-radio value="replace">替换数据</a-radio>
                  <a-radio value="update">更新数据</a-radio>
                </a-radio-group>
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="重复数据处理">
                <a-select v-model:value="importSettings.duplicateHandling">
                  <a-select-option value="skip">跳过重复</a-select-option>
                  <a-select-option value="overwrite">覆盖重复</a-select-option>
                  <a-select-option value="rename">重命名重复</a-select-option>
                </a-select>
              </a-form-item>
            </a-col>
          </a-row>

          <a-form-item label="字段映射">
            <a-table
              :columns="mappingColumns"
              :data-source="fieldMappings"
              :pagination="false"
              size="small"
            />
          </a-form-item>

          <a-form-item>
            <a-checkbox v-model:checked="importSettings.validateData">
              导入前验证数据
            </a-checkbox>
          </a-form-item>
        </a-form>
      </div>

      <!-- 步骤4: 导入进度 -->
      <div v-if="currentStep === 3" class="step-content">
        <div class="import-progress">
          <a-progress
            :percent="importProgress"
            :status="importStatus"
            :stroke-color="importStatus === 'exception' ? '#ff4d4f' : '#1890ff'"
          />
          <p class="progress-text">{{ progressText }}</p>
        </div>

        <div v-if="importResult" class="import-result">
          <a-result
            :status="importResult.success ? 'success' : 'error'"
            :title="importResult.title"
            :sub-title="importResult.message"
          >
            <template #extra>
              <a-space>
                <a-button v-if="importResult.success" type="primary" @click="handleComplete">
                  完成
                </a-button>
                <a-button v-else @click="handleRetry">重新导入</a-button>
                <a-button @click="handleCancel">关闭</a-button>
              </a-space>
            </template>
          </a-result>
        </div>
      </div>
    </div>

    <!-- 公共窗口底部 -->
    <div class="window-footer">
      <a-space>
        <a-button v-if="currentStep > 0 && currentStep < 3" @click="prevStep">
          上一步
        </a-button>
        <a-button
          v-if="currentStep < 2"
          type="primary"
          :disabled="!canNextStep"
          @click="nextStep"
        >
          下一步
        </a-button>
        <a-button
          v-if="currentStep === 2"
          type="primary"
          :loading="importing"
          @click="startImport"
        >
          开始导入
        </a-button>
        <a-button @click="handleCancel">取消</a-button>
      </a-space>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, defineProps, defineEmits, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { InboxOutlined } from '@ant-design/icons-vue'
import ModalWindowHeader from '../ModalWindowHeader.vue'
import { useWindowControls } from '../../composables/useWindowControls.js'

// Props
const props = defineProps({
  operateType: String,
  operateLabel: String,
  data: Array,
  tableData: Array,
  parentWindow: String,
  appId: String,
  selectedCount: Number,
  totalCount: Number,
  hasSelection: Boolean
})

// Emits
const emit = defineEmits(['submit', 'cancel', 'close', 'update'])

// 使用窗口控制 composable
const { isMaximized, handleMinimize, handleMaximize, handleClose } = useWindowControls()

// 响应式数据
const loading = ref(true)
const currentStep = ref(0)
const fileList = ref([])
const selectedFile = ref(null)
const previewData = ref([])
const previewColumns = ref([])
const importing = ref(false)
const importProgress = ref(0)
const importStatus = ref('active')
const importResult = ref(null)

// 导入设置
const importSettings = ref({
  mode: 'append',
  duplicateHandling: 'skip',
  validateData: true
})

// 字段映射
const fieldMappings = ref([
  { key: 'name', sourceField: '项目名称', targetField: 'name', required: true },
  { key: 'code', sourceField: '项目编码', targetField: 'code', required: true },
  { key: 'amount', sourceField: '金额', targetField: 'amount', required: false },
  { key: 'unit', sourceField: '单位', targetField: 'unit', required: false }
])

const mappingColumns = [
  { title: '源字段', dataIndex: 'sourceField', key: 'sourceField' },
  { title: '目标字段', dataIndex: 'targetField', key: 'targetField' },
  { title: '必填', dataIndex: 'required', key: 'required', 
    customRender: ({ record }) => record.required ? '是' : '否' }
]

// 计算属性
const canNextStep = computed(() => {
  if (currentStep.value === 0) return selectedFile.value !== null
  if (currentStep.value === 1) return previewData.value.length > 0
  return true
})

const progressText = computed(() => {
  if (importProgress.value === 0) return '准备导入...'
  if (importProgress.value === 100) return '导入完成'
  return `正在导入... ${importProgress.value}%`
})

// 文件处理
const beforeUpload = (file) => {
  selectedFile.value = file
  parseFile(file)
  return false
}

const handleRemove = () => {
  selectedFile.value = null
  previewData.value = []
  previewColumns.value = []
}

const parseFile = async (file) => {
  try {
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    previewData.value = [
      { name: '土建工程', code: 'EST001', amount: 1000000, unit: '项' },
      { name: '安装工程', code: 'EST002', amount: 500000, unit: '项' },
      { name: '装饰工程', code: 'EST003', amount: 300000, unit: '项' }
    ]
    
    previewColumns.value = [
      { title: '项目名称', dataIndex: 'name', key: 'name' },
      { title: '项目编码', dataIndex: 'code', key: 'code' },
      { title: '金额', dataIndex: 'amount', key: 'amount' },
      { title: '单位', dataIndex: 'unit', key: 'unit' }
    ]
    
    message.success('文件解析成功')
  } catch (error) {
    message.error('文件解析失败')
    console.error(error)
  }
}

// 步骤控制
const nextStep = () => {
  if (canNextStep.value) {
    currentStep.value++
  }
}

const prevStep = () => {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}

// 导入处理
const startImport = async () => {
  importing.value = true
  importProgress.value = 0
  importStatus.value = 'active'
  currentStep.value = 3

  try {
    for (let i = 0; i <= 100; i += 10) {
      await new Promise(resolve => setTimeout(resolve, 200))
      importProgress.value = i
    }

    importResult.value = {
      success: true,
      title: '导入成功',
      message: `成功导入 ${previewData.value.length} 条数据`
    }
    importStatus.value = 'success'
  } catch (error) {
    importResult.value = {
      success: false,
      title: '导入失败',
      message: error.message || '导入过程中发生错误'
    }
    importStatus.value = 'exception'
  } finally {
    importing.value = false
  }
}

// 事件处理
const handleComplete = () => {
  emit('submit', {
    type: 'import',
    data: previewData.value,
    settings: importSettings.value
  })
}

const handleRetry = () => {
  currentStep.value = 0
  importResult.value = null
  importProgress.value = 0
}

const handleCancel = async () => {
  await handleClose(emit)
}

// 工具函数
const formatFileSize = (bytes) => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

// 初始化
onMounted(async () => {
  // 模拟加载延迟 - 减少到300ms
  await new Promise(resolve => setTimeout(resolve, 300))
  loading.value = false
})
</script>

<style scoped>
.data-import-window {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #fff;
}

.loading-skeleton {
  padding: 24px;
  flex: 1;
}

.window-content {
  flex: 1;
  overflow-y: auto;
  padding: 0 24px;
}

.import-steps {
  margin-bottom: 32px;
}

.step-content {
  min-height: 300px;
}

.file-info {
  margin-top: 16px;
}

.preview-header {
  margin-bottom: 16px;
}

.import-progress {
  text-align: center;
  padding: 40px 0;
}

.progress-text {
  margin-top: 16px;
  font-size: 16px;
}

.import-result {
  margin-top: 32px;
}

.window-footer {
  padding: 16px 24px;
  border-top: 1px solid #f0f0f0;
  text-align: right;
  background: #fafafa;
}
</style>
