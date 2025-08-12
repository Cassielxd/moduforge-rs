<template>
  <div class="cost-table-container">
    <div class="table-toolbar">
      <a-space>
        <a-button type="primary" @click="addRow">新增</a-button>
        <a-button danger @click="deleteSelected">删除</a-button>
        <a-button @click="exportData">导出</a-button>
      </a-space>
      <div class="table-search">
        <a-input-search
          v-model:value="searchText"
          placeholder="搜索..."
          @search="handleSearch"
          @change="handleSearch"
          allow-clear
        />
      </div>
    </div>
    
    <div ref="tableContainer" class="table-content"></div>
    
    <div class="table-footer">
      <div class="summary-info">
        <span>总计: {{ summary.total }}</span>
        <span>已选: {{ summary.selected }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch, computed } from 'vue'
import { TabulatorFull as Tabulator } from 'tabulator-tables'
import 'tabulator-tables/dist/css/tabulator.min.css'
import { useTableOperations } from '../composables/useTableOperations.js'
import { useCostCalculation } from '../composables/useCostCalculation.js'

const props = defineProps({
  data: {
    type: Array,
    default: () => []
  },
  columns: {
    type: Array,
    required: true
  },
  tableType: {
    type: String,
    default: 'budget' // budget, estimate, settlement
  },
  editable: {
    type: Boolean,
    default: true
  }
})

const emit = defineEmits(['dataChange', 'rowSelect', 'cellEdit'])

const tableContainer = ref(null)
const searchText = ref('')
const tabulator = ref(null)

const { addRow, deleteSelected, exportData, handleSearch } = useTableOperations()
const { calculateTotal, calculateSelected } = useCostCalculation()

const summary = computed(() => ({
  total: calculateTotal(props.data),
  selected: calculateSelected(getSelectedRows())
}))

onMounted(() => {
  initTable()
})

watch(() => props.data, (newData) => {
  if (tabulator.value) {
    tabulator.value.setData(newData)
  }
}, { deep: true })

function initTable() {
  const tableConfig = {
    data: props.data,
    columns: props.columns,
    layout: "fitColumns",
    responsiveLayout: "hide",
    pagination: "local",
    paginationSize: 50,
    selectable: true,
    selectableCheck: function(row) {
      return row.getData().selectable !== false
    },
    rowSelected: function(row) {
      emit('rowSelect', row.getData())
    },
    cellEdited: function(cell) {
      emit('cellEdit', {
        row: cell.getRow().getData(),
        field: cell.getField(),
        value: cell.getValue()
      })
      emit('dataChange', tabulator.value.getData())
    }
  }

  if (!props.editable) {
    tableConfig.columns = tableConfig.columns.map(col => ({
      ...col,
      editor: false
    }))
  }

  tabulator.value = new Tabulator(tableContainer.value, tableConfig)
}

function getSelectedRows() {
  return tabulator.value ? tabulator.value.getSelectedData() : []
}

defineExpose({
  getTableData: () => tabulator.value?.getData() || [],
  getSelectedData: getSelectedRows,
  clearSelection: () => tabulator.value?.deselectRow(),
  refreshTable: () => tabulator.value?.redraw()
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
  padding: 12px 0;
  border-bottom: 1px solid #f0f0f0;
}

.table-search {
  width: 300px;
}

.table-content {
  flex: 1;
  min-height: 400px;
}

.table-footer {
  padding: 8px 0;
  border-top: 1px solid #e4e7ed;
  background: #f5f7fa;
}

.summary-info {
  display: flex;
  gap: 20px;
  font-size: 14px;
  color: #606266;
}
</style>
