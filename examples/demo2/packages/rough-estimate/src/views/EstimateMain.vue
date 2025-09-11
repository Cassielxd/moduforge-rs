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
         
            <CostTable
              :data="treeEstimateData"
              :columns="costTableColumns"
              :editable="true"
              :show-toolbar="true"
              :show-summary="true"
              :range-selection="true"
              :tree-props="treeProps"
              :scroll-config="{ x: 1200, y: 400 }"
              table-type="estimate"
              row-key="id"
              :default-expand-all="true"
              :custom-form-component="EstimateFormComponent"
              :form-props="{ tableType: 'estimate' }"
              @data-change="handleCostTableDataChange"
              @cell-edited="handleCellEdited"
              @row-select="handleCostTableRowSelect"
              @add-row="handleAddRow"
              @delete-row="handleDeleteRow"
              @copy-rows="handleCopyRows"
              @paste-rows="handlePasteRows"
              @open-form="handleOpenForm"
            >

              <!-- 自定义右键菜单 -->
              <template #context-menu="{ record, column }">
                <a-menu @click="(e) => handleContextMenuClick(e, record)">
                  <a-sub-menu key="add" title="插入">
                    <a-menu-item key="add-parent">添加父级项目</a-menu-item>
                    <a-menu-item key="add-child">添加子项目</a-menu-item>
                    <a-menu-item key="add-sibling">添加同级项目</a-menu-item>
                  </a-sub-menu>
                  <a-menu-divider />
                  <a-menu-item key="copy">复制</a-menu-item>
                  <a-menu-item key="paste">粘贴</a-menu-item>
                  <a-menu-divider />
                  <a-menu-item key="edit">编辑</a-menu-item>
                  <a-menu-item key="delete" danger>删除</a-menu-item>
                  <a-menu-divider />
                  <a-menu-item key="calculate">重新计算</a-menu-item>
                </a-menu>
              </template>
            </CostTable>
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
  ReloadOutlined,
  CalculatorOutlined
} from '@ant-design/icons-vue'
import {
  CostTable,
  TableContextMenu,
  AppLayout,
  AsideTree,
  OperateBar,



  useGlobalOperateWindow
} from '@cost-app/shared-components'
// import { invoke } from '@tauri-apps/api/core'
import operateList, { updateOperateByName, menuPolymerizeHandler } from '../data/operateConfig'

// 标题
const headerTitle = ref('概算管理系统')

// 自定义表单组件
const EstimateFormComponent = {
  props: ['value', 'record', 'tableType'],
  emits: ['update:value', 'submit', 'cancel'],
  template: `
    <div class="estimate-form">
      <a-form layout="vertical">
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="项目编码">
              <a-input v-model:value="formData.code" placeholder="请输入项目编码" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="项目名称">
              <a-input v-model:value="formData.name" placeholder="请输入项目名称" />
            </a-form-item>
          </a-col>
        </a-row>
        <a-row :gutter="16">
          <a-col :span="8">
            <a-form-item label="数量">
              <a-input-number v-model:value="formData.quantity" :min="0" style="width: 100%" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="单价">
              <a-input-number v-model:value="formData.unitPrice" :min="0" :precision="2" style="width: 100%" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="金额">
              <a-input-number v-model:value="computedAmount" :precision="2" disabled style="width: 100%" />
            </a-form-item>
          </a-col>
        </a-row>
        <a-form-item label="备注">
          <a-textarea v-model:value="formData.remark" :rows="3" placeholder="请输入备注信息" />
        </a-form-item>
      </a-form>
    </div>
  `,
  setup(props, { emit }) {
    const formData = ref({ ...props.record })

    const computedAmount = computed(() => {
      return (formData.value.quantity || 0) * (formData.value.unitPrice || 0)
    })

    watch(formData, (newValue) => {
      emit('update:value', newValue)
    }, { deep: true })

    return {
      formData,
      computedAmount
    }
  }
}

// 简化的窗口信息
const windowInfo = computed(() => ({
  label: 'rough-estimate-main',
  isMaximized: false,
  isReady: true
}))



// 使用通用Tauri窗口管理器
const operateWindowManager = useGlobalOperateWindow({
  appId: 'rough-estimate',
  defaultPort: '5174',
  routePath: 'operate-page'
})

// 窗口管理已简化

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

