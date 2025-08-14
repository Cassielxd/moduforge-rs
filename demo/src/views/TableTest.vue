<template>
  <div class="table-test">
    <h2>CostTable 测试页面</h2>
    
    <div class="test-section">
      <h3>基本表格功能测试</h3>
      
      <CostTable
        :data="testData"
        :columns="testColumns"
        table-type="estimate"
        :editable="true"
        @open-form="handleOpenForm"
        @edit-row="handleEditRow"
        @delete-row="handleDeleteRow"
        @row-select="handleRowSelect"
      />
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
import { ref } from 'vue'
import { message } from 'ant-design-vue'
import { CostTable } from '@cost-app/shared-components'

const logs = ref([])

const addLog = (msg) => {
  logs.value.unshift(`${new Date().toLocaleTimeString()}: ${msg}`)
  if (logs.value.length > 10) {
    logs.value.pop()
  }
}

const testColumns = [
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
    title: '金额', 
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
    title: '操作',
    key: 'action',
    width: 200,
    fixed: 'right'
  }
]

const testData = ref([
  {
    id: 1,
    name: '测试项目1',
    type: 'building',
    amount: 1000000,
    status: 'approved'
  },
  {
    id: 2,
    name: '测试项目2',
    type: 'infrastructure',
    amount: 2000000,
    status: 'reviewing'
  },
  {
    id: 3,
    name: '测试项目3',
    type: 'renovation',
    amount: 500000,
    status: 'draft'
  }
])

const handleOpenForm = ({ type, data }) => {
  addLog(`打开表单: ${type}, 数据: ${data ? data.name : '无'}`)
  message.info(`打开表单: ${type}`)
}

const handleEditRow = (record) => {
  addLog(`编辑行: ${record.name}`)
  message.info(`编辑: ${record.name}`)
}

const handleDeleteRow = (record) => {
  addLog(`删除行: ${record.name}`)
  const index = testData.value.findIndex(item => item.id === record.id)
  if (index > -1) {
    testData.value.splice(index, 1)
    message.success(`已删除: ${record.name}`)
  }
}

const handleRowSelect = (rows) => {
  addLog(`选中行数: ${rows.length}`)
  if (rows.length > 0) {
    message.info(`已选中 ${rows.length} 项`)
  }
}
</script>

<style scoped>
.table-test {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
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
  max-height: 200px;
  overflow-y: auto;
  background: #f5f5f5;
  padding: 12px;
  border-radius: 4px;
}

.log-item {
  padding: 4px 0;
  border-bottom: 1px solid #eee;
  font-family: monospace;
  font-size: 12px;
}

.log-item:last-child {
  border-bottom: none;
}

h2 {
  color: #1890ff;
  margin-bottom: 24px;
}

h3 {
  color: #262626;
  margin-bottom: 16px;
}
</style>
