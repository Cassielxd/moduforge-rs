<template>
  <div class="cost-table-container">
    <!-- 表格内容 -->
    <div class="table-content">
      <s-table
        ref="stableRef"
        :columns="computedColumns"
        :data-source="filteredData"
        :delay="200"
        :animateRows="false"
        :pagination="false"
        :loading="loading"
        :scroll="scrollConfig"
        :size="tableSize"
        :bordered="bordered"
        :row-key="rowKey"
        :expandable="null"
        :range-selection="rangeSelection"
        :row-selection="rowSelection"
        :custom-row="customRow"
        :custom-cell="customCell"
        :custom-header-cell="customHeaderCell"
        :row-class-name="rowClassName"
        @change="handleTableChange"
        @cell-click="handleCellClick"
        @cell-dblclick="handleCellDblClick"
        @row-click="handleRowClick"
        @row-dblclick="handleRowDblClick"
        @cell-keydown="handleCellKeydown"
        @mouseup="handleMouseUp"
        @mousedown="handleMouseDown"
        @open-editor="handleOpenEditor"
        @close-editor="handleCloseEditor"
        @cell-edited="handleCellEdited"
      >
        <!-- 自定义表头 -->
        <template #headerCell="{ title, column }">
          <span class="custom-header" style="font-weight: bold">
            <i class="vxe-icon-edit" v-show="column.editable"></i>
            &nbsp;{{ title }}
            <CloseOutlined
              class="icon-close-s"
              @click="hideColumn(column)"
              v-if="column.closable !== false"
            />
          </span>
        </template>

        <!-- 自定义单元格内容 -->
        <template #bodyCell="{ text, record, index, column, key, openEditor, closeEditor }">
          <!-- 树形结构列（编码列，包含展开按钮） -->
          <div v-if="column.field === treeCodeField" class="tree-cell">
            <div class="tree-content">
              <!-- 树形辅助线 -->
              <div class="tree-lines" v-if="record.level > 0">
                <!-- 垂直线 -->
                <div
                  v-for="lineLevel in record.level"
                  :key="'v-' + lineLevel"
                  class="tree-line tree-line-vertical"
                  :style="{ left: (lineLevel - 1) * 20 + 10 + 'px' }"
                ></div>
                <!-- 水平线 -->
                <div
                  class="tree-line tree-line-horizontal"
                  :style="{
                    left: (record.level - 1) * 20 + 10 + 'px',
                    width: '10px'
                  }"
                ></div>
                <!-- 最后一个子节点的垂直线只到中间 -->
                <div
                  v-if="isLastChild(record)"
                  class="tree-line tree-line-vertical tree-line-last"
                  :style="{ left: (record.level - 1) * 20 + 10 + 'px' }"
                ></div>
              </div>

              <!-- 缩进 -->
              <div :style="{ paddingLeft: getTreeIndent(record) + 'px' }" class="tree-indent">
                <!-- 展开/收起按钮 -->
                <div class="tree-expand-wrapper">
                  <div
                    v-if="hasChildren(record)"
                    :class="getTreeExpandIcon(record)"
                    @click="toggleExpand(record)"
                  ></div>
                  <div v-else class="tree-expand-placeholder"></div>
                </div>

                <!-- 编码文本 -->
                <span class="tree-text">{{ record[column.field] }}</span>
              </div>
            </div>
          </div>

          <!-- 可编辑单元格 -->
          <div v-else-if="isCellEditable(column, record)" class="editable-cell">
            <div
              class="cell-content"
              @dblclick="openEditor(key)"
              @mouseover="handleCellMouseEnter(record, column)"
              @mouseout="handleCellMouseLeave(record, column)"
            >
              <span>{{ getCellDisplayValue(record, column) }}</span>
              <i class="edit-icon" v-if="!record.isLocked">✏️</i>
            </div>
          </div>

          <!-- 普通单元格 -->
          <div v-else>
            <span>{{ getCellDisplayValue(record, column) }}</span>
          </div>

          <!-- 自定义单元格插槽 -->
          <slot
            :name="`cell-${column.field}`"
            :record="record"
            :column="column"
            :text="text"
            :index="index"
          ></slot>
        </template>

        <!-- 右键菜单 -->
        <template #contextmenuPopup="{ record, column }">
          <slot name="context-menu" :record="record" :column="column">
            <div class="context-menu">
              <a-menu @click="handleContextMenuClick">
                <a-menu-item key="copy" :disabled="!hasSelection">
                  <CopyOutlined /> 复制
                </a-menu-item>
                <a-menu-item key="paste" :disabled="!canPaste" v-if="editable">
                  <FileAddOutlined /> 粘贴
                </a-menu-item>
                <a-menu-divider v-if="editable" />
                <a-menu-item key="add" v-if="editable">
                  <PlusOutlined /> 新增行
                </a-menu-item>
                <a-menu-item key="delete" :disabled="!hasSelection" v-if="editable">
                  <DeleteOutlined /> 删除行
                </a-menu-item>
                <a-menu-divider v-if="editable" />
                <a-menu-item key="moveUp" :disabled="!canMoveUp">
                  <ArrowUpOutlined /> 上移
                </a-menu-item>
                <a-menu-item key="moveDown" :disabled="!canMoveDown">
                  <ArrowDownOutlined /> 下移
                </a-menu-item>
              </a-menu>
            </div>
          </slot>
        </template>
      </s-table>
    </div>



    <!-- 表单编辑弹窗 -->
    <a-modal
      v-model:visible="formVisible"
      title="表单编辑"
      width="800px"
      @ok="handleFormSubmit"
      @cancel="handleFormCancel"
    >
      <component
        v-if="formComponent"
        :is="formComponent"
        v-model:value="formData"
        :columns="formColumns"
        :record="currentRecord"
        @submit="handleFormSubmit"
        @cancel="handleFormCancel"
      />
      <slot name="form" :record="currentRecord" :data="formData"></slot>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
