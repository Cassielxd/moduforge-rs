import { reactive, ref, computed } from 'vue'

/**
 * 全局共享状态管理
 * 使用 Vue 3 的响应式系统实现简单的状态管理
 */

// 全局状态
const globalState = reactive({
  // 用户信息
  user: {
    id: null,
    name: '',
    role: '',
    permissions: []
  },
  
  // 应用配置
  app: {
    theme: 'light',
    language: 'zh-CN',
    sidebarCollapsed: false
  },
  
  // 概算数据
  estimate: {
    currentProject: null,
    projects: [],
    selectedItems: [],
    filters: {
      status: '',
      dateRange: null,
      searchText: ''
    }
  },
  
  // 表单状态
  form: {
    activeWindows: new Map(), // 活动的表单窗口
    drafts: new Map(), // 草稿数据
    lastSaved: null
  },
  
  // 通知和消息
  notifications: [],
  
  // 加载状态
  loading: {
    global: false,
    estimate: false,
    form: false
  }
})

// 计算属性
const getters = {
  // 当前用户是否已登录
  isLoggedIn: computed(() => !!globalState.user.id),
  
  // 当前用户权限
  hasPermission: computed(() => (permission) => {
    return globalState.user.permissions.includes(permission)
  }),
  
  // 概算项目总数
  totalProjects: computed(() => globalState.estimate.projects.length),
  
  // 已选中的概算项目数量
  selectedCount: computed(() => globalState.estimate.selectedItems.length),
  
  // 过滤后的项目列表
  filteredProjects: computed(() => {
    let projects = globalState.estimate.projects
    const { status, searchText } = globalState.estimate.filters
    
    if (status) {
      projects = projects.filter(p => p.status === status)
    }
    
    if (searchText) {
      const search = searchText.toLowerCase()
      projects = projects.filter(p => 
        p.name.toLowerCase().includes(search) ||
        p.creator.toLowerCase().includes(search)
      )
    }
    
    return projects
  }),
  
  // 活动表单窗口数量
  activeWindowCount: computed(() => globalState.form.activeWindows.size),
  
  // 未读通知数量
  unreadNotifications: computed(() => 
    globalState.notifications.filter(n => !n.read).length
  )
}

// 操作方法
const actions = {
  // 用户相关
  setUser(user) {
    Object.assign(globalState.user, user)
  },
  
  clearUser() {
    globalState.user.id = null
    globalState.user.name = ''
    globalState.user.role = ''
    globalState.user.permissions = []
  },
  
  // 应用配置
  setTheme(theme) {
    globalState.app.theme = theme
    // 可以在这里添加主题切换逻辑
  },
  
  setLanguage(language) {
    globalState.app.language = language
  },
  
  toggleSidebar() {
    globalState.app.sidebarCollapsed = !globalState.app.sidebarCollapsed
  },
  
  // 概算数据管理
  setCurrentProject(project) {
    globalState.estimate.currentProject = project
  },
  
  addProject(project) {
    globalState.estimate.projects.push({
      ...project,
      id: project.id || Date.now(),
      createTime: project.createTime || new Date().toISOString()
    })
  },
  
  updateProject(id, updates) {
    const index = globalState.estimate.projects.findIndex(p => p.id === id)
    if (index > -1) {
      Object.assign(globalState.estimate.projects[index], updates)
    }
  },
  
  deleteProject(id) {
    const index = globalState.estimate.projects.findIndex(p => p.id === id)
    if (index > -1) {
      globalState.estimate.projects.splice(index, 1)
    }
    // 同时清除选中状态
    actions.clearSelection()
  },
  
  setProjects(projects) {
    globalState.estimate.projects = projects
  },
  
  // 选择管理
  selectItems(items) {
    globalState.estimate.selectedItems = Array.isArray(items) ? items : [items]
  },
  
  addToSelection(item) {
    if (!globalState.estimate.selectedItems.find(i => i.id === item.id)) {
      globalState.estimate.selectedItems.push(item)
    }
  },
  
  removeFromSelection(itemId) {
    const index = globalState.estimate.selectedItems.findIndex(i => i.id === itemId)
    if (index > -1) {
      globalState.estimate.selectedItems.splice(index, 1)
    }
  },
  
  clearSelection() {
    globalState.estimate.selectedItems = []
  },
  
  // 筛选器
  setFilter(key, value) {
    globalState.estimate.filters[key] = value
  },
  
  clearFilters() {
    globalState.estimate.filters.status = ''
    globalState.estimate.filters.dateRange = null
    globalState.estimate.filters.searchText = ''
  },
  
  // 表单窗口管理
  registerWindow(windowId, windowInfo) {
    globalState.form.activeWindows.set(windowId, {
      ...windowInfo,
      openTime: Date.now()
    })
  },
  
  unregisterWindow(windowId) {
    globalState.form.activeWindows.delete(windowId)
    globalState.form.drafts.delete(windowId)
  },
  
  // 草稿管理
  saveDraft(windowId, data) {
    globalState.form.drafts.set(windowId, {
      data,
      saveTime: Date.now()
    })
    globalState.form.lastSaved = Date.now()
  },
  
  getDraft(windowId) {
    return globalState.form.drafts.get(windowId)
  },
  
  clearDraft(windowId) {
    globalState.form.drafts.delete(windowId)
  },
  
  // 通知管理
  addNotification(notification) {
    globalState.notifications.push({
      id: Date.now(),
      read: false,
      createTime: Date.now(),
      ...notification
    })
  },
  
  markNotificationRead(id) {
    const notification = globalState.notifications.find(n => n.id === id)
    if (notification) {
      notification.read = true
    }
  },
  
  clearNotifications() {
    globalState.notifications = []
  },
  
  // 加载状态
  setLoading(key, value) {
    globalState.loading[key] = value
  }
}

// 持久化相关
const persistence = {
  // 保存到 localStorage
  save() {
    try {
      const dataToSave = {
        user: globalState.user,
        app: globalState.app,
        estimate: {
          projects: globalState.estimate.projects,
          filters: globalState.estimate.filters
        }
      }
      localStorage.setItem('cost-app-state', JSON.stringify(dataToSave))
    } catch (error) {
      console.error('保存状态失败:', error)
    }
  },
  
  // 从 localStorage 加载
  load() {
    try {
      const saved = localStorage.getItem('cost-app-state')
      if (saved) {
        const data = JSON.parse(saved)
        
        // 恢复用户信息
        if (data.user) {
          Object.assign(globalState.user, data.user)
        }
        
        // 恢复应用配置
        if (data.app) {
          Object.assign(globalState.app, data.app)
        }
        
        // 恢复概算数据
        if (data.estimate) {
          if (data.estimate.projects) {
            globalState.estimate.projects = data.estimate.projects
          }
          if (data.estimate.filters) {
            Object.assign(globalState.estimate.filters, data.estimate.filters)
          }
        }
      }
    } catch (error) {
      console.error('加载状态失败:', error)
    }
  },
  
  // 清除持久化数据
  clear() {
    localStorage.removeItem('cost-app-state')
  }
}

// 导出状态管理
export {
  globalState,
  getters,
  actions,
  persistence
}

// 默认导出一个包含所有功能的对象
export default {
  state: globalState,
  getters,
  actions,
  persistence
}
