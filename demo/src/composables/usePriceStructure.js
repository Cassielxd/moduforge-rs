// 单价构成组合式函数
import { ref, computed } from "vue";

export function usePriceStructure() {
  // 单价构成数据
  const structureData = ref([
    {
      id: 1,
      component: "人工费",
      basePrice: 150.0,
      coefficient: 2.5,
      amount: 375.0,
      percentage: 0,
      description: "普通工日单价",
    },
    {
      id: 2,
      component: "材料费",
      basePrice: 480.0,
      coefficient: 1.2,
      amount: 576.0,
      percentage: 0,
      description: "主要材料费用",
    },
    {
      id: 3,
      component: "机械费",
      basePrice: 320.0,
      coefficient: 0.8,
      amount: 256.0,
      percentage: 0,
      description: "机械使用费",
    },
    {
      id: 4,
      component: "管理费",
      basePrice: 1207.0,
      coefficient: 0.08,
      amount: 96.56,
      percentage: 0,
      description: "现场管理费",
    },
    {
      id: 5,
      component: "利润",
      basePrice: 1207.0,
      coefficient: 0.05,
      amount: 60.35,
      percentage: 0,
      description: "企业利润",
    },
  ]);

  // 表格列配置（兼容useSubTabulator格式）
  const structureColumns = [
    { prop: "component", label: "构成要素", width: 120 },
    { prop: "basePrice", label: "基价", width: 100, align: "right"  },
    { prop: "coefficient", label: "系数", width: 80, align: "right"  },
    { prop: "amount", label: "金额", width: 100, align: "right"  },
    {
      prop: "percentage",
      label: "占比(%)",
      width: 100,
      align: "right" ,
    },
    { prop: "description", label: "说明", width: 150 },
  ];

  // 计算总金额
  const totalAmount = computed(() => {
    return structureData.value.reduce((sum, item) => sum + item.amount, 0);
  });

  // 计算各项占比
  const calculatePercentages = () => {
    const total = totalAmount.value;
    if (total > 0) {
      structureData.value.forEach((item) => {
        item.percentage = (item.amount / total) * 100;
      });
    }
  };

  // 人材机费用小计
  const directCost = computed(() => {
    return structureData.value
      .filter((item) => ["人工费", "材料费", "机械费"].includes(item.component))
      .reduce((sum, item) => sum + item.amount, 0);
  });

  // 间接费用小计
  const indirectCost = computed(() => {
    return structureData.value
      .filter((item) => ["管理费", "利润"].includes(item.component))
      .reduce((sum, item) => sum + item.amount, 0);
  });

  // 添加构成要素
  const addStructureRow = (currentRow) => {
    const newRow = {
      id: Date.now(),
      component: "新要素",
      basePrice: 0,
      coefficient: 1,
      amount: 0,
      percentage: 0,
      description: "",
    };

    if (currentRow) {
      const currentIndex = structureData.value.findIndex(
        (item) => item.id === currentRow.id
      );
      if (currentIndex !== -1) {
        structureData.value.splice(currentIndex + 1, 0, newRow);
      } else {
        structureData.value.push(newRow);
      }
    } else {
      structureData.value.push(newRow);
    }

    calculatePercentages();
    return newRow;
  };

  // 删除构成要素
  const deleteStructureRow = (row) => {
    const index = structureData.value.findIndex((item) => item.id === row.id);
    if (index > -1) {
      structureData.value.splice(index, 1);
      calculatePercentages();
      return true;
    }
    return false;
  };

  // 复制构成要素
  const copyStructureRow = (row) => {
    const newRow = {
      ...row,
      id: Date.now(),
      component: `${row.component} (复制)`,
    };
    structureData.value.push(newRow);
    calculatePercentages();
    return newRow;
  };

  // 编辑构成要素
  const editStructureRow = (row) => {
    return row;
  };

  // 更新金额（当基价或系数变化时）
  const updateAmount = (row) => {
    row.amount = row.basePrice * row.coefficient;
    calculatePercentages();
  };

  // 批量更新系数（用于调价）
  const updateCoefficients = (adjustmentRate) => {
    structureData.value.forEach((item) => {
      if (!["管理费", "利润"].includes(item.component)) {
        item.coefficient *= 1 + adjustmentRate;
        item.amount = item.basePrice * item.coefficient;
      }
    });
    calculatePercentages();
  };

  // 获取单价构成报告
  const getPriceStructureReport = () => {
    return {
      items: structureData.value,
      totalAmount: totalAmount.value,
      directCost: directCost.value,
      indirectCost: indirectCost.value,
      directCostPercentage:
        totalAmount.value > 0
          ? (directCost.value / totalAmount.value) * 100
          : 0,
      indirectCostPercentage:
        totalAmount.value > 0
          ? (indirectCost.value / totalAmount.value) * 100
          : 0,
    };
  };

  // 初始化时计算占比
  calculatePercentages();

  return {
    // 数据
    structureData,
    structureColumns,
    // 计算属性
    totalAmount,
    directCost,
    indirectCost,
    // 方法
    addStructureRow,
    deleteStructureRow,
    copyStructureRow,
    editStructureRow,
    updateAmount,
    updateCoefficients,
    calculatePercentages,
    getPriceStructureReport,
  };
}
