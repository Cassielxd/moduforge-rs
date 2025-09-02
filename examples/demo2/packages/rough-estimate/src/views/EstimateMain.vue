<template>
  <AppLayout
    class="child-app-layout"
    :title="headerTitle"
    :loading="loading"
    loading-tip="正在处理中..."
    :is-maximized="isMaximized"
    :show-operate="true"
    :show-aside="true"
    :show-footer="true"
    @minimize="onMinimize"
    @maximize="onMaximize"
    @close="onClose"
    @aside-toggle="onAsideToggle"
  >
    <!-- Header 右侧内容 -->
    <template #header-right>
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
          <a-tag v-if="isReady" color="green">{{ windowLabel }}</a-tag>
          <a-tag v-else color="orange">初始化中...</a-tag>
        </a-space>
      </div>
    </template>

    <!-- 操作条 -->
    <template #operate>
      <OperateBar
        :operate-list="operateList"
        :component-type="'estimate'"
        :level-type="3"
        :window-type="'parentPage'"
        :project-type="'estimate'"
        :height="'60px'"
        :show-expand-toggle="true"
        :default-expanded="isOperateExpanded"
        @operate-click="handleOperateClick"
        @select-click="handleSelectClick"
        @expand-change="handleExpandChange"
      />
    </template>

    <!-- 左侧项目树 -->
    <template #aside>
      <AsideTree
        :tree-data="projectTree"
        :selected-keys="selectedProjectKeys"
        :expanded-keys="expandedProjectKeys"
        :draggable="false"
        :show-actions="true"
        @select="handleProjectSelect"
        @expand="handleProjectExpand"
        @refresh="handleProjectRefresh"
        @expand-all="handleProjectExpandAll"
        @collapse-all="handleProjectCollapseAll"
      >
        <template #title="{ title, key, dataRef }">
          <span :class="{ 'project-active': selectedProjectKeys.includes(key) }">
            {{ title }}
          </span>
        </template>
        <template #icon="{ dataRef }">
          <component 
            :is="getProjectIcon(dataRef)" 
            :style="{ color: getProjectColor(dataRef) }" 
          />
        </template>
      </AsideTree>
    </template>

    <!-- 主要内容区域 -->
    <template #content>
      <div class="main-content">
        <!-- 内容头部 -->
        <div class="content-header">
          <a-breadcrumb>
            <a-breadcrumb-item>
              <HomeOutlined />
              概算管理
            </a-breadcrumb-item>
            <a-breadcrumb-item>{{ currentProject?.title || '项目列表' }}</a-breadcrumb-item>
          </a-breadcrumb>
        </div>

        <!-- 内容主体 -->
        <div class="content-body">
          <a-card title="概算项目列表" :bordered="false">
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
      </div>
    </template>

    <!-- Footer 内容 -->
    <template #footer>
      <div class="footer-content">
        <div class="footer-left">
          <a-typography-text type="secondary">
            共 {{ filteredData.length }} 个项目 | 总预算: ¥{{ totalBudget.toLocaleString() }}
          </a-typography-text>
        </div>
        <div class="footer-right">
          <a-typography-text type="secondary">
            ModuForge-RS 概算管理系统 v1.0.0
          </a-typography-text>
        </div>
      </div>
    </template>
  </AppLayout>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { message } from 'ant-design-vue'
import {
  PlusOutlined,
  ImportOutlined,
  ExportOutlined,
  HomeOutlined,
  FolderOutlined,
  ProjectOutlined,
  FileOutlined,
  EditOutlined,
  DeleteOutlined,
  SettingOutlined,
  ReloadOutlined
} from '@ant-design/icons-vue'
import { 
  CostTable, 
  useEstimate, 
  AppLayout, 
  AsideTree, 
  OperateBar,
  useParentWindowDataExchange, 
  useChildAppWindowManager, 
  useEstimateFormWindow 
} from '@cost-app/shared-components'
import { invoke } from '@tauri-apps/api/core'
import { useChildWindowManagement } from '@cost-app/shared-components'
import operateList, { updateOperateByName, menuPolymerizeHandler } from '../data/operateConfig'

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

