<script lang="ts">
import { defineComponent, ref, nextTick, onMounted, onUnmounted, watch } from "vue";
import { ElMessage, ElMessageBox, ElDialog, ElButton, ElColorPicker, ElIcon, ElTabs, ElTabPane } from "element-plus";
// @ts-ignore
import { TabulatorFull as Tabulator } from 'tabulator-tables';
// @ts-ignore
import type { RowComponent, CellComponent } from 'tabulator-tables';
import 'tabulator-tables/dist/css/tabulator.min.css';
import { Plus, Edit, Delete, CopyDocument, Brush, Folder, Document } from '@element-plus/icons-vue';

// 导入新的组合式函数
import { useMainTabulator } from '@/composables/useMainTabulator';
import { useSubTabulator } from '@/composables/useSubTabulator';
import { useFbfxData, convertToMainTabulatorFormat } from '@/composables/useFbfxData';
import { useFbfxActions } from '@/composables/useFbfxActions';
import { useColorDialog } from '@/composables/useColorDialog';
import { useFbfxUtils } from '@/composables/useFbfxUtils';
import { useFbfxTables } from '@/composables/useFbfxTables';

// 导入专门的子表格组合式函数
import { useRcjDetail } from '@/composables/useRcjDetail';
import { usePriceStructure } from '@/composables/usePriceStructure';
import { useStandardConversion } from '@/composables/useStandardConversion';

