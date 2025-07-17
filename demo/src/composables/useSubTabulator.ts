// 子表格相关的组合式函数
import { ref } from "vue";
// @ts-ignore
import { TabulatorFull as Tabulator } from "tabulator-tables";
import "tabulator-tables/dist/css/tabulator.min.css";
import { ElMessage } from "element-plus";

// 接口定义
interface TableColumn {
  prop: string;
  label: string;
  minWidth?: number;
  width?: number;
  align?: "left" | "center" | "right";
  type?: "date";
}

// 子表格数据类型
interface SubTableData {
  id: number | string;
  [key: string]: any;
}

export function useSubTabulator() {
  // 为每个标签页维护独立的实例
  const subTabulators = ref<Record<string, any>>({});
  const currentActiveTab = ref<string>("detail");

  // 默认数据
  const defaultData = {
    detail: [
      { id: 1, property: "文件类型", value: "目录/文件" },
      { id: 2, property: "创建时间", value: "2024-07-29" },
      { id: 3, property: "修改时间", value: "2024-07-29" },
      { id: 4, property: "文件大小", value: "2.5MB" },
      { id: 5, property: "权限", value: "读写执行" },
      { id: 6, property: "所有者", value: "当前用户" },
    ],
    statistics: [
      {
        id: 1,
        name: "总文件数",
        value: "156",
        unit: "个",
        description: "包含所有文件和文件夹",
      },
      {
        id: 2,
        name: "文件夹数",
        value: "23",
        unit: "个",
        description: "目录文件夹数量",
      },
      {
        id: 3,
        name: "总大小",
        value: "45.2",
        unit: "MB",
        description: "所有文件总大小",
      },
      {
        id: 4,
        name: "最近修改",
        value: "12",
        unit: "个",
        description: "近7天内修改的文件",
      },
    ],
    history: [
      {
        id: 1,
        time: "2024-07-29 15:30",
        action: "创建",
        file: "项目说明.docx",
        user: "张三",
      },
      {
        id: 2,
        time: "2024-07-29 14:20",
        action: "修改",
        file: "需求文档.pdf",
        user: "李四",
      },
      {
        id: 3,
        time: "2024-07-29 13:15",
        action: "删除",
        file: "临时文件.tmp",
        user: "王五",
      },
      {
        id: 4,
        time: "2024-07-28 16:45",
        action: "创建",
        file: "代码目录",
        user: "赵六",
      },
    ],
  };

  // 默认列配置
  const defaultColumns = {
    detail: [
      { prop: "property", label: "属性", width: 120 },
      { prop: "value", label: "值", minWidth: 200 },
    ],
    statistics: [
      { prop: "name", label: "统计项", width: 120 },
      { prop: "value", label: "数值", width: 80, align: "right" as const },
      { prop: "unit", label: "单位", width: 60 },
      { prop: "description", label: "说明", minWidth: 200 },
    ],
    history: [
      { prop: "time", label: "时间", width: 150 },
      { prop: "action", label: "操作", width: 80 },
      { prop: "file", label: "文件", minWidth: 200 },
      { prop: "user", label: "用户", width: 100 },
    ],
  };

  // 当前数据状态
  const currentData = ref<SubTableData[]>([]);
  const currentColumns = ref<TableColumn[]>([]);

  // 事件处理函数
  const eventHandlers = {
    onAddRow: () => {
      ElMessage.info("子表格新增功能");
    },
    onEditRow: (row: any) => {
      ElMessage.info("子表格编辑功能，可以双击单元格编辑");
    },
    onDeleteRow: (row: any) => {
      ElMessage.info("子表格删除功能");
    },
    onCopyRow: (row: any) => {
      ElMessage.info("子表格复制功能");
    },
    onCellEdited: (cell: any) => {
      ElMessage.info("子表格编辑功能");
    },
  };

  // 设置事件处理器
  const setEventHandlers = (handlers: Partial<typeof eventHandlers>) => {
    Object.assign(eventHandlers, handlers);
  };

  // 获取列配置
  const getColumns = (columns: TableColumn[]): any[] => [
    {
      title: "",
      width: 50,
      headerSort: false,
      formatter: () => "",
    },
    ...columns.map((col) => ({
      title: col.label,
      field: col.prop,
      width: col.width || col.minWidth || 120,
      headerSort: true,
      editor: "input",
      cellEdited: (cell: any) => {
        eventHandlers.onCellEdited(cell);
      },
    })),
  ];

  // 初始化子表格
  const initSubTabulator = (
    element: HTMLElement,
    tabName: string,
    customData?: SubTableData[],
    customColumns?: TableColumn[]
  ) => {
    if (!element) return;

    // 设置当前活动标签
    currentActiveTab.value = tabName;

    // 确定数据和列配置
    const data =
      customData || defaultData[tabName as keyof typeof defaultData] || [];
    const columns =
      customColumns ||
      defaultColumns[tabName as keyof typeof defaultColumns] ||
      [];

    currentData.value = data;
    currentColumns.value = columns;

    // 销毁旧实例
    if (subTabulators.value[tabName]) {
      subTabulators.value[tabName].destroy();
    }

    // 创建新的 Tabulator 实例
    console.log("创建Tabulator实例，当前事件处理器:", eventHandlers);

    subTabulators.value[tabName] = new Tabulator(element, {
      data: data,
      columns: getColumns(columns),
      dataTree: true,
      dataTreeChildField: "children",
      dataTreeStartExpanded: true,
      dataTreeBranchElement: true,
      dataTreeChildIndent: 20,
      layout: "fitColumns",
      selectable: true,
      height: 400,
      maxHeight: 300,
      editTriggerEvent: "dblclick", // 设置双击编辑
      cellEdited: (cell: any) => {
        console.log("子表格单元格编辑:", cell.getData());
        eventHandlers.onCellEdited(cell);
      },
      rowDblClick: (e: UIEvent, row: any) => {
        console.log("双击子表格行:", row.getData());
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

    console.log("Tabulator实例创建完成:", tabName);
  };

  // 更新数据
  const updateData = (newData: SubTableData[]) => {
    if (!subTabulators.value[currentActiveTab.value]) return;
    currentData.value = newData;
    // @ts-ignore
    subTabulators.value[currentActiveTab.value].setData(newData);
  };

  // 切换标签页
  const switchTab = (
    element: HTMLElement,
    tabName: string,
    customData?: SubTableData[],
    customColumns?: TableColumn[]
  ) => {
    initSubTabulator(element, tabName, customData, customColumns);
  };

  // 导出的数据操作方法
  const exportedMethods = {
    // 数据查询方法
    getCurrentData: () => {
      if (!subTabulators.value[currentActiveTab.value]) return [];
      // @ts-ignore
      return subTabulators.value[
        currentActiveTab.value
      ].getData() as SubTableData[];
    },

    getAllRows: () => {
      if (!subTabulators.value[currentActiveTab.value]) return [];
      // @ts-ignore
      return subTabulators.value[currentActiveTab.value].getRows();
    },

    getRowById: (id: string | number) => {
      if (!subTabulators.value[currentActiveTab.value]) return null;
      try {
        // @ts-ignore
        const row = subTabulators.value[currentActiveTab.value].getRow(id);
        return row ? (row.getData() as SubTableData) : null;
      } catch {
        return null;
      }
    },

    getSelectedRows: () => {
      if (!subTabulators.value[currentActiveTab.value]) return [];
      // @ts-ignore
      return subTabulators.value[currentActiveTab.value]
        .getSelectedRows()
        .map((row: any) => row.getData() as SubTableData);
    },

    // 数据操作方法
    addRow: (data: SubTableData, position?: "top" | "bottom") => {
      if (!subTabulators.value[currentActiveTab.value]) return false;
      try {
        // @ts-ignore
        subTabulators.value[currentActiveTab.value].addRow(
          data,
          position === "top"
        );
        currentData.value.push(data);
        return true;
      } catch {
        return false;
      }
    },

    updateRow: (id: string | number, data: Partial<SubTableData>) => {
      if (!subTabulators.value[currentActiveTab.value]) return false;
      try {
        // @ts-ignore
        const row = subTabulators.value[currentActiveTab.value].getRow(id);
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

    deleteRow: (id: string | number) => {
      if (!subTabulators.value[currentActiveTab.value]) return false;
      try {
        // @ts-ignore
        const row = subTabulators.value[currentActiveTab.value].getRow(id);
        if (row) {
          // @ts-ignore
          row.delete();
          // 从当前数据中移除
          const index = currentData.value.findIndex((item) => item.id === id);
          if (index > -1) {
            currentData.value.splice(index, 1);
          }
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },

    // 选择操作方法
    selectRow: (id: string | number) => {
      if (!subTabulators.value[currentActiveTab.value]) return false;
      try {
        // @ts-ignore
        const row = subTabulators.value[currentActiveTab.value].getRow(id);
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

    selectAll: () => {
      if (!subTabulators.value[currentActiveTab.value]) return;
      // @ts-ignore
      subTabulators.value[currentActiveTab.value].selectRow();
    },

    deselectAll: () => {
      if (!subTabulators.value[currentActiveTab.value]) return;
      // @ts-ignore
      subTabulators.value[currentActiveTab.value].deselectRow();
    },

    // 排序和过滤方法
    sortBy: (field: string, direction: "asc" | "desc") => {
      if (!subTabulators.value[currentActiveTab.value]) return;
      // @ts-ignore
      subTabulators.value[currentActiveTab.value].setSort(field, direction);
    },

    setFilter: (field: string, type: string, value: any) => {
      if (!subTabulators.value[currentActiveTab.value]) return;
      // @ts-ignore
      subTabulators.value[currentActiveTab.value].setFilter(field, type, value);
    },

    clearFilter: () => {
      if (!subTabulators.value[currentActiveTab.value]) return;
      // @ts-ignore
      subTabulators.value[currentActiveTab.value].clearFilter();
    },

    // 导出方法
    exportToCSV: (filename?: string) => {
      if (!subTabulators.value[currentActiveTab.value]) return;
      // @ts-ignore
      subTabulators.value[currentActiveTab.value].download(
        "csv",
        filename || "sub_table_data.csv"
      );
    },

    exportToJSON: (filename?: string) => {
      if (!subTabulators.value[currentActiveTab.value]) return;
      // @ts-ignore
      subTabulators.value[currentActiveTab.value].download(
        "json",
        filename || "sub_table_data.json"
      );
    },

    // 工具方法
    refresh: () => {
      if (!subTabulators.value[currentActiveTab.value]) return;
      // @ts-ignore
      subTabulators.value[currentActiveTab.value].redraw();
    },

    destroy: () => {
      // 销毁所有标签页的实例
      Object.keys(subTabulators.value).forEach((tabName) => {
        if (subTabulators.value[tabName]) {
          // @ts-ignore
          subTabulators.value[tabName].destroy();
        }
      });
      subTabulators.value = {};
    },

    // 获取表格状态
    getTableStats: () => {
      if (!subTabulators.value[currentActiveTab.value]) return null;
      // @ts-ignore
      const allRows = subTabulators.value[currentActiveTab.value].getRows();
      // @ts-ignore
      const selectedRows =
        subTabulators.value[currentActiveTab.value].getSelectedRows();
      return {
        totalRows: allRows.length,
        selectedRows: selectedRows.length,
        activeTab: currentActiveTab.value,
      };
    },
  };

  return {
    subTabulators,
    currentData,
    currentColumns,
    currentActiveTab,
    defaultData,
    defaultColumns,
    initSubTabulator,
    updateData,
    switchTab,
    setEventHandlers,
    ...exportedMethods,
  };
}
