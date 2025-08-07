import { ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'

export function useTableOperations() {
  const loading = ref(false)

  const addRow = () => {
    // 添加新行的逻辑
    console.log('添加新行')
    ElMessage.success('已添加新行')
  }

  const deleteSelected = async () => {
    try {
      await ElMessageBox.confirm('确定要删除选中的行吗？', '确认删除', {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      })
      
      // 删除逻辑
      console.log('删除选中行')
      ElMessage.success('删除成功')
    } catch {
      ElMessage.info('已取消删除')
    }
  }

  const exportData = () => {
    // 导出数据逻辑
    console.log('导出数据')
    ElMessage.success('导出成功')
  }

  const handleSearch = (searchText) => {
    // 搜索逻辑
    console.log('搜索:', searchText)
  }

  return {
    loading,
    addRow,
    deleteSelected,
    exportData,
    handleSearch
  }
}
