<script lang="ts">
import { defineComponent, ref, nextTick, onMounted, onUnmounted, watch } from "vue";
import { ElMessage, ElMessageBox, ElDialog, ElButton, ElColorPicker, ElIcon } from "element-plus";
// @ts-ignore
import { TabulatorFull as Tabulator } from 'tabulator-tables';
// @ts-ignore
import type { RowComponent, CellComponent } from 'tabulator-tables';
import 'tabulator-tables/dist/css/tabulator.min.css';
import { Plus, Edit, Delete, CopyDocument, Brush } from '@element-plus/icons-vue';

// 导入组合式函数
import { useMainTabulator } from '@/composables/useMainTabulator';

interface TableData {
  id: number | string;
  date: string;
  level: string;
  message: string;
}

// 转换数据格式
const convertToCxxmTabulatorFormat = (data: TableData[]): any[] => {
  return data.map((item) => ({
    id: String(item.id),
    name: item.message, // 使用message作为name字段
    type: item.level,   // 使用level作为type字段
    subType: "",
    description: item.date,
    _row_color: undefined,
  }));
};

export default defineComponent({
  name: "CxxmView",
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
  },
  setup(props, { expose }) {
    // 主表格引用
    const mainTableRef = ref<HTMLElement>();

    // 使用主表格组合式函数
    const mainTabulatorComposable = useMainTabulator();

    // 表格数据
    const tableData = ref<TableData[]>([
      { id: 1, date: "2024-07-29", level: "info", message: "用户登录" },
      { id: 2, date: "2024-07-29", level: "warn", message: "密码即将过期" },
      { id: 3, date: "2024-07-30", level: "error", message: "登录失败" },
      { id: 4, date: "2024-07-30", level: "info", message: "数据同步完成" },
    ]);

    // 颜色对话框相关
    const colorDialogVisible = ref(false);
    const colorValue = ref("#409EFF");
    const currentRow = ref<any>(null);

    const init = (id: string | number | null) => {
      console.log("CxxmView.vue init called with ID:", id);
      // 如果需要，这里也可以根据ID刷新日志数据
      if (id) {
        tableData.value = [
          {
            id: id,
            date: new Date().toISOString().split('T')[0],
            level: "info",
            message: `Logs related to tree node ${id}`,
          },
        ];
      }
    };

    // 颜色变更处理
    const handleColorChange = (id: string, color: string) => {
      const item = tableData.value.find(item => String(item.id) === id);
      if (item) {
        (item as any)._row_color = color;
        console.log("CxxmView: 颜色变更", id, color);
      }
    };

    // 行点击处理
    const handleRowClick = (row: any) => {
      currentRow.value = row;
      console.log("CxxmView: 行点击", row);
    };

    // 处理表格事件
    const handleAddRow = (currentRow?: TableData) => {
      const newRow: TableData = {
        id: Date.now(),
        date: new Date().toISOString().split("T")[0],
        level: "info",
        message: "新日志记录",
      };

      if (currentRow) {
        // 在指定行的下一行插入新行
        const currentIndex = tableData.value.findIndex((item) => item.id === currentRow.id);
        if (currentIndex !== -1) {
          tableData.value.splice(currentIndex + 1, 0, newRow);
          ElMessage.success("在选中行下方添加成功");
        } else {
          tableData.value.push(newRow);
          ElMessage.success("添加成功");
        }
      } else {
        tableData.value.push(newRow);
        ElMessage.success("添加成功");
      }
    };

    const handleAddChild = (parentRow: TableData) => {
      // 普通表格不支持子项，显示提示
      ElMessage.warning("此表格不支持添加子项");
    };

    const handleEditRow = (row: TableData) => {
      // 可以触发编辑模式或显示编辑弹窗
      ElMessage.info("编辑功能，可以双击单元格编辑");
    };

    const handleDeleteRow = (row: TableData) => {
      ElMessageBox.confirm("确定要删除这一行吗？", "警告", {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning",
      })
        .then(() => {
          const index = tableData.value.findIndex((item) => item.id === row.id);
          if (index > -1) {
            tableData.value.splice(index, 1);
            ElMessage.success("删除成功");
          }
        })
        .catch(() => { });
    };

    const handleCopyRow = (row: TableData) => {
      const newItem: TableData = {
        ...row,
        id: Date.now(),
        message: `${row.message} (复制)`,
      };
      tableData.value.push(newItem);
      ElMessage.success("复制成功");
    };

    // 颜色对话框处理
    const openColorDialog = () => {
      colorDialogVisible.value = true;
    };

    const handleColorSubmit = () => {
      if (currentRow.value) {
        handleColorChange(currentRow.value.id, colorValue.value);
      }
      colorDialogVisible.value = false;
    };

    // 生命周期
    onMounted(() => {
      console.log('CxxmView mounted, 开始初始化表格');
      nextTick(() => {
        console.log('CxxmView nextTick 执行，开始检查主表格容器');
        console.log('主表格容器引用:', mainTableRef.value);

        if (mainTableRef.value) {
          console.log('主表格容器尺寸:', {
            width: mainTableRef.value.offsetWidth,
            height: mainTableRef.value.offsetHeight,
            clientWidth: mainTableRef.value.clientWidth,
            clientHeight: mainTableRef.value.clientHeight
          });

          console.log('原始数据:', tableData.value);
          const convertedData = convertToCxxmTabulatorFormat(tableData.value);
          console.log('转换后的主表格数据:', convertedData);

          try {
            mainTabulatorComposable.initMainTabulator(
              mainTableRef.value,
              convertedData,
              handleColorChange,
              handleRowClick
            );
            console.log('CxxmView: 主表格初始化完成');
          } catch (error) {
            console.error('CxxmView: 主表格初始化失败:', error);
          }
        } else {
          console.error('CxxmView: 主表格容器未找到 - mainTableRef.value 为:', mainTableRef.value);
        }
      });
    });

    onUnmounted(() => {
      mainTabulatorComposable.destroy();
    });

    // 监听数据变化
    watch(() => tableData.value, (newData) => {
      const convertedData = convertToCxxmTabulatorFormat(newData);
      mainTabulatorComposable.updateData(convertedData);
    }, { deep: true });

    expose({ init });

    return {
      // DOM refs
      mainTableRef,
      // 数据
      tableData,
      // 颜色对话框
      colorDialogVisible,
      colorValue,
      // 事件处理
      handleAddRow,
      handleAddChild,
      handleEditRow,
      handleDeleteRow,
      handleCopyRow,
      handleColorSubmit,
      openColorDialog,
      convertToCxxmTabulatorFormat,
      // 暴露主表格组合式函数的所有方法
      ...mainTabulatorComposable,
    };
  },
});
</script>

