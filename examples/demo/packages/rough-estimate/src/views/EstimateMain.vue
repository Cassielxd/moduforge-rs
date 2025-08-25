<template>
  <div class="estimate-main">
    <a-layout class="layout">
      <!-- 使用共享头部组件 -->
      <SimpleHeader
        :title="headerTitle"
        :show-window-controls="true"
        :is-maximized="isMaximized"
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
              <a-button type="dashed" @click="showWindowManager">
                <template #icon><WindowsOutlined /></template>
                窗体管理演示
              </a-button>
              <a-tag v-if="isReady" color="green">{{ windowLabel }}</a-tag>
              <a-tag v-else color="orange">初始化中...</a-tag>
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
                  allow-clear
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
  ExportOutlined,
  WindowsOutlined
} from '@ant-design/icons-vue'
import { CostTable, useEstimate, SimpleHeader, useParentWindowDataExchange, useChildAppWindowManager } from '@cost-app/shared-components'
import { invoke } from '@tauri-apps/api/core'
import { useChildWindowManagement } from '@cost-app/shared-components'

// 标题
const headerTitle = ref('概算管理系统')

// 使用新的窗口管理封装
const {
  currentWindow,
  isMaximized,
  isReady,
  windowLabel,
  minimizeWindow,
  toggleMaximize,
  closeWindow
} = useChildWindowManagement()

// 兼容现有代码的 windowInfo
const windowInfo = computed(() => ({
  label: windowLabel.value,
  isMaximized: isMaximized.value,
  isReady: isReady.value
}))

// 使用数据交换系统
const dataExchange = useParentWindowDataExchange()
const {
  onFormSubmit,
  onFormDataUpdate,
  onDataRefreshRequest
} = dataExchange

// 使用子应用窗体管理器
const windowManager = useChildAppWindowManager()

// windowReady、windowLabel、isMaximized 等都已从 useChildWindowManagement 获取

// 数据状态
const loading = ref(false)
const searchText = ref('')
const statusFilter = ref('') // 确保默认为空，显示全部数据
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

// 使用本地数据而不是共享状态（暂时修复）
const estimateData = ref([
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
  },
  {
    id: 5,
    name: '水电安装工程',
    type: 'installation',
    amount: 1200000,
    status: 'reviewing',
    creator: '孙七',
    createTime: '2024-02-05',
    description: '综合楼水电管线安装工程'
  },
  {
    id: 6,
    name: '停车场建设',
    type: 'infrastructure',
    amount: 2500000,
    status: 'draft',
    creator: '周八',
    createTime: '2024-02-10',
    description: '地下停车场建设项目'
  },
  {
    id: 7,
    name: '消防系统改造',
    type: 'safety',
    amount: 900000,
    status: 'approved',
    creator: '吴九',
    createTime: '2024-02-15',
    description: '建筑消防系统升级改造'
  },
  {
    id: 8,
    name: '智能化系统集成',
    type: 'technology',
    amount: 1500000,
    status: 'reviewing',
    creator: '郑十',
    createTime: '2024-02-20',
    description: '楼宇智能化管理系统集成项目'
  }
])

// 本地操作方法
const addProject = (project) => {
  estimateData.value.push(project)
}

const updateProject = (id, updates) => {
  const index = estimateData.value.findIndex(p => p.id === id)
  if (index > -1) {
    Object.assign(estimateData.value[index], updates)
  }
}

const deleteProject = (id) => {
  const index = estimateData.value.findIndex(p => p.id === id)
  if (index > -1) {
    estimateData.value.splice(index, 1)
  }
}

console.log('本地数据初始化完成，数据长度:', estimateData.value.length)


