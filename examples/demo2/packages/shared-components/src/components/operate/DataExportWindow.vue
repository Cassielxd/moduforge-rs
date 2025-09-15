<template>
  <div class="data-export-window">
    <!-- 公共窗口头部 -->
    <ModalWindowHeader
      :title="operateLabel || '数据导出'"
      :subtitle="'导出表格数据到文件'"
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
      <a-form layout="vertical">
        <!-- 导出格式 -->
        <a-form-item label="导出格式">
          <a-radio-group v-model:value="exportSettings.format" @change="handleFormatChange">
            <a-radio-button value="excel">Excel (.xlsx)</a-radio-button>
            <a-radio-button value="csv">CSV (.csv)</a-radio-button>
            <a-radio-button value="pdf">PDF (.pdf)</a-radio-button>
            <a-radio-button value="json">JSON (.json)</a-radio-button>
          </a-radio-group>
        </a-form-item>

        <!-- 导出范围 -->
        <a-form-item label="导出范围">
          <a-radio-group v-model:value="exportSettings.range">
            <a-radio value="all">全部数据 ({{ totalCount }} 条)</a-radio>
            <a-radio value="selected" :disabled="!hasSelection">
              选中数据 ({{ selectedCount }} 条)
            </a-radio>
            <a-radio value="filtered">当前筛选结果</a-radio>
            <a-radio value="custom">自定义范围</a-radio>
          </a-radio-group>
        </a-form-item>

        <!-- 自定义范围 -->
        <a-form-item v-if="exportSettings.range === 'custom'" label="数据范围">
          <a-row :gutter="8">
            <a-col :span="8">
              <a-input-number
                v-model:value="exportSettings.customRange.start"
                :min="1"
                :max="totalCount"
                placeholder="起始行"
                style="width: 100%"
              />
            </a-col>
            <a-col :span="2" style="text-align: center; line-height: 32px;">至</a-col>
            <a-col :span="8">
              <a-input-number
                v-model:value="exportSettings.customRange.end"
                :min="exportSettings.customRange.start || 1"
                :max="totalCount"
                placeholder="结束行"
                style="width: 100%"
              />
            </a-col>
          </a-row>
        </a-form-item>

        <!-- 字段选择 -->
        <a-form-item label="导出字段">
          <div class="field-selection">
            <div class="field-actions">
              <a-space>
                <a-button size="small" @click="selectAllFields">全选</a-button>
                <a-button size="small" @click="clearAllFields">清空</a-button>
                <a-button size="small" @click="resetFields">重置</a-button>
              </a-space>
            </div>
            <a-checkbox-group v-model:value="exportSettings.fields" class="field-list">
              <a-row>
                <a-col v-for="field in availableFields" :key="field.key" :span="8">
                  <a-checkbox :value="field.key">{{ field.label }}</a-checkbox>
                </a-col>
              </a-row>
            </a-checkbox-group>
          </div>
        </a-form-item>

        <!-- 导出选项 -->
        <a-form-item label="导出选项">
          <a-space direction="vertical" style="width: 100%">
            <a-checkbox v-model:checked="exportSettings.includeHeader">
              包含表头
            </a-checkbox>
            <a-checkbox v-model:checked="exportSettings.includeIndex">
              包含序号
            </a-checkbox>
            <a-checkbox v-model:checked="exportSettings.includeTotal">
              包含合计行
            </a-checkbox>
            <a-checkbox 
              v-model:checked="exportSettings.compressFile"
              v-if="exportSettings.format !== 'pdf'"
            >
              压缩文件
            </a-checkbox>
          </a-space>
        </a-form-item>

        <!-- 文件名设置 -->
        <a-form-item label="文件名">
          <a-input
            v-model:value="exportSettings.fileName"
            placeholder="请输入文件名"
            :suffix="getFileExtension()"
          />
        </a-form-item>

        <!-- 预览信息 -->
        <a-form-item label="导出预览">
          <a-descriptions bordered size="small">
            <a-descriptions-item label="导出格式">
              {{ formatLabels[exportSettings.format] }}
            </a-descriptions-item>
            <a-descriptions-item label="数据行数">
              {{ getExportRowCount() }} 行
            </a-descriptions-item>
            <a-descriptions-item label="字段数量">
              {{ exportSettings.fields.length }} 个字段
            </a-descriptions-item>
            <a-descriptions-item label="预估大小">
              {{ estimateFileSize() }}
            </a-descriptions-item>
          </a-descriptions>
        </a-form-item>
      </a-form>

      <!-- 导出进度 -->
      <div v-if="exporting" class="export-progress">
        <a-progress
          :percent="exportProgress"
          :status="exportStatus"
          :stroke-color="exportStatus === 'exception' ? '#ff4d4f' : '#1890ff'"
        />
        <p class="progress-text">{{ progressText }}</p>
      </div>
    </div>

    <!-- 公共窗口底部 -->
    <div class="window-footer">
      <a-space>
        <a-button
          type="primary"
          :loading="exporting"
          :disabled="!canExport"
          @click="startExport"
        >
          开始导出
        </a-button>
        <a-button @click="handleCancel">取消</a-button>
      </a-space>
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
const exporting = ref(false)
const exportProgress = ref(0)
const exportStatus = ref('active')

