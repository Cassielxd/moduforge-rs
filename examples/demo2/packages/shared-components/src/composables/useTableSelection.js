/**
 * 表格选择功能组合式函数
 * @description 提供表格行选择、多选、范围选择等功能
 */
import { computed } from 'vue'

export function useTableSelection(selectedRowKeys, tableData) {
  // 获取选中的行数据
  const getSelectedRows = () => {
    if (!tableData.value || !Array.isArray(tableData.value)) {
      return []
    }
    return tableData.value.filter(item =>
      selectedRowKeys.value.includes(getRowKey(item))
    )
  }

  // 获取行的唯一标识
  const getRowKey = (record) => {
    return record.id || record.sequenceNbr || record.key
  }

  // 选中单行
  const selectRow = (record) => {
    const rowKey = getRowKey(record)
    if (!selectedRowKeys.value.includes(rowKey)) {
      selectedRowKeys.value.push(rowKey)
    }
  }

  // 取消选中单行
  const unselectRow = (record) => {
    const rowKey = getRowKey(record)
    const index = selectedRowKeys.value.indexOf(rowKey)
    if (index > -1) {
      selectedRowKeys.value.splice(index, 1)
    }
  }

  // 选中多行
  const selectRows = (records) => {
    const rowKeys = records.map(getRowKey)
    selectedRowKeys.value = Array.from(new Set([...selectedRowKeys.value, ...rowKeys]))
  }

  // 取消选中多行
  const unselectRows = (records) => {
    const rowKeys = records.map(getRowKey)
    selectedRowKeys.value = selectedRowKeys.value.filter(key => !rowKeys.includes(key))
  }

  // 全选
  const selectAll = () => {
    if (!tableData.value || !Array.isArray(tableData.value)) {
      return
    }
    selectedRowKeys.value = tableData.value.map(getRowKey)
  }

  // 清空选择
  const clearSelection = () => {
    selectedRowKeys.value = []
  }

  // 反选
  const invertSelection = () => {
    if (!tableData.value || !Array.isArray(tableData.value)) {
      return
    }
    const allKeys = tableData.value.map(getRowKey)
    selectedRowKeys.value = allKeys.filter(key => !selectedRowKeys.value.includes(key))
  }

  // 判断行是否被选中
  const isRowSelected = (record) => {
    return selectedRowKeys.value.includes(getRowKey(record))
  }

  // 判断是否有选中项
  const hasSelection = computed(() => selectedRowKeys.value.length > 0)

  // 判断是否全选
  const isAllSelected = computed(() => {
    if (!tableData.value || !Array.isArray(tableData.value)) {
      return false
    }
    return tableData.value.length > 0 &&
           selectedRowKeys.value.length === tableData.value.length
  })

  // 判断是否部分选中
  const isIndeterminate = computed(() => {
    if (!tableData.value || !Array.isArray(tableData.value)) {
      return false
    }
    return selectedRowKeys.value.length > 0 &&
           selectedRowKeys.value.length < tableData.value.length
  })

  // 范围选择
  const selectRange = (startRecord, endRecord) => {
    if (!tableData.value || !Array.isArray(tableData.value)) {
      return
    }
    const startIndex = tableData.value.findIndex(item => getRowKey(item) === getRowKey(startRecord))
    const endIndex = tableData.value.findIndex(item => getRowKey(item) === getRowKey(endRecord))

    if (startIndex === -1 || endIndex === -1) return

    const minIndex = Math.min(startIndex, endIndex)
    const maxIndex = Math.max(startIndex, endIndex)

    const rangeRecords = tableData.value.slice(minIndex, maxIndex + 1)
    selectRows(rangeRecords)
  }

  // 切换行选择状态
  const toggleRowSelection = (record) => {
    if (isRowSelected(record)) {
      unselectRow(record)
    } else {
      selectRow(record)
    }
  }

  // 切换全选状态
  const toggleSelectAll = () => {
    if (isAllSelected.value) {
      clearSelection()
    } else {
      selectAll()
    }
  }

  // 根据条件选择行
  const selectByCondition = (predicate) => {
    if (!tableData.value || !Array.isArray(tableData.value)) {
      return
    }
    const matchingRecords = tableData.value.filter(predicate)
    selectRows(matchingRecords)
  }

  // 获取选中行的索引
  const getSelectedIndexes = () => {
    if (!tableData.value || !Array.isArray(tableData.value)) {
      return []
    }
    return selectedRowKeys.value.map(key => {
      return tableData.value.findIndex(item => getRowKey(item) === key)
    }).filter(index => index !== -1)
  }

  // 获取选中行的数量
  const getSelectionCount = computed(() => selectedRowKeys.value.length)

  return {
    // 状态
    hasSelection,
    isAllSelected,
    isIndeterminate,
    getSelectionCount,
    
    // 方法
    selectRow,
    unselectRow,
    selectRows,
    unselectRows,
    selectAll,
    clearSelection,
    invertSelection,
    selectRange,
    selectByCondition,
    toggleRowSelection,
    toggleSelectAll,
    
    // 查询方法
    isRowSelected,
    getSelectedRows,
    getSelectedIndexes,
    getRowKey
  }
}
