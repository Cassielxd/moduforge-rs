<script lang="ts">
import { defineComponent, ref } from "vue";
import { ElMessage } from "element-plus";
import RightTablePanel, { TableColumn } from "../components/RightTablePanel.vue";

interface ProjectInfo {
  id: number | string;
  label: string;
  value: string;
  editable?: boolean;
}

export default defineComponent({
  name: "ProjectInfoView",
  components: { RightTablePanel },
  setup(props, { expose }) {
    // 项目基本信息数据
    const projectInfoData = ref<ProjectInfo[]>([
      { id: 1, label: "项目名称", value: "ModuForge Demo 工程项目", editable: true },
      { id: 2, label: "项目编号", value: "PROJ-2024-001", editable: true },
      { id: 3, label: "项目经理", value: "张三", editable: true },
      { id: 4, label: "创建时间", value: "2024-07-29", editable: false },
      { id: 5, label: "项目状态", value: "进行中", editable: true },
      { id: 6, label: "预算总额", value: "¥15,876,543.20", editable: true },
      { id: 7, label: "项目地址", value: "上海市浦东新区张江高科技园区", editable: true },
      { id: 8, label: "联系电话", value: "021-12345678", editable: true },
      {
        id: 9,
        label: "项目描述",
        value: "基于现代化技术栈的综合性工程管理系统",
        editable: true,
      },
      { id: 10, label: "计划工期", value: "12个月", editable: true },
      { id: 11, label: "技术负责人", value: "李四", editable: true },
      { id: 12, label: "质量负责人", value: "王五", editable: true },
    ]);

    // 项目信息表格列定义
    const projectInfoColumns: TableColumn[] = [
      { prop: "label", label: "项目属性", width: 150 },
      { prop: "value", label: "属性值", minWidth: 300 },
    ];

    const rightTablePanelRef = ref();

    const init = (id: string | number | null) => {
      console.log("ProjectInfoView.vue init called with ID:", id);
      // 根据不同的节点ID可以加载不同的项目信息
      if (id) {
        // 这里可以根据ID从服务器获取对应的项目信息
        console.log(`Loading project info for node: ${id}`);
      }
    };

    // 处理表格事件
    const handleAddRow = (currentRow?: ProjectInfo) => {
      const newRow: ProjectInfo = {
        id: Date.now(),
        label: "新属性",
        value: "请输入值",
        editable: true,
      };

      if (currentRow) {
        // 在指定行的下一行插入新行
        const currentIndex = projectInfoData.value.findIndex((item) => item.id === currentRow.id);
        if (currentIndex !== -1) {
          projectInfoData.value.splice(currentIndex + 1, 0, newRow);
          ElMessage.success("在选中行下方添加成功");
        } else {
          projectInfoData.value.push(newRow);
          ElMessage.success("添加成功");
        }
      } else {
        projectInfoData.value.push(newRow);
        ElMessage.success("添加成功");
      }

      // 选中新添加的行
      if (rightTablePanelRef.value) {
        rightTablePanelRef.value.setCurrentRow(newRow.id);
      }
    };

    const handleAddChild = () => {
      ElMessage.warning("项目信息不支持添加子项");
    };

    const handleEditRow = (row: ProjectInfo) => {
      if (row.editable) {
        ElMessage.info("双击单元格可以编辑属性值");
      } else {
        ElMessage.warning("该属性不允许编辑");
      }
    };

    const handleDeleteRow = (row: ProjectInfo) => {
      if (row.editable) {
        const index = projectInfoData.value.findIndex((item) => item.id === row.id);
        if (index > -1) {
          projectInfoData.value.splice(index, 1);
          ElMessage.success("删除成功");
        }
      } else {
        ElMessage.warning("系统属性不允许删除");
      }
    };

    const handleCopyRow = (row: ProjectInfo) => {
      const newItem: ProjectInfo = {
        ...row,
        id: Date.now(),
        label: `${row.label} (复制)`,
      };
      projectInfoData.value.push(newItem);
      ElMessage.success("复制成功");
    };

    expose({ init });

    return {
      rightTablePanelRef,
      projectInfoData,
      projectInfoColumns,
      handleAddRow,
      handleAddChild,
      handleEditRow,
      handleDeleteRow,
      handleCopyRow,
    };
  },
});
</script>

<template>
  <div class="project-info-container">
    <!-- 详细信息表格 -->
    <RightTablePanel
      ref="rightTablePanelRef"
      :table-data="projectInfoData"
      :table-columns="projectInfoColumns"
      :is-tree-table="false"
      @add-row="handleAddRow"
      @add-child="handleAddChild"
      @edit-row="handleEditRow"
      @delete-row="handleDeleteRow"
      @copy-row="handleCopyRow"
    />
  </div>
</template>

<style scoped>
.project-info-container {
  height: 100%;
}
</style>
