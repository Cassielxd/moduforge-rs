// 主表格相关的组合式函数
import { ref, nextTick } from "vue";
// @ts-ignore
import { TabulatorFull as Tabulator } from "tabulator-tables";
import "tabulator-tables/dist/css/tabulator.min.css";
import { ElMessage } from "element-plus";

// 接口定义
interface TableItem {
  id: string;
  name: string;
  type: string;
  subType?: string;
  description?: string;
  children?: TableItem[];
  _row_color?: string;
}

interface TableState {
  expandedRows: Set<string>;
  selectedRows: Set<string>;
  scrollPosition: number;
  filters: any[];
  sorting: any[];
}

export function useMainTabulator() {
  const mainTabulator = ref<any>(null);
  const isEditing = ref(false);
  const tableState = ref<TableState>({
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
    onAddChildRow: (row: any) => {
      ElMessage.info(
        "添加子项功能 - 请在外部组件中通过 setEventHandlers 自定义实现"
      );
    },
    onEditRow: (row: any) => {
      ElMessage.info("主表格编辑功能，可以双击单元格编辑");
    },
    onDeleteRow: (row: any) => {
      ElMessage.info("主表格删除功能");
    },
    onCopyRow: (row: any) => {
      ElMessage.info("主表格复制功能");
    },
    onCellEdited: (cell: any) => {
      ElMessage.info("主表格编辑功能");
    },
  };

  // 设置事件处理器
  const setEventHandlers = (handlers: Partial<typeof eventHandlers>) => {
    Object.assign(eventHandlers, handlers);
  };

  // 保存表格状态
  const saveTableState = () => {
    if (!mainTabulator.value) return;

    // @ts-ignore
    const expandedRows = mainTabulator.value
      .getRows()
      .filter((row: any) => row.isTreeExpanded());
    tableState.value.expandedRows = new Set(
      expandedRows.map((row: any) => (row.getData() as TableItem).id)
    );

    // @ts-ignore
    const selectedRows = mainTabulator.value.getSelectedRows();
    tableState.value.selectedRows = new Set(
      selectedRows.map((row: any) => (row.getData() as TableItem).id)
    );

    // @ts-ignore
    tableState.value.scrollPosition = mainTabulator.value.getScrollPosition();
  };

  // 恢复表格状态
  const restoreTableState = async () => {
    if (!mainTabulator.value) return;

    await nextTick();

    // 恢复展开状态
    tableState.value.expandedRows.forEach((id) => {
      // @ts-ignore
      const row = mainTabulator.value.getRow(id);
      if (row) {
        // @ts-ignore
        row.treeExpand();
      }
    });

    // 恢复选中状态
    tableState.value.selectedRows.forEach((id) => {
      // @ts-ignore
      const row = mainTabulator.value.getRow(id);
      if (row) {
        // @ts-ignore
        row.select();
      }
    });

    // 恢复滚动位置
    if (tableState.value.scrollPosition > 0) {
      // @ts-ignore
      mainTabulator.value.scrollToPosition(
        0,
        tableState.value.scrollPosition,
        false
      );
    }
  };

  // 行格式化器 - 添加颜色边框
  const rowFormatter = (row: any) => {
    const data = row.getData() as TableItem;
    if (data._row_color) {
      const element = row.getElement();
      element.style.borderLeft = `4px solid ${data._row_color}`;
    }
  };

  // 定义表格列
  const getColumns = (
    onColorChange: (id: string, color: string) => void
  ): any[] => [
    {
      title: "名称",
      field: "name",
      width: 250,
      editor: "input",
      formatter: "tree" as any,
      headerSort: false,
    },
    { title: "类型", field: "type", width: 150, editor: "input" },
    { title: "子类型", field: "subType", width: 150, editor: "input" },
    { title: "描述", field: "description", width: 300, editor: "input" },
    {
      title: "颜色",
      field: "_row_color",
      width: 100,
      formatter: (cell: any) => {
        const value = cell.getValue();
        if (value) {
          return `<div style="width: 20px; height: 20px; background-color: ${value}; border: 1px solid #ccc; border-radius: 3px; display: inline-block;"></div>`;
        }
        return '<span style="color: #999;">无颜色</span>';
      },
      cellClick: (e: UIEvent, cell: any) => {
        const data = cell.getData() as TableItem;
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
    element: HTMLElement,
    data: TableItem[],
    onColorChange: (id: string, color: string) => void,
    onRowClick?: (row: TableItem) => void
  ) => {
    if (mainTabulator.value) {
      // @ts-ignore
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
      dataTreeChildIndent: 20,
      selectable: true,
      rowFormatter: rowFormatter,
      editTriggerEvent: "dblclick", // 设置双击编辑
      cellEdited: (cell: any) => {
        isEditing.value = true;
        console.log("Cell edited:", cell.getData());
        eventHandlers.onCellEdited(cell);
      },
      rowClick: (e: UIEvent, row: any) => {
        const data = row.getData() as TableItem;
        if (onRowClick) {
          onRowClick(data);
        }
      },
      rowDblClick: (e: UIEvent, row: any) => {
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
          action: (e: Event, row: any) => {
            console.log("右键菜单 - 添加子项被点击");
            eventHandlers.onAddChildRow(row.getData());
          },
        },
        {
          label: "编辑",
          action: (e: Event, row: any) => {
            console.log("右键菜单 - 编辑被点击，行数据:", row.getData());
            eventHandlers.onEditRow(row.getData());
          },
        },
        {
          label: "删除",
          action: (e: Event, row: any) => {
            console.log("右键菜单 - 删除被点击，行数据:", row.getData());
            eventHandlers.onDeleteRow(row.getData());
          },
        },
        {
          label: "复制",
          action: (e: Event, row: any) => {
            console.log("右键菜单 - 复制被点击，行数据:", row.getData());
            eventHandlers.onCopyRow(row.getData());
          },
        },
      ],
    });

    console.log("Tabulator instance created:", mainTabulator.value);
  };

  // 更新数据
  const updateData = async (data: TableItem[]) => {
    if (!mainTabulator.value) return;

    if (isEditing.value) {
      // 如果是编辑操作，保存状态
      saveTableState();
      isEditing.value = false;
    }

    // @ts-ignore
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
      // @ts-ignore
      const selectedRows = mainTabulator.value.getSelectedRows();
      return selectedRows.length > 0
        ? (selectedRows[0].getData() as TableItem)
        : null;
    },

    getAllData: () => {
      if (!mainTabulator.value) return [];
      // @ts-ignore
      return mainTabulator.value.getData() as TableItem[];
    },

    getRowById: (id: string) => {
      if (!mainTabulator.value) return null;
      try {
        // @ts-ignore
        const row = mainTabulator.value.getRow(id);
        return row ? (row.getData() as TableItem) : null;
      } catch {
        return null;
      }
    },

    findRows: (criteria: Partial<TableItem>) => {
      if (!mainTabulator.value) return [];
      // @ts-ignore
      return mainTabulator.value
        .searchRows(criteria)
        .map((row: any) => row.getData() as TableItem);
    },

    // 数据操作方法
    updateRow: (id: string, data: Partial<TableItem>) => {
      if (!mainTabulator.value) return false;
      try {
        // @ts-ignore
        const row = mainTabulator.value.getRow(id);
        if (row) {
          // @ts-ignore
          row.update(data);
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },

    deleteRow: (id: string) => {
      if (!mainTabulator.value) return false;
      try {
        // @ts-ignore
        const row = mainTabulator.value.getRow(id);
        if (row) {
          // @ts-ignore
          row.delete();
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },

    addRow: (
      data: TableItem,
      position?: "top" | "bottom",
      parentId?: string
    ) => {
      if (!mainTabulator.value) return false;
      try {
        if (parentId) {
          // @ts-ignore
          const parentRow = mainTabulator.value.getRow(parentId);
          if (parentRow) {
            // @ts-ignore
            parentRow.addTreeChild(data);
            return true;
          }
        } else {
          // @ts-ignore
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
      // @ts-ignore
      mainTabulator.value.getRows().forEach((row) => {
        if (row.getTreeChildren().length > 0) {
          // @ts-ignore
          row.treeExpand();
        }
      });
    },

    collapseAll: () => {
      if (!mainTabulator.value) return;
      // @ts-ignore
      mainTabulator.value.getRows().forEach((row) => {
        if (row.getTreeChildren().length > 0) {
          // @ts-ignore
          row.treeCollapse();
        }
      });
    },

    expandRow: (id: string) => {
      if (!mainTabulator.value) return false;
      try {
        // @ts-ignore
        const row = mainTabulator.value.getRow(id);
        if (row) {
          // @ts-ignore
          row.treeExpand();
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },

    collapseRow: (id: string) => {
      if (!mainTabulator.value) return false;
      try {
        // @ts-ignore
        const row = mainTabulator.value.getRow(id);
        if (row) {
          // @ts-ignore
          row.treeCollapse();
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },

    isRowExpanded: (id: string) => {
      if (!mainTabulator.value) return false;
      try {
        // @ts-ignore
        const row = mainTabulator.value.getRow(id);
        // @ts-ignore
        return row ? row.isTreeExpanded() : false;
      } catch {
        return false;
      }
    },

    // 选择操作方法
    selectRow: (id: string) => {
      if (!mainTabulator.value) return false;
      try {
        // @ts-ignore
        const row = mainTabulator.value.getRow(id);
        if (row) {
          // @ts-ignore
          row.select();
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },

    deselectRow: (id: string) => {
      if (!mainTabulator.value) return false;
      try {
        // @ts-ignore
        const row = mainTabulator.value.getRow(id);
        if (row) {
          // @ts-ignore
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
      // @ts-ignore
      return mainTabulator.value
        .getSelectedRows()
        .map((row: any) => row.getData() as TableItem);
    },

    selectAll: () => {
      if (!mainTabulator.value) return;
      // @ts-ignore
      mainTabulator.value.selectRow();
    },

    deselectAll: () => {
      if (!mainTabulator.value) return;
      // @ts-ignore
      mainTabulator.value.deselectRow();
    },

    // 排序和过滤方法
    sortBy: (field: string, direction: "asc" | "desc") => {
      if (!mainTabulator.value) return;
      // @ts-ignore
      mainTabulator.value.setSort(field, direction);
    },

    setFilter: (field: string, type: string, value: any) => {
      if (!mainTabulator.value) return;
      // @ts-ignore
      mainTabulator.value.setFilter(field, type, value);
    },

    clearFilter: () => {
      if (!mainTabulator.value) return;
      // @ts-ignore
      mainTabulator.value.clearFilter();
    },

    // 分页方法
    setPage: (page: number) => {
      if (!mainTabulator.value) return;
      // @ts-ignore
      mainTabulator.value.setPage(page);
    },

    getCurrentPage: () => {
      if (!mainTabulator.value) return 1;
      // @ts-ignore
      return mainTabulator.value.getPage();
    },

    setPageSize: (size: number) => {
      if (!mainTabulator.value) return;
      // @ts-ignore
      mainTabulator.value.setPageSize(size);
    },

    // 滚动方法
    scrollToRow: (id: string, position?: "top" | "center" | "bottom") => {
      if (!mainTabulator.value) return false;
      try {
        // @ts-ignore
        const row = mainTabulator.value.getRow(id);
        if (row) {
          // @ts-ignore
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
      // @ts-ignore
      mainTabulator.value.scrollToRow(1, "top", false);
    },

    scrollToBottom: () => {
      if (!mainTabulator.value) return;
      // @ts-ignore
      const rows = mainTabulator.value.getRows();
      if (rows.length > 0) {
        // @ts-ignore
        rows[rows.length - 1].scrollTo("bottom");
      }
    },

    // 导出方法
    exportToCSV: (filename?: string) => {
      if (!mainTabulator.value) return;
      // @ts-ignore
      mainTabulator.value.download("csv", filename || "data.csv");
    },

    exportToJSON: (filename?: string) => {
      if (!mainTabulator.value) return;
      // @ts-ignore
      mainTabulator.value.download("json", filename || "data.json");
    },

    exportToPDF: (filename?: string) => {
      if (!mainTabulator.value) return;
      // @ts-ignore
      mainTabulator.value.download("pdf", filename || "data.pdf");
    },

    // 状态管理方法
    getTableState: () => {
      return { ...tableState.value };
    },

    setTableState: (newState: Partial<TableState>) => {
      tableState.value = { ...tableState.value, ...newState };
    },

    getTableStats: () => {
      if (!mainTabulator.value) return null;
      // @ts-ignore
      const allRows = mainTabulator.value.getRows();
      // @ts-ignore
      const selectedRows = mainTabulator.value.getSelectedRows();
      return {
        totalRows: allRows.length,
        selectedRows: selectedRows.length,
        // @ts-ignore
        filteredRows: mainTabulator.value.getDataCount("active"),
      };
    },

    // 刷新和重建方法
    refresh: () => {
      if (!mainTabulator.value) return;
      // @ts-ignore
      mainTabulator.value.redraw();
    },

    destroy: () => {
      if (mainTabulator.value) {
        // @ts-ignore
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
