<template>
  <div class="data-page">
    <div class="data-header">
      <h2>📊 数据查看器</h2>
      <p>查看项目数据和统计信息</p>
    </div>

    <a-row :gutter="16" style="margin-bottom: 24px;">
      <a-col :span="6">
        <a-card>
          <a-statistic
            title="总项目数"
            :value="statistics.totalProjects"
            :value-style="{ color: '#3f8600' }"
          >
            <template #prefix>
              <ProjectOutlined />
            </template>
          </a-statistic>
        </a-card>
      </a-col>
      <a-col :span="6">
        <a-card>
          <a-statistic
            title="总预算金额"
            :value="statistics.totalBudget"
            :precision="2"
            suffix="万元"
            :value-style="{ color: '#1890ff' }"
          >
            <template #prefix>
              <DollarOutlined />
            </template>
          </a-statistic>
        </a-card>
      </a-col>
      <a-col :span="6">
        <a-card>
          <a-statistic
            title="进行中项目"
            :value="statistics.activeProjects"
            :value-style="{ color: '#cf1322' }"
          >
            <template #prefix>
              <ClockCircleOutlined />
            </template>
          </a-statistic>
        </a-card>
      </a-col>
      <a-col :span="6">
        <a-card>
          <a-statistic
            title="完成率"
            :value="statistics.completionRate"
            precision="1"
            suffix="%"
            :value-style="{ color: '#722ed1' }"
          >
            <template #prefix>
              <CheckCircleOutlined />
            </template>
          </a-statistic>
        </a-card>
      </a-col>
    </a-row>

    <a-tabs v-model:activeKey="activeTab" type="card">
      <a-tab-pane key="projects" tab="项目列表">
        <a-table
          :columns="projectColumns"
          :data-source="projectData"
          :pagination="{ pageSize: 10 }"
          :scroll="{ x: 1200 }"
          bordered
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.key === 'status'">
              <a-tag :color="getStatusColor(record.status)">
                {{ getStatusText(record.status) }}
              </a-tag>
            </template>
            <template v-else-if="column.key === 'budget'">
              ¥{{ record.budget.toLocaleString() }}
            </template>
            <template v-else-if="column.key === 'progress'">
              <a-progress :percent="record.progress" size="small" />
            </template>
            <template v-else-if="column.key === 'action'">
              <a-button type="link" @click="viewProject(record)">查看详情</a-button>
            </template>
          </template>
        </a-table>
      </a-tab-pane>

      <a-tab-pane key="charts" tab="图表分析">
        <a-row :gutter="16">
          <a-col :span="12">
            <a-card title="项目状态分布" style="margin-bottom: 16px;">
              <div ref="pieChart" style="height: 300px;"></div>
            </a-card>
          </a-col>
          <a-col :span="12">
            <a-card title="月度预算趋势" style="margin-bottom: 16px;">
              <div ref="lineChart" style="height: 300px;"></div>
            </a-card>
          </a-col>
        </a-row>
        <a-row :gutter="16">
          <a-col :span="24">
            <a-card title="项目预算对比">
              <div ref="barChart" style="height: 400px;"></div>
            </a-card>
          </a-col>
        </a-row>
      </a-tab-pane>

      <a-tab-pane key="reports" tab="报表导出">
        <a-card>
          <a-form layout="vertical">
            <a-row :gutter="16">
              <a-col :span="8">
                <a-form-item label="报表类型">
                  <a-select v-model:value="reportConfig.type" placeholder="选择报表类型">
                    <a-select-option value="summary">项目汇总报表</a-select-option>
                    <a-select-option value="detail">详细费用报表</a-select-option>
                    <a-select-option value="progress">进度跟踪报表</a-select-option>
                    <a-select-option value="budget">预算分析报表</a-select-option>
                  </a-select>
                </a-form-item>
              </a-col>
              <a-col :span="8">
                <a-form-item label="时间范围">
                  <a-range-picker v-model:value="reportConfig.dateRange" style="width: 100%" />
                </a-form-item>
              </a-col>
              <a-col :span="8">
                <a-form-item label="导出格式">
                  <a-select v-model:value="reportConfig.format" placeholder="选择格式">
                    <a-select-option value="excel">Excel</a-select-option>
                    <a-select-option value="pdf">PDF</a-select-option>
                    <a-select-option value="csv">CSV</a-select-option>
                  </a-select>
                </a-form-item>
              </a-col>
            </a-row>
            
            <a-form-item>
              <a-button type="primary" @click="exportReport" :loading="exporting">
                <template #icon><DownloadOutlined /></template>
                导出报表
              </a-button>
            </a-form-item>
          </a-form>
        </a-card>
      </a-tab-pane>
    </a-tabs>

    <div class="data-actions">
      <a-button @click="refreshData" :loading="loading">
        <template #icon><ReloadOutlined /></template>
        刷新数据
      </a-button>
      <a-button @click="closeWindow">关闭窗口</a-button>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { useChildWindowManagement } from '@cost-app/shared-components'
