// 导入样式文件
import './styles/simple-header.css'
import './styles/project-header.css'
import './styles/modal-window-header.css'

// 导出STable相关
export { STable, setupSTable } from './plugins/stable.js'

// 导出共享组件
export { default as CostTable } from './components/CostTable.vue'
export { default as TableContextMenu } from './components/TableContextMenu.vue'
export { default as CostForm } from './components/CostForm.vue'

export { default as ModalWindowHeader } from './components/ModalWindowHeader.vue'

// 导出窗口控制 composable
export { useWindowControls } from './composables/useWindowControls.js'

// 导出操作窗口组件
export { default as DataImportWindow } from './components/operate/DataImportWindow.vue'
export { default as DataExportWindow } from './components/operate/DataExportWindow.vue'
export { default as BatchOperationWindow } from './components/operate/BatchOperationWindow.vue'
export { default as FormWindow } from './components/operate/FormWindow.vue'

// 导出共享的composables
export { useCostCalculation } from './composables/useCostCalculation.js'
export { useTableOperations } from './composables/useTableOperations.js'
export { useTableEditing } from './composables/useTableEditing.js'
export { useTableSelection } from './composables/useTableSelection.js'
export { useTreeTable } from './composables/useTreeTable.js'



// 导出通用Tauri窗口管理
export {
  useOperateWindowManager,
  useGlobalOperateWindow,
  registerOperateWindow,
  operateWindowPresets
} from './composables/useOperateWindowManager.js'



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
export { 
  AppHeader, 
  SimpleHeader, 
  ProjectHeaderExample,
  AppLayout, 
  AsideTree, 
  OperateBar, 
  op 
} from './layouts/index.js'

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


