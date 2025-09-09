<template>
  <div class="table-test">
    <h2>CostTable 功能演示</h2>

    <div class="demo-controls">
      <a-space>
        <a-button type="primary" @click="addRootItem">
          <template #icon><PlusOutlined /></template>
          添加根节点
        </a-button>
        <a-button @click="expandAll">
          <template #icon><ExpandOutlined /></template>
          展开全部
        </a-button>
        <a-button @click="collapseAll">
          <template #icon><ShrinkOutlined /></template>
          折叠全部
        </a-button>
        <a-button @click="clearSelection">
          <template #icon><ClearOutlined /></template>
          清除选择
        </a-button>
      </a-space>
    </div>

    <div class="test-section">
      <h3>树形表格 - 支持单元格编辑、多选、右键菜单</h3>

      <CostTable
        :data="treeData"
        :columns="treeColumns"
        :editable="true"
        :show-toolbar="true"
        :show-summary="true"
        :range-selection="true"
        :tree-props="treeProps"
        table-type="subItem"
        row-key="id"
        @data-change="handleDataChange"
        @cell-edited="handleCellEdited"
        @row-select="handleRowSelect"
        @add-row="handleAddRow"
        @delete-row="handleDeleteRow"
        @copy-rows="handleCopyRows"
        @paste-rows="handlePasteRows"
      >
        <!-- 自定义工具栏 -->
        <template #toolbar-extra>
          <a-button @click="calculateTotal">
            <template #icon><CalculatorOutlined /></template>
            重新计算
          </a-button>
        </template>

        <!-- 自定义单元格编辑器 -->
        <template #cell-type="{ record, column, text }">
          <a-select
            v-if="record.isEditing"
            v-model:value="record.type"
            :options="typeOptions"
            style="width: 100%"
            @change="handleTypeChange(record)"
          />
          <span v-else>{{ text }}</span>
        </template>

        <template #cell-unit="{ record, column, text }">
          <a-select
            v-if="record.isEditing"
            v-model:value="record.unit"
            :options="unitOptions"
            style="width: 100%"
            show-search
          />
          <span v-else>{{ text }}</span>
        </template>

        <!-- 自定义右键菜单 -->
        <template #context-menu="{ record, column }">
          <a-menu-item key="addChild" @click="addChildItem(record)">
            <PlusOutlined /> 添加子项
          </a-menu-item>
          <a-menu-item key="insertBefore" @click="insertBefore(record)">
            <ArrowUpOutlined /> 在上方插入
          </a-menu-item>
          <a-menu-item key="insertAfter" @click="insertAfter(record)">
            <ArrowDownOutlined /> 在下方插入
          </a-menu-item>
          <a-menu-divider />
          <a-menu-item key="delete" @click="deleteItem(record)" danger>
            <DeleteOutlined /> 删除
          </a-menu-item>
        </template>
      </CostTable>
    </div>

    <div class="info-section">
      <h3>操作日志</h3>
      <div class="log-container">
        <div v-for="(log, index) in logs" :key="index" class="log-item">
          {{ log }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, nextTick } from 'vue'
import { message } from 'ant-design-vue'
import {
  PlusOutlined,
  ExpandOutlined,
  ShrinkOutlined,
  ClearOutlined,
  CalculatorOutlined,
  ArrowUpOutlined,
  ArrowDownOutlined,
  DeleteOutlined
} from '@ant-design/icons-vue'
import { CostTable } from '@cost-app/shared-components'

const logs = ref([])

const addLog = (msg) => {
  logs.value.unshift(`${new Date().toLocaleTimeString()}: ${msg}`)
  if (logs.value.length > 20) {
    logs.value.pop()
  }
}

// 树形表格配置
const treeProps = {
  children: 'children',
  hasChildren: 'hasChildren'
}

