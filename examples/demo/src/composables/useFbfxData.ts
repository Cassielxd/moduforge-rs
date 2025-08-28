// 数据管理组合式函数
import { ref } from "vue";

export interface TreeTableData {
  id: number | string;
  name: string;
  type: string;
  size?: string;
  date: string;
  children?: TreeTableData[];
  color?: string;
}

export interface TableColumn {
  prop: string;
  label: string;
  minWidth?: number;
  width?: number;
  align?: "left" | "center" | "right";
  type?: "date";
}

// 将主表格数据转换为useMainTabulator所需的格式
export const convertToMainTabulatorFormat = (data: TreeTableData[]): any[] => {
  return data.map((item) => ({
    id: String(item.id),
    name: item.name,
    type: item.type,
    subType: item.size || "",
    description: item.date,
    children: item.children
      ? convertToMainTabulatorFormat(item.children)
      : undefined,
    _row_color: item.color,
  }));
};

export function useFbfxData() {
  // 主表格数据
  const tableTreeData = ref<TreeTableData[]>([
    {
      id: 1,
      name: "文档目录",
      type: "folder",
      date: "2024-07-29",
      children: [
        {
          id: 11,
          name: "项目说明.docx",
          type: "file",
          size: "2.5MB",
          date: "2024-07-29",
        },
        {
          id: 12,
          name: "需求文档.pdf",
          type: "file",
          size: "1.8MB",
          date: "2024-07-28",
        },
      ],
    },
    {
      id: 2,
      name: "代码目录",
      type: "folder",
      date: "2024-07-29",
      children: [
        {
          id: 21,
          name: "src",
          type: "folder",
          date: "2024-07-29",
          children: [
            {
              id: 211,
              name: "main.ts",
              type: "file",
              size: "1.2KB",
              date: "2024-07-29",
            },
            {
              id: 212,
              name: "App.vue",
              type: "file",
              size: "3.4KB",
              date: "2024-07-29",
            },
          ],
        },
        {
          id: 22,
          name: "package.json",
          type: "file",
          size: "2.1KB",
          date: "2024-07-28",
        },
      ],
    },
    {
      id: 3,
      name: "readme.md",
      type: "file",
      size: "5.2KB",
      date: "2024-07-27",
    },
  ]);

  // 主表格列配置
  const tableColumns: TableColumn[] = [
    { prop: "name", label: "名称", width: 200 },
    { prop: "type", label: "类型", width: 100 },
    { prop: "size", label: "大小", width: 100 },
    { prop: "date", label: "修改时间", width: 150 },
  ];

  // 子表格数据
  const historyData = ref([
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
  ]);

  const detailData = ref([
    { id: 1, property: "文件类型", value: "目录/文件" },
    { id: 2, property: "创建时间", value: "2024-07-29" },
    { id: 3, property: "修改时间", value: "2024-07-29" },
    { id: 4, property: "文件大小", value: "2.5MB" },
  ]);

  const statisticsData = ref([
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
  ]);

  // 子表格列配置
  const historyColumns = [
    { prop: "time", label: "时间", width: 150 },
    { prop: "action", label: "操作", width: 80 },
    { prop: "file", label: "文件", minWidth: 200 },
    { prop: "user", label: "用户", width: 100 },
  ];

  const detailColumns = [
    { prop: "property", label: "属性", width: 120 },
    { prop: "value", label: "值", minWidth: 200 },
  ];

  const statisticsColumns = [
    { prop: "name", label: "统计项", width: 120 },
    { prop: "value", label: "数值", width: 80, align: "right" as const },
    { prop: "unit", label: "单位", width: 60 },
    { prop: "description", label: "说明", minWidth: 200 },
  ];

  return {
    // 数据
    tableTreeData,
    historyData,
    detailData,
    statisticsData,
    // 列配置
    tableColumns,
    historyColumns,
    detailColumns,
    statisticsColumns,
    // 工具函数
    convertToMainTabulatorFormat,
  };
}