// 导出设置
const exportSettings = ref({
  format: 'excel',
  range: 'all',
  customRange: { start: 1, end: props.totalCount },
  fields: [],
  includeHeader: true,
  includeIndex: true,
  includeTotal: false,
  compressFile: false,
  fileName: `数据导出_${new Date().toISOString().slice(0, 10)}`
})

// 可用字段
const availableFields = ref([
  { key: 'dispNo', label: '序号' },
  { key: 'name', label: '项目名称' },
  { key: 'code', label: '项目编码' },
  { key: 'amount', label: '金额' },
  { key: 'unit', label: '单位' },
  { key: 'quantity', label: '数量' },
  { key: 'unitPrice', label: '单价' },
  { key: 'status', label: '状态' },
  { key: 'createTime', label: '创建时间' },
  { key: 'updateTime', label: '更新时间' }
])

// 格式标签
const formatLabels = {
  excel: 'Microsoft Excel',
  csv: 'CSV 逗号分隔值',
  pdf: 'PDF 文档',
  json: 'JSON 数据'
}

// 计算属性
const canExport = computed(() => {
  return exportSettings.value.fields.length > 0 && 
         exportSettings.value.fileName.trim() !== ''
})

const progressText = computed(() => {
  if (exportProgress.value === 0) return '准备导出...'
  if (exportProgress.value === 100) return '导出完成'
  return `正在导出... ${exportProgress.value}%`
})

// 方法
const handleFormatChange = () => {
  if (exportSettings.value.format === 'pdf') {
    exportSettings.value.compressFile = false
  }
}

const getFileExtension = () => {
  const extensions = {
    excel: '.xlsx',
    csv: '.csv',
    pdf: '.pdf',
    json: '.json'
  }
  return extensions[exportSettings.value.format] || ''
}

const getExportRowCount = () => {
  switch (exportSettings.value.range) {
    case 'all':
      return props.totalCount
    case 'selected':
      return props.selectedCount
    case 'custom':
      const start = exportSettings.value.customRange.start || 1
      const end = exportSettings.value.customRange.end || props.totalCount
      return Math.max(0, end - start + 1)
    default:
      return props.totalCount
  }
}

const estimateFileSize = () => {
  const rowCount = getExportRowCount()
  const fieldCount = exportSettings.value.fields.length
  const avgCellSize = 20
  
  let size = rowCount * fieldCount * avgCellSize
  
  if (exportSettings.value.format === 'excel') {
    size *= 1.5
  } else if (exportSettings.value.format === 'pdf') {
    size *= 3
  }
  
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / (1024 * 1024)).toFixed(1)} MB`
}

const selectAllFields = () => {
  exportSettings.value.fields = availableFields.value.map(field => field.key)
}

const clearAllFields = () => {
  exportSettings.value.fields = []
}

const resetFields = () => {
  exportSettings.value.fields = ['name', 'code', 'amount', 'unit']
}

const startExport = async () => {
  exporting.value = true
  exportProgress.value = 0
  exportStatus.value = 'active'

  try {
    for (let i = 0; i <= 100; i += 10) {
      await new Promise(resolve => setTimeout(resolve, 200))
      exportProgress.value = i
    }

    exportStatus.value = 'success'
    message.success('导出完成')
    
    emit('submit', {
      type: 'export',
      settings: exportSettings.value,
      rowCount: getExportRowCount()
    })
  } catch (error) {
    exportStatus.value = 'exception'
    message.error('导出失败')
    console.error(error)
  } finally {
    exporting.value = false
  }
}

const handleCancel = async () => {
  await handleClose(emit)
}

// 初始化
onMounted(async () => {
  // 模拟加载延迟 - 减少到300ms
  await new Promise(resolve => setTimeout(resolve, 300))
  loading.value = false

  // 默认选择常用字段
  resetFields()

  // 如果有选中数据，默认导出选中数据
  if (props.hasSelection) {
    exportSettings.value.range = 'selected'
  }
})
</script>

<style scoped>
.data-export-window {
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

.field-selection {
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  padding: 12px;
}

.field-actions {
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #f0f0f0;
}

.field-list {
  width: 100%;
}

.field-list .ant-checkbox-wrapper {
  margin-bottom: 8px;
}

.export-progress {
  margin-top: 24px;
  padding: 20px;
  background: #fafafa;
  border-radius: 6px;
  text-align: center;
}

.progress-text {
  margin-top: 12px;
  font-size: 14px;
  color: #666;
}

.window-footer {
  padding: 16px 24px;
  border-top: 1px solid #f0f0f0;
  text-align: right;
  background: #fafafa;
}
</style>
