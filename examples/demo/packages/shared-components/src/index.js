// 导入样式文件
import './styles/simple-header.css'
import './styles/modal-window-header.css'

// 导出STable相关
export { STable, setupSTable } from './plugins/stable.js'

// 导出共享组件
export { default as CostTable } from './components/CostTable.vue'
export { default as CostForm } from './components/CostForm.vue'
export { default as FormWindow } from './components/FormWindow.vue'
export { default as SimpleFormWindow } from './components/SimpleFormWindow.vue'
export { default as ModalWindowHeader } from './components/ModalWindowHeader.vue'

// 导出共享的composables
export { useCostCalculation } from './composables/useCostCalculation.js'
export { useTableOperations } from './composables/useTableOperations.js'

// 窗口管理 - 新的简化稳定版本
export { 
  useSimpleWindowManagement,
  useMainWindowManagement, 
  useChildWindowManagement
} from './composables/useSimpleWindowManagement.js'
export {
  useWindowDataExchange,
  useChildWindowDataExchange,
  useParentWindowDataExchange
} from './composables/useWindowDataExchange.js'
export {
  useUniversalWindowManager,
  useGlobalWindowManager,
  useChildAppWindowManager,
  useMainAppWindowManager
} from './composables/useUniversalWindowManager.js'

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

// 导出布局组件
export { AppHeader, SimpleHeader } from './layouts/index.js'

// 导出Web环境工具函数
export {
  isTauriEnvironment,
  supportsFullscreen,
  isFullscreen,
  enterFullscreen,
  exitFullscreen,
  toggleFullscreen,
  simulateMinimize,
  WebWindowController,
  webWindowController
} from './utils/webEnvironment.js'


