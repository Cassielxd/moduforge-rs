// 标准换算组合式函数
import { ref, computed } from "vue";

export function useStandardConversion() {
  // 换算规则数据
  const conversionRules = ref([
    {
      id: 1,
      itemName: "混凝土",
      originalUnit: "m³",
      targetUnit: "吨",
      conversionFactor: 2.4,
      formula: "体积(m³) × 2.4 = 重量(吨)",
      remark: "C30混凝土密度",
      isActive: true,
    },
    {
      id: 2,
      itemName: "钢筋",
      originalUnit: "吨",
      targetUnit: "根",
      conversionFactor: 0.617,
      formula: "重量(吨) ÷ 0.617 = 根数",
      remark: "Φ12钢筋，每米重量0.617kg",
      isActive: true,
    },
    {
      id: 3,
      itemName: "砂石",
      originalUnit: "m³",
      targetUnit: "吨",
      conversionFactor: 1.5,
      formula: "体积(m³) × 1.5 = 重量(吨)",
      remark: "天然砂石堆积密度",
      isActive: true,
    },
    {
      id: 4,
      itemName: "砖块",
      originalUnit: "m³",
      targetUnit: "块",
      conversionFactor: 512,
      formula: "体积(m³) × 512 = 块数",
      remark: "标准红砖240×115×53mm",
      isActive: true,
    },
  ]);

  // 换算历史记录
  const conversionHistory = ref([
    {
      id: 1,
      timestamp: "2024-07-29 15:30:22",
      operation: "正向换算",
      itemName: "混凝土",
      originalValue: 100,
      originalUnit: "m³",
      convertedValue: 240,
      targetUnit: "吨",
      conversionFactor: 2.4,
      operator: "张三",
    },
    {
      id: 2,
      timestamp: "2024-07-29 14:20:15",
      operation: "反向换算",
      itemName: "钢筋",
      originalValue: 500,
      originalUnit: "根",
      convertedValue: 308.5,
      targetUnit: "吨",
      conversionFactor: 0.617,
      operator: "李四",
    },
  ]);

  // 换算规则表格列配置（兼容useSubTabulator格式） - 暂时使用换算历史的数据和列
  const rulesColumns = [
    { prop: "timestamp", label: "时间", width: 150 },
    { prop: "operation", label: "操作", width: 80 },
    { prop: "itemName", label: "项目", width: 100 },
    {
      prop: "originalValue",
      label: "原始值",
      width: 80,
      align: "right",
    },
    { prop: "originalUnit", label: "原单位", width: 60 },
    {
      prop: "convertedValue",
      label: "转换值",
      width: 80,
      align: "right",
    },
    { prop: "targetUnit", label: "目标单位", width: 60 },
    { prop: "operator", label: "操作人", width: 80 },
  ];

  // 换算历史表格列配置（兼容useSubTabulator格式）
  const historyColumns = [
    { prop: "timestamp", label: "时间", width: 150 },
    { prop: "operation", label: "操作", width: 80 },
    { prop: "itemName", label: "项目", width: 100 },
    {
      prop: "originalValue",
      label: "原始值",
      width: 80,
      align: "right",
    },
    { prop: "originalUnit", label: "原单位", width: 60 },
    {
      prop: "convertedValue",
      label: "转换值",
      width: 80,
      align: "right",
    },
    { prop: "targetUnit", label: "目标单位", width: 60 },
    { prop: "operator", label: "操作人", width: 80 },
  ];

  // 活跃的换算规则
  const activeRules = computed(() => {
    return conversionRules.value.filter((rule) => rule.isActive);
  });

  // 执行换算
  const performConversion = (
    itemName,
    originalValue,
    isReverse = false,
    operator = "当前用户"
  ) => {
    const rule = conversionRules.value.find(
      (r) => r.itemName === itemName && r.isActive
    );
    if (!rule) {
      throw new Error(`未找到项目 ${itemName} 的换算规则`);
    }

    let convertedValue;
    let operation;
    let originalUnit;
    let targetUnit;

    if (isReverse) {
      // 反向换算
      convertedValue = originalValue / rule.conversionFactor;
      operation = "反向换算";
      originalUnit = rule.targetUnit;
      targetUnit = rule.originalUnit;
    } else {
      // 正向换算
      convertedValue = originalValue * rule.conversionFactor;
      operation = "正向换算";
      originalUnit = rule.originalUnit;
      targetUnit = rule.targetUnit;
    }

    // 添加到历史记录
    const historyRecord = {
      id: Date.now(),
      timestamp: new Date().toLocaleString(),
      operation,
      itemName,
      originalValue,
      originalUnit,
      convertedValue,
      targetUnit,
      conversionFactor: rule.conversionFactor,
      operator,
    };

    conversionHistory.value.unshift(historyRecord);

    // 限制历史记录数量
    if (conversionHistory.value.length > 100) {
      conversionHistory.value = conversionHistory.value.slice(0, 100);
    }

    return {
      originalValue,
      originalUnit,
      convertedValue,
      targetUnit,
      conversionFactor: rule.conversionFactor,
      formula: rule.formula,
    };
  };

  // 添加换算规则
  const addConversionRule = (currentRow) => {
    const newRule = {
      id: Date.now(),
      itemName: "新项目",
      originalUnit: "单位1",
      targetUnit: "单位2",
      conversionFactor: 1,
      formula: "原值 × 1 = 转换值",
      remark: "",
      isActive: true,
    };

    if (currentRow) {
      const currentIndex = conversionRules.value.findIndex(
        (item) => item.id === currentRow.id
      );
      if (currentIndex !== -1) {
        conversionRules.value.splice(currentIndex + 1, 0, newRule);
      } else {
        conversionRules.value.push(newRule);
      }
    } else {
      conversionRules.value.push(newRule);
    }

    return newRule;
  };

  // 删除换算规则
  const deleteConversionRule = (row) => {
    const index = conversionRules.value.findIndex((item) => item.id === row.id);
    if (index > -1) {
      conversionRules.value.splice(index, 1);
      return true;
    }
    return false;
  };

  // 复制换算规则
  const copyConversionRule = (row) => {
    const newRule = {
      ...row,
      id: Date.now(),
      itemName: `${row.itemName} (复制)`,
    };
    conversionRules.value.push(newRule);
    return newRule;
  };

  // 编辑换算规则
  const editConversionRule = (row) => {
    return row;
  };

  // 清空历史记录
  const clearHistory = () => {
    conversionHistory.value = [];
  };

  // 导出换算规则
  const exportRules = () => {
    return {
      rules: conversionRules.value,
      activeRulesCount: activeRules.value.length,
      totalRulesCount: conversionRules.value.length,
    };
  };

  // 导入换算规则
  const importRules = (rules) => {
    conversionRules.value = [...conversionRules.value, ...rules];
  };

  // 批量执行换算
  const batchConversion = (
    conversions
  ) => {
    const results = [];
    for (const conversion of conversions) {
      try {
        const result = performConversion(
          conversion.itemName,
          conversion.originalValue,
          conversion.isReverse || false,
          "当前用户"
        );
        results.push({ success: true, result });
      } catch (error) {
        results.push({ success: false, error: (error ).message });
      }
    }
    return results;
  };

  return {
    // 数据
    conversionRules,
    conversionHistory,
    rulesColumns,
    historyColumns,
    // 计算属性
    activeRules,
    // 方法
    performConversion,
    addConversionRule,
    deleteConversionRule,
    copyConversionRule,
    editConversionRule,
    clearHistory,
    exportRules,
    importRules,
    batchConversion,
  };
}
