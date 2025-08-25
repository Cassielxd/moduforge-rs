import { getHistory } from "@/api/common";
import { defineStore } from "pinia";
import { ref } from "vue";

// 历史记录项类型
export interface HistoryItem {
  state_version: number;
  description: string;
  timestamp: string;
  type: "创建" | "修改" | "删除";
}

// 全局历史记录 store
export const useHistoryStore = defineStore("history", {
  state: () => ({ historyList: [] as HistoryItem[] }),
  actions: {
    addHistory(item: HistoryItem) {
      this.historyList.unshift(item);
    },
    clearHistory() {
      this.historyList = [];
    },
    setHistoryList(list: HistoryItem[]) {
      this.historyList = list;
    },
    async refreshHistory(id: string) {
      const res = await getHistory({ editor_name: id });
      this.historyList = res as HistoryItem[];
    },
  },
});
