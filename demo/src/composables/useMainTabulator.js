// 主表格相关的组合式函数
import { ref, nextTick } from "vue";
import { TabulatorFull as Tabulator } from "tabulator-tables";
import "tabulator-tables/dist/css/tabulator.min.css";
import { ElMessage } from "element-plus";


export function useMainTabulator() {
  const mainTabulator = ref(null);
  const isEditing = ref(false);
  const tableState = ref({
    expandedRows: new Set(),
    selectedRows: new Set(),
    scrollPosition: 0,
    filters: [],
    sorting: [],
  });

  // 事件处理函数
  const eventHandlers = {
    onAddRow: () => {
      ElMessage.info("主表格新增功能");
    },
    onAddChildRow: (row) => {
      ElMessage.info(
        "添加子项功能 - 请在外部组件中通过 setEventHandlers 自定义实现"
      );
    },
    onEditRow: (row) => {
      ElMessage.info("主表格编辑功能，可以双击单元格编辑");
    },
    onDeleteRow: (row) => {
      ElMessage.info("主表格删除功能");
    },
    onCopyRow: (row) => {
      ElMessage.info("主表格复制功能");
    },
    onCellEdited: (cell) => {
      ElMessage.info("主表格编辑功能");
    },
  };

  // 设置事件处理器
  const setEventHandlers = (handlers) => {
    Object.assign(eventHandlers, handlers);
  };

  // 保存表格状态
  const saveTableState = () => {
    if (!mainTabulator.value) return;

    const expandedRows = mainTabulator.value
      .getRows()
      .filter((row) => row.isTreeExpanded());
    tableState.value.expandedRows = new Set(
      expandedRows.map((row) => (row.getData() ).id)
    );

    const selectedRows = mainTabulator.value.getSelectedRows();
    tableState.value.selectedRows = new Set(
      selectedRows.map((row) => (row.getData() ).id)
    );

    tableState.value.scrollPosition = mainTabulator.value.getScrollPosition();
  };

  // 恢复表格状态
  const restoreTableState = async () => {
    if (!mainTabulator.value) return;

    await nextTick();

    // 恢复展开状态
    tableState.value.expandedRows.forEach((id) => {
      const row = mainTabulator.value.getRow(id);
      if (row) {
        row.treeExpand();
      }
    });

    // 恢复选中状态
    tableState.value.selectedRows.forEach((id) => {
      const row = mainTabulator.value.getRow(id);
      if (row) {
        row.select();
      }
    });

    // 恢复滚动位置
    if (tableState.value.scrollPosition > 0) {
      mainTabulator.value.scrollToPosition(
        0,
        tableState.value.scrollPosition,
        false
      );
    }
  };

  // 行格式化器 - 添加颜色边框
  const rowFormatter = (row) => {
    const data = row.getData() ;
    if (data._row_color) {
      const element = row.getElement();
      element.style.borderLeft = `4px solid ${data._row_color}`;
    }
  };

  // 定义表格列
  const getColumns = (
    onColorChange
  ) => [
    {
      title: "名称",
      field: "name",
      width: 250,
      editor: "input",
      formatter: "tree",
      headerSort: false,
    },
    { title: "类型", field: "type", width: 150, editor: "input" },
    { title: "子类型", field: "subType", width: 150, editor: "input" },
    { title: "描述", field: "description", width: 300, editor: "input" },
    {
      title: "颜色",
      field: "_row_color",
      width: 100,
      formatter: (cell) => {
        const value = cell.getValue();
        if (value) {
          return `<div style="width: 20px; height: 20px; background-color: ${value}; border: 1px solid #ccc; border-radius: 3px; display: inline-block;"></div>`;
        }
        return '<span style="color: #999;">无颜色</span>';
      },
      cellClick: (e, cell) => {
        const data = cell.getData() ;
        const input = document.createElement("input");
        input.type = "color";
        input.value = data._row_color || "#000000";
        input.style.position = "absolute";
        input.style.left = "-9999px";
        document.body.appendChild(input);

        input.addEventListener("change", () => {
          onColorChange(data.id, input.value);
          document.body.removeChild(input);
        });

        input.click();
      },
    },
  ];

  // 初始化主表格
  const initMainTabulator = (
    element,
    data,
    onColorChange,
    onRowClick
  ) => {
    if (mainTabulator.value) {
      mainTabulator.value.destroy();
    }

    console.log("Initializing Tabulator with data:", data);
    console.log("Element:", element);
    console.log(
      "Element dimensions:",
      element.offsetWidth,
      element.offsetHeight
    );

    mainTabulator.value = new Tabulator(element, {
      data: data,
      height: "100%", // 恢复使用百分比高度
      layout: "fitColumns",
      columns: getColumns(onColorChange),
      dataTree: true,
      dataTreeChildField: "children",
      dataTreeStartExpanded: true,
      dataTreeBranchElement: true,
      selectableRange:true,
      selectableRangeColumns:true,
      selectableRangeRows:true,
      dataTreeChildIndent: 20,
      selectable: true,
      rowFormatter: rowFormatter,
      editTriggerEvent: "dblclick", // 设置双击编辑
      cellEdited: (cell) => {
        isEditing.value = true;
        console.log("Cell edited:", cell.getData());
        eventHandlers.onCellEdited(cell);
      },
      rowClick: (e, row) => {
        const data = row.getData() ;
        if (onRowClick) {
          onRowClick(data);
        }
      },
      rowDblClick: (e, row) => {
        console.log("双击行:", row.getData());
        // 双击行时可以触发编辑事件
        eventHandlers.onEditRow(row.getData());
      },
      rowContextMenu: [
        {
          label: "添加行",
          action: () => {
            console.log("右键菜单 - 添加行被点击");
            eventHandlers.onAddRow();
          },
        },
        {
          label: "添加子项",
          action: (e, row) => {
            console.log("右键菜单 - 添加子项被点击");
            eventHandlers.onAddChildRow(row.getData());
          },
        },
        {
          label: "编辑",
          action: (e, row) => {
            console.log("右键菜单 - 编辑被点击，行数据:", row.getData());
            eventHandlers.onEditRow(row.getData());
          },
        },
        {
          label: "删除",
          action: (e, row) => {
            console.log("右键菜单 - 删除被点击，行数据:", row.getData());
            eventHandlers.onDeleteRow(row.getData());
          },
        },
        {
          label: "复制",
          action: (e, row) => {
            console.log("右键菜单 - 复制被点击，行数据:", row.getData());
            eventHandlers.onCopyRow(row.getData());
          },
        },
      ],
    });

    console.log("Tabulator instance created:", mainTabulator.value);
  };

  // 更新数据
  const updateData = async (data) => {
    if (!mainTabulator.value) return;

    if (isEditing.value) {
      // 如果是编辑操作，保存状态
      saveTableState();
      isEditing.value = false;
    }

    mainTabulator.value.setData(data);

    // 如果有保存的状态，恢复它
    if (tableState.value.expandedRows.size > 0) {
      await restoreTableState();
    }
  };

  // 导出的方法
  const exportedMethods = {
    // 数据查询方法
    getCurrentRow: () => {
      if (!mainTabulator.value) return null;
      const selectedRows = mainTabulator.value.getSelectedRows();
      return selectedRows.length > 0
        ? (selectedRows[0].getData() )
        : null;
    },

    getAllData: () => {
      if (!mainTabulator.value) return [];
      return mainTabulator.value.getData();
    },

    getRowById: (id) => {
      if (!mainTabulator.value) return null;
      try {
        const row = mainTabulator.value.getRow(id);
        return row ? (row.getData() ) : null;
      } catch {
        return null;
      }
    },

    findRows: (criteria) => {
      if (!mainTabulator.value) return [];
      return mainTabulator.value
        .searchRows(criteria)
        .map((row) => row.getData() );
    },

    // 数据操作方法
    updateRow: (id, data) => {
      if (!mainTabulator.value) return false;
      try {
        const row = mainTabulator.value.getRow(id);
        if (row) {
          row.update(data);
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },

    deleteRow: (id) => {
      if (!mainTabulator.value) return false;
      try {
        const row = mainTabulator.value.getRow(id);
        if (row) {
          row.delete();
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },

    addRow: (
      data,
      position,
      parentId
    ) => {
      if (!mainTabulator.value) return false;
      try {
        if (parentId) {
          const parentRow = mainTabulator.value.getRow(parentId);
          if (parentRow) {
            parentRow.addTreeChild(data);
            return true;
          }
        } else {
          mainTabulator.value.addRow(data, position === "top");
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },

    // 树形操作方法
    expandAll: () => {
      if (!mainTabulator.value) return;
      mainTabulator.value.getRows().forEach((row) => {
        if (row.getTreeChildren().length > 0) {
          row.treeExpand();
        }
      });
    },

    collapseAll: () => {
      if (!mainTabulator.value) return;
      mainTabulator.value.getRows().forEach((row) => {
        if (row.getTreeChildren().length > 0) {
          row.treeCollapse();
        }
      });
    },

    expandRow: (id) => {
      if (!mainTabulator.value) return false;
      try {
        const row = mainTabulator.value.getRow(id);
        if (row) {
          row.treeExpand();
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },

    collapseRow: (id) => {
      if (!mainTabulator.value) return false;
      try {
        const row = mainTabulator.value.getRow(id);
        if (row) {
          row.treeCollapse();
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },

    isRowExpanded: (id) => {
      if (!mainTabulator.value) return false;
      try {
        const row = mainTabulator.value.getRow(id);
        return row ? row.isTreeExpanded() : false;
      } catch {
        return false;
      }
    },

    // 选择操作方法
    selectRow: (id) => {
      if (!mainTabulator.value) return false;
      try {
        const row = mainTabulator.value.getRow(id);
        if (row) {
          row.select();
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },

    deselectRow: (id) => {
      if (!mainTabulator.value) return false;
      try {
        const row = mainTabulator.value.getRow(id);
        if (row) {
          row.deselect();
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },

    getSelectedRows: () => {
      if (!mainTabulator.value) return [];
      return mainTabulator.value
        .getSelectedRows()
        .map((row) => row.getData() );
    },

    selectAll: () => {
      if (!mainTabulator.value) return;
      mainTabulator.value.selectRow();
    },

    deselectAll: () => {
      if (!mainTabulator.value) return;
      mainTabulator.value.deselectRow();
    },

    // 排序和过滤方法
    sortBy: (field, direction) => {
      if (!mainTabulator.value) return;
      mainTabulator.value.setSort(field, direction);
    },

    setFilter: (field, type, value) => {
      if (!mainTabulator.value) return;
      mainTabulator.value.setFilter(field, type, value);
    },

    clearFilter: () => {
      if (!mainTabulator.value) return;
      mainTabulator.value.clearFilter();
    },

    // 分页方法
    setPage: (page) => {
      if (!mainTabulator.value) return;
      mainTabulator.value.setPage(page);
    },

    getCurrentPage: () => {
      if (!mainTabulator.value) return 1;
      return mainTabulator.value.getPage();
    },

    setPageSize: (size) => {
      if (!mainTabulator.value) return;
      mainTabulator.value.setPageSize(size);
    },

    // 滚动方法
    scrollToRow: (id, position) => {
      if (!mainTabulator.value) return false;
      try {
        const row = mainTabulator.value.getRow(id);
        if (row) {
          row.scrollTo(position || "center");
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },

    scrollToTop: () => {
      if (!mainTabulator.value) return;
      mainTabulator.value.scrollToRow(1, "top", false);
    },

    scrollToBottom: () => {
      const rows = mainTabulator.value.getRows();
      if (rows.length > 0) {
        rows[rows.length - 1].scrollTo("bottom");
      }
    },

    // 导出方法
    exportToCSV: (filename) => {
      if (!mainTabulator.value) return;
      mainTabulator.value.download("csv", filename || "data.csv");
    },

    exportToJSON: (filename) => {
      if (!mainTabulator.value) return;
      mainTabulator.value.download("json", filename || "data.json");
    },

    exportToPDF: (filename) => {
      if (!mainTabulator.value) return;
      mainTabulator.value.download("pdf", filename || "data.pdf");
    },

    // 状态管理方法
    getTableState: () => {
      return { ...tableState.value };
    },

    setTableState: (newState) => {
      tableState.value = { ...tableState.value, ...newState };
    },

    getTableStats: () => {
      if (!mainTabulator.value) return null;
      const allRows = mainTabulator.value.getRows();
      const selectedRows = mainTabulator.value.getSelectedRows();
      return {
        totalRows: allRows.length,
        selectedRows: selectedRows.length,
        filteredRows: mainTabulator.value.getDataCount("active"),
      };
    },

    // 刷新和重建方法
    refresh: () => {
      if (!mainTabulator.value) return;
      mainTabulator.value.redraw();
    },

    destroy: () => {
      if (mainTabulator.value) {
        mainTabulator.value.destroy();
        mainTabulator.value = null;
      }
    },
  };

  return {
    mainTabulator,
    initMainTabulator,
    updateData,
    setEventHandlers,
    ...exportedMethods,
  };
}
