<template>
  <div id="app">
    <a-layout class="layout">
      <!-- 使用共享头部组件 -->
      <SimpleHeader
        title="概算管理系统"
        :show-window-controls="true"
        @minimize="onMinimize"
        @maximize="onMaximize"
        @close="onClose"
      >
        <template #right>
          <div class="actions">
            <a-space>
              <a-button type="primary" @click="newEstimate">
                <template #icon><PlusOutlined /></template>
                新建概算
              </a-button>
              <a-button @click="importData">
                <template #icon><ImportOutlined /></template>
                导入数据
              </a-button>
              <a-button @click="exportData">
                <template #icon><ExportOutlined /></template>
                导出数据
              </a-button>
            </a-space>
          </div>
        </template>
      </SimpleHeader>

      <!-- 主内容区 -->
      <a-layout-content class="content">
        <div class="content-wrapper">
          <!-- 工具栏 -->
          <a-card class="toolbar-card" :bordered="false">
            <a-row :gutter="16" align="middle">
              <a-col :span="8">
                <a-input-search
                  v-model:value="searchText"
                  placeholder="搜索概算项目..."
                  @search="handleSearch"
                  style="width: 100%"
                />
              </a-col>
              <a-col :span="4">
                <a-select
                  v-model:value="statusFilter"
                  placeholder="状态筛选"
                  style="width: 100%"
                  @change="handleStatusFilter"
                >
                  <a-select-option value="">全部状态</a-select-option>
                  <a-select-option value="draft">草稿</a-select-option>
                  <a-select-option value="reviewing">审核中</a-select-option>
                  <a-select-option value="approved">已批准</a-select-option>
                  <a-select-option value="rejected">已拒绝</a-select-option>
                </a-select>
              </a-col>
              <a-col :span="4">
                <a-date-picker
                  v-model:value="dateFilter"
                  placeholder="选择日期"
                  style="width: 100%"
                  @change="handleDateFilter"
                />
              </a-col>
              <a-col :span="8" style="text-align: right">
                <a-space>
                  <a-button @click="handleRefresh">
                    刷新
                  </a-button>
                  <a-button type="primary" @click="newEstimate">
                    新建概算
                  </a-button>
                </a-space>
              </a-col>
            </a-row>
          </a-card>

          <!-- 数据表格 -->
          <a-card class="table-card" :bordered="false">
            <CostTable
              :data="filteredData"
              :columns="costTableColumns"
              table-type="estimate"
              :editable="false"
              @data-change="handleCostTableDataChange"
              @row-select="handleCostTableRowSelect"
              @cell-edit="handleCostTableCellEdit"
              @open-form="handleOpenForm"
              @edit-row="handleEditRow"
              @delete-row="handleDeleteRow"
            />
          </a-card>
        </div>
      </a-layout-content>
    </a-layout>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import {
  PlusOutlined,
  ImportOutlined,
  ExportOutlined
} from '@ant-design/icons-vue'
import { CostTable, useEstimate, useGlobalStore, SimpleHeader } from '@cost-app/shared-components'
import { invoke } from '@tauri-apps/api/core'


// 数据状态
const loading = ref(false)
const searchText = ref('')
const statusFilter = ref('')
const dateFilter = ref(null)

// 表格配置（Ant Table 用）
const columns = [
  {
    title: '项目名称',
    dataIndex: 'name',
    key: 'name',
    width: 200,
  },
  {
    title: '概算金额',
    dataIndex: 'amount',
    key: 'amount',
    width: 150,
    sorter: (a, b) => a.amount - b.amount,
  },
  {
    title: '状态',
    dataIndex: 'status',
    key: 'status',
    width: 100,
  },
  {
    title: '创建时间',
    dataIndex: 'createTime',
    key: 'createTime',
    width: 150,
    sorter: (a, b) => new Date(a.createTime) - new Date(b.createTime),
  },
  {
    title: '创建人',
    dataIndex: 'creator',
    key: 'creator',
    width: 100,
  },
  {
    title: '操作',
    key: 'action',
    width: 200,
  },
]

