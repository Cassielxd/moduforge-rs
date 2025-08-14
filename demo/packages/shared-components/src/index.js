// 导出共享组件
export { default as CostTable } from './components/CostTable.vue'
export { default as CostForm } from './components/CostForm.vue'

// 导出共享的composables
export { useCostCalculation } from './composables/useCostCalculation.js'
export { useTableOperations } from './composables/useTableOperations.js'

// 导出状态管理
export {
  useGlobalStore,
  useUser,
  useAppConfig,
  useEstimate,
  useFormWindows,
  useNotifications,
  useLoading,
  usePersistence,
  useDataSync,
  useStoreDebug
} from './composables/useGlobalStore.js'

// 导出状态管理核心
export { globalState, getters, actions, persistence } from './store/index.js'
