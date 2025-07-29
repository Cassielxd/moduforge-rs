// 业务操作组合式函数
import { ref, nextTick } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";

export function useFbfxActions(
  tableTreeData,
  mainTabulatorComposable
) {
  const currentTableItem = ref(null);
  const currentRowKey = ref(null);

  // 查找行数据的通用函数
  const findRowById = (
    data,
    id
  ) => {
    for (const item of data) {
      if (String(item.id) === String(id)) {
        return item;
      }
      if (item.children) {
        const found = findRowById(item.children, id);
        if (found) return found;
      }
    }
    return null;
  };

  // 获取目标行的通用函数（优先使用传入的row，否则使用当前选中的行）
  const getTargetRow = (row) => {
    if (row) return row;
    if (currentRowKey.value) {
      return findRowById(tableTreeData.value, currentRowKey.value);
    }
    return null;
  };

  // 颜色变化处理函数
  const handleColorChange = (id, color) => {
    // 在原始数据中更新颜色
    const updateColor = (items) => {
      for (const item of items) {
        if (String(item.id) === id) {
          item.color = color;
          return;
        }
        if (item.children) {
          updateColor(item.children);
        }
      }
    };

    updateColor(tableTreeData.value);
    ElMessage.success(`已设置颜色：${color}`);
  };

  // 行点击处理函数
  const handleRowClick = (data) => {
    currentRowKey.value = data.id;
    console.log("选中行:", data);
  };

  // 添加行
  const handleAddRow = (currentRow) => {
    const newRow = {
      id: Date.now(),
      name: "新文件",
      type: "file",
      size: "0KB",
      date: new Date().toISOString().split("T")[0],
    };

    // 如果没有传入currentRow，尝试获取当前选中的行
    if (!currentRow) {
      const foundRow = getTargetRow();
      if (foundRow) {
        currentRow = foundRow;
      }
    }

    if (currentRow) {
      // 在指定行的下一行插入新行
      const insertAfter = (
        data,
        targetId
      ) => {
        for (let i = 0; i < data.length; i++) {
          if (data[i].id === targetId) {
            // 在当前位置的下一行插入
            data.splice(i + 1, 0, newRow);
            return true;
          }
          // 如果有子节点，递归查找
          if (data[i].children && insertAfter(data[i].children, targetId)) {
            return true;
          }
        }
        return false;
      };

      if (!insertAfter(tableTreeData.value, currentRow.id)) {
        // 如果没有找到指定行，就添加到末尾
        tableTreeData.value.push(newRow);
      }
    } else {
      // 没有指定行时，添加到末尾
      tableTreeData.value.push(newRow);
    }

    // 选中新添加的行
    nextTick(() => {
      setCurrentRow(newRow.id);
    });
  };

  // 添加子项
  const handleAddChild = (parentRow) => {
    const newChild = {
      id: Date.now(),
      name: "新子项",
      type: "file",
      size: "0KB",
      date: new Date().toISOString().split("T")[0],
    };

    console.log("添加子项到父行:", parentRow);

    if (!parentRow.children) {
      parentRow.children = [];
    }
    parentRow.children.push(newChild);

    ElMessage.success("添加子项成功");
  };

  // 编辑行
  const handleEditRow = (row) => {
    const targetRow = getTargetRow(row);

    if (targetRow) {
      ElMessage.info(`选中编辑行: ${targetRow.name}，双击单元格进行编辑`);
    } else {
      ElMessage.info("请先选择要编辑的行，然后双击单元格进行编辑");
    }
  };

  // 删除行
  const handleDeleteRow = (row) => {
    const targetRow = getTargetRow(row);

    if (!targetRow) {
      ElMessage.warning("请先选择要删除的行");
      return;
    }
    // 递归查找并删除节点
    const deleteNode = (
      data,
      targetId
    ) => {
      for (let i = 0; i < data.length; i++) {
        // 使用字符串比较确保匹配
        if (String(data[i].id) === String(targetId)) {
          data.splice(i, 1);
          return true;
        }
        if (data[i].children && deleteNode(data[i].children, targetId)) {
          return true;
        }
      }
      return false;
    };

    if (deleteNode(tableTreeData.value, targetRow.id)) {
      ElMessage.success("删除成功");
      // 清空当前选中行
      if (String(currentRowKey.value) === String(targetRow.id)) {
        currentRowKey.value = null;
      }
    } else {
      ElMessage.error("删除失败，未找到指定行");
    }
  };

  // 复制行
  const handleCopyRow = (row) => {
    const targetRow = getTargetRow(row);

    if (!targetRow) {
      ElMessage.warning("请先选择要复制的行");
      return;
    }

    const newItem = {
      ...targetRow,
      id: Date.now(),
      name: `${targetRow.name} (复制)`,
      children: undefined, // 复制时不包含子项
    };
    tableTreeData.value.push(newItem);

    ElMessage.success(`复制成功: ${targetRow.name}`);

    // 选中复制的新行
    nextTick(() => {
      setCurrentRow(newItem.id);
    });
  };

  // 设置当前行
  const setCurrentRow = (rowId) => {
    currentRowKey.value = rowId;
    // 使用组合式函数选择行
    mainTabulatorComposable.selectRow(String(rowId));
  };

  // 子表格事件处理
  const handleSubTableAddRow = () => {
    ElMessage.info("子表格新增功能");
  };

  const handleSubTableEditRow = (row) => {
    ElMessage.info("子表格编辑功能，双击单元格进行编辑");
  };

  const handleSubTableDeleteRow = (row) => {
    ElMessage.info("子表格删除功能");
  };

  const handleSubTableCopyRow = (row) => {
    ElMessage.info("子表格复制功能");
  };

  return {
    // 状态
    currentTableItem,
    currentRowKey,
    // 主表格操作
    handleColorChange,
    handleRowClick,
    handleAddRow,
    handleAddChild,
    handleEditRow,
    handleDeleteRow,
    handleCopyRow,
    setCurrentRow,
    // 子表格操作
    handleSubTableAddRow,
    handleSubTableEditRow,
    handleSubTableDeleteRow,
    handleSubTableCopyRow,
  };
}