// 共享表格组件（CostTable）相关配置
const costTableColumns = [
  {
    title: '项目名称',
    dataIndex: 'name',
    key: 'name',
    width: 200,
    sorter: true
  },
  {
    title: '项目类型',
    dataIndex: 'type',
    key: 'type',
    width: 120
  },
  {
    title: '概算金额',
    dataIndex: 'amount',
    key: 'amount',
    width: 150,
    sorter: true
  },
  {
    title: '状态',
    dataIndex: 'status',
    key: 'status',
    width: 100
  },
  {
    title: '负责人',
    dataIndex: 'creator',
    key: 'creator',
    width: 100
  },
  {
    title: '创建时间',
    dataIndex: 'createTime',
    key: 'createTime',
    width: 150,
    sorter: true
  },
  {
    title: '操作',
    key: 'action',
    width: 200,
    fixed: 'right'
  }
]

const handleCostTableDataChange = (newData) => {
  estimateData.value = newData
  pagination.value.total = newData.length
}

const handleCostTableRowSelect = (row) => {
  message.info(`选中：${row.name}`)
}



// 分页配置
const pagination = ref({
  current: 1,
  pageSize: 10,
  total: 0,
  showSizeChanger: true,
  showQuickJumper: true,
  showTotal: (total, range) => `第 ${range[0]}-${range[1]} 条，共 ${total} 条`,
})

// 使用共享状态管理
const {
  projects: estimateData,
  selectedItems,
  filters,
  filteredProjects,
  totalProjects,
  selectedCount,
  addProject,
  updateProject,
  deleteProject,
  setProjects,
  selectItems,
  clearSelection,
  setFilter,
  clearFilters
} = useEstimate()

// 初始化数据（如果没有数据的话）
if (estimateData.length === 0) {
  setProjects([
    {
      id: 1,
      name: '办公楼建设项目',
      type: 'building',
      amount: 5000000,
      status: 'approved',
      createTime: '2024-01-15',
      creator: '张三',
      description: '新建办公楼项目，包含主体建筑和配套设施',
      manager: '张三',
      startDate: '2024-02-01',
      endDate: '2024-12-31'
    },
    {
      id: 2,
      name: '道路改造工程',
      type: 'infrastructure',
      amount: 3200000,
      status: 'reviewing',
      createTime: '2024-01-20',
      creator: '李四',
      description: '城市主干道改造升级工程',
      manager: '李四',
      startDate: '2024-03-01',
      endDate: '2024-10-31'
    },
    {
      id: 3,
      name: '绿化景观项目',
      type: 'landscape',
      amount: 1800000,
      status: 'draft',
      createTime: '2024-01-25',
      creator: '王五',
      description: '公园绿化和景观设计项目',
      manager: '王五',
      startDate: '2024-04-01',
      endDate: '2024-08-31'
    },
    {
      id: 4,
      name: '装修改造工程',
      type: 'renovation',
      amount: 800000,
      status: 'approved',
      createTime: '2024-02-01',
      creator: '赵六',
      description: '办公区域装修改造项目',
      manager: '赵六',
      startDate: '2024-03-15',
      endDate: '2024-06-30'
    }
  ])
}

// 使用共享状态的过滤数据，同时支持本地筛选
const filteredData = computed(() => {
  let data = filteredProjects.value

  // 本地搜索文本筛选
  if (searchText.value) {
    data = data.filter(item =>
      item.name.toLowerCase().includes(searchText.value.toLowerCase()) ||
      item.creator.toLowerCase().includes(searchText.value.toLowerCase())
    )
  }

  // 本地状态筛选
  if (statusFilter.value) {
    data = data.filter(item => item.status === statusFilter.value)
  }

  return data
})

