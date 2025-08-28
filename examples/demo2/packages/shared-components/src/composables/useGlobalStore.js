import { globalState, getters, actions, persistence } from '../store/index.js'
import { onMounted, onUnmounted, watch } from 'vue'

/**
 * 全局状态管理组合式函数
 * 提供便捷的状态访问和操作方法
 */

// 用户相关
export function useUser() {
  return {
    user: globalState.user,
    isLoggedIn: getters.isLoggedIn,
    hasPermission: getters.hasPermission,
    setUser: actions.setUser,
    clearUser: actions.clearUser
  }
}

// 应用配置相关
export function useAppConfig() {
  return {
    config: globalState.app,
    theme: globalState.app.theme,
    language: globalState.app.language,
    sidebarCollapsed: globalState.app.sidebarCollapsed,
    setTheme: actions.setTheme,
    setLanguage: actions.setLanguage,
    toggleSidebar: actions.toggleSidebar
  }
}

// 概算数据相关
export function useEstimate() {
  return {
    // 状态
    currentProject: globalState.estimate.currentProject,
    projects: globalState.estimate.projects,
    selectedItems: globalState.estimate.selectedItems,
    filters: globalState.estimate.filters,
    
    // 计算属性
    totalProjects: getters.totalProjects,
    selectedCount: getters.selectedCount,
    filteredProjects: getters.filteredProjects,
    
    // 操作方法
    setCurrentProject: actions.setCurrentProject,
    addProject: actions.addProject,
    updateProject: actions.updateProject,
    deleteProject: actions.deleteProject,
    setProjects: actions.setProjects,
    
    // 选择管理
    selectItems: actions.selectItems,
    addToSelection: actions.addToSelection,
    removeFromSelection: actions.removeFromSelection,
    clearSelection: actions.clearSelection,
    
    // 筛选器
    setFilter: actions.setFilter,
    clearFilters: actions.clearFilters
  }
}

// 表单窗口管理
export function useFormWindows() {
  return {
    activeWindows: globalState.form.activeWindows,
    drafts: globalState.form.drafts,
    lastSaved: globalState.form.lastSaved,
    activeWindowCount: getters.activeWindowCount,
    
    registerWindow: actions.registerWindow,
    unregisterWindow: actions.unregisterWindow,
    saveDraft: actions.saveDraft,
    getDraft: actions.getDraft,
    clearDraft: actions.clearDraft
  }
}

// 通知管理
export function useNotifications() {
  return {
    notifications: globalState.notifications,
    unreadCount: getters.unreadNotifications,
    
    addNotification: actions.addNotification,
    markRead: actions.markNotificationRead,
    clear: actions.clearNotifications
  }
}

// 加载状态管理
export function useLoading() {
  return {
    loading: globalState.loading,
    setLoading: actions.setLoading,
    
    // 便捷方法
    setGlobalLoading: (value) => actions.setLoading('global', value),
    setEstimateLoading: (value) => actions.setLoading('estimate', value),
    setFormLoading: (value) => actions.setLoading('form', value)
  }
}

// 持久化管理
export function usePersistence() {
  // 自动保存
  const startAutoSave = (interval = 30000) => { // 默认30秒
    const timer = setInterval(() => {
      persistence.save()
    }, interval)
    
    onUnmounted(() => {
      clearInterval(timer)
    })
    
    return timer
  }
  
  // 监听状态变化自动保存
  const watchAndSave = () => {
    // 监听用户状态变化
    watch(() => globalState.user, () => {
      persistence.save()
    }, { deep: true })
    
    // 监听应用配置变化
    watch(() => globalState.app, () => {
      persistence.save()
    }, { deep: true })
    
    // 监听概算数据变化
    watch(() => globalState.estimate.projects, () => {
      persistence.save()
    }, { deep: true })
  }
  
  return {
    save: persistence.save,
    load: persistence.load,
    clear: persistence.clear,
    startAutoSave,
    watchAndSave
  }
}

// 完整的状态管理钩子
export function useGlobalStore() {
  const user = useUser()
  const appConfig = useAppConfig()
  const estimate = useEstimate()
  const formWindows = useFormWindows()
  const notifications = useNotifications()
  const loading = useLoading()
  const persistence = usePersistence()
  
  // 初始化
  onMounted(() => {
    // 加载持久化数据
    persistence.load()
    
    // 开始自动保存
    persistence.startAutoSave()
    
    // 监听变化自动保存
    persistence.watchAndSave()
  })
  
  return {
    // 状态
    state: globalState,
    getters,
    
    // 分类功能
    user,
    appConfig,
    estimate,
    formWindows,
    notifications,
    loading,
    persistence,
    
    // 全局操作
    actions
  }
}

// 数据同步相关
export function useDataSync() {
  // 跨窗口数据同步
  const syncAcrossWindows = () => {
    // 监听 storage 事件实现跨窗口同步
    const handleStorageChange = (e) => {
      if (e.key === 'cost-app-state' && e.newValue) {
        try {
          const newData = JSON.parse(e.newValue)
          
          // 更新本地状态
          if (newData.user) {
            Object.assign(globalState.user, newData.user)
          }
          if (newData.app) {
            Object.assign(globalState.app, newData.app)
          }
          if (newData.estimate) {
            if (newData.estimate.projects) {
              globalState.estimate.projects = newData.estimate.projects
            }
          }
        } catch (error) {
          console.error('同步数据失败:', error)
        }
      }
    }
    
    window.addEventListener('storage', handleStorageChange)
    
    onUnmounted(() => {
      window.removeEventListener('storage', handleStorageChange)
    })
  }
  
  // 广播数据变化
  const broadcastChange = (type, data) => {
    // 使用 BroadcastChannel API 进行跨窗口通信
    if (typeof BroadcastChannel !== 'undefined') {
      const channel = new BroadcastChannel('cost-app-sync')
      channel.postMessage({
        type,
        data,
        timestamp: Date.now()
      })
    }
  }
  
  // 监听广播消息
  const listenToBroadcast = () => {
    if (typeof BroadcastChannel !== 'undefined') {
      const channel = new BroadcastChannel('cost-app-sync')
      
      channel.onmessage = (event) => {
        const { type, data } = event.data
        
        switch (type) {
          case 'project-updated':
            actions.updateProject(data.id, data.updates)
            break
          case 'project-added':
            actions.addProject(data)
            break
          case 'project-deleted':
            actions.deleteProject(data.id)
            break
          case 'selection-changed':
            actions.selectItems(data)
            break
        }
      }
      
      onUnmounted(() => {
        channel.close()
      })
    }
  }
  
  return {
    syncAcrossWindows,
    broadcastChange,
    listenToBroadcast
  }
}

// 调试工具
export function useStoreDebug() {
  const logState = () => {
    console.log('Global State:', globalState)
  }
  
  const logGetters = () => {
    console.log('Getters:', {
      isLoggedIn: getters.isLoggedIn.value,
      totalProjects: getters.totalProjects.value,
      selectedCount: getters.selectedCount.value,
      activeWindowCount: getters.activeWindowCount.value,
      unreadNotifications: getters.unreadNotifications.value
    })
  }
  
  const exportState = () => {
    return JSON.stringify(globalState, null, 2)
  }
  
  const importState = (stateJson) => {
    try {
      const newState = JSON.parse(stateJson)
      Object.assign(globalState, newState)
    } catch (error) {
      console.error('导入状态失败:', error)
    }
  }
  
  return {
    logState,
    logGetters,
    exportState,
    importState
  }
}