// import { message } from 'ant-design-vue'
import {
  CloseOutlined,
  CopyOutlined,
  FileAddOutlined,
  PlusOutlined,
  DeleteOutlined,
  ArrowUpOutlined,
  ArrowDownOutlined
} from '@ant-design/icons-vue'
import { useTableSelection } from '../composables/useTableSelection.js'
import { useTableEditing } from '../composables/useTableEditing.js'
import { useTreeTable } from '../composables/useTreeTable.js'

// 简单的消息提示替代方案
const message = {
  success: (msg) => console.log('✓', msg),
  warning: (msg) => console.warn('⚠', msg),
  error: (msg) => console.error('✗', msg),
  info: (msg) => console.info('ℹ', msg)
}

// Props定义
const props = defineProps({
  // 数据相关
  data: {
    type: Array,
    default: () => []
  },
  columns: {
    type: Array,
    required: true
  },

  // 表格配置
  tableType: {
    type: String,
    default: 'budget' // budget, estimate, settlement, measures
  },
  rowKey: {
    type: String,
    default: 'id'
  },
  treeProps: {
    type: Object,
    default: () => ({
      children: 'children',
      hasChildren: 'hasChildren'
    })
  },
  treeCodeField: {
    type: String,
    default: 'code' // 树形结构显示的编码字段
  },

  // 功能开关
  editable: {
    type: Boolean,
    default: true
  },
  rangeSelection: {
    type: Boolean,
    default: true
  },
  bordered: {
    type: Boolean,
    default: true
  },

  // 表格样式
  tableSize: {
    type: String,
    default: 'small'
  },
  scrollConfig: {
    type: Object,
    default: () => ({ x: 1200, y: 'calc(100vh - 300px)' })
  },

  // 表单组件
  formComponent: {
    type: [Object, String],
    default: null
  },
  formColumns: {
    type: Array,
    default: () => []
  },

  // 自定义配置
  customRowClassName: {
    type: Function,
    default: null
  },
  customCellRender: {
    type: Function,
    default: null
  }
})

// Emits定义
const emit = defineEmits([
  'dataChange',
  'rowSelect',
  'cellEdit',
  'cellEdited',
  'openForm',
  'editRow',
  'deleteRow',
  'addRow',
  'copyRows',
  'pasteRows',
  'moveRow',
  'expandChange',
  'selectionChange'
])