// 表格列配置 - 支持编辑
const treeColumns = ref([
  {
    title: '项目编码',
    field: 'code',
    dataIndex: 'code',
    width: 150,
    align: 'left',
    fixed: 'left',
    editable: true,
    required: true
  },
  {
    title: '类型',
    field: 'type',
    dataIndex: 'type',
    width: 80,
    align: 'center',
    editable: true,
    slot: true
  },
  {
    title: '项目名称',
    field: 'name',
    dataIndex: 'name',
    width: 200,
    align: 'left',
    editable: true,
    required: true
  },
  {
    title: '规格',
    field: 'specification',
    dataIndex: 'specification',
    width: 120,
    editable: true
  },
  {
    title: '单位',
    field: 'unit',
    dataIndex: 'unit',
    width: 80,
    align: 'center',
    editable: true,
    slot: true
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
    field: 'price',
    dataIndex: 'price',
    width: 100,
    align: 'right',
    editable: true,
    dataType: 'number',
    formatter: (value) => Number(value || 0).toFixed(2)
  },
  {
    title: '合计',
    field: 'total',
    dataIndex: 'total',
    width: 120,
    align: 'right',
    formatter: (value) => Number(value || 0).toFixed(2)
  }
])

// 树形测试数据
const treeData = ref([
  {
    id: '1',
    code: '010101',
    type: '分部',
    name: '土石方工程',
    specification: '',
    unit: '',
    quantity: 0,
    price: 0,
    total: 255000,
    children: [
      {
        id: '1-1',
        code: '010101001',
        type: '分项',
        name: '人工挖土',
        specification: '挖土深度2m以内',
        unit: 'm³',
        quantity: 10000,
        price: 25.50,
        total: 255000,
        children: []
      },
      {
        id: '1-2',
        code: '010101002',
        type: '分项',
        name: '机械挖土',
        specification: '挖掘机挖土',
        unit: 'm³',
        quantity: 5000,
        price: 15.80,
        total: 79000,
        children: []
      }
    ]
  },
  {
    id: '2',
    code: '010102',
    type: '分部',
    name: '地基处理',
    specification: '',
    unit: '',
    quantity: 0,
    price: 0,
    total: 180000,
    children: [
      {
        id: '2-1',
        code: '010102001',
        type: '分项',
        name: '换填垫层',
        specification: '砂石垫层',
        unit: 'm³',
        quantity: 2000,
        price: 90.00,
        total: 180000,
        children: []
      }
    ]
  }
])

// 选项数据
const typeOptions = ref([
  { label: '分部', value: '分部' },
  { label: '分项', value: '分项' },
  { label: '措施', value: '措施' }
])

const unitOptions = ref([
  { label: 'm³', value: 'm³' },
  { label: 'm²', value: 'm²' },
  { label: 'm', value: 'm' },
  { label: 't', value: 't' },
  { label: '个', value: '个' },
  { label: '项', value: '项' }
])

// 事件处理方法
const handleDataChange = (newData) => {
  treeData.value = newData
  addLog('表格数据已更新')
}

const handleCellEdited = ({ record, column, newValue, oldValue }) => {
  addLog(`单元格编辑: ${record.name} - ${column.title}: ${oldValue} → ${newValue}`)

  // 自动计算合计
  if (column.field === 'quantity' || column.field === 'price') {
    record.total = (record.quantity || 0) * (record.price || 0)
    addLog(`自动计算合计: ${record.name} = ${record.total}`)
  }

  message.success('编辑成功')
}

const handleRowSelect = (rows) => {
  addLog(`选中行数: ${rows.length}`)
  if (rows.length > 0) {
    const names = rows.map(r => r.name).join(', ')
    message.info(`已选中 ${rows.length} 项: ${names}`)
  }
}

const handleAddRow = (newRow) => {
  addLog(`新增行: ${newRow.name}`)
  message.success('新增成功')
}

const handleDeleteRow = (deletedRows) => {
  const names = deletedRows.map(r => r.name).join(', ')
  addLog(`删除行: ${names}`)
  message.success(`删除了 ${deletedRows.length} 项`)
}

const handleCopyRows = (copiedRows) => {
  const names = copiedRows.map(r => r.name).join(', ')
  addLog(`复制行: ${names}`)
  message.info(`已复制 ${copiedRows.length} 项`)
}