// 共享表格组件（CostTable）相关配置 - 支持编辑和树形结构
const costTableColumns = [
  {
    title: '项目编码',
    field: 'code',
    dataIndex: 'code',
    width: 260,
    align: 'left',
    fixed: 'left',
    editable: true,
    required: true
  },
  {
    title: '项目名称',
    field: 'name',
    dataIndex: 'name',
    width: 200,
    align: 'left',
    editable: true,
    required: true,
    sorter: true
  },
  {
    title: '项目类型',
    field: 'type',
    dataIndex: 'type',
    width: 120,
    align: 'center',
    editable: true
  },
  {
    title: '单位',
    field: 'unit',
    dataIndex: 'unit',
    width: 80,
    align: 'center',
    editable: true
  },
  {
    title: '数量',
    field: 'quantity',
    dataIndex: 'quantity',
    width: 100,
    align: 'right',
    editable: true,
    dataType: 'number',
    formatter: (value) => Number(value || 0).toFixed(2)
  },
  {
    title: '单价',
    field: 'unitPrice',
    dataIndex: 'unitPrice',
    width: 120,
    align: 'right',
    editable: true,
    dataType: 'number',
    formatter: (value) => Number(value || 0).toFixed(2)
  },
  {
    title: '概算金额',
    field: 'amount',
    dataIndex: 'amount',
    width: 150,
    align: 'right',
    sorter: true,
    formatter: (value) => Number(value || 0).toFixed(2)
  },
  {
    title: '状态',
    field: 'status',
    dataIndex: 'status',
    width: 100,
    align: 'center'
  },
  {
    title: '负责人',
    field: 'creator',
    dataIndex: 'creator',
    width: 100,
    align: 'center',
    editable: true
  },
  {
    title: '创建时间',
    field: 'createTime',
    dataIndex: 'createTime',
    width: 150,
    align: 'center',
    sorter: true
  }
]

// 树形表格配置
const treeProps = {
  children: 'children',
  hasChildren: 'hasChildren'
}

// 右键菜单配置
const contextMenuOptions = ref([
  {
    code: "add",
    name: "插入1",
    visible: true,
    disabled: false,
    children: [
      {
        code: "add-parent",
        name: "添加父级项目",
        kind: "01",
        visible: true,
        disabled: false,
      },
      {
        code: "add-child",
        name: "添加子项目",
        kind: "02",
        visible: true,
        disabled: false,
      },
      {
        code: "add-sibling",
        name: "添加同级项目",
        kind: "03",
        visible: true,
        disabled: false,
      }
    ],
  },
  {
    code: "cut",
    name: "剪切",
    visible: true,
    disabled: false,
  },
  {
    code: "copy",
    name: "复制",
    visible: true,
    disabled: false,
  },
  {
    code: "paste",
    name: "粘贴",
    visible: true,
    disabled: false,
  },
  {
    code: "delete",
    name: "删除",
    visible: true,
    disabled: false,
  },
  {
    code: "edit",
    name: "编辑",
    visible: true,
    disabled: false,
  },
  {
    code: "calculate",
    name: "重新计算",
    visible: true,
    disabled: false,
  }
])

// 转换为树形数据结构
const treeEstimateData = computed(() => {
  // 递归添加层级信息
  const addLevelInfo = (items, level = 0, parent = null) => {
    return items.map(item => {
      const processedItem = {
        ...item,
        code: item.code || `EST${String(item.id).padStart(3, '0')}`,
        unit: item.unit || '项',
        quantity: item.quantity || 1,
        unitPrice: item.unitPrice || item.amount || 0,
        level: level,
        parent: parent,
        hasChildren: !!(item.children && item.children.length > 0)
      }

      if (item.children && item.children.length > 0) {
        processedItem.children = addLevelInfo(item.children, level + 1, processedItem)
      }

      return processedItem
    })
  }

  return addLevelInfo(estimateData.value)
})

const handleCostTableDataChange = (newData) => {
  estimateData.value = newData
  pagination.value.total = newData.length
}

const handleCellEdited = ({ record, column, newValue, oldValue }) => {
  console.log('单元格编辑:', { record, column, newValue, oldValue })

  // 自动计算金额
  if (column.field === 'quantity' || column.field === 'unitPrice') {
    record.amount = (record.quantity || 0) * (record.unitPrice || 0)
  }

  message.success('编辑成功')
}

