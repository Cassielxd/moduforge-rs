<template>
  <div class="batch-operation-window">
    <!-- 公共窗口头部 -->
    <ModalWindowHeader
      :title="operateLabel || '批量操作'"
      :subtitle="'对选中的数据执行批量操作'"
      :show-close="true"
      :is-maximized="isMaximized"
      @minimize="handleMinimize"
      @maximize="handleMaximize"
      @close="handleCancel"
    />

    <!-- 加载骨架屏 -->
    <div v-if="loading" class="loading-skeleton">
      <a-skeleton active :paragraph="{ rows: 6 }" />
    </div>

    <!-- 窗口内容 -->
    <div v-else class="window-content">
      <!-- 选中数据概览 -->
      <a-card title="选中数据概览" size="small" class="data-overview">
        <a-descriptions :column="2" size="small">
          <a-descriptions-item label="选中数量">{{ selectedCount }} 条</a-descriptions-item>
          <a-descriptions-item label="总数量">{{ totalCount }} 条</a-descriptions-item>
          <a-descriptions-item label="选中比例">{{ selectionPercentage }}%</a-descriptions-item>
          <a-descriptions-item label="操作类型">{{ operateLabel }}</a-descriptions-item>
        </a-descriptions>
      </a-card>

      <!-- 批量操作选项 -->
      <a-form layout="vertical" class="operation-form">
        <a-form-item label="操作类型">
          <a-radio-group v-model:value="operationSettings.type">
            <a-radio-button value="approve">批量审批</a-radio-button>
            <a-radio-button value="reject">批量驳回</a-radio-button>
            <a-radio-button value="delete">批量删除</a-radio-button>
            <a-radio-button value="export">批量导出</a-radio-button>
            <a-radio-button value="update">批量更新</a-radio-button>
          </a-radio-group>
        </a-form-item>

        <!-- 审批/驳回选项 -->
        <div v-if="operationSettings.type === 'approve' || operationSettings.type === 'reject'">
          <a-form-item label="审批意见">
            <a-textarea
              v-model:value="operationSettings.comment"
              :placeholder="operationSettings.type === 'approve' ? '请输入审批意见' : '请输入驳回原因'"
              :rows="3"
            />
          </a-form-item>
          
          <a-form-item>
            <a-checkbox v-model:checked="operationSettings.notifyUsers">
              通知相关用户
            </a-checkbox>
          </a-form-item>
        </div>

        <!-- 删除选项 -->
        <div v-if="operationSettings.type === 'delete'">
          <a-alert
            message="警告"
            description="删除操作不可恢复，请确认是否继续"
            type="warning"
            show-icon
            class="delete-warning"
          />
          
          <a-form-item>
            <a-checkbox v-model:checked="operationSettings.confirmDelete">
              我确认要删除这些数据
            </a-checkbox>
          </a-form-item>
        </div>

        <!-- 导出选项 -->
        <div v-if="operationSettings.type === 'export'">
          <a-form-item label="导出格式">
            <a-select v-model:value="operationSettings.exportFormat">
              <a-select-option value="excel">Excel (.xlsx)</a-select-option>
              <a-select-option value="csv">CSV (.csv)</a-select-option>
              <a-select-option value="pdf">PDF (.pdf)</a-select-option>
            </a-select>
          </a-form-item>
          
          <a-form-item>
            <a-checkbox v-model:checked="operationSettings.includeHeader">
              包含表头
            </a-checkbox>
          </a-form-item>
        </div>

        <!-- 更新选项 -->
        <div v-if="operationSettings.type === 'update'">
          <a-form-item label="更新字段">
            <a-select v-model:value="operationSettings.updateField" placeholder="选择要更新的字段">
              <a-select-option value="status">状态</a-select-option>
              <a-select-option value="category">分类</a-select-option>
              <a-select-option value="priority">优先级</a-select-option>
              <a-select-option value="assignee">负责人</a-select-option>
            </a-select>
          </a-form-item>
          
          <a-form-item label="新值">
            <a-input
              v-model:value="operationSettings.updateValue"
              placeholder="请输入新的值"
            />
          </a-form-item>
        </div>

        <!-- 执行计划 -->
        <a-form-item label="执行方式">
          <a-radio-group v-model:value="operationSettings.executeMode">
            <a-radio value="immediate">立即执行</a-radio>
            <a-radio value="scheduled">定时执行</a-radio>
          </a-radio-group>
        </a-form-item>

        <a-form-item v-if="operationSettings.executeMode === 'scheduled'" label="执行时间">
          <a-date-picker
            v-model:value="operationSettings.scheduleTime"
            show-time
            placeholder="选择执行时间"
            style="width: 100%"
          />
        </a-form-item>
      </a-form>

      <!-- 操作进度 -->
      <div v-if="processing" class="operation-progress">
        <a-progress
          :percent="operationProgress"
          :status="operationStatus"
          :stroke-color="operationStatus === 'exception' ? '#ff4d4f' : '#1890ff'"
        />
        <p class="progress-text">{{ progressText }}</p>
      </div>
    </div>

    <!-- 公共窗口底部 -->
    <div class="window-footer">
      <a-space>
        <a-button
          type="primary"
          :loading="processing"
          :disabled="!canExecute"
          @click="executeOperation"
        >
          执行操作
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
const processing = ref(false)
const operationProgress = ref(0)
const operationStatus = ref('active')

// 操作设置
const operationSettings = ref({
  type: 'approve',
  comment: '',
  notifyUsers: true,
  confirmDelete: false,
  exportFormat: 'excel',
  includeHeader: true,
  updateField: '',
  updateValue: '',
  executeMode: 'immediate',
  scheduleTime: null
})

// 计算属性
const selectionPercentage = computed(() => {
  if (props.totalCount === 0) return 0
  return Math.round((props.selectedCount / props.totalCount) * 100)
})

const canExecute = computed(() => {
  const settings = operationSettings.value
  
  switch (settings.type) {
    case 'delete':
      return settings.confirmDelete
    case 'update':
      return settings.updateField && settings.updateValue
    case 'approve':
    case 'reject':
      return settings.comment.trim() !== ''
    default:
      return true
  }
})

const progressText = computed(() => {
  if (operationProgress.value === 0) return '准备执行操作...'
  if (operationProgress.value === 100) return '操作完成'
  return `正在执行操作... ${operationProgress.value}%`
})

// 方法
const executeOperation = async () => {
  processing.value = true
  operationProgress.value = 0
  operationStatus.value = 'active'

  try {
    // 模拟操作进度
    for (let i = 0; i <= 100; i += 10) {
      await new Promise(resolve => setTimeout(resolve, 200))
      operationProgress.value = i
    }

    operationStatus.value = 'success'
    message.success(`${operationSettings.value.type} 操作完成`)
    
    // 发送操作完成事件
    emit('submit', {
      type: 'batch-operation',
      operation: operationSettings.value.type,
      settings: operationSettings.value,
      affectedCount: props.selectedCount
    })
  } catch (error) {
    operationStatus.value = 'exception'
    message.error('操作失败')
    console.error(error)
  } finally {
    processing.value = false
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
})
</script>

<style scoped>
.batch-operation-window {
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

.data-overview {
  margin-bottom: 24px;
}

.operation-form {
  margin-bottom: 24px;
}

.delete-warning {
  margin-bottom: 16px;
}

.operation-progress {
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