// 使用新的表单窗口管理器
const formWindowManager = useEstimateFormWindow()

// windowReady、windowLabel、isMaximized 等都已从 useChildWindowManagement 获取

// 数据状态
const loading = ref(false)
const searchText = ref('')
const statusFilter = ref('') // 确保默认为空，显示全部数据
const dateFilter = ref(null)
const refreshLoading = ref(false)
const isOperateExpanded = ref(true) // 操作栏展开状态

// 项目树数据
const projectTree = ref([
  {
    key: 'building',
    title: '建筑工程',
    type: 'category',
    children: [
      { key: 'office', title: '办公楼项目', type: 'project' },
      { key: 'parking', title: '停车场项目', type: 'project' }
    ]
  },
  {
    key: 'infrastructure',
    title: '基础设施',
    type: 'category',
    children: [
      { key: 'road', title: '道路改造', type: 'project' },
      { key: 'pipeline', title: '管线工程', type: 'project' }
    ]
  },
  {
    key: 'decoration',
    title: '装修工程',
    type: 'category',
    children: [
      { key: 'interior', title: '室内装修', type: 'project' },
      { key: 'landscape', title: '园林绿化', type: 'project' }
    ]
  }
])

const selectedProjectKeys = ref(['office'])
const expandedProjectKeys = ref(['building', 'infrastructure'])
const currentProject = ref(null)

// 表格选择
const selectedRowKeys = ref([])
const hasTableSelection = computed(() => selectedRowKeys.value.length > 0)

// 当前选中项目过滤条件
const currentStatusFilter = ref('all')
const currentTypeFilter = ref([])


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