const handleAddRow = (newRow) => {
  console.log('新增行:', newRow)
  message.success('新增成功')
}

const handleDeleteRow = (deletedRows) => {
  // 处理单个记录或多个记录
  const rowsToDelete = Array.isArray(deletedRows) ? deletedRows : [deletedRows]

  rowsToDelete.forEach(record => {
    deleteProject(record)
  })

  const names = rowsToDelete.map(r => r.name).join(', ')
  console.log('删除行:', names)
  message.success(`删除了 ${rowsToDelete.length} 项: ${names}`)
}

const handleCopyRows = (copiedRows) => {
  const names = copiedRows.map(r => r.name).join(', ')
  console.log('复制行:', names)
  message.info(`已复制 ${copiedRows.length} 项`)
}

const handlePasteRows = (pastedRows) => {
  const names = pastedRows.map(r => r.name).join(', ')
  console.log('粘贴行:', names)
  message.success(`已粘贴 ${pastedRows.length} 项`)
}

const calculateTotal = () => {
  // 递归计算所有项目的金额
  const calculateNode = (node) => {
    if (node.children && node.children.length > 0) {
      node.amount = node.children.reduce((sum, child) => {
        return sum + calculateNode(child)
      }, 0)
    } else {
      node.amount = (node.quantity || 0) * (node.unitPrice || 0)
    }
    return node.amount
  }

  estimateData.value.forEach(calculateNode)
  message.success('重新计算完成')
}

const importTemplate = () => {
  message.info('导入模板功能开发中...')
}

const addChildProject = (parentRecord) => {
  const newChild = {
    id: `${parentRecord.id}-${Date.now()}`,
    code: `${parentRecord.code}001`,
    name: '新建子项目',
    type: 'subproject',
    unit: '项',
    quantity: 1,
    unitPrice: 0,
    amount: 0,
    status: 'draft',
    creator: '当前用户',
    createTime: new Date().toISOString().split('T')[0],
    children: []
  }

  if (!parentRecord.children) {
    parentRecord.children = []
  }
  parentRecord.children.push(newChild)

  message.success('添加子项目成功')
}

const editProject = (record) => {
  console.log('编辑项目:', record)
  message.info('编辑项目功能开发中...')
}