// 状态相关方法
const getStatusColor = (status) => {
  const colors = {
    draft: 'default',
    reviewing: 'processing',
    approved: 'success',
    rejected: 'error'
  }
  return colors[status] || 'default'
}

const getStatusText = (status) => {
  const texts = {
    draft: '草稿',
    reviewing: '审核中',
    approved: '已批准',
    rejected: '已拒绝'
  }
  return texts[status] || '未知'
}

// 操作方法
const newEstimate = async () => {
  try {
    await openFormWindow('create', null)
  } catch (error) {
    console.error('打开新建表单失败:', error)
    message.error('打开表单失败')
  }
}

const importData = () => {
  message.info('导入数据功能开发中...')
}

const exportData = () => {
  message.success('数据导出成功！')
}

// 打开表单窗口
const openFormWindow = async (mode = 'create', data = null) => {
  try {
    const windowId = `estimate-form-${mode}-${Date.now()}`
    const title = mode === 'create' ? '新建概算' : mode === 'edit' ? '编辑概算' : '查看概算'

    // 构建URL参数
    const params = new URLSearchParams({
      mode,
      formType: 'estimate'
    })

    if (data) {
      params.append('data', JSON.stringify(data))
    }

    await invoke('create_child_window', {
      windowId,
      title,
      url: `/form-page?${params.toString()}`,
      modal: mode !== 'view', // 查看模式使用非模态，编辑模式使用模态
      width: 1200,
      height: 800,
      parentWindow: 'main'
    })

    message.success(`${title}窗口已打开`)
  } catch (error) {
    console.error('打开表单窗口失败:', error)
    message.error('打开窗口失败')
  }
}

// 表格事件处理
const handleOpenForm = ({ type, data }) => {
  openFormWindow(type, data)
}

const handleEditRow = (record) => {
  openFormWindow('edit', record)
}

const handleDeleteRow = (record) => {
  // 使用共享状态的删除方法
  deleteProject(record.id)
  message.success(`已删除 ${record.name}`)
}





const handleCostTableCellEdit = (editInfo) => {
  console.log('单元格编辑:', editInfo)
  message.success('数据已更新')
}

const handleSearch = () => {
  console.log('搜索:', searchText.value)
}

const handleStatusFilter = () => {
  console.log('状态筛选:', statusFilter.value)
}

const handleDateFilter = () => {
  console.log('日期筛选:', dateFilter.value)
}

const handleRefresh = () => {
  message.success('数据已刷新')
}

// 窗口控制方法
const onMinimize = () => {
  console.log('概算窗口最小化')
  // SimpleHeader 组件会自动处理窗口控制逻辑
}

const onMaximize = () => {
  console.log('概算窗口最大化/还原')
  // SimpleHeader 组件会自动处理窗口控制逻辑
}

const onClose = () => {
  console.log('概算窗口关闭')
  // SimpleHeader 组件会自动处理窗口控制逻辑
}

const handleTableChange = (pag, filters, sorter) => {
  pagination.value = pag
  console.log('表格变更:', pag, filters, sorter)
}

const viewDetail = (record) => {
  message.info(`查看 ${record.name} 详情功能开发中...`)
}

const editRecord = (record) => {
  message.info(`编辑 ${record.name} 功能开发中...`)
}

const deleteRecord = (record) => {
  message.warning(`删除概算: ${record.name}`)
}

onMounted(() => {
  console.log('概算管理系统已加载')
  pagination.value.total = estimateData.value?.length
})
</script>

<style scoped>
.layout {
  height: 100vh;
}

/* 头部样式已移至共享组件 */
.actions {
  display: flex;
  align-items: center;
}

.content {
  padding: 24px;
  background: #f0f2f5;
}

.content-wrapper {
  max-width: 1200px;
  margin: 0 auto;
}

.toolbar-card {
  margin-bottom: 16px;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.table-card {
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}
</style>
