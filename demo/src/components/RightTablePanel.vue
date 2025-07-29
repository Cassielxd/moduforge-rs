<script>
import { defineComponent, ref, reactive, onMounted, onUnmounted, nextTick, watch } from "vue";
import { ElMessage, ElMessageBox, ElDialog, ElButton, ElColorPicker, ElIcon } from "element-plus";
// @ts-ignore
import { TabulatorFull as Tabulator } from 'tabulator-tables';
import 'tabulator-tables/dist/css/tabulator.min.css';
import { Plus, Edit, Delete, CopyDocument, Brush, Folder, Document } from '@element-plus/icons-vue';

// --- Data Interfaces ---

export default defineComponent({
  name: "RightTablePanel",
  components: {
    ElDialog,
    ElButton,
    ElColorPicker,
    ElIcon,
    Plus,
    Edit,
    Delete,
    CopyDocument,
    Brush,
    Folder,
    Document,
  },
  props: {
    tableData: {
      type: Array,
      required: true,
    },
    tableColumns: {
      type: Array,
      required: true,
    },
    isTreeTable: {
      type: Boolean,
      default: false,
    },
  },
  emits: [
    "update:tableData",
    "add-row",
    "add-child",
    "edit-row",
    "delete-row",
    "copy-row",
  ],
  setup(props, { emit, expose }) {
    const localTableData = ref(props.tableData);
    const tableRef = ref();
    const tabulator = ref(null);

    const tableContextMenuVisible = ref(false);
    const tableContextMenuPosition = reactive({ x: 0, y: 0 });
    const currentTableItem = ref(null);
    const currentRowKey = ref(null);

    // 颜色选择相关
    const colorDialogVisible = ref(false);
    const colorValue = ref("#409EFF");

    // 添加编辑标志
    const isEditing = ref(false);

    // 初始化 Tabulator
    const initTabulator = () => {
      console.log("RightTablePanel: 开始初始化 Tabulator");
      console.log("tableRef.value:", tableRef.value);
      console.log("props.tableData:", props.tableData);
      console.log("props.tableColumns:", props.tableColumns);

      if (!tableRef.value) {
        console.error("RightTablePanel: tableRef.value 为空，无法初始化");
        return;
      }

      console.log("RightTablePanel: 容器尺寸:", {
        width: tableRef.value.offsetWidth,
        height: tableRef.value.offsetHeight,
        clientWidth: tableRef.value.clientWidth,
        clientHeight: tableRef.value.clientHeight
      });

      // 使用最简单的配置进行测试
      const simpleConfig = {
        data: props.tableData.length > 0 ? props.tableData : [
          { id: 1, name: "测试数据1", value: "值1" },
          { id: 2, name: "测试数据2", value: "值2" }
        ],
        columns: [
          { title: "ID", field: "id", width: 100 },
          { title: "名称", field: props.tableColumns[0]?.prop || "name", width: 200 },
          { title: "值", field: props.tableColumns[1]?.prop || "value", width: 200 }
        ],
        height: "100%",
        layout: "fitColumns"
      };

      console.log("RightTablePanel: 使用简化配置:", simpleConfig);

      try {
        tabulator.value = new Tabulator(tableRef.value, simpleConfig);
        console.log("RightTablePanel: Tabulator 实例创建成功:", tabulator.value);

        // 验证表格是否渲染成功
        setTimeout(() => {
          console.log("RightTablePanel: 延迟检查 - 表格行数:", tabulator.value?.getRows().length);
          console.log("RightTablePanel: 延迟检查 - 表格数据:", tabulator.value?.getData());
          console.log("RightTablePanel: 延迟检查 - 表格DOM:", tableRef.value?.innerHTML);
        }, 1000);

      } catch (error) {
        console.error("RightTablePanel: Tabulator 实例创建失败:", error);

        // 如果Tabulator创建失败，尝试手动创建一个简单的HTML表格作为fallback
        if (tableRef.value) {
          tableRef.value.innerHTML = `
            <div style="border: 1px solid #ccc; padding: 10px;">
              <h4>表格加载失败，使用简单HTML显示:</h4>
              <table style="width: 100%; border-collapse: collapse;">
                <thead>
                  <tr style="background: #f5f5f5;">
                    <th style="border: 1px solid #ccc; padding: 8px;">ID</th>
                    <th style="border: 1px solid #ccc; padding: 8px;">名称</th>
                    <th style="border: 1px solid #ccc; padding: 8px;">值</th>
                  </tr>
                </thead>
                <tbody>
                  ${props.tableData.map(item => `
                    <tr>
                      <td style="border: 1px solid #ccc; padding: 8px;">${item.id}</td>
                      <td style="border: 1px solid #ccc; padding: 8px;">${item[props.tableColumns[0]?.prop] || '无数据'}</td>
                      <td style="border: 1px solid #ccc; padding: 8px;">${item[props.tableColumns[1]?.prop] || '无数据'}</td>
                    </tr>
                  `).join('')}
                </tbody>
              </table>
            </div>
          `;
        }
        return;
      }

      // 监听行选择 - 只有在Tabulator成功创建后才添加监听器
      if (tabulator.value) {
        tabulator.value.on("rowClick", (e, row) => {
          currentRowKey.value = row.getData().id;

          // 清除所有行的选中状态
          tabulator.value?.getRows().forEach((r) => {
            r.getElement().classList.remove('row-selected');
          });

          // 为当前选中行添加选中样式
          row.getElement().classList.add('row-selected');
        });
      }
    };

    const handleTableContextMenu = (row, column, event) => {
      event.preventDefault();
      currentTableItem.value = row;
      tableContextMenuPosition.x = event.clientX;
      tableContextMenuPosition.y = event.clientY;
      tableContextMenuVisible.value = true;
      document.addEventListener("click", closeTableContextMenu);
    };

    const closeTableContextMenu = () => {
      tableContextMenuVisible.value = false;
      document.removeEventListener("click", closeTableContextMenu);
    };

    const handleTableCommand = (command) => {
      if (!currentTableItem.value) return;

      switch (command) {
        case "add-row":
          emit("add-row", currentTableItem.value);
          break;
        case "edit":
          emit("edit-row", currentTableItem.value);
          break;
        case "delete":
          emit("delete-row", currentTableItem.value);
          break;
        case "copy":
          emit("copy-row", currentTableItem.value);
          break;
        case "add-child":
          emit("add-child", currentTableItem.value);
          break;
      }
      tableContextMenuVisible.value = false;
    };

    const handleAddTableRow = () => {
      emit("add-row");
    };

    const openColorDialog = () => {
      colorValue.value = currentTableItem.value?.color || "#409EFF";
      colorDialogVisible.value = true;
      tableContextMenuVisible.value = false;
    };

    const handleColorSubmit = () => {
      if (!colorValue.value) {
        ElMessage.warning("请选择颜色");
        return;
      }
      if (currentTableItem.value && tabulator.value) {
        // 更新 Tabulator 中的数据
        const rows = tabulator.value.getRows();
        const targetRow = rows.find((row) => row.getData().id === currentTableItem.value.id);
        if (targetRow) {
          targetRow.update({ ...targetRow.getData(), color: colorValue.value });
          localTableData.value = tabulator.value.getData();
          emit("update:tableData", localTableData.value);
        }
      }
      colorDialogVisible.value = false;
    };

    // 设置当前选中行的方法，供父组件调用
    const setCurrentRow = (rowId) => {
      currentRowKey.value = rowId;
      if (tabulator.value) {
        // 使用 nextTick 确保数据已更新并渲染完成
        nextTick(() => {
          // 添加一个小延迟，确保数据监听器完成更新
          setTimeout(() => {
            if (!tabulator.value) return;

            const rows = tabulator.value.getRows();

            // 清除所有行的选中状态
            rows.forEach((r) => {
              r.getElement().classList.remove('row-selected');
            });

            const targetRow = rows.find((row) => row.getData().id === rowId);
            if (targetRow) {
              targetRow.select();
              // 添加自定义选中样式
              targetRow.getElement().classList.add('row-selected');

              // 如果是树形表格，确保目标行可见（滚动到视图中）
              if (props.isTreeTable) {
                try {
                  targetRow.getElement().scrollIntoView({
                    behavior: 'smooth',
                    block: 'nearest'
                  });
                } catch (error) {
                  console.warn('滚动到目标行失败:', error);
                }
              }
            } else {
              console.log('未找到要选中的行:', rowId);
              console.log('可用的行ID:', rows.map((r) => r.getData().id));
            }
          }, 100); // 100ms 延迟确保数据更新完成
        });
      }
    };

    // 简化：移除展开功能，只保留基本的树形表格显示

    // 更新数据的方法
    const updateData = (newData) => {
      const previousSelectedId = currentRowKey.value;
      localTableData.value = newData;
      if (tabulator.value) {
        tabulator.value.setData(newData);

        // 数据更新后恢复选中状态
        if (previousSelectedId) {
          nextTick(() => {
            setCurrentRow(previousSelectedId);
          });
        }
      }
    };

    // 刷新表格数据（强制重新渲染）
    const refreshTable = () => {
      if (tabulator.value) {
        tabulator.value.setData(localTableData.value);
      }
    };

    onMounted(() => {
      nextTick(() => {
        initTabulator();
      });
    });

    onUnmounted(() => {
      if (tabulator.value) {
        tabulator.value.destroy();
      }
    });

    // 监听数据变化
    watch(() => props.tableData, (newData) => {
      console.log('props.tableData 变化:', newData);
      localTableData.value = newData;
      if (tabulator.value) {
        // 检查是否是编辑操作导致的数据变化
        const currentData = tabulator.value.getData();
        const isEditOperation = isEditing.value || (currentData.length === newData.length);

        if (isEditOperation) {
          // 如果是编辑操作，只更新数据，不重建表格
          // 保存当前的展开状态
          const expandedRows = tabulator.value.getRows()
            .filter((row) => {
              // 安全检查：确保isTreeExpanded方法存在且为树形表格
              return props.isTreeTable &&
                typeof (row ).isTreeExpanded === 'function' &&
                (row ).isTreeExpanded();
            })
            .map((row) => row.getData().id);

          // 更新数据
          tabulator.value.setData(newData);

          // 恢复展开状态
          nextTick(() => {
            if (tabulator.value && props.isTreeTable) {
              expandedRows.forEach((id) => {
                const row = tabulator.value.getRows().find((r) => r.getData().id === id);
                if (row && row.getData().children && row.getData().children.length > 0) {
                  // 安全检查：确保treeExpand方法存在
                  if (typeof (row ).treeExpand === 'function') {
                    (row ).treeExpand();
                  }
                }
              });
            }
          });
        } else {
          // 如果是结构变化（添加/删除行），才重建表格
          tabulator.value.setData([]);
          nextTick(() => {
            if (tabulator.value) {
              tabulator.value.setData(newData);
            }
          });
        }
      }
    }, { deep: true, immediate: true });

    // 清除选中状态的方法
    const clearSelection = () => {
      currentRowKey.value = null;
      if (tabulator.value) {
        tabulator.value.getRows().forEach((r) => {
          r.getElement().classList.remove('row-selected');
        });
        tabulator.value.deselectRow();
      }
    };

    // ========== 新增方法 ==========

    // 获取当前选中行数据
    const getCurrentRow = () => {
      if (!tabulator.value || !currentRowKey.value) return null;
      const rows = tabulator.value.getRows();
      return rows.find((row) => row.getData().id === currentRowKey.value)?.getData() || null;
    };

    // 获取所有行数据
    const getAllData = () => {
      return tabulator.value?.getData() || [];
    };

    // 根据ID获取行数据
    const getRowById = (id) => {
      if (!tabulator.value) return null;
      const rows = tabulator.value.getRows();
      return rows.find((row) => row.getData().id === id)?.getData() || null;
    };

    // 根据条件查找行
    const findRows = (predicate) => {
      if (!tabulator.value) return [];
      const rows = tabulator.value.getRows();
      return rows
        .map((row) => row.getData())
        .filter(predicate);
    };

    // 更新指定行的数据
    const updateRow = (id, newData) => {
      if (!tabulator.value) return false;
      const rows = tabulator.value.getRows();
      const targetRow = rows.find((row) => row.getData().id === id);
      if (targetRow) {
        targetRow.update({ ...targetRow.getData(), ...newData });
        localTableData.value = tabulator.value.getData();
        emit("update:tableData", localTableData.value);
        return true;
      }
      return false;
    };

    // 删除指定行
    const deleteRow = (id) => {
      if (!tabulator.value) return false;
      const rows = tabulator.value.getRows();
      const targetRow = rows.find((row) => row.getData().id === id);
      if (targetRow) {
        // 使用类型断言来访问 delete 方法
        (targetRow ).delete();
        localTableData.value = tabulator.value.getData();
        emit("update:tableData", localTableData.value);
        return true;
      }
      return false;
    };

    // 添加新行
    const addRow = (rowData) => {
      if (!tabulator.value) return false;
      // 使用类型断言来访问 addRow 方法
      (tabulator.value ).addRow(rowData);
      localTableData.value = tabulator.value.getData();
      emit("update:tableData", localTableData.value);
      return true;
    };

    // 树形表格相关方法
    const expandAll = () => {
      if (!tabulator.value || !props.isTreeTable) return;
      const rows = tabulator.value.getRows();
      rows.forEach((row) => {
        if (row.getData().children && row.getData().children.length > 0) {
          // 安全检查：确保treeExpand方法存在
          if (typeof (row ).treeExpand === 'function') {
            (row ).treeExpand();
          }
        }
      });
    };

    const collapseAll = () => {
      if (!tabulator.value || !props.isTreeTable) return;
      const rows = tabulator.value.getRows();
      rows.forEach((row) => {
        // 安全检查：确保isTreeExpanded和treeCollapse方法存在
        if (typeof (row ).isTreeExpanded === 'function' &&
          typeof (row ).treeCollapse === 'function' &&
          (row ).isTreeExpanded()) {
          (row ).treeCollapse();
        }
      });
    };

    const expandRow = (id) => {
      if (!tabulator.value || !props.isTreeTable) return false;
      const rows = tabulator.value.getRows();
      const targetRow = rows.find((row) => row.getData().id === id);
      if (targetRow && targetRow.getData().children && targetRow.getData().children.length > 0) {
        // 安全检查：确保treeExpand方法存在
        if (typeof (targetRow ).treeExpand === 'function') {
          (targetRow ).treeExpand();
          return true;
        }
      }
      return false;
    };

    const collapseRow = (id) => {
      if (!tabulator.value || !props.isTreeTable) return false;
      const rows = tabulator.value.getRows();
      const targetRow = rows.find((row) => row.getData().id === id);
      if (targetRow &&
        typeof (targetRow ).isTreeExpanded === 'function' &&
        typeof (targetRow ).treeCollapse === 'function' &&
        (targetRow ).isTreeExpanded()) {
        (targetRow ).treeCollapse();
        return true;
      }
      return false;
    };

    const isRowExpanded = (id) => {
      if (!tabulator.value || !props.isTreeTable) return false;
      const rows = tabulator.value.getRows();
      const targetRow = rows.find((row) => row.getData().id === id);
      return targetRow ? targetRow.isTreeExpanded() : false;
    };

    // 获取展开的行ID列表
    const getExpandedRows = () => {
      if (!tabulator.value || !props.isTreeTable) return [];
      const rows = tabulator.value.getRows();
      return rows.filter((row) => row.isTreeExpanded()).map((row) => row.getData().id);
    };

    // 设置展开状态
    const setExpandedRows = (ids) => {
      if (!tabulator.value || !props.isTreeTable) return;
      const rows = tabulator.value.getRows();

      // 先收起所有行
      rows.forEach((row) => {
        if (row.isTreeExpanded()) {
          row.treeCollapse();
        }
      });

      // 展开指定的行
      ids.forEach((id) => {
        const targetRow = rows.find((row) => row.getData().id === id);
        if (targetRow && targetRow.getData().children && targetRow.getData().children.length > 0) {
          targetRow.treeExpand();
        }
      });
    };

    // 排序相关方法
    const sortBy = (field, dir) => {
      if (!tabulator.value) return;
      (tabulator.value ).setSort(field, dir);
    };

    const clearSort = () => {
      if (!tabulator.value) return;
      (tabulator.value ).clearSort();
    };

    // 过滤相关方法
    const setFilter = (field, type, value) => {
      if (!tabulator.value) return;
      (tabulator.value ).setFilter(field, type, value);
    };

    const clearFilter = () => {
      if (!tabulator.value) return;
      (tabulator.value ).clearFilter();
    };

    // 分页相关方法
    const setPage = (page) => {
      if (!tabulator.value) return;
      (tabulator.value ).setPage(page);
    };

    const getCurrentPage = () => {
      if (!tabulator.value) return 1;
      return (tabulator.value ).getPage();
    };

    const getPageSize = () => {
      if (!tabulator.value) return 10;
      return (tabulator.value ).getPageSize();
    };

    const setPageSize = (size) => {
      if (!tabulator.value) return;
      (tabulator.value ).setPageSize(size);
    };

    // 选择相关方法
    const selectRow = (id) => {
      if (!tabulator.value) return false;
      const rows = tabulator.value.getRows();
      const targetRow = rows.find((row) => row.getData().id === id);
      if (targetRow) {
        targetRow.select();
        return true;
      }
      return false;
    };

    const deselectRow = (id) => {
      if (!tabulator.value) return false;
      const rows = tabulator.value.getRows();
      const targetRow = rows.find((row) => row.getData().id === id);
      if (targetRow) {
        (targetRow ).deselect();
        return true;
      }
      return false;
    };

    const getSelectedRows = () => {
      if (!tabulator.value) return [];
      return (tabulator.value ).getSelectedRows().map((row) => row.getData());
    };

    const selectAll = () => {
      if (!tabulator.value) return;
      (tabulator.value ).selectRow();
    };

    const deselectAll = () => {
      if (!tabulator.value) return;
      tabulator.value.deselectRow();
    };

    // 滚动相关方法
    const scrollToRow = (id, position) => {
      if (!tabulator.value) return false;
      const rows = tabulator.value.getRows();
      const targetRow = rows.find((row) => row.getData().id === id);
      if (targetRow) {
        targetRow.getElement().scrollIntoView({
          behavior: 'smooth',
          block: position
        });
        return true;
      }
      return false;
    };

    const scrollToTop = () => {
      if (!tabulator.value) return;
      const tableElement = (tabulator.value ).getElement();
      if (tableElement) {
        tableElement.scrollTop = 0;
      }
    };

    const scrollToBottom = () => {
      if (!tabulator.value) return;
      const tableElement = (tabulator.value ).getElement();
      if (tableElement) {
        tableElement.scrollTop = tableElement.scrollHeight;
      }
    };

    // 导出相关方法
    const exportToCSV = (filename) => {
      if (!tabulator.value) return;
      (tabulator.value ).download('csv', filename || 'table-data.csv');
    };

    const exportToJSON = (filename) => {
      if (!tabulator.value) return;
      (tabulator.value ).download('json', filename || 'table-data.json');
    };

    const exportToPDF = (filename) => {
      if (!tabulator.value) return;
      (tabulator.value ).download('pdf', filename || 'table-data.pdf');
    };

    // 表格状态方法
    const getTableState = () => {
      if (!tabulator.value) return null;
      return {
        currentPage: (tabulator.value ).getPage(),
        pageSize: (tabulator.value ).getPageSize(),
        sort: (tabulator.value ).getSorters(),
        filter: (tabulator.value ).getFilters(),
        selectedRows: (tabulator.value ).getSelectedRows().map((row) => row.getData().id),
        expandedRows: props.isTreeTable ? getExpandedRows() : [],
        currentRow: currentRowKey.value
      };
    };

    const setTableState = (state) => {
      if (!tabulator.value) return;

      if (state.sort) {
        (tabulator.value ).setSort(state.sort);
      }

      if (state.filter) {
        (tabulator.value ).setFilter(state.filter);
      }

      if (state.pageSize) {
        (tabulator.value ).setPageSize(state.pageSize);
      }

      if (state.currentPage) {
        (tabulator.value ).setPage(state.currentPage);
      }

      if (state.expandedRows && props.isTreeTable) {
        setExpandedRows(state.expandedRows);
      }

      if (state.currentRow) {
        setCurrentRow(state.currentRow);
      }
    };

    // 重新初始化表格
    const reinitializeTable = () => {
      if (tabulator.value) {
        tabulator.value.destroy();
      }
      nextTick(() => {
        initTabulator();
      });
    };

    // 获取表格统计信息
    const getTableStats = () => {
      if (!tabulator.value) return null;
      const rows = tabulator.value.getRows();
      return {
        totalRows: rows.length,
        visibleRows: rows.filter((row) => (row ).isVisible()).length,
        selectedRows: (tabulator.value ).getSelectedRows().length,
        expandedRows: props.isTreeTable ? getExpandedRows().length : 0,
        currentPage: (tabulator.value ).getPage(),
        totalPages: (tabulator.value ).getPageMax()
      };
    };

    // 暴露方法给父组件
    expose({
      setCurrentRow,
      updateData,
      clearSelection,
      refreshTable,
      getCurrentRow,
      getAllData,
      getRowById,
      findRows,
      updateRow,
      deleteRow,
      addRow,
      expandAll,
      collapseAll,
      expandRow,
      collapseRow,
      isRowExpanded,
      getExpandedRows,
      setExpandedRows,
      sortBy,
      clearSort,
      setFilter,
      clearFilter,
      setPage,
      getCurrentPage,
      getPageSize,
      setPageSize,
      selectRow,
      deselectRow,
      getSelectedRows,
      selectAll,
      deselectAll,
      scrollToRow,
      scrollToTop,
      scrollToBottom,
      exportToCSV,
      exportToJSON,
      exportToPDF,
      getTableState,
      setTableState,
      reinitializeTable,
      getTableStats,
    });

    return {
      localTableData,
      tableRef,
      tabulator,
      tableContextMenuVisible,
      tableContextMenuPosition,
      handleTableContextMenu,
      handleTableCommand,
      closeTableContextMenu,
      handleAddTableRow,
      colorDialogVisible,
      colorValue,
      openColorDialog,
      handleColorSubmit,
      currentRowKey,
      setCurrentRow,
      clearSelection,
      refreshTable,
    };
  },
});
</script>

