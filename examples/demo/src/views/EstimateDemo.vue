<template>
  <div class="estimate-demo">
    <div class="demo-header">
      <h2>📊 概算表单窗口演示</h2>
      <p>演示在概算中打开表单窗口，使用共享组件进行数据操作</p>
    </div>

    <div class="demo-content">
      <!-- 表格展示区域 -->
      <a-card title="概算数据列表" class="table-section">
        <template #extra>
          <a-space>
            <a-button type="primary" @click="openCreateForm">
              <template #icon><PlusOutlined /></template>
              新建概算
            </a-button>
            <a-button @click="refreshData">
              <template #icon><ReloadOutlined /></template>
              刷新
            </a-button>
          </a-space>
        </template>

        <CostTable
          :data="estimateData"
          :columns="tableColumns"
          table-type="estimate"
          :editable="false"
          @open-form="handleOpenForm"
          @edit-row="handleEditRow"
          @delete-row="handleDeleteRow"
          @row-select="handleRowSelect"
        />
      </a-card>

      <!-- 操作说明 -->
      <a-card title="功能说明" class="info-section">
        <a-descriptions :column="1" bordered>
          <a-descriptions-item label="新建概算">
            点击"新建概算"按钮，打开模态表单窗口进行数据录入
          </a-descriptions-item>
          <a-descriptions-item label="编辑概算">
            点击表格中的"编辑"按钮，在新窗口中编辑选中的数据
          </a-descriptions-item>
          <a-descriptions-item label="表单编辑">
            点击表格工具栏的"表单编辑"按钮，打开通用表单窗口
          </a-descriptions-item>
          <a-descriptions-item label="数据同步">
            表单窗口中的数据修改会同步到主窗口的表格中
          </a-descriptions-item>
          <a-descriptions-item label="窗口模式">
            支持模态和非模态两种窗口模式，可根据需要选择
          </a-descriptions-item>
        </a-descriptions>
      </a-card>

      <!-- 数据统计 -->
      <a-card title="数据统计" class="stats-section">
        <a-row :gutter="16">
          <a-col :span="6">
            <a-statistic
              title="总项目数"
              :value="estimateData.length"
              :value-style="{ color: '#3f8600' }"
            >
              <template #prefix>
                <ProjectOutlined />
              </template>
            </a-statistic>
          </a-col>
          <a-col :span="6">
            <a-statistic
              title="总金额"
              :value="totalAmount"
              :precision="2"
              suffix="万元"
              :value-style="{ color: '#1890ff' }"
            >
              <template #prefix>
                <DollarOutlined />
              </template>
            </a-statistic>
          </a-col>
          <a-col :span="6">
            <a-statistic
              title="已批准项目"
              :value="approvedCount"
              :value-style="{ color: '#52c41a' }"
            >
              <template #prefix>
                <CheckCircleOutlined />
              </template>
            </a-statistic>
          </a-col>
          <a-col :span="6">
            <a-statistic
              title="平均金额"
              :value="averageAmount"
              :precision="2"
              suffix="万元"
              :value-style="{ color: '#722ed1' }"
            >
              <template #prefix>
                <BarChartOutlined />
              </template>
            </a-statistic>
          </a-col>
        </a-row>
      </a-card>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import {
  PlusOutlined,
  ReloadOutlined,
  ProjectOutlined,
  DollarOutlined,
  CheckCircleOutlined,
  BarChartOutlined
} from '@ant-design/icons-vue'
import { CostTable } from '@cost-app/shared-components'

// 表格列配置
const tableColumns = [
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
    title: '预算金额',
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
    dataIndex: 'manager',
    key: 'manager',
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

// 模拟数据
const estimateData = ref([
  {
    id: 1,
    name: '办公楼建设项目',
    type: 'building',
    amount: 5000000,
    status: 'approved',
    manager: '张三',
    createTime: '2024-01-15',
    description: '新建办公楼项目，包含主体建筑和配套设施'
  },
  {
    id: 2,
    name: '道路改造工程',
    type: 'infrastructure',
    amount: 3200000,
    status: 'reviewing',
    manager: '李四',
    createTime: '2024-01-20',
    description: '城市主干道改造升级工程'
  },
  {
    id: 3,
    name: '绿化景观项目',
    type: 'landscape',
    amount: 1800000,
    status: 'draft',
    manager: '王五',
    createTime: '2024-01-25',
    description: '公园绿化和景观设计项目'
  },
  {
    id: 4,
    name: '装修改造工程',
    type: 'renovation',
    amount: 800000,
    status: 'approved',
    manager: '赵六',
    createTime: '2024-02-01',
    description: '办公区域装修改造项目'
  }
])

// 计算属性
const totalAmount = computed(() => {
  return estimateData.value.reduce((sum, item) => sum + item.amount, 0) / 10000
})

const approvedCount = computed(() => {
  return estimateData.value.filter(item => item.status === 'approved').length
})

const averageAmount = computed(() => {
  return estimateData.value.length > 0 ? totalAmount.value / estimateData.value.length : 0
})

// 方法
const openCreateForm = async () => {
  try {
    await openFormWindow('create', null)
  } catch (error) {
    console.error('打开新建表单失败:', error)
    message.error('打开表单失败')
  }
}

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

const handleOpenForm = ({ type, data }) => {
  openFormWindow(type, data)
}

const handleEditRow = (record) => {
  openFormWindow('edit', record)
}

const handleDeleteRow = (record) => {
  const index = estimateData.value.findIndex(item => item.id === record.id)
  if (index > -1) {
    estimateData.value.splice(index, 1)
    message.success(`已删除 ${record.name}`)
  }
}

const handleRowSelect = (rows) => {
  console.log('选中的行:', rows)
  if (rows.length > 0) {
    message.info(`已选中 ${rows.length} 项`)
  }
}

const refreshData = () => {
  message.success('数据已刷新')
}
</script>

<style scoped>
.estimate-demo {
  padding: 24px;
  background: #f0f2f5;
  min-height: 100vh;
}

.demo-header {
  text-align: center;
  margin-bottom: 32px;
  padding: 24px;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.demo-header h2 {
  margin-bottom: 8px;
  color: #1890ff;
  font-size: 24px;
}

.demo-content {
  max-width: 1400px;
  margin: 0 auto;
}

.table-section {
  margin-bottom: 24px;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.info-section {
  margin-bottom: 24px;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.stats-section {
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

:deep(.ant-card-head) {
  background: #fafafa;
}

:deep(.ant-statistic-title) {
  font-size: 14px;
  color: #666;
}

:deep(.ant-statistic-content) {
  font-size: 20px;
  font-weight: 600;
}
</style>