// 响应式数据
const stableRef = ref(null)
const searchText = ref('')
const loading = ref(false)
const selectedRowKeys = ref([])
const expandedRowKeys = ref([])
const copyData = ref(null)
const formVisible = ref(false)
const formData = ref({})
const currentRecord = ref(null)

// 使用组合式函数
const {
  clearSelection,
  getSelectedRows
} = useTableSelection(selectedRowKeys, props.data)

const {
  openCellEditor,
  closeCellEditor
} = useTableEditing()

const {
  toggleExpand,
  isRowExpanded,
  hasChildren,
  getRowKey
} = useTreeTable(expandedRowKeys, props.treeProps)

// 计算属性
const filteredData = computed(() => {
  // 将树形数据扁平化，根据展开状态显示
  const flattenTreeData = (data, level = 0) => {
    const result = []

    data.forEach(item => {
      // 添加层级信息
      const processedItem = {
        ...item,
        level: level,
        hasChildren: !!(item.children && item.children.length > 0)
      }

      result.push(processedItem)

      // 如果有子节点且当前节点已展开，递归处理子节点
      if (item.children && item.children.length > 0 && isRowExpanded(getRowKey(item))) {
        result.push(...flattenTreeData(item.children, level + 1))
      }
    })

    return result
  }

  const flatData = flattenTreeData(props.data)

  // 如果有搜索条件，进行过滤
  if (!searchText.value) return flatData

  return flatData.filter(item => {
    const searchLower = searchText.value.toLowerCase()
    return Object.values(item).some(value =>
      String(value).toLowerCase().includes(searchLower)
    )
  })
})

const computedColumns = computed(() => {
  const baseColumns = []


  // 处理用户定义的列
  const userColumns = props.columns.map(col => ({
    ...col,
    key: col.dataIndex || col.field,
    dataIndex: col.dataIndex || col.field,
    sorter: col.sorter !== false,
    ellipsis: col.ellipsis !== false,
    editable: col.editable || false,
    closable: col.closable !== false
  }))

  return [...baseColumns, ...userColumns]
})

const hasSelection = computed(() => selectedRowKeys.value.length > 0)

const canPaste = computed(() => copyData.value && copyData.value.length > 0)

const canMoveUp = computed(() => {
  if (!hasSelection.value) return false
  const firstSelectedIndex = filteredData.value.findIndex(
    item => selectedRowKeys.value.includes(getRowKey(item))
  )
  return firstSelectedIndex > 0
})

const canMoveDown = computed(() => {
  if (!hasSelection.value) return false
  const lastSelectedIndex = filteredData.value.findLastIndex(
    item => selectedRowKeys.value.includes(getRowKey(item))
  )
  return lastSelectedIndex < filteredData.value.length - 1
})

const rowSelection = computed(() => ({
  selectedRowKeys: selectedRowKeys.value,
  hideSelectAll: true,
  fixed: true,
  onChange: (keys, rows) => {
    selectedRowKeys.value = keys
    emit('rowSelect', rows)
    emit('selectionChange', { keys, rows })
  },
  getCheckboxProps: (record) => ({
    disabled: record.disabled === true || record.isLocked === true,
    name: record.name,
  }),
}))



const getCellDisplayValue = (record, column) => {
  const value = record[column.field || column.dataIndex]
  if (column.formatter && typeof column.formatter === 'function') {
    return column.formatter(value, record, column)
  }
  return value || ''
}

const isCellEditable = (column, record) => {
  if (!props.editable || record.isLocked) return false

  if (typeof column.editable === 'function') {
    return column.editable({ record, column })
  }

  return column.editable === true || column.editable === 'cellEditorSlot'
}

// 新的树形展开图标样式
const getTreeExpandIcon = (record) => {
  if (!hasChildren(record)) return 'tree-expand-placeholder'
  return isRowExpanded(getRowKey(record)) ? 'tree-expand-icon tree-expand-icon-expanded' : 'tree-expand-icon tree-expand-icon-collapsed'
}

// 获取树形缩进
const getTreeIndent = (record) => {
  const level = record.level || 0
  return level * 20 // 每级缩进20px
}