const deleteProject = (record) => {
  // 如果传入的是记录对象，使用其id；如果传入的是id，直接使用
  const id = typeof record === 'object' ? record.id : record
  const index = estimateData.value.findIndex(p => p.id === id)
  if (index > -1) {
    const deletedItem = estimateData.value[index]
    estimateData.value.splice(index, 1)
    message.success(`已删除项目: ${deletedItem.name}`)
  }
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

// 使用本地数据 - 树形结构概算数据（多层级）
const estimateData = ref([
  {
    id: 1,
    code: 'EST001',
    name: '办公楼建设项目',
    type: 'building',
    unit: '项',
    quantity: 1,
    unitPrice: 50000000,
    amount: 50000000,
    status: 'approved',
    createTime: '2024-01-15',
    creator: '张三',
    description: '新建办公楼项目，包含主体建筑和配套设施',
    children: [
      {
        id: 11,
        code: 'EST001001',
        name: '土建工程',
        type: 'construction',
        unit: 'm²',
        quantity: 10000,
        unitPrice: 300,
        amount: 3000000,
        status: 'approved',
        creator: '张三',
        createTime: '2024-01-15',
        children: [
          {
            id: 111,
            code: 'EST001001001',
            name: '基础工程',
            type: 'foundation',
            unit: 'm³',
            quantity: 500,
            unitPrice: 800,
            amount: 400000,
            status: 'approved',
            creator: '张三',
            createTime: '2024-01-15',
            children: [
              {
                id: 1111,
                code: 'EST001001001001',
                name: '挖土方',
                type: 'earthwork',
                unit: 'm³',
                quantity: 200,
                unitPrice: 50,
                amount: 10000,
                status: 'approved',
                creator: '张三',
                createTime: '2024-01-15',
                children: []
              },
              {
                id: 1112,
                code: 'EST001001001002',
                name: '混凝土浇筑',
                type: 'concrete',
                unit: 'm³',
                quantity: 300,
                unitPrice: 1300,
                amount: 390000,
                status: 'approved',
                creator: '张三',
                createTime: '2024-01-15',
                children: []
              }
            ]
          },
          {
            id: 112,
            code: 'EST001001002',
            name: '主体结构',
            type: 'structure',
            unit: 'm²',
            quantity: 8000,
            unitPrice: 325,
            amount: 2600000,
            status: 'approved',
            creator: '张三',
            createTime: '2024-01-15',
            children: [
              {
                id: 1121,
                code: 'EST001001002001',
                name: '钢筋工程',
                type: 'steel',
                unit: 't',
                quantity: 100,
                unitPrice: 8000,
                amount: 800000,
                status: 'approved',
                creator: '张三',
                createTime: '2024-01-15',
                children: []
              },
              {
                id: 1122,
                code: 'EST001001002002',
                name: '模板工程',
                type: 'formwork',
                unit: 'm²',
                quantity: 5000,
                unitPrice: 120,
                amount: 600000,
                status: 'approved',
                creator: '张三',
                createTime: '2024-01-15',
                children: []
              },
              {
                id: 1123,
                code: 'EST001001002003',
                name: '混凝土工程',
                type: 'concrete',
                unit: 'm³',
                quantity: 800,
                unitPrice: 1500,
                amount: 1200000,
                status: 'approved',
                creator: '张三',
                createTime: '2024-01-15',
                children: []
              }
            ]
          }
        ]
      },
      {
        id: 12,
        code: 'EST001002',
        name: '装修工程',
        type: 'decoration',
        unit: 'm²',
        quantity: 8000,
        unitPrice: 250,
        amount: 2000000,
        status: 'approved',
        creator: '张三',
        createTime: '2024-01-15',
        children: [
          {
            id: 121,
            code: 'EST001002001',
            name: '地面装修',
            type: 'floor',
            unit: 'm²',
            quantity: 6000,
            unitPrice: 150,
            amount: 900000,
            status: 'approved',
            creator: '张三',
            createTime: '2024-01-15',
            children: []
          },
          {
            id: 122,
            code: 'EST001002002',
            name: '墙面装修',
            type: 'wall',
            unit: 'm²',
            quantity: 12000,
            unitPrice: 80,
            amount: 960000,
            status: 'approved',
            creator: '张三',
            createTime: '2024-01-15',
            children: []
          },
          {
            id: 123,
            code: 'EST001002003',
            name: '天花装修',
            type: 'ceiling',
            unit: 'm²',
            quantity: 6000,
            unitPrice: 120,
            amount: 720000,
            status: 'approved',
            creator: '张三',
            createTime: '2024-01-15',
            children: []
          }
        ]
      }
    ]
  },
  {
    id: 2,
    code: 'EST002',
    name: '道路改造工程',
    type: 'infrastructure',
    unit: 'km',
    quantity: 5,
    unitPrice: 6400000,
    amount: 32000000,
    status: 'reviewing',
    createTime: '2024-01-20',
    creator: '李四',
    description: '城市主干道改造升级工程',
    children: [
      {
        id: 21,
        code: 'EST002001',
        name: '路面工程',
        type: 'road',
        unit: 'm²',
        quantity: 25000,
        unitPrice: 800,
        amount: 20000000,
        status: 'reviewing',
        creator: '李四',
        createTime: '2024-01-20',
        children: [
          {
            id: 211,
            code: 'EST002001001',
            name: '路基处理',
            type: 'roadbed',
            unit: 'm²',
            quantity: 25000,
            unitPrice: 200,
            amount: 5000000,
            status: 'reviewing',
            creator: '李四',
            createTime: '2024-01-20',
            children: []
          },
          {
            id: 212,
            code: 'EST002001002',
            name: '沥青铺设',
            type: 'asphalt',
            unit: 'm²',
            quantity: 25000,
            unitPrice: 600,
            amount: 15000000,
            status: 'reviewing',
            creator: '李四',
            createTime: '2024-01-20',
            children: []
          }
        ]
      },
      {
        id: 22,
        code: 'EST002002',
        name: '排水工程',
        type: 'drainage',
        unit: 'm',
        quantity: 5000,
        unitPrice: 2400,
        amount: 12000000,
        status: 'reviewing',
        creator: '李四',
        createTime: '2024-01-20',
        children: []
      }
    ]
  },
  {
    id: 3,
    code: 'EST003',
    name: '绿化景观项目',
    type: 'landscape',
    unit: 'm²',
    quantity: 15000,
    unitPrice: 120,
    amount: 1800000,
    status: 'draft',
    createTime: '2024-01-25',
    creator: '王五',
    description: '公园绿化和景观设计项目',
    children: []
  },
  {
    id: 4,
    code: 'EST004',
    name: '装修改造工程',
    type: 'renovation',
    unit: 'm²',
    quantity: 4000,
    unitPrice: 200,
    amount: 800000,
    status: 'approved',
    createTime: '2024-02-01',
    creator: '赵六',
    description: '办公区域装修改造项目',
    children: []
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

// 右键菜单处理函数
const handleContextMenuClick = ({ key }, record) => {
  console.log('右键菜单点击:', key, record)

  switch (key) {
    case 'add-parent':
      addParentProject(record)
      break
    case 'add-child':
      addChildProject(record)
      break
    case 'add-sibling':
      addSiblingProject(record)
      break
    case 'cut':
      cutProject(record)
      break
    case 'copy':
      copyProject(record)
      break
    case 'paste':
      pasteProject(record)
      break
    case 'delete':
      deleteProject(record)
      break
    case 'edit':
      editProject(record)
      break
    case 'calculate':
      calculateTotal()
      break
    default:
      message.info(`执行操作: ${key}`)
  }
}

// 右键菜单相关操作函数
const addParentProject = (record) => {
  message.info(`为 ${record.name} 添加父级项目功能开发中...`)
}

const addSiblingProject = (record) => {
  message.info(`为 ${record.name} 添加同级项目功能开发中...`)
}

const cutProject = (record) => {
  message.info(`剪切项目 ${record.name} 功能开发中...`)
}

const copyProject = (record) => {
  message.info(`复制项目 ${record.name} 功能开发中...`)
}

const pasteProject = (record) => {
  message.info(`在 ${record.name} 处粘贴项目功能开发中...`)
}

// 新Layout相关事件处理
const onAsideToggle = (collapsed) => {
  console.log('侧边栏切换:', collapsed)
}

// 操作栏事件处理
const handleOperateClick = (item, parentItem = null) => {
  console.log('操作按钮点击:', item.name, item)

  // 定义需要使用Tauri窗口的操作
  const tauriWindowOperations = [
    'create', 'edit', 'view',  // 表单操作
    'import-data',      // 数据导入
    'export-table',     // 数据导出
    'batch-operation',  // 批量操作
    'system-settings',  // 系统设置
    'template-manage',  // 模板管理
    'data-analysis',    // 数据分析
    'tax-settings'      // 计税设置
  ]

  // 如果是Tauri窗口操作，使用窗口管理器
  if (tauriWindowOperations.includes(item.name)) {
    const selectedData = estimateData.value.filter(row => selectedRowKeys.value.includes(row.id))

    // 表单操作使用专门的方法
    if (['create', 'edit', 'view'].includes(item.name)) {
      if (item.name === 'create') {
        operateWindowManager.createRecord({
          windowInfo: windowInfo.value
        })
      } else if (item.name === 'edit') {
        if (selectedData.length === 0) {
          message.warning('请先选择要编辑的记录')
          return
        }
        operateWindowManager.editRecord(selectedData[0], {
          windowInfo: windowInfo.value
        })
      } else if (item.name === 'view') {
        if (selectedData.length === 0) {
          message.warning('请先选择要查看的记录')
          return
        }
        operateWindowManager.viewRecord(selectedData[0], {
          windowInfo: windowInfo.value
        })
      }
    } else {
      // 其他操作使用通用方法
      operateWindowManager.openOperateWindow(item, {
        windowInfo: windowInfo.value,
        data: selectedData.length > 0 ? selectedData : null,
        tableData: estimateData.value,
        customParams: {
          selectedCount: selectedData.length,
          totalCount: estimateData.value.length,
          hasSelection: selectedData.length > 0
        }
      })
    }
    return
  }

  // 原有的操作逻辑
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
      // 使用弹窗方式导出
      openModalByOperate(item, {
        data: estimateData.value,
        componentProps: {
          tableData: estimateData.value
        }
      })
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
  
  // 窗口管理已简化
  
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