const handleCostTableRowSelect = (selectedKeys, selectedRows) => {
  selectedRowKeys.value = selectedKeys
  console.log('表格选择变化:', selectedKeys, selectedRows)
  if (selectedRows && selectedRows.length > 0) {
    message.info(`选中 ${selectedRows.length} 个项目`)
  }
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

// 计算属性
const totalBudget = computed(() => {
  return filteredData.value.reduce((total, item) => total + (item.amount || 0), 0)
})

const statusText = computed(() => {
  const total = filteredData.value.length
  const reviewing = filteredData.value.filter(item => item.status === 'reviewing').length
  const approved = filteredData.value.filter(item => item.status === 'approved').length
  return `共 ${total} 项，审核中 ${reviewing} 项，已批准 ${approved} 项`
})

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
  if (currentStatusFilter.value && currentStatusFilter.value !== 'all') {
    data = data.filter(item => item.status === currentStatusFilter.value)
  }

  // 类型筛选
  if (currentTypeFilter.value && currentTypeFilter.value.length > 0) {
    data = data.filter(item => currentTypeFilter.value.includes(item.type))
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
  console.log(windowInfo.value)
  try {
    await formWindowManager.createForm({
      windowInfo: windowInfo.value
    })
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



// 表格事件处理
const handleOpenForm = ({ type, data }) => {
  if (type === 'create') {
    formWindowManager.createForm({ windowInfo: windowInfo.value })
  } else if (type === 'edit') {
    formWindowManager.editForm(data, { windowInfo: windowInfo.value })
  } else if (type === 'view') {
    formWindowManager.viewForm(data, { windowInfo: windowInfo.value })
  }
}

const handleEditRow = (record) => {
  formWindowManager.editForm(record, { windowInfo: windowInfo.value })
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

const handleRefresh = async () => {
  try {
    refreshLoading.value = true
    // 模拟数据刷新
    await new Promise(resolve => setTimeout(resolve, 1000))
    message.success('数据已刷新')
  } finally {
    refreshLoading.value = false
  }
}

// 新Layout相关事件处理
const onAsideToggle = (collapsed) => {
  console.log('侧边栏切换:', collapsed)
}

// 操作栏事件处理
const handleOperateClick = (item, parentItem = null) => {
  console.log('操作按钮点击:', item.name, item)
  
  switch (item.name) {
    case 'create':
      newEstimate()
      break
    case 'edit':
      if (!hasTableSelection.value) {
        message.warning('请先选择要编辑的项目')
        return
      }
      handleEditSelected()
      break
    case 'delete':
      if (!hasTableSelection.value) {
        message.warning('请先选择要删除的项目')
        return
      }
      handleDeleteSelected()
      break
    case 'save':
      handleSave()
      break
    case 'export-table':
      exportData()
      break
    case 'batch-approve':
      handleBatchApprove()
      break
    case 'batch-export':
      handleBatchExport()
      break
    case 'batch-delete':
      handleBatchDelete()
      break
    case 'import-data':
      importData()
      break
    case 'refresh':
      handleRefresh()
      break
    case 'view-detail':
      handleViewDetail()
      break
    case 'copy-project':
      handleCopyProject()
      break
    case 'system-settings':
      handleSystemSettings()
      break
    default:
      message.info(`执行操作: ${item.label}`)
  }
}

const handleSelectClick = (selectItem, item, parentItem = null) => {
  console.log('下拉选择:', selectItem, item)
  
  switch (item.name) {
    case 'approval-status':
      currentStatusFilter.value = selectItem.kind
      message.info(`筛选状态: ${selectItem.name}`)
      break
    case 'project-type-filter':
      if (!currentTypeFilter.value.includes(selectItem.kind)) {
        currentTypeFilter.value.push(selectItem.kind)
      } else {
        currentTypeFilter.value = currentTypeFilter.value.filter(type => type !== selectItem.kind)
      }
      message.info(`筛选类型: ${selectItem.name}`)
      break
    case 'project-template':
      handleTemplateAction(selectItem.kind, selectItem.name)
      break
    default:
      message.info(`选择: ${selectItem.name}`)
  }
}

const handleExpandChange = (expanded) => {
  isOperateExpanded.value = expanded
  console.log('操作栏展开状态:', expanded)
}

// 具体操作方法
const handleEditSelected = () => {
  const selectedRows = estimateData.value.filter(item => selectedRowKeys.value.includes(item.id))
  if (selectedRows.length === 1) {
    formWindowManager.editForm(selectedRows[0], { windowInfo: windowInfo.value })
  } else {
    message.warning('请选择一个项目进行编辑')
  }
}

const handleDeleteSelected = () => {
  if (selectedRowKeys.value.length === 0) {
    message.warning('请先选择要删除的项目')
    return
  }
  
  selectedRowKeys.value.forEach(id => {
    deleteProject(id)
  })
  selectedRowKeys.value = []
  message.success(`已删除 ${selectedRowKeys.value.length} 个项目`)
}

const handleSave = () => {
  message.success('数据已保存')
}

const handleBatchApprove = () => {
  if (!hasTableSelection.value) {
    message.warning('请先选择要审批的项目')
    return
  }
  message.info(`批量审批 ${selectedRowKeys.value.length} 个项目`)
}

const handleBatchExport = () => {
  if (!hasTableSelection.value) {
    message.warning('请先选择要导出的项目')
    return
  }
  message.info(`批量导出 ${selectedRowKeys.value.length} 个项目`)
}

const handleBatchDelete = () => {
  if (!hasTableSelection.value) {
    message.warning('请先选择要删除的项目')
    return
  }
  handleDeleteSelected()
}

const handleViewDetail = () => {
  if (!hasTableSelection.value) {
    message.warning('请先选择要查看的项目')
    return
  }
  const selectedRows = estimateData.value.filter(item => selectedRowKeys.value.includes(item.id))
  if (selectedRows.length === 1) {
    formWindowManager.viewForm(selectedRows[0], { windowInfo: windowInfo.value })
  } else {
    message.warning('请选择一个项目进行查看')
  }
}

const handleCopyProject = () => {
  if (!hasTableSelection.value) {
    message.warning('请先选择要复制的项目')
    return
  }
  message.info('复制项目功能开发中...')
}

const handleTemplateAction = (action, name) => {
  switch (action) {
    case 'save-template':
      message.info('保存为模板功能开发中...')
      break
    case 'create-from-template':
      message.info('从模板创建功能开发中...')
      break
    case 'template-manage':
      message.info('模板管理功能开发中...')
      break
  }
}

const handleSystemSettings = () => {
  message.info('系统设置功能开发中...')
}


// 项目树事件处理
const handleProjectSelect = (selectedKeys, info) => {
  selectedProjectKeys.value = selectedKeys
  if (selectedKeys.length > 0) {
    const node = info.node.dataRef || info.node
    currentProject.value = node
    message.info(`选择项目分类: ${node.title}`)
  }
}

const handleProjectExpand = (expandedKeys) => {
  expandedProjectKeys.value = expandedKeys
}

const handleProjectRefresh = () => {
  message.info('刷新项目树')
}

const handleProjectExpandAll = (keys) => {
  expandedProjectKeys.value = keys
  message.info('展开所有项目')
}

const handleProjectCollapseAll = () => {
  expandedProjectKeys.value = []
  message.info('收起所有项目')
}

// 工具函数
const getProjectIcon = (dataRef) => {
  const iconMap = {
    category: FolderOutlined,
    project: ProjectOutlined,
    default: FileOutlined
  }
  return iconMap[dataRef.type] || iconMap.default
}

const getProjectColor = (dataRef) => {
  const colorMap = {
    category: '#1890ff',
    project: '#52c41a',
    default: '#666'
  }
  return colorMap[dataRef.type] || colorMap.default
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
  
  // 初始化操作按钮配置
  initializeOperateConfig()
  
  // 窗口缩放监听，窗口缩放时菜单聚合显示处理
  window.addEventListener('resize', () => {
    menuPolymerizeHandler()
  })
  
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

// 初始化操作按钮配置
const initializeOperateConfig = () => {
  // 设置审批状态选择的初始值
  updateOperateByName('approval-status', item => {
    item.value = currentStatusFilter.value
  })
  
  // 设置项目类型过滤的初始值
  updateOperateByName('project-type-filter', item => {
    item.value = currentTypeFilter.value
  })
  
  // 执行菜单聚合处理
  menuPolymerizeHandler()
}

// 监听筛选条件变化
watch(() => currentStatusFilter.value, (newVal) => {
  updateOperateByName('approval-status', item => {
    item.value = newVal
  })
})

watch(() => currentTypeFilter.value, (newVal) => {
  updateOperateByName('project-type-filter', item => {
    item.value = newVal
  })
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
  console.log('操作按钮配置已加载:', operateList.value.length, '个按钮')
}
</script>

<style scoped>
.actions {
  display: flex;
  align-items: center;
}

.main-content {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.content-header {
  height: 60px;
  flex-shrink: 0;
  padding: 8px 24px;
  background: #fff;
  border-bottom: 1px solid #f0f0f0;
  display: flex;
  align-items: center;
}

.content-body {
  height: calc(100vh - 180px);
  padding: 8px;
  background: #f0f2f5;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.content-body :deep(.ant-card) {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.content-body :deep(.ant-card-head) {
  padding: 8px 16px;
  min-height: auto;
}

.content-body :deep(.ant-card-body) {
  flex: 1;
  padding: 8px;
  overflow: hidden;
  min-height: 0;
}

.footer-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.project-active {
  color: #1890ff;
  font-weight: 600;
}

/* 子应用布局特定样式 */
.child-app-layout :deep(.layout-header) {
  height: 60px;
  flex-shrink: 0;
}

.child-app-layout :deep(.layout-operate) {
  height: 60px;
  flex-shrink: 0;
}

.child-app-layout :deep(.layout-main) {
  height: calc(100vh - 180px);
  flex-shrink: 0;
  overflow: hidden;
}

.child-app-layout :deep(.layout-footer) {
  height: 60px;
  flex-shrink: 0;
}

/* 响应式样式 */
@media (max-width: 768px) {
  .content-header {
    padding: 12px 16px;
  }
  
  .content-body {
    padding: 12px;
  }
}
</style>