// 判断是否为最后一个子节点
const isLastChild = (record) => {
  // 在扁平化的数据中查找父子关系
  const findParentAndSiblings = (target, data) => {
    for (const item of data) {
      if (item.children && item.children.includes(target)) {
        return { parent: item, siblings: item.children }
      }
      if (item.children && item.children.length > 0) {
        const result = findParentAndSiblings(target, item.children)
        if (result) return result
      }
    }
    return null
  }

  const parentInfo = findParentAndSiblings(record, props.data)
  if (!parentInfo) return true // 根节点

  const { siblings } = parentInfo
  return siblings[siblings.length - 1] === record
}



const handleTableChange = (pagination, filters, sorter) => {
  console.log('Table changed:', pagination, filters, sorter)
}







// 表格事件处理方法
const handleCellClick = (record, column, event) => {
  // 阻止事件冒泡，避免触发行选择
  event.stopPropagation()
  emit('cellEdit', { record, column, event })
}

const handleCellDblClick = (record, column, event) => {
  if (isCellEditable(column, record)) {
    openCellEditor(record, column)
  }
}

const handleRowClick = (record, event) => {
  if (!event.ctrlKey) {
    clearSelection()
  }
  selectRow(record)
  emit('rowSelect', [record])
}

const handleRowDblClick = (record, event) => {
  emit('editRow', record)
}



const handleCellKeydown = (event, { cellPosition, isEditing }) => {
  // 处理键盘事件，如方向键导航、回车确认等
  if (event.code === 'Enter' && !isEditing) {
    const { rowIndex, column } = cellPosition
    const record = filteredData.value[rowIndex]
    if (isCellEditable(column, record)) {
      openCellEditor(record, column)
    }
  }
}

const handleMouseUp = (event) => {
  // 处理拖选结束
  if (stableRef.value) {
    const selectedRange = stableRef.value.getSelectedRange()
    if (selectedRange && selectedRange.length > 0) {
      const rangeKeys = []
      selectedRange.forEach(range => {
        for (let i = range.startRow.rowIndex; i <= range.endRow.rowIndex; i++) {
          const record = filteredData.value[i]
          if (record) {
            rangeKeys.push(getRowKey(record))
          }
        }
      })
      selectedRowKeys.value = Array.from(new Set(rangeKeys))
    }
  }
}

const handleMouseDown = (event) => {
  // 处理鼠标按下事件
}

const handleOpenEditor = (cellInfo) => {
  openCellEditor(cellInfo.record, cellInfo.column)
}

const handleCloseEditor = (cellInfo) => {
  closeCellEditor()
}

const handleCellEdited = (cellInfo, newValue, oldValue) => {
  const { record, column } = cellInfo
  record[column.field || column.dataIndex] = newValue
  emit('cellEdited', { record, column, newValue, oldValue })
}

const handleCellMouseEnter = (record, column) => {
  // 鼠标进入单元格
}

const handleCellMouseLeave = (record, column) => {
  // 鼠标离开单元格
}

// 右键菜单处理
const handleContextMenuClick = ({ key }) => {
  switch (key) {
    case 'copy':
      copySelected()
      break
    case 'paste':
      pasteRows()
      break
    case 'add':
      addRow()
      break
    case 'delete':
      deleteSelected()
      break
    case 'moveUp':
      moveSelectedUp()
      break
    case 'moveDown':
      moveSelectedDown()
      break
  }
}

// 新的工具栏操作方法
const addRow = () => {
  emit('addRow')
}

const deleteSelected = () => {
  if (!hasSelection.value) return

  const selectedRows = getSelectedRows()
  selectedRows.forEach(record => {
    emit('deleteRow', record)
  })

  selectedRowKeys.value = []
  message.success(`已删除 ${selectedRows.length} 条记录`)
}

const copySelected = () => {
  if (!hasSelection.value) return

  const selectedRows = getSelectedRows()
  copyData.value = JSON.parse(JSON.stringify(selectedRows))
  message.success(`已复制 ${selectedRows.length} 项`)
  emit('copyRows', selectedRows)
}

