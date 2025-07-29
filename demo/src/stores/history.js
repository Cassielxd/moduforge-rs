import { getHistory } from "@/api/common";
import { defineStore } from "pinia";
import { ref } from "vue";


// 全局历史记录 store
export const useHistoryStore = defineStore("history", {
  state: () => ({ historyList: []}),
  actions: {
    addHistory(item) {
      this.historyList.unshift(item);
    },
    clearHistory() {
      this.historyList = [];
    },
    setHistoryList(list) {
      this.historyList = list;
    },
    async refreshHistory(id) {
      const res = await getHistory({ editor_name: id });
      this.historyList = res;
    },
  },
});
