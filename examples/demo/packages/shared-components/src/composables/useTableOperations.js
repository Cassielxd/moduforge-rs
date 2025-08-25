import { ref } from 'vue'
import { message, Modal } from 'ant-design-vue'

export function useTableOperations() {
  const loading = ref(false)

  const addRow = () => {
    // 添加新行的逻辑
    console.log('添加新行')
    message.success('已添加新行')
  }

  const deleteSelected = async () => {
    try {
      await new Promise((resolve, reject) => {
        Modal.confirm({
          title: '确认删除',
          content: '确定要删除选中的行吗？',
          okText: '确定',
          cancelText: '取消',
          onOk: resolve,
          onCancel: reject
        })
      })

      // 删除逻辑
      console.log('删除选中行')
      message.success('删除成功')
    } catch {
      message.info('已取消删除')
    }
  }

  const exportData = () => {
    // 导出数据逻辑
    console.log('导出数据')
    message.success('导出成功')
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