const pasteRows = () => {
  if (!canPaste.value) return

  const newRows = copyData.value.map(row => ({
    ...row,
    [props.rowKey]: Date.now() + Math.random(),
    children: row.children ? [...row.children] : []
  }))

  const newData = [...props.data, ...newRows]
  emit('dataChange', newData)
  emit('pasteRows', newRows)
  message.success(`已粘贴 ${newRows.length} 项`)
}

const moveSelectedUp = () => {
  // 上移选中行
  emit('moveRow', { direction: 'up', rows: getSelectedRows() })
}

const moveSelectedDown = () => {
  // 下移选中行
  emit('moveRow', { direction: 'down', rows: getSelectedRows() })
}

// 辅助方法
const getDescendants = (record) => {
  const descendants = []
  const children = record[props.treeProps.children] || []

  children.forEach(child => {
    descendants.push(child)
    descendants.push(...getDescendants(child))
  })

  return descendants
}

const hideColumn = (column) => {
  // 隐藏列的逻辑
  console.log('隐藏列:', column)
}

const customRow = (record) => {
  return {
    onClick: (event) => handleRowClick(record, event),
    onDblclick: (event) => handleRowDblClick(record, event),
    class: props.customRowClassName ? props.customRowClassName(record) : ''
  }
}

const customCell = (record, column) => {
  return {
    onClick: (event) => {
      // 阻止事件冒泡，避免触发行选择
      event.stopPropagation()
      handleCellClick(record, column, event)
    },
    onDblclick: (event) => {
      event.stopPropagation()
      handleCellDblClick(record, column, event)
    }
  }
}

const customHeaderCell = (column) => {
  return {
    class: 'custom-header-cell'
  }
}

const rowClassName = (record, index) => {
  const classes = []

  if (selectedRowKeys.value.includes(getRowKey(record))) {
    classes.push('selected-row')
  }

  if (record.isLocked) {
    classes.push('locked-row')
  }

  if (props.customRowClassName) {
    const customClass = props.customRowClassName(record, index)
    if (customClass) {
      classes.push(customClass)
    }
  }

  return classes.join(' ')
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
  padding: 8px 0;
  border-bottom: 1px solid #f0f0f0;
  margin-bottom: 8px;
}

.table-search {
  width: 300px;
}