import {
  ProjectOutlined,
  DollarOutlined,
  ClockCircleOutlined,
  CheckCircleOutlined,
  DownloadOutlined,
  ReloadOutlined
} from '@ant-design/icons-vue'

const activeTab = ref('projects')
const loading = ref(false)
const exporting = ref(false)

const statistics = reactive({
  totalProjects: 156,
  totalBudget: 2580.5,
  activeProjects: 23,
  completionRate: 78.5
})

const reportConfig = reactive({
  type: '',
  dateRange: null,
  format: 'excel'
})

const projectColumns = [
  { title: '项目名称', dataIndex: 'name', key: 'name', width: 200, fixed: 'left' },
  { title: '项目类型', dataIndex: 'type', key: 'type', width: 120 },
  { title: '预算金额', dataIndex: 'budget', key: 'budget', width: 150 },
  { title: '状态', dataIndex: 'status', key: 'status', width: 100 },
  { title: '进度', dataIndex: 'progress', key: 'progress', width: 120 },
  { title: '开始日期', dataIndex: 'startDate', key: 'startDate', width: 120 },
  { title: '负责人', dataIndex: 'manager', key: 'manager', width: 100 },
  { title: '操作', key: 'action', width: 100, fixed: 'right' }
]

const projectData = ref([
  {
    key: '1',
    name: '办公楼建设项目',
    type: '建筑工程',
    budget: 1500000,
    status: 'active',
    progress: 65,
    startDate: '2024-01-15',
    manager: '张三'
  },
  {
    key: '2',
    name: '道路改造工程',
    type: '基础设施',
    budget: 800000,
    status: 'completed',
    progress: 100,
    startDate: '2024-02-01',
    manager: '李四'
  },
  {
    key: '3',
    name: '装修改造项目',
    type: '装修工程',
    budget: 350000,
    status: 'planning',
    progress: 15,
    startDate: '2024-03-10',
    manager: '王五'
  }
])

const getStatusColor = (status) => {
  const colors = {
    planning: 'blue',
    active: 'green',
    completed: 'purple',
    paused: 'orange',
    cancelled: 'red'
  }
  return colors[status] || 'default'
}

const getStatusText = (status) => {
  const texts = {
    planning: '规划中',
    active: '进行中',
    completed: '已完成',
    paused: '暂停',
    cancelled: '已取消'
  }
  return texts[status] || '未知'
}

const viewProject = (project) => {
  message.info(`查看项目：${project.name}`)
}

const refreshData = async () => {
  try {
    loading.value = true
    // 模拟数据刷新
    await new Promise(resolve => setTimeout(resolve, 1000))
    message.success('数据刷新成功')
  } catch (error) {
    message.error('数据刷新失败')
  } finally {
    loading.value = false
  }
}

const exportReport = async () => {
  try {
    exporting.value = true
    
    if (!reportConfig.type) {
      message.error('请选择报表类型')
      return
    }
    
    // 模拟报表导出
    await new Promise(resolve => setTimeout(resolve, 2000))
    message.success('报表导出成功')
    
  } catch (error) {
    message.error('报表导出失败')
  } finally {
    exporting.value = false
  }
}

const { closeCurrentWindow } = useChildWindowManagement()

const closeWindow = async () => {
  try {
    await closeCurrentWindow()
  } catch (error) {
    console.error('关闭窗口失败:', error)
    message.error('关闭窗口失败')
  }
}

onMounted(() => {
  console.log('数据查看器已加载')
  // 这里可以初始化图表
})
</script>

<style scoped>
.data-page {
  padding: 24px;
  background: #fff;
  min-height: 100vh;
}

.data-header {
  text-align: center;
  margin-bottom: 32px;
  padding-bottom: 16px;
  border-bottom: 1px solid #f0f0f0;
}

.data-header h2 {
  margin-bottom: 8px;
  color: #1890ff;
}

.data-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 32px;
  padding-top: 16px;
  border-top: 1px solid #f0f0f0;
}
</style>
