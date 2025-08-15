<template>
  <div class="cost-table-container">
    <div class="table-toolbar">
      <a-space>
        <a-button type="primary" @click="addRow">
          <template #icon><PlusOutlined /></template>
          新增
        </a-button>
        <a-button danger @click="deleteSelected" :disabled="!hasSelection">
          <template #icon><DeleteOutlined /></template>
          删除
        </a-button>
        <a-button @click="exportData">
          <template #icon><ExportOutlined /></template>
          导出
        </a-button>
        <a-button @click="openFormWindow" type="primary" ghost>
          <template #icon><FormOutlined /></template>
          表单编辑
        </a-button>
      </a-space>
      <div class="table-search">
        <a-input-search
          v-model:value="searchText"
          placeholder="搜索..."
          @search="handleSearch"
          @change="handleSearch"
          allow-clear
        />
      </div>
    </div>

    <div class="table-content">
      <s-table
        :columns="tableColumns"
        :data-source="filteredData"
        :row-selection="rowSelection"
        :loading="loading"
        :scroll="{ x: 1200, y: 400 }"
        size="middle"
        bordered
        @change="handleTableChange"
      >
      
      </s-table>
    </div>

    <div class="table-footer">
      <div class="summary-info">
        <a-space>
          <span>总计: ¥{{ formatAmount(summary.total) }}</span>
          <span>已选: {{ selectedRowKeys.length }} 项</span>
          <span>选中金额: ¥{{ formatAmount(summary.selected) }}</span>
        </a-space>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { message } from 'ant-design-vue'
import {
  PlusOutlined,
  DeleteOutlined,
  ExportOutlined,
  FormOutlined
} from '@ant-design/icons-vue'
import { useTableOperations } from '../composables/useTableOperations.js'
import { useCostCalculation } from '../composables/useCostCalculation.js'

const props = defineProps({
  data: {
    type: Array,
    default: () => []
  },
  columns: {
    type: Array,
    required: true
  },
  tableType: {
    type: String,
    default: 'budget' // budget, estimate, settlement
  },
  editable: {
    type: Boolean,
    default: true
  },
  showFormButton: {
    type: Boolean,
    default: true
  }
})

const emit = defineEmits(['dataChange', 'rowSelect', 'cellEdit', 'openForm', 'editRow', 'deleteRow'])

const searchText = ref('')
const loading = ref(false)
const selectedRowKeys = ref([])

const { calculateTotal, calculateSelected } = useCostCalculation()

// 计算属性
const filteredData = computed(() => {
  if (!searchText.value) return props.data

  return props.data.filter(item => {
    const searchLower = searchText.value.toLowerCase()
    return Object.values(item).some(value =>
      String(value).toLowerCase().includes(searchLower)
    )
  })
})

const tableColumns = computed(() => {
  return props.columns.map(col => ({
    ...col,
    key: col.dataIndex || col.field,
    dataIndex: col.dataIndex || col.field,
    sorter: col.sorter !== false,
    ellipsis: true
  }))
})

const summary = computed(() => {
  const selectedData = props.data.filter(item => selectedRowKeys.value.includes(item.id))
  return {
    total: calculateTotal(props.data),
    selected: calculateSelected(selectedData)
  }
})

const hasSelection = computed(() => selectedRowKeys.value.length > 0)

const paginationConfig = computed(() => ({
  current: 1,
  pageSize: 20,
  total: filteredData.value.length,
  showSizeChanger: true,
  showQuickJumper: true,
  showTotal: (total, range) => `第 ${range[0]}-${range[1]} 条，共 ${total} 条`,
  pageSizeOptions: ['10', '20', '50', '100']
}))

const rowSelection = computed(() => ({
  selectedRowKeys: selectedRowKeys.value,
  onChange: (keys, rows) => {
    selectedRowKeys.value = keys
    emit('rowSelect', rows)
  },
  getCheckboxProps: (record) => ({
    disabled: record.disabled === true,
    name: record.name,
  }),
}))

// 方法
const formatAmount = (amount) => {
  if (!amount) return '0.00'
  return Number(amount).toLocaleString('zh-CN', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  })
}

const getStatusColor = (status) => {
  const colors = {
    draft: 'default',
    reviewing: 'processing',
    approved: 'success',
    rejected: 'error',
    active: 'blue',
    completed: 'green',
    cancelled: 'red'
  }
  return colors[status] || 'default'
}

const getStatusText = (status) => {
  const texts = {
    draft: '草稿',
    reviewing: '审核中',
    approved: '已批准',
    rejected: '已拒绝',
    active: '进行中',
    completed: '已完成',
    cancelled: '已取消'
  }
  return texts[status] || status
}

const handleTableChange = (pagination, filters, sorter) => {
  console.log('Table changed:', pagination, filters, sorter)
}

const addRow = () => {
  emit('openForm', {
    type: 'create',
    data: null
  })
}

const deleteSelected = () => {
  if (selectedRowKeys.value.length === 0) {
    message.warning('请先选择要删除的数据')
    return
  }

  const selectedData = props.data.filter(item => selectedRowKeys.value.includes(item.id))
  selectedData.forEach(record => {
    emit('deleteRow', record)
  })

  selectedRowKeys.value = []
  message.success(`已删除 ${selectedData.length} 条记录`)
}

const exportData = () => {
  const dataToExport = selectedRowKeys.value.length > 0
    ? props.data.filter(item => selectedRowKeys.value.includes(item.id))
    : props.data

  console.log('导出数据:', dataToExport)
  message.success(`已导出 ${dataToExport.length} 条数据`)
}

const handleSearch = () => {
  // 搜索逻辑已在 computed 中处理
}

const openFormWindow = () => {
  emit('openForm', {
    type: 'create',
    data: null
  })
}

const editRow = (record) => {
  emit('editRow', record)
}

const viewDetail = (record) => {
  message.info(`查看 ${record.name} 详情`)
}

const deleteRow = (record) => {
  emit('deleteRow', record)
}

// 监听数据变化
watch(() => props.data, () => {
  // 清除无效的选中项
  selectedRowKeys.value = selectedRowKeys.value.filter(key =>
    props.data.some(item => item.id === key)
  )
}, { deep: true })

defineExpose({
  getTableData: () => filteredData.value,
  getSelectedData: () => props.data.filter(item => selectedRowKeys.value.includes(item.id)),
  clearSelection: () => { selectedRowKeys.value = [] },
  refreshTable: () => { /* Ant Table 自动刷新 */ }
})
</script>

<style scoped>
.cost-table-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.table-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 0;
  border-bottom: 1px solid #f0f0f0;
  margin-bottom: 16px;
}

.table-search {
  width: 300px;
}

.table-content {
  flex: 1;
  min-height: 400px;
}

.table-footer {
  padding: 12px 0;
  border-top: 1px solid #f0f0f0;
  background: #fafafa;
  margin-top: 16px;
}

.summary-info {
  font-size: 14px;
  color: #666;
  font-weight: 500;
}

.amount-cell {
  font-weight: 600;
  color: #1890ff;
}

:deep(.ant-table-thead > tr > th) {
  background: #fafafa;
  font-weight: 600;
}

:deep(.ant-table-tbody > tr:hover > td) {
  background: #e6f7ff;
}

:deep(.ant-table-row-selected > td) {
  background: #bae7ff;
}
</style>