<template>
  <div class="cxxm-view-container">
    <!-- 颜色选择对话框 -->
    <el-dialog v-model="colorDialogVisible" title="设置边框颜色" width="300px">
      <el-color-picker v-model="colorValue" />
      <template #footer>
        <el-button @click="colorDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleColorSubmit">确定</el-button>
      </template>
    </el-dialog>

    <!-- 主表格内容 -->
    <div class="main-table-section">
      <div ref="mainTableRef" class="main-table-container"></div>
    </div>
  </div>
</template>

<style scoped>
.cxxm-view-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 主表格内容 */
.main-table-section {
  height: 100%;
  flex-shrink: 0;
  border: 1px solid #e4e7ed;
  overflow: hidden;
  border-radius: 4px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

/* 表格容器样式 */
.main-table-container {
  width: 100%;
  height: 100%;
  min-height: 400px;
  /* 确保容器有足够高度 */
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

/* 有边框颜色的行样式 */
:deep(.tabulator .tabulator-row.colored-border) {
  border-width: 2px !important;
  border-style: solid !important;
  margin: 1px 0 !important;
}

/* 确保边框颜色在悬停时也保持 */
:deep(.tabulator .tabulator-row.colored-border:hover) {
  border-width: 2px !important;
  border-style: solid !important;
}
</style>
