// 表格初始化和管理组合式函数
import { ref, nextTick } from "vue";

export function useFbfxTables() {
  // 子标签页状态
  const activeSubTab = ref("detail");

  // ========== Tabulator 相关变量 ==========
  const subTableRef = ref<HTMLElement>();
  // 为每个子标签页创建独立的ref
  const detailTableRef = ref<HTMLElement>();
  const statisticsTableRef = ref<HTMLElement>();
  const historyTableRef = ref<HTMLElement>();

  // 初始化子表格
  const initSubTabulator = (
    subTabulatorComposable: any,
    detailData: any,
    statisticsData: any,
    historyData: any,
    detailColumns: any,
    statisticsColumns: any,
    historyColumns: any,
    eventHandlers: any
  ) => {
    console.log("initSubTabulator 被调用, activeSubTab:", activeSubTab.value);

    let currentRef: HTMLElement | undefined;

    // 根据当前活动标签页选择对应的ref
    switch (activeSubTab.value) {
      case "detail":
        currentRef = detailTableRef.value;
        console.log("detailTableRef:", detailTableRef.value);
        break;
      case "statistics":
        currentRef = statisticsTableRef.value;
        console.log("statisticsTableRef:", statisticsTableRef.value);
        break;
      case "history":
        currentRef = historyTableRef.value;
        console.log("historyTableRef:", historyTableRef.value);
        break;
      default:
        currentRef = detailTableRef.value;
        console.log("使用默认detailTableRef:", detailTableRef.value);
    }

    if (!currentRef) {
      console.warn("子表格容器未找到:", activeSubTab.value);
      return;
    }

    let data, columns;
    switch (activeSubTab.value) {
      case "detail":
        data = detailData.value;
        columns = detailColumns;
        break;
      case "statistics":
        data = statisticsData.value;
        columns = statisticsColumns;
        break;
      case "history":
        data = historyData.value;
        columns = historyColumns;
        break;
      default:
        data = detailData.value;
        columns = detailColumns;
    }

    console.log("子表格数据:", data);
    console.log("子表格列配置:", columns);
    console.log("子表格事件处理器:", eventHandlers);

    // 先设置子表格事件处理器
    subTabulatorComposable.setEventHandlers(eventHandlers);

    // 然后使用子表格组合式函数初始化
    subTabulatorComposable.initSubTabulator(
      currentRef,
      activeSubTab.value,
      data,
      columns
    );

    console.log("子表格初始化完成");
  };

  // 处理子标签页切换
  const handleSubTabChange = (
    tabName: string | number,
    subTabulatorComposable: any,
    detailData: any,
    statisticsData: any,
    historyData: any,
    detailColumns: any,
    statisticsColumns: any,
    historyColumns: any,
    eventHandlers: any
  ) => {
    console.log("FbfxView: Sub tab changed to:", tabName);
    activeSubTab.value = tabName as string;

    // 重新初始化子表格
    nextTick(() => {
      console.log("标签页切换后，开始初始化子表格");
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
    });
  };

  return {
    // 状态
    activeSubTab,
    // DOM refs
    subTableRef,
    detailTableRef,
    statisticsTableRef,
    historyTableRef,
    // 方法
    initSubTabulator,
    handleSubTabChange,
  };
}