<template>
  <div class="table-panel-container">
    <el-dialog v-model="colorDialogVisible" title="设置边框颜色" width="300px">
      <el-color-picker v-model="colorValue" />
      <template #footer>
        <el-button @click="colorDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleColorSubmit">确定</el-button>
      </template>
    </el-dialog>

    <!-- Tabulator 表格容器 -->
    <div ref="tableRef" class="tabulator-table"></div>
  </div>
</template>

<style scoped>
.table-panel-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.tabulator-table {
  flex: 1;
  height: 100%;
  min-height: 300px;
  /* 确保最小高度 */
}

/* Tabulator 自定义样式 */
:deep(.tabulator) {
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  background: #fff;
}

:deep(.tabulator .tabulator-header) {
  background: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
}

:deep(.tabulator .tabulator-header .tabulator-col) {
  background: #f5f7fa;
  border-right: 1px solid #e4e7ed;
  font-weight: 600;
  color: #606266;
}

:deep(.tabulator .tabulator-row) {
  border-bottom: 1px solid #e4e7ed;
  position: relative;
}

:deep(.tabulator .tabulator-row .tabulator-cell) {
  border-right: 1px solid #e4e7ed;
  padding: 8px 12px;
}

/* 有颜色边框的行，调整单元格边框 */
:deep(.tabulator .tabulator-row.colored-border .tabulator-cell) {
  border-right: none !important;
}