export default defineComponent({
  name: "FbfxView",
  components: {
    ElDialog,
    ElButton,
    ElColorPicker,
    ElIcon,
    ElTabs,
    ElTabPane,
    Plus,
    Edit,
    Delete,
    CopyDocument,
    Brush,
    Folder,
    Document,
  },
  setup(props, { expose }) {
    // 使用组合式函数
    const mainTabulatorComposable = useMainTabulator();
    const subTabulatorComposable = useSubTabulator();

    // 主表格数据管理
    const fbfxData = useFbfxData();
    const { tableTreeData, tableColumns } = fbfxData;

    // 子表格专门的组合式函数
    const rcjDetail = useRcjDetail();
    const priceStructure = usePriceStructure();
    const standardConversion = useStandardConversion();

    // 获取子表格数据和配置
    const detailData = rcjDetail.detailData;
    const detailColumns = rcjDetail.detailColumns;

    const statisticsData = priceStructure.structureData;
    const statisticsColumns = priceStructure.structureColumns;

    const historyData = standardConversion.conversionHistory;
    const historyColumns = standardConversion.historyColumns;

    // 主表格引用直接在这里定义
    const mainTableRef = ref<HTMLElement>();

    // 表格管理
    const fbfxTables = useFbfxTables();
    const {
      activeSubTab,
      detailTableRef,
      statisticsTableRef,
      historyTableRef,
      initSubTabulator,
      handleSubTabChange: _handleSubTabChange,
    } = fbfxTables;

    // 业务操作
    const fbfxActions = useFbfxActions(tableTreeData, mainTabulatorComposable);
    const {
      currentTableItem,
      currentRowKey,
      handleColorChange,
      handleRowClick,
      handleAddRow,
      handleAddChild,
      handleEditRow,
      handleDeleteRow,
      handleCopyRow,
      setCurrentRow,
      handleSubTableAddRow,
      handleSubTableEditRow,
      handleSubTableDeleteRow,
      handleSubTableCopyRow,
    } = fbfxActions;

    // 颜色对话框
    const colorDialog = useColorDialog(currentTableItem, handleColorChange);
    const {
      colorDialogVisible,
      colorValue,
      openColorDialog,
      handleColorSubmit,
    } = colorDialog;

    // 工具函数
    const fbfxUtils = useFbfxUtils(tableTreeData);
    const { getFolderCount, getFileCount, getCurrentTime } = fbfxUtils;

    // 子表格事件处理器 - 根据不同标签页使用不同的处理方法
    const getSubTableEventHandlers = (tabName: string) => {
      switch (tabName) {
        case 'detail':
          return {
            onAddRow: () => {
              rcjDetail.addDetailRow();
              ElMessage.success("添加成功");
            },
            onEditRow: (row: any) => {
              rcjDetail.editDetailRow(row);
              ElMessage.info("可以双击单元格进行编辑");
            },
            onDeleteRow: (row: any) => {
              const success = rcjDetail.deleteDetailRow(row);
              if (success) {
                ElMessage.success("删除成功");
              }
            },
            onCopyRow: (row: any) => {
              rcjDetail.copyDetailRow(row);
              ElMessage.success("复制成功");
            },
          };
        case 'statistics':
          return {
            onAddRow: () => {
              priceStructure.addStructureRow();
              ElMessage.success("添加成功");
            },
            onEditRow: (row: any) => {
              priceStructure.editStructureRow(row);
              ElMessage.info("可以双击单元格进行编辑");
            },
            onDeleteRow: (row: any) => {
              const success = priceStructure.deleteStructureRow(row);
              if (success) {
                ElMessage.success("删除成功");
              }
            },
            onCopyRow: (row: any) => {
              priceStructure.copyStructureRow(row);
              ElMessage.success("复制成功");
            },
          };
        case 'history':
          return {
            onAddRow: () => {
              standardConversion.addConversionRule();
              ElMessage.success("添加成功");
            },
            onEditRow: (row: any) => {
              standardConversion.editConversionRule(row);
              ElMessage.info("可以双击单元格进行编辑");
            },
            onDeleteRow: (row: any) => {
              const success = standardConversion.deleteConversionRule(row);
              if (success) {
                ElMessage.success("删除成功");
              }
            },
            onCopyRow: (row: any) => {
              standardConversion.copyConversionRule(row);
              ElMessage.success("复制成功");
            },
          };
        default:
          return {
            onAddRow: handleSubTableAddRow,
            onEditRow: handleSubTableEditRow,
            onDeleteRow: handleSubTableDeleteRow,
            onCopyRow: handleSubTableCopyRow,
          };
      }
    };

    // 简化的子表格初始化函数
    const initSubTabulatorSimple = () => {
      const eventHandlers = getSubTableEventHandlers(activeSubTab.value);
      initSubTabulator(
        subTabulatorComposable,
        detailData,
        statisticsData,
        historyData,
        detailColumns,
        statisticsColumns,
        historyColumns,
        eventHandlers
      );
    };

    // 处理子标签页切换
    const handleSubTabChange = (tabName: string | number) => {
      const eventHandlers = getSubTableEventHandlers(tabName as string);
      _handleSubTabChange(
        tabName,
        subTabulatorComposable,
        detailData,
        statisticsData,
        historyData,
        detailColumns,
        statisticsColumns,
        historyColumns,
        eventHandlers
      );
    };

    // 初始化函数
    const init = (id: string | number | null) => {
      console.log("FbfxView.vue init called with ID:", id);
    };

    // ========== 生命周期 ==========
    onMounted(() => {
      console.log('FbfxView mounted, 开始初始化表格');
      console.log('mainTableRef:', mainTableRef);

      nextTick(() => {
        console.log('nextTick 执行，开始检查主表格容器');
        console.log('主表格容器引用:', mainTableRef.value);

        if (mainTableRef.value) {
          console.log('主表格容器尺寸:', {
            width: mainTableRef.value.offsetWidth,
            height: mainTableRef.value.offsetHeight,
            clientWidth: mainTableRef.value.clientWidth,
            clientHeight: mainTableRef.value.clientHeight
          });

          console.log('原始数据:', tableTreeData.value);
          const convertedData = convertToMainTabulatorFormat(tableTreeData.value);
          console.log('转换后的主表格数据:', convertedData);

          try {
            // 设置主表格的事件处理器，关联useFbfxActions中的函数
            mainTabulatorComposable.setEventHandlers({
              onAddRow: () => {
                handleAddRow();
                ElMessage.success("添加成功");
              },
              onAddChildRow: (parentRow: any) => {
                // 添加子项功能实现
                const newChild = {
                  id: `child_${parentRow.id}_${Date.now()}`,
                  name: `${parentRow.name} 的子项`,
                  type: parentRow.type || "子类型",
                  subType: "新建子项",
                  description: "请编辑描述信息",
                  children: [],
                  _row_color: parentRow._row_color || "#409EFF"
                };

                if (mainTabulatorComposable.addRow(newChild, "bottom", parentRow.id)) {
                  ElMessage.success(`已为 "${parentRow.name}" 添加子项`);
                  // 展开父行以显示新添加的子项
                  setTimeout(() => {
                    mainTabulatorComposable.expandRow(parentRow.id);
                  }, 100);
                } else {
                  ElMessage.error("添加子项失败");
                }
              },
              onEditRow: (row: any) => {
                handleEditRow(row);
              },
              onDeleteRow: (row: any) => {
                handleDeleteRow(row);
              },
              onCopyRow: (row: any) => {
                handleCopyRow(row);
              },
            });

            mainTabulatorComposable.initMainTabulator(
              mainTableRef.value,
              convertedData,
              handleColorChange,
              handleRowClick
            );
            console.log('主表格初始化完成');
          } catch (error) {
            console.error('主表格初始化失败:', error);
          }
        } else {
          console.error('主表格容器未找到 - mainTableRef.value 为:', mainTableRef.value);
        }

        console.log('开始初始化子表格');
        try {
          initSubTabulatorSimple();
          console.log('子表格初始化完成');
        } catch (error) {
          console.error('子表格初始化失败:', error);
        }
      });
    });

    onUnmounted(() => {
      mainTabulatorComposable.destroy();
      subTabulatorComposable.destroy();
    });

    // 监听主表格数据变化
    watch(() => tableTreeData.value, (newData) => {
      const convertedData = convertToMainTabulatorFormat(newData);
      mainTabulatorComposable.updateData(convertedData);
    }, { deep: true });

    // 监听子表格数据变化
    watch(() => detailData.value, (newData) => {
      if (activeSubTab.value === 'detail') {
        subTabulatorComposable.updateData(newData);
      }
    }, { deep: true });

    watch(() => statisticsData.value, (newData) => {
      if (activeSubTab.value === 'statistics') {
        subTabulatorComposable.updateData(newData);
      }
    }, { deep: true });

    watch(() => historyData.value, (newData) => {
      if (activeSubTab.value === 'history') {
        subTabulatorComposable.updateData(newData);
      }
    }, { deep: true });

    expose({ init });

    return {
      // DOM refs
      mainTableRef,
      detailTableRef,
      statisticsTableRef,
      historyTableRef,
      // 主表格数据
      tableTreeData,
      tableColumns,
      // 子表格数据（使用新的命名避免冲突）
      activeSubTab,
      rcjDetailData: detailData,
      priceStructureData: statisticsData,
      conversionHistoryData: historyData,
      rcjDetailColumns: detailColumns,
      priceStructureColumns: statisticsColumns,
      conversionHistoryColumns: historyColumns,
      // 颜色对话框
      colorDialogVisible,
      colorValue,
      // 事件处理
      handleSubTabChange,
      handleAddRow,
      handleAddChild,
      handleEditRow,
      handleDeleteRow,
      handleCopyRow,
      handleSubTableAddRow,
      handleSubTableEditRow,
      handleSubTableDeleteRow,
      handleSubTableCopyRow,
      handleColorSubmit,
      openColorDialog,
      // 工具函数
      getFolderCount,
      getFileCount,
      getCurrentTime,
      convertToMainTabulatorFormat,
      // 暴露专门的子表格组合式函数的方法（排除已经重新命名的数据）
      totalAmount: rcjDetail.totalAmount,
      categoryStats: rcjDetail.categoryStats,
      addDetailRow: rcjDetail.addDetailRow,
      deleteDetailRow: rcjDetail.deleteDetailRow,
      copyDetailRow: rcjDetail.copyDetailRow,
      editDetailRow: rcjDetail.editDetailRow,
      updateTotalPrice: rcjDetail.updateTotalPrice,
      exportDetailData: rcjDetail.exportDetailData,
      // 单价构成相关方法
      structureTotalAmount: priceStructure.totalAmount,
      directCost: priceStructure.directCost,
      indirectCost: priceStructure.indirectCost,
      addStructureRow: priceStructure.addStructureRow,
      deleteStructureRow: priceStructure.deleteStructureRow,
      copyStructureRow: priceStructure.copyStructureRow,
      editStructureRow: priceStructure.editStructureRow,
      updateAmount: priceStructure.updateAmount,
      updateCoefficients: priceStructure.updateCoefficients,
      calculatePercentages: priceStructure.calculatePercentages,
      getPriceStructureReport: priceStructure.getPriceStructureReport,
      // 标准换算相关方法
      conversionRules: standardConversion.conversionRules,
      activeRules: standardConversion.activeRules,
      performConversion: standardConversion.performConversion,
      addConversionRule: standardConversion.addConversionRule,
      deleteConversionRule: standardConversion.deleteConversionRule,
      copyConversionRule: standardConversion.copyConversionRule,
      editConversionRule: standardConversion.editConversionRule,
      clearHistory: standardConversion.clearHistory,
      exportRules: standardConversion.exportRules,
      importRules: standardConversion.importRules,
      batchConversion: standardConversion.batchConversion,
      // 暴露主表格组合式函数的所有方法
      ...mainTabulatorComposable,
    };
  },
});
</script>