// 使用共享状态的过滤数据，确保状态同步
const filteredData = computed(() => {
  let data = estimateData.value || []

  // 搜索文本筛选
  if (searchText.value) {
    data = data.filter(item =>
      item.name.toLowerCase().includes(searchText.value.toLowerCase()) ||
      item.creator.toLowerCase().includes(searchText.value.toLowerCase())
    )
  }

  // 状态筛选
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

// 窗体管理演示 - 打开工作台的窗体管理演示页面
const showWindowManager = async () => {
  try {
    // 打开工作台的窗体管理演示页面，建立父子窗口关系
    await windowManager.openWindow('window-manager-demo', {
      width: 1000,
      height: 700
    })
    message.success('窗体管理演示窗口已打开')
  } catch (error) {
    console.error('打开窗体管理演示失败:', error)
    message.error('打开窗体管理演示失败')
  }
}

// 打开表单窗口
const openFormWindow = async (mode = 'create', data = null) => {
  try {
    // 对于相同模式，使用固定的窗口ID，这样可以重用窗口
    const baseWindowId = data?.id ? `estimate-form-${mode}-${data.id}` : `estimate-form-${mode}`
    const windowId = baseWindowId
    const title = mode === 'create' ? '新建概算' : mode === 'edit' ? '编辑概算' : '查看概算'
    const isModal = mode !== 'view' // 编辑和创建使用模态，查看使用非模态

    console.log('打开表单窗口:', {
      windowId,
      title,
      mode,
      modal: isModal,
      dataId: data?.id
    })

    // 构建URL参数
    const parentWindowLabel = windowInfo.value.label || 'module-rough-estimate'
    const currentPort = window.location.port || '5176' // 默认端口
    const params = new URLSearchParams({
      mode,
      formType: 'estimate',
      modal: isModal.toString(),
      parentWindow: parentWindowLabel,
      appId: 'rough-estimate',  // 应用标识
      appPort: currentPort  // 应用端口（动态获取）
    })

    console.log('传递给表单的参数:', {
      mode,
      formType: 'estimate',
      modal: isModal,
      parentWindow: parentWindowLabel
    })

    if (data) {
      params.append('data', JSON.stringify(data))
    }

    // 先尝试显示已存在的窗口
    try {
      await invoke('show_existing_window', { windowId })
      console.log('已显示现有窗口:', windowId)
      message.success(`${title}窗口已显示`)
      return
    } catch (error) {
      // 窗口不存在，继续创建新窗口
      console.log('窗口不存在，创建新窗口:', windowId)
    }

    // 构建完整的URL，使用当前应用的端口访问form-page路由
    const currentOrigin = window.location.origin
    const fullUrl = `${currentOrigin}/form-page?${params.toString()}`
    
    console.log('子窗口完整URL:', fullUrl)

    // 使用invoke直接调用后端创建窗口
    await invoke('create_child_window', {
      windowId,
      title,
      url: fullUrl,  // 使用完整URL而不是相对路径
      modal: !isModal,
      width: isModal ? 700 : 800,  // 比概算窗口更小
      height: isModal ? 500 : 600,  // 比概算窗口更小
      parentWindow: windowInfo.value.label  // 确保使用概算窗口的label作为父窗口
    })
    
    console.log('创建子窗口，父窗口:', windowInfo.value.label, '模态:', isModal)

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
  // 本地筛选，不需要同步到共享状态
}

const handleDateFilter = () => {
  console.log('日期筛选:', dateFilter.value)
}

const handleRefresh = () => {
  message.success('数据已刷新')
}

// 窗口禁用/启用相关功能
const addDisabledOverlay = () => {
  // 移除现有遮罩（如果存在）
  removeDisabledOverlay()
  
  // 创建禁用遮罩
  const overlay = document.createElement('div')
  overlay.id = 'window-disabled-overlay'
  overlay.style.cssText = `
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.3);
    z-index: 9999;
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-size: 18px;
    font-weight: 600;
    pointer-events: all;
    user-select: none;
  `
  overlay.innerHTML = '<div>模态窗口打开中，请先关闭模态窗口</div>'
  document.body.appendChild(overlay)
}

const removeDisabledOverlay = () => {
  const overlay = document.getElementById('window-disabled-overlay')
  if (overlay) {
    overlay.remove()
  }
}

// 窗口控制方法
// 使用新封装的窗口操作函数，添加用户反馈
const onMinimize = async () => {
  try {
    await minimizeWindow()
    message.success('窗口已最小化')
  } catch (error) {
    message.error('最小化失败')
  }
}

const onMaximize = async () => {
  try {
    await toggleMaximize()
  } catch (error) {
    message.error('窗口操作失败')
  }
}

const onClose = async () => {
  try {
    await closeWindow()
  } catch (error) {
    message.error('关闭失败')
  }
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

onMounted(async () => {
  console.log('概算管理系统已加载')
  console.log('头部标题:', headerTitle.value)
  pagination.value.total = estimateData.value?.length
  
  // 窗口管理通过 useChildWindowManagement 自动初始化
  
  // 保留模态窗口监听（这部分功能暂时保持独立）
  try {
    if (currentWindow.value) {
      // 监听窗口禁用/启用事件（用于模态窗口）
      const unlisten2 = await currentWindow.value.listen('window-disabled', () => {
        console.log('概算窗口被禁用')
        // 添加禁用遮罩
        addDisabledOverlay()
      })
      
      const unlisten3 = await currentWindow.value.listen('window-enabled', () => {
        console.log('概算窗口被启用')  
        // 移除禁用遮罩
        removeDisabledOverlay()
      })
    }
  } catch (error) {
    console.error('设置模态窗口监听失败:', error)
  }

  // 设置数据交换事件处理
  setupDataExchangeHandlers()
})

// 设置数据交换处理器
const setupDataExchangeHandlers = () => {
  // 处理表单提交事件
  onFormSubmit((formSubmitData, fromWindow) => {
    console.log('收到表单提交数据:', formSubmitData, '来源:', fromWindow)
    
    const { action, formType, data } = formSubmitData
    
    if (formType === 'estimate') {
      if (action === 'create') {
        // 添加新项目
        const newProject = {
          id: Date.now(),
          ...data,
          status: 'draft',
          createTime: new Date().toISOString().split('T')[0]
        }
        addProject(newProject)
        message.success(`新建概算项目 "${data.name}" 成功`)
      } else if (action === 'edit') {
        // 更新现有项目
        updateProject(data.id, data)
        message.success(`更新概算项目 "${data.name}" 成功`)
      }
      
      // 刷新表格数据
      pagination.value.total = estimateData.value?.length
    }
  })
  
  // 处理表单数据更新事件（实时预览）
  onFormDataUpdate((updateData, fromWindow) => {
    console.log('收到表单数据更新:', updateData, '来源:', fromWindow)
    // 可以用于实时预览或验证
  })
  
  // 处理数据刷新请求
  onDataRefreshRequest((requestData, fromWindow) => {
    console.log('收到数据刷新请求:', requestData, '来源:', fromWindow)
    // 可以重新加载数据或刷新界面
    handleRefresh()
  })
  
  console.log('数据交换处理器已设置')
}
</script>

<style scoped>
.estimate-main {
  height: 100vh;
}

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