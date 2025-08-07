<template>
  <div class="budget-module">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>预算管理</span>
          <el-button type="primary" @click="showDemo">演示共享组件</el-button>
        </div>
      </template>

      <div class="module-content">
        <el-alert
          title="微前端模块演示"
          type="info"
          description="这是预算模块的演示页面。在完整的微前端架构中，这将是一个独立的应用。"
          show-icon
          :closable="false"
          style="margin-bottom: 20px;"
        />

        <!-- 演示共享组件 -->
        <div v-if="showTable" class="demo-table">
          <h3>预算清单</h3>
          <CostTable
            :data="budgetData"
            :columns="budgetColumns"
            table-type="budget"
            :editable="true"
            @data-change="handleDataChange"
            @row-select="handleRowSelect"
            @cell-edit="handleCellEdit"
          />
        </div>

        <div v-else class="placeholder">
          <el-empty description="点击上方按钮查看共享表格组件演示" />
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
// 这里演示如何使用共享组件
// import { CostTable } from '@cost-app/shared-components'

// 临时的表格组件占位
const CostTable = {
  props: ['data', 'columns', 'tableType', 'editable'],
  emits: ['dataChange', 'rowSelect', 'cellEdit'],
  template: `
    <div class="demo-cost-table">
      <div class="table-toolbar">
        <el-button-group>
          <el-button type="primary" icon="Plus">新增</el-button>
          <el-button type="danger" icon="Delete">删除</el-button>
          <el-button icon="Download">导出</el-button>
        </el-button-group>
      </div>
      <el-table :data="data" style="width: 100%; margin-top: 10px;">
        <el-table-column v-for="col in columns" :key="col.field" 
          :prop="col.field" :label="col.title" :width="col.width" />
      </el-table>
      <div class="table-summary">
        <span>总计: ¥{{ calculateTotal() }}</span>
      </div>
    </div>
  `,
  methods: {
    calculateTotal() {
      return this.data.reduce((sum, item) => sum + (item.amount || 0), 0).toFixed(2)
    }
  }
}

const showTable = ref(false)

const budgetData = ref([
  { id: 1, name: '土建工程', quantity: 1000, unitPrice: 500, amount: 500000, unit: 'm²' },
  { id: 2, name: '装修工程', quantity: 800, unitPrice: 300, amount: 240000, unit: 'm²' },
  { id: 3, name: '机电工程', quantity: 1, unitPrice: 150000, amount: 150000, unit: '项' },
  { id: 4, name: '园林绿化', quantity: 500, unitPrice: 200, amount: 100000, unit: 'm²' }
])

const budgetColumns = ref([
  { field: 'name', title: '项目名称', width: 200 },
  { field: 'quantity', title: '工程量', width: 120 },
  { field: 'unit', title: '单位', width: 80 },
  { field: 'unitPrice', title: '单价', width: 120 },
  { field: 'amount', title: '金额', width: 150 }
])

const showDemo = () => {
  showTable.value = true
  ElMessage.success('已加载共享表格组件演示')
}

const handleDataChange = (data) => {
  console.log('数据变更:', data)
  ElMessage.info('数据已更新')
}

const handleRowSelect = (row) => {
  console.log('选中行:', row)
  ElMessage.info(`已选中: ${row.name}`)
}

const handleCellEdit = (editInfo) => {
  console.log('单元格编辑:', editInfo)
  ElMessage.success('单元格已更新')
}
</script>

<style scoped>
.budget-module {
  max-width: 1200px;
  margin: 0 auto;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.module-content {
  min-height: 400px;
}

.demo-table h3 {
  margin-bottom: 15px;
  color: #303133;
}

.placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 300px;
}

.demo-cost-table .table-toolbar {
  margin-bottom: 10px;
}

.demo-cost-table .table-summary {
  margin-top: 10px;
  text-align: right;
  font-weight: 600;
  color: #409eff;
}
</style>
