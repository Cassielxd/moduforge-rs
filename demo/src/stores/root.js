import { defineStore } from "pinia";

// 全局根节点 ID store
export const useRootStore = defineStore("root", {
  state: () => ({
    rootId: undefined,
  }),
  actions: {
    setRootId(id) {
      this.rootId = id;
    },
    clearRootId() {
      this.rootId = undefined;
    },
  },
  getters: {
    getRootId: (state) => state.rootId,
  },
});
