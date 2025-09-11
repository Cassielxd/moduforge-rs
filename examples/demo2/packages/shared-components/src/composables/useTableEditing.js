/**
 * 表格编辑功能组合式函数
 * @description 提供表格单元格编辑、键盘导航、数据验证等功能
 */
import { ref, reactive, nextTick } from 'vue'

export function useTableEditing() {
  // 当前编辑的单元格信息
  const editingCell = ref(null)
  
  // 编辑状态
  const editingState = reactive({
    isEditing: false,
    record: null,
    column: null,
    originalValue: null,
    currentValue: null
  })

  // 编辑历史记录
  const editHistory = ref([])
  const historyIndex = ref(-1)

  // 打开单元格编辑器
  const openCellEditor = async (record, column) => {
    if (!record || !column) return false
    
    // 检查是否可编辑
    if (column.editable === false) return false
    
    // 如果是函数，执行检查
    if (typeof column.editable === 'function') {
      const canEdit = column.editable({ record, column })
      if (!canEdit) return false
    }
    
    // 关闭当前编辑器
    if (editingState.isEditing) {
      await closeCellEditor()
    }
    
    // 设置编辑状态
    editingState.isEditing = true
    editingState.record = record
    editingState.column = column
    editingState.originalValue = record[column.field || column.dataIndex]
    editingState.currentValue = editingState.originalValue
    
    editingCell.value = {
      record,
      column,
      field: column.field || column.dataIndex
    }
    
    // 等待DOM更新后聚焦
    await nextTick()
    focusEditor()
    
    return true
  }

  // 关闭单元格编辑器
  const closeCellEditor = async (save = true) => {
    if (!editingState.isEditing) return
    
    const { record, column, originalValue, currentValue } = editingState
    
    if (save && currentValue !== originalValue) {
      // 验证数据
      const isValid = await validateCellValue(currentValue, record, column)
      if (!isValid) return false
      
      // 保存数据
      const field = column.field || column.dataIndex
      record[field] = currentValue
      
      // 添加到历史记录
      addToHistory({
        type: 'edit',
        record,
        column,
        field,
        oldValue: originalValue,
        newValue: currentValue,
        timestamp: Date.now()
      })
      
      // 触发编辑事件
      if (column.onCellEdit) {
        column.onCellEdit({
          record,
          column,
          field,
          oldValue: originalValue,
          newValue: currentValue
        })
      }
    }
    
    // 重置编辑状态
    editingState.isEditing = false
    editingState.record = null
    editingState.column = null
    editingState.originalValue = null
    editingState.currentValue = null
    editingCell.value = null
    
    return true
  }

  // 取消编辑
  const cancelCellEdit = () => {
    editingState.currentValue = editingState.originalValue
    closeCellEditor(false)
  }

  // 验证单元格值
  const validateCellValue = async (value, record, column) => {
    // 必填验证
    if (column.required && (value === null || value === undefined || value === '')) {
      return false
    }
    
    // 类型验证
    if (column.dataType) {
      switch (column.dataType) {
        case 'number':
          if (isNaN(Number(value))) return false
          break
        case 'email':
          const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
          if (!emailRegex.test(value)) return false
          break
        case 'phone':
          const phoneRegex = /^1[3-9]\d{9}$/
          if (!phoneRegex.test(value)) return false
          break
      }
    }
    
    // 自定义验证器
    if (column.validator && typeof column.validator === 'function') {
      const result = await column.validator(value, record, column)
      if (!result) return false
    }
    
    return true
  }

  // 聚焦编辑器
  const focusEditor = () => {
    const { column } = editingState
    if (!column || !column.editRender) return
    
    const selector = column.editRender.autofocus || '.vxe-input--inner'
    
    nextTick(() => {
      const editor = document.querySelector(selector)
      if (editor) {
        editor.focus()
        
        // 如果是输入框，选中所有文本
        if (editor.select) {
          editor.select()
        }
      }
    })
  }

  // 处理键盘事件
  const handleKeydown = (event) => {
    if (!editingState.isEditing) return
    
    switch (event.code) {
      case 'Enter':
        event.preventDefault()
        closeCellEditor(true)
        break
      case 'Escape':
        event.preventDefault()
        cancelCellEdit()
        break
      case 'Tab':
        event.preventDefault()
        // 移动到下一个可编辑单元格
        moveToNextCell(event.shiftKey ? -1 : 1)
        break
    }
  }

  // 移动到下一个单元格
  const moveToNextCell = (direction = 1) => {
    // 这里需要根据表格结构实现
    // 暂时简单关闭当前编辑器
    closeCellEditor(true)
  }

  // 添加到历史记录
  const addToHistory = (action) => {
    // 清除当前位置之后的历史记录
    editHistory.value = editHistory.value.slice(0, historyIndex.value + 1)
    
    // 添加新的历史记录
    editHistory.value.push(action)
    historyIndex.value = editHistory.value.length - 1
    
    // 限制历史记录数量
    if (editHistory.value.length > 100) {
      editHistory.value.shift()
      historyIndex.value--
    }
  }

  // 撤销操作
  const undo = () => {
    if (historyIndex.value < 0) return false
    
    const action = editHistory.value[historyIndex.value]
    if (action.type === 'edit') {
      const { record, field, oldValue } = action
      record[field] = oldValue
    }
    
    historyIndex.value--
    return true
  }

  // 重做操作
  const redo = () => {
    if (historyIndex.value >= editHistory.value.length - 1) return false
    
    historyIndex.value++
    const action = editHistory.value[historyIndex.value]
    
    if (action.type === 'edit') {
      const { record, field, newValue } = action
      record[field] = newValue
    }
    
    return true
  }

  // 判断是否正在编辑
  const isCellEditing = (record, column) => {
    if (!editingState.isEditing) return false
    
    return editingState.record === record && 
           editingState.column === column
  }

  // 获取单元格编辑器类型
  const getCellEditorType = (column) => {
    if (!column.editable) return null
    
    if (column.editRender && column.editRender.name) {
      return column.editRender.name
    }
    
    // 根据数据类型推断编辑器类型
    switch (column.dataType) {
      case 'number':
        return 'input-number'
      case 'date':
        return 'date-picker'
      case 'select':
        return 'select'
      case 'textarea':
        return 'textarea'
      default:
        return 'input'
    }
  }

  // 清除历史记录
  const clearHistory = () => {
    editHistory.value = []
    historyIndex.value = -1
  }

  return {
    // 状态
    editingCell,
    editingState,
    editHistory,
    
    // 编辑方法
    openCellEditor,
    closeCellEditor,
    cancelCellEdit,
    
    // 验证方法
    validateCellValue,
    
    // 键盘导航
    handleKeydown,
    moveToNextCell,
    
    // 历史记录
    undo,
    redo,
    clearHistory,
    
    // 查询方法
    isCellEditing,
    getCellEditorType
  }
}
