// 人材机明细组合式函数
import { ref, computed } from "vue";

export interface RcjDetailItem {
  id: number | string;
  category: string; // 类别：人工、材料、机械
  name: string; // 名称
  specification: string; // 规格
  unit: string; // 单位
  quantity: number; // 数量
  unitPrice: number; // 单价
  totalPrice: number; // 合价
  remark?: string; // 备注
  children?: RcjDetailItem[]; // 子项
}

export function useRcjDetail() {
  // 人材机明细数据
  const detailData = ref<RcjDetailItem[]>([
    {
      id: 1,
      category: "人工",
      name: "普通工",
      specification: "8小时工日",
      unit: "工日",
      quantity: 2.5,
      unitPrice: 150.0,
      totalPrice: 375.0,
      remark: "技术工人",
      children: [
        {
          id: 11,
          category: "人工",
          name: "普通工",
          specification: "8小时工日",
          unit: "工日",
          quantity: 2.5,
          unitPrice: 150.0,
          totalPrice: 375.0,
          remark: "技术工人",
        },
      ],
    },
    {
      id: 2,
      category: "材料",
      name: "水泥",
      specification: "P.O42.5",
      unit: "吨",
      quantity: 1.2,
      unitPrice: 480.0,
      totalPrice: 576.0,
      remark: "普通硅酸盐水泥",
    },
    {
      id: 3,
      category: "机械",
      name: "混凝土搅拌机",
      specification: "0.4m³",
      unit: "台班",
      quantity: 0.8,
      unitPrice: 320.0,
      totalPrice: 256.0,
      remark: "小型搅拌机",
    },
  ]);

  // 表格列配置（兼容useSubTabulator格式）
  const detailColumns = [
    { prop: "category", label: "类别", width: 80 },
    { prop: "name", label: "名称", width: 150 },
    { prop: "specification", label: "规格", width: 120 },
    { prop: "unit", label: "单位", width: 80 },
    { prop: "quantity", label: "数量", width: 80, align: "right" as const },
    { prop: "unitPrice", label: "单价", width: 100, align: "right" as const },
    { prop: "totalPrice", label: "合价", width: 100, align: "right" as const },
    { prop: "remark", label: "备注", width: 150 },
  ];

  // 计算总价
  const totalAmount = computed(() => {
    return detailData.value.reduce((sum, item) => sum + item.totalPrice, 0);
  });

  // 按类别统计
  const categoryStats = computed(() => {
    const stats = {
      人工: { count: 0, amount: 0 },
      材料: { count: 0, amount: 0 },
      机械: { count: 0, amount: 0 },
    };

    detailData.value.forEach((item) => {
      if (stats[item.category as keyof typeof stats]) {
        stats[item.category as keyof typeof stats].count++;
        stats[item.category as keyof typeof stats].amount += item.totalPrice;
      }
    });

    return stats;
  });

  // 添加行
  const addDetailRow = (currentRow?: RcjDetailItem) => {
    const newRow: RcjDetailItem = {
      id: Date.now(),
      category: "人工",
      name: "新项目",
      specification: "",
      unit: "个",
      quantity: 1,
      unitPrice: 0,
      totalPrice: 0,
      remark: "",
    };

    if (currentRow) {
      const currentIndex = detailData.value.findIndex(
        (item) => item.id === currentRow.id
      );
      if (currentIndex !== -1) {
        detailData.value.splice(currentIndex + 1, 0, newRow);
      } else {
        detailData.value.push(newRow);
      }
    } else {
      detailData.value.push(newRow);
    }

    return newRow;
  };

  // 删除行
  const deleteDetailRow = (row: RcjDetailItem) => {
    const index = detailData.value.findIndex((item) => item.id === row.id);
    if (index > -1) {
      detailData.value.splice(index, 1);
      return true;
    }
    return false;
  };

  // 复制行
  const copyDetailRow = (row: RcjDetailItem) => {
    const newRow: RcjDetailItem = {
      ...row,
      id: Date.now(),
      name: `${row.name} (复制)`,
    };
    detailData.value.push(newRow);
    return newRow;
  };

  // 编辑行
  const editDetailRow = (row: RcjDetailItem) => {
    // 这里可以触发编辑模式或显示编辑弹窗
    return row;
  };

  // 更新合价（当数量或单价变化时）
  const updateTotalPrice = (row: RcjDetailItem) => {
    row.totalPrice = row.quantity * row.unitPrice;
  };

  // 导出数据（用于报表）
  const exportDetailData = () => {
    return {
      items: detailData.value,
      totalAmount: totalAmount.value,
      categoryStats: categoryStats.value,
    };
  };

  return {
    // 数据
    detailData,
    detailColumns,
    // 计算属性
    totalAmount,
    categoryStats,
    // 方法
    addDetailRow,
    deleteDetailRow,
    copyDetailRow,
    editDetailRow,
    updateTotalPrice,
    exportDetailData,
  };
}