<template>
  <div class="fbfx-view-container">
    <!-- 颜色选择对话框 -->
    <el-dialog v-model="colorDialogVisible" title="设置边框颜色" width="300px">
      <el-color-picker v-model="colorValue" />
      <template #footer>
        <el-button @click="colorDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleColorSubmit">确定</el-button>
      </template>
    </el-dialog>

    <!-- 上半部分：主要表格内容 -->
    <div class="main-content-section">
      <div ref="mainTableRef" class="main-table-container"></div>
    </div>

    <!-- 下半部分：子标签页内容 -->
    <div class="sub-tabs-section">
      <el-tabs v-model="activeSubTab" type="card" @tab-change="handleSubTabChange">
        <el-tab-pane label="人材机明细" name="detail">
          <div ref="detailTableRef" class="sub-table-container"></div>
        </el-tab-pane>
        <el-tab-pane label="单价构成" name="statistics">
          <div ref="statisticsTableRef" class="sub-table-container"></div>
        </el-tab-pane>
        <el-tab-pane label="标准换算" name="history">
          <div ref="historyTableRef" class="sub-table-container"></div>
        </el-tab-pane>
      </el-tabs>
    </div>

    <!-- 底部统计信息 -->
    <div class="footer-stats">
      <div class="stats-container">
        <div class="stat-item">
          <span class="stat-label">总文件数</span>
          <span class="stat-value">{{ getFolderCount() + getFileCount() }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">文件夹数</span>
          <span class="stat-value">{{ getFolderCount() }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">总大小</span>
          <span class="stat-value">{{ (getFolderCount() + getFileCount()) * 0.001 }} MB</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">最近修改</span>
          <span class="stat-value">{{ getCurrentTime() }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fbfx-view-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 上半部分：主要表格内容 */
.main-content-section {
  height: 55%;
  flex-shrink: 0;
  border-bottom: 1px solid #e4e7ed;
  overflow: hidden;
  margin-bottom: 8px;
  min-height: 300px;
  /* 确保最小高度 */
}

/* 下半部分：子标签页内容 */
.sub-tabs-section {
  height: calc(45% - 8px);
  flex-shrink: 0;
  background: #fff;
  overflow: hidden;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

/* 底部统计信息 */
.footer-stats {
  height: 32px;
  flex-shrink: 0;
  background: #f5f7fa;
  border-top: 1px solid #e4e7ed;
  display: flex;
  align-items: center;
  padding: 0 16px;
}

.stats-container {
  display: flex;
  align-items: center;
  gap: 16px;
  white-space: nowrap;
  overflow: hidden;
}

.stat-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.stat-label {
  font-size: 12px;
  color: #606266;
  font-weight: 500;
}

.stat-value {
  font-size: 12px;
  color: #303133;
  font-weight: 600;
}

.stat-value.highlight {
  color: #409eff;
  font-weight: 700;
}

.stat-separator {
  color: #dcdfe6;
  font-size: 12px;
  margin: 0 4px;
}

/* 主要表格内容的 RightTablePanel 样式调整 */
.main-content-section :deep(.table-panel-container) {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.main-content-section :deep(.el-table) {
  flex: 1;
  height: 100%;
}

.main-content-section :deep(.el-table .el-table__body-wrapper) {
  max-height: calc(100% - 40px);
  overflow-y: auto;
}

.main-content-section :deep(.el-table__header-wrapper) {
  flex-shrink: 0;
}

/* 子标签页中的 RightTablePanel 样式调整 */
.sub-tabs-section :deep(.table-panel-container) {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.sub-tabs-section :deep(.el-table) {
  flex: 1;
  font-size: 12px;
  height: 100%;
}

.sub-tabs-section :deep(.el-table .el-table__body-wrapper) {
  max-height: calc(100% - 40px);
  overflow-y: auto;
  min-height: 200px;
}

.sub-tabs-section :deep(.el-table th) {
  padding: 8px 0;
  font-size: 12px;
}

.sub-tabs-section :deep(.el-table td) {
  padding: 6px 0;
  font-size: 12px;
}

/* 标签页样式调整 */
:deep(.el-tabs__header) {
  margin: 0;
  border-bottom: 1px solid #e4e7ed;
}

:deep(.el-tabs__content) {
  height: calc(100% - 40px);
  padding: 8px;
  overflow: hidden;
}

:deep(.el-tab-pane) {
  height: calc(100% - 16px);
  overflow: hidden;
}

/* 表格容器样式 */
.main-table-container {
  width: 100%;
  height: 100%;
  min-height: 280px;
}

.sub-table-container {
  width: 100%;
  height: 300px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
}

.main-table-container {
  /* 移除调试样式，保持正常显示 */
}

/* Tabulator 自定义样式 */
:deep(.tabulator) {
  border: none !important;
  border-radius: 0;
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