.table-content {
  flex: 1;
  overflow: hidden;
  min-height: 0;
  height: 0; /* 强制计算高度 */
  max-height: calc(100vh - 200px); /* 设置最大高度 */
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

:deep(.ant-table) {
  height: 100%;
  display: flex;
  flex-direction: column;
}

:deep(.ant-table-container) {
  height: 100%;
  display: flex;
  flex-direction: column;
}

:deep(.ant-table-header) {
  flex-shrink: 0;
}

:deep(.ant-table-body) {
  flex: 1;
  overflow: auto !important;
  min-height: 0;
}

:deep(.surely-table-body-viewport-container) {
  max-height: 100% !important;
  overflow: auto !important;
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

/* 树形表格样式 */
.tree-cell {
  position: relative;
  width: 100%;
}

.tree-content {
  position: relative;
  width: 100%;
}

/* 树形辅助线 */
.tree-lines {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 100%;
  pointer-events: none;
  z-index: 1;
}

.tree-line {
  position: absolute;
  background: #d9d9d9;
}

/* 垂直线 */
.tree-line-vertical {
  width: 1px;
  height: 100%;
  top: 0;
}

/* 水平线 */
.tree-line-horizontal {
  height: 1px;
  top: 50%;
}

/* 最后一个子节点的垂直线只到中间 */
.tree-line-last {
  height: 50% !important;
}

/* 树形缩进容器 */
.tree-indent {
  display: flex;
  align-items: center;
  position: relative;
  z-index: 2;
}

/* 展开/收起按钮 */
.tree-expand-wrapper {
  position: relative;
  width: 16px;
  height: 16px;
  margin-right: 8px;
  flex-shrink: 0;
}

.tree-expand-icon {
  width: 16px;
  height: 16px;
  border: 1px solid #d9d9d9;
  border-radius: 2px;
  background: #fff;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: bold;
  line-height: 1;
  position: relative;
  z-index: 3;
}

.tree-expand-icon:hover {
  border-color: #1890ff;
  color: #1890ff;
}

.tree-expand-icon-expanded {
  background: #1890ff;
  border-color: #1890ff;
  color: #fff;
}

.tree-expand-icon-expanded::before {
  content: '−';
}

.tree-expand-icon-collapsed {
  background: #fff;
  border-color: #d9d9d9;
  color: #666;
}

.tree-expand-icon-collapsed::before {
  content: '+';
}

.tree-expand-placeholder {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.tree-text {
  flex: 1;
  margin-left: 4px;
}

/* 序号列样式 */
.sequence-cell {
  text-align: center;
  padding: 4px 8px;
}

/* 隐藏s-table默认的展开列和图标 */
:deep(.ant-table-row-expand-icon-cell) {
  display: none !important;
  width: 0 !important;
  padding: 0 !important;
}

:deep(.ant-table-row-expand-icon) {
  display: none !important;
  width: 0 !important;
  height: 0 !important;
  margin: 0 !important;
  padding: 0 !important;
}

:deep(.ant-table-expand-icon-col) {
  display: none !important;
  width: 0 !important;
}

/* 隐藏所有可能的展开相关元素 */
:deep(.ant-table-row-expand-icon-spaced) {
  display: none !important;
}

:deep(.ant-table-row-indent) {
  display: none !important;
}

:deep(.ant-table-row-expand-icon-collapsed),
:deep(.ant-table-row-expand-icon-expanded) {
  display: none !important;
}

/* 确保第一列不显示任何展开图标 */
:deep(.ant-table-tbody > tr > td:first-child .ant-table-row-expand-icon) {
  display: none !important;
}

:deep(.ant-table-tbody > tr > td:first-child .ant-table-row-indent) {
  display: none !important;
}

/* 隐藏 surely-table 的默认树形展开图标 */
:deep(.surely-table-row-indent + .surely-table-row-expand-icon) {
  display: none !important;
}

:deep(.ant-table-row-indent) {
  display: none !important;
}

:deep(.ant-table-tbody .ant-table-row .ant-table-cell:first-child) {
  padding-left: 8px !important;
}

/* 移除默认的树形层级缩进 */
:deep(.ant-table-row-level-1 .ant-table-cell:first-child),
:deep(.ant-table-row-level-2 .ant-table-cell:first-child),
:deep(.ant-table-row-level-3 .ant-table-cell:first-child),
:deep(.ant-table-row-level-4 .ant-table-cell:first-child) {
  padding-left: 8px !important;
}

/* 可编辑单元格样式 */
.editable-cell {
  position: relative;
}

.cell-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 24px;
  padding: 2px 4px;
  border-radius: 2px;
  transition: background-color 0.2s;
}

.cell-content:hover {
  background-color: #f5f5f5;
}

.edit-icon {
  opacity: 0;
  font-size: 12px;
  color: #999;
  transition: opacity 0.2s;
}

.cell-content:hover .edit-icon {
  opacity: 1;
}

/* 自定义表头样式 */
.custom-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.icon-close-s {
  opacity: 0;
  cursor: pointer;
  color: #999;
  font-size: 12px;
  transition: opacity 0.2s;
}

.custom-header:hover .icon-close-s {
  opacity: 1;
}

.icon-close-s:hover {
  color: #ff4d4f;
}

/* 右键菜单样式 */
.context-menu {
  background: white;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 4px 0;
  min-width: 120px;
}

/* 选中行样式 */
.selected-row {
  background-color: #e6f7ff !important;
}

.locked-row {
  background-color: #f5f5f5 !important;
  color: #999;
}

/* 表格工具栏样式 */
.table-toolbar {
  background: #fafafa;
  border-radius: 6px 6px 0 0;
}

/* 表格底部汇总样式 */
.table-footer {
  background: #fafafa;
  border-radius: 0 0 6px 6px;
}

/* 响应式样式 */
@media (max-width: 768px) {
  .table-toolbar {
    flex-direction: column;
    gap: 8px;
  }

  .table-search {
    width: 100%;
  }
}
</style>
