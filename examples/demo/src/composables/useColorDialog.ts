// 颜色对话框管理组合式函数
import { ref } from "vue";
import { ElMessage } from "element-plus";
import type { TreeTableData } from "./useFbfxData";

export function useColorDialog(
  currentTableItem: any,
  handleColorChange: (id: string, color: string) => void
) {
  const colorDialogVisible = ref(false);
  const colorValue = ref("#409EFF");

  // 打开颜色选择对话框
  const openColorDialog = () => {
    colorValue.value = currentTableItem.value?.color || "#409EFF";
    colorDialogVisible.value = true;
  };

  // 处理颜色提交
  const handleColorSubmit = () => {
    if (!colorValue.value) {
      ElMessage.warning("请选择颜色");
      return;
    }
    if (currentTableItem.value) {
      // 使用传入的颜色更新功能
      handleColorChange(String(currentTableItem.value.id), colorValue.value);
    }
    colorDialogVisible.value = false;
  };

  return {
    colorDialogVisible,
    colorValue,
    openColorDialog,
    handleColorSubmit,
  };
}