:deep(.tabulator .tabulator-row:hover) {
  background-color: #f5f7fa;
}

:deep(.tabulator .tabulator-row.tabulator-selected) {
  background-color: #ecf5ff;
}

/* 自定义行选中样式 */
:deep(.tabulator .tabulator-row.row-selected) {
  background-color: #e6f7ff !important;
  border-left: 1px solid #1890ff !important;
  position: relative;
}

/* 选中行悬停效果 */
:deep(.tabulator .tabulator-row.row-selected:hover) {
  background-color: #bae7ff !important;
}

/* 选中行的单元格样式 */
:deep(.tabulator .tabulator-row.row-selected .tabulator-cell) {
  border-color: #91d5ff;
  font-weight: 500;
}

/* 确保有颜色边框的选中行样式正确叠加 */
:deep(.tabulator .tabulator-row.row-selected.colored-border) {
  box-shadow: inset 0 0 0 2px #1890ff, 0 0 0 2px currentColor;
}

/* 树形表格样式 */
.tree-name-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.folder-icon {
  color: #e6a23c;
  font-size: 16px;
}

.file-icon {
  color: #909399;
  font-size: 16px;
}

.type-tag {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.folder-tag {
  background: #fdf6ec;
  color: #e6a23c;
  border: 1px solid #f5dab1;
}

.file-tag {
  background: #f4f4f5;
  color: #909399;
  border: 1px solid #d3d4d6;
}

/* 树形控制列样式 */
:deep(.tabulator .tabulator-cell[data-field="tree_control"]) {
  padding: 4px 8px;
  text-align: center;
}

:deep(.tabulator .tabulator-cell[data-field="tree_control"] .tabulator-tree-control) {
  display: inline-block;
  width: 16px;
  height: 16px;
  line-height: 16px;
  text-align: center;
  cursor: pointer;
  border-radius: 2px;
  transition: background-color 0.2s;
}

:deep(.tabulator .tabulator-cell[data-field="tree_control"] .tabulator-tree-control:hover) {
  background-color: #f0f0f0;
}

:deep(.tabulator .tabulator-cell[data-field="tree_control"] .tabulator-tree-control .tabulator-tree-control-expand) {
  color: #409eff;
  font-weight: bold;
}

:deep(.tabulator .tabulator-cell[data-field="tree_control"] .tabulator-tree-control .tabulator-tree-control-collapse) {
  color: #409eff;
  font-weight: bold;
}

/* 右键菜单样式覆盖 */
:deep(.tabulator-menu) {
  background: #fff;
  border: 1px solid #e4e7ed;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
  border-radius: 4px;
  padding: 4px 0;
  font-size: 14px;
}

:deep(.tabulator-menu .tabulator-menu-item) {
  padding: 8px 16px;
  cursor: pointer;
  transition: background 0.2s;
}

:deep(.tabulator-menu .tabulator-menu-item:hover) {
  background-color: #f5f7fa;
}

/* 有边框颜色的行样式 */
:deep(.tabulator .tabulator-row.colored-border) {
  border-width: 2px !important;
  border-style: solid !important;
  margin: 1px 0 !important;
  /* 防止边框重叠 */
}

/* 确保边框颜色在悬停时也保持 */
:deep(.tabulator .tabulator-row.colored-border:hover) {
  border-width: 2px !important;
  border-style: solid !important;
}
</style>