const handlePasteRows = (pastedRows) => {
  const names = pastedRows.map(r => r.name).join(', ')
  addLog(`粘贴行: ${names}`)
  message.success(`已粘贴 ${pastedRows.length} 项`)
}

const handleTypeChange = (record) => {
  addLog(`类型变更: ${record.name} → ${record.type}`)
  // 根据类型调整其他字段
  if (record.type === '分部') {
    record.unit = ''
    record.quantity = 0
    record.price = 0
    record.total = 0
  }
}

// 工具栏操作
const addRootItem = () => {
  const newItem = {
    id: `new-${Date.now()}`,
    code: `NEW${String(treeData.value.length + 1).padStart(3, '0')}`,
    type: '分部',
    name: '新建项目',
    specification: '',
    unit: '',
    quantity: 0,
    price: 0,
    total: 0,
    children: []
  }
  treeData.value.push(newItem)
  addLog(`添加根节点: ${newItem.name}`)
  message.success('添加根节点成功')
}

const expandAll = () => {
  addLog('展开全部节点')
  message.info('展开全部节点')
}

const collapseAll = () => {
  addLog('折叠全部节点')
  message.info('折叠全部节点')
}

const clearSelection = () => {
  addLog('清除所有选择')
  message.info('已清除选择')
}

const calculateTotal = () => {
  // 递归计算所有项目的合计
  const calculateNode = (node) => {
    if (node.children && node.children.length > 0) {
      node.total = node.children.reduce((sum, child) => {
        return sum + calculateNode(child)
      }, 0)
    } else {
      node.total = (node.quantity || 0) * (node.price || 0)
    }
    return node.total
  }

  treeData.value.forEach(calculateNode)
  addLog('重新计算所有合计')
  message.success('计算完成')
}

// 右键菜单操作
const addChildItem = (parentRecord) => {
  const newChild = {
    id: `${parentRecord.id}-${Date.now()}`,
    code: `${parentRecord.code}001`,
    type: '分项',
    name: '新建子项',
    specification: '',
    unit: 'm³',
    quantity: 0,
    price: 0,
    total: 0,
    children: []
  }

  if (!parentRecord.children) {
    parentRecord.children = []
  }
  parentRecord.children.push(newChild)

  addLog(`添加子项: ${parentRecord.name} → ${newChild.name}`)
  message.success('添加子项成功')
}

const insertBefore = (record) => {
  addLog(`在上方插入: ${record.name}`)
  message.info('在上方插入功能开发中...')
}

const insertAfter = (record) => {
  addLog(`在下方插入: ${record.name}`)
  message.info('在下方插入功能开发中...')
}

const deleteItem = (record) => {
  addLog(`删除项目: ${record.name}`)
  message.success(`已删除: ${record.name}`)
}
</script>

<style scoped>
.table-test {
  padding: 24px;
  max-width: 1400px;
  margin: 0 auto;
  background: #f5f5f5;
  min-height: 100vh;
}

.demo-controls {
  margin-bottom: 24px;
  padding: 16px;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.test-section {
  margin-bottom: 32px;
  padding: 24px;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.info-section {
  padding: 24px;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.log-container {
  max-height: 300px;
  overflow-y: auto;
  background: #f8f9fa;
  padding: 12px;
  border-radius: 4px;
  border: 1px solid #e9ecef;
}

.log-item {
  padding: 6px 0;
  border-bottom: 1px solid #dee2e6;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  color: #495057;
  line-height: 1.4;
}

.log-item:last-child {
  border-bottom: none;
}

h2 {
  color: #1890ff;
  margin-bottom: 24px;
  font-size: 28px;
  font-weight: 600;
}

h3 {
  color: #262626;
  margin-bottom: 16px;
  font-size: 18px;
  font-weight: 500;
}

:deep(.ant-table-tbody > tr > td) {
  padding: 8px 12px;
}

:deep(.ant-table-thead > tr > th) {
  background: #fafafa;
  font-weight: 600;
}
</style>
