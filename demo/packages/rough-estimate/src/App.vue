<template>
  <div id="app">
    <a-layout class="layout">
      <!-- 顶部导航 -->
      <a-layout-header class="header">
        <div class="header-content">
          <div class="logo">
            <h3>概算管理系统</h3>
          </div>
          <div class="header-center">
            <!-- 可拖动区域 -->
          </div>
          <div class="header-right">
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
          </div>
        </div>
      </a-layout-header>

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
            <a-table
              :columns="columns"
              :data-source="filteredData"
              :pagination="pagination"
              :loading="loading"
              @change="handleTableChange"
              row-key="id"
            >
              <template #bodyCell="{ column, record }">
                <template v-if="column.key === 'status'">
                  <a-tag :color="getStatusColor(record.status)">
                    {{ getStatusText(record.status) }}
                  </a-tag>
                </template>
                <template v-else-if="column.key === 'amount'">
                  <span style="color: #1890ff; font-weight: 500;">
                    ¥{{ record.amount.toLocaleString() }}
                  </span>
                </template>
                <template v-else-if="column.key === 'action'">
                  <a-space>
                    <a-button type="link" size="small" @click="viewDetail(record)">
                      查看
                    </a-button>
                    <a-button type="link" size="small" @click="editRecord(record)">
                      编辑
                    </a-button>
                    <a-button type="link" size="small" danger @click="deleteRecord(record)">
                      删除
                    </a-button>
                  </a-space>
                </template>
              </template>
            </a-table>
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

// 数据状态
const loading = ref(false)
const searchText = ref('')
const statusFilter = ref('')
const dateFilter = ref(null)

// 表格配置
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

// 分页配置
const pagination = ref({
  current: 1,
  pageSize: 10,
  total: 0,
  showSizeChanger: true,
  showQuickJumper: true,
  showTotal: (total, range) => `第 ${range[0]}-${range[1]} 条，共 ${total} 条`,
})

// 模拟数据
const estimateData = ref([
  {
    id: 1,
    name: '办公楼建设项目',
    amount: 5000000,
    status: 'approved',
    createTime: '2024-01-15',
    creator: '张三',
  },
  {
    id: 2,
    name: '道路改造工程',
    amount: 3200000,
    status: 'reviewing',
    createTime: '2024-01-20',
    creator: '李四',
  },
  {
    id: 3,
    name: '绿化景观项目',
    amount: 1800000,
    status: 'draft',
    createTime: '2024-01-25',
    creator: '王五',
  },
])

// 过滤后的数据
const filteredData = computed(() => {
  let data = estimateData.value

  if (searchText.value) {
    data = data.filter(item => 
      item.name.toLowerCase().includes(searchText.value.toLowerCase()) ||
      item.creator.toLowerCase().includes(searchText.value.toLowerCase())
    )
  }

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
const newEstimate = () => {
  message.info('新建概算功能开发中...')
}

const importData = () => {
  message.info('导入数据功能开发中...')
}

const exportData = () => {
  message.success('数据导出成功！')
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
  pagination.value.total = estimateData.value.length
})
</script>

<style scoped>
.layout {
  height: 100vh;
}

.header {
  background: linear-gradient(135deg, #1890ff 0%, #096dd9 100%);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.15);
  padding: 0;
}

.header-content {
  display: flex;
  align-items: center;
  height: 100%;
  padding: 0 24px;
}

.logo {
  flex-shrink: 0;
}

.logo h3 {
  color: white;
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.header-center {
  flex: 1;
  height: 100%;
}

.header-right {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 16px;
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
