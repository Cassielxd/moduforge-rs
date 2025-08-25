import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { useSimpleWindowManagement, useChildWindowManagement, useMainWindowManagement } from './useSimpleWindowManagement.js'

/**
 * 通用窗体管理器
 * 适用于工作台和所有子应用，提供统一的窗体管理接口
 */
export function useUniversalWindowManager(options = {}) {
  const {
    isChildWindow = false,
    autoDetect = true,
    autoInitialize = true
  } = options

  // 内部状态
  const initialized = ref(false)
  const windowManager = ref(null)
  const openWindows = reactive(new Map())
  const windowCounter = ref(0)
  
  // 预定义的窗口配置
  const windowTemplates = {
    'estimate-demo': {
      title: '概算演示',
      url: '#/estimate-demo',
      width: 1200,
      height: 800,
      modal: false
    },
    'table-test': {
      title: '表格测试',
      url: '#/table-test',
      width: 1000,
      height: 700,
      modal: false
    },
    'data-viewer': {
      title: '数据查看器',
      url: '#/data-page',
      width: 900,
      height: 600,
      modal: false
    },
    'form-page': {
      title: '表单页面',
      url: '#/form-page',
      width: 800,
      height: 600,
      modal: false
    },
    'settings': {
      title: '系统设置',
      url: '#/settings-page',
      width: 700,
      height: 500,
      modal: true
    },
    'window-manager-demo': {
      title: '窗体管理演示',
      url: '/window-manager', // 使用工作台的路由
      width: 1000,
      height: 700,
      modal: false
    },
    // 子应用相关窗口
    'rough-estimate-main': {
      title: '概算管理',
      url: '/packages/rough-estimate/dist/index.html',
      width: 1200,
      height: 800,
      modal: false
    },
    'main-shell-dashboard': {
      title: '主工作台',
      url: '/packages/main-shell/dist/index.html',
      width: 1400,
      height: 900,
      modal: false
    }
  }

  // 计算属性
  const windowCount = computed(() => openWindows.size)
  const hasOpenWindows = computed(() => windowCount.value > 0)
  const openWindowList = computed(() => {
    return Array.from(openWindows.entries()).map(([id, info]) => ({
      id,
      ...info
    }))
  })

  /**
   * 初始化窗体管理器
   */
  const init = async () => {
    if (initialized.value) {
      console.warn('通用窗体管理器已经初始化')
      return
    }

    try {
      // 自动检测窗口类型
      let actualIsChildWindow = isChildWindow
      if (autoDetect) {
        // 可以通过 URL 参数或其他方式检测
        const urlParams = new URLSearchParams(window.location.search)
        actualIsChildWindow = urlParams.has('child') || window.opener !== null
      }

      // 根据窗口类型创建对应的管理器
      if (actualIsChildWindow) {
        windowManager.value = useChildWindowManagement()
      } else {
        windowManager.value = useMainWindowManagement()
      }

      // 等待底层管理器初始化完成
      if (!autoInitialize) {
        await windowManager.value.initializeWindow()
      }

      initialized.value = true
      console.log('通用窗体管理器初始化完成', { isChildWindow: actualIsChildWindow })
    } catch (error) {
      console.error('通用窗体管理器初始化失败:', error)
      throw error
    }
  }

  /**
   * 生成唯一的窗口ID
   */
  const generateWindowId = (type, customId) => {
    if (customId) return customId
    windowCounter.value++
    return `${type}-${Date.now()}-${windowCounter.value}`
  }

  /**
   * 打开预定义的窗口
   */
  const openWindow = async (type, customOptions = {}) => {
    if (!initialized.value) {
      throw new Error('窗体管理器未初始化，请先调用 init()')
    }

    const template = windowTemplates[type]
    if (!template) {
      throw new Error(`未知的窗口类型: ${type}`)
    }

    const windowId = generateWindowId(type, customOptions.windowId)
    const windowOptions = {
      windowId,
      ...template,
      ...customOptions
    }

    try {
      if (template.modal) {
        await windowManager.value.createModalDialog(windowOptions)
      } else {
        await windowManager.value.createChildWindow(windowOptions)
      }

      // 记录窗口信息
      openWindows.set(windowId, {
        type,
        title: windowOptions.title,
        url: windowOptions.url,
        modal: windowOptions.modal,
        openTime: new Date(),
        options: windowOptions
      })

      console.log(`窗口打开成功: ${type} (${windowId})`)
      return windowId
    } catch (error) {
      console.error(`打开窗口失败: ${type}`, error)
      throw error
    }
  }

  /**
   * 打开自定义窗口
   */
  const openCustomWindow = async (options) => {
    if (!initialized.value) {
      throw new Error('窗体管理器未初始化，请先调用 init()')
    }

    const { windowId, title, url, type = 'custom' } = options
    
    if (!windowId || !title || !url) {
      throw new Error('窗口ID、标题和URL是必需的参数')
    }

    const defaultOptions = {
      width: 800,
      height: 600,
      modal: false,
      ...options
    }

    try {
      if (defaultOptions.modal) {
        await windowManager.value.createModalDialog(defaultOptions)
      } else {
        await windowManager.value.createChildWindow(defaultOptions)
      }

      // 记录窗口信息
      openWindows.set(windowId, {
        type,
        title: defaultOptions.title,
        url: defaultOptions.url,
        modal: defaultOptions.modal,
        openTime: new Date(),
        options: defaultOptions
      })

      console.log(`自定义窗口打开成功: ${windowId}`)
      return windowId
    } catch (error) {
      console.error('打开自定义窗口失败:', error)
      throw error
    }
  }

  /**
   * 关闭指定窗口
   */
  const closeWindow = async (windowId) => {
    if (!initialized.value) {
      throw new Error('窗体管理器未初始化')
    }

    try {
      await windowManager.value.closeChildWindow(windowId)
      openWindows.delete(windowId)
      console.log(`窗口关闭成功: ${windowId}`)
    } catch (error) {
      console.error(`关闭窗口失败: ${windowId}`, error)
      throw error
    }
  }

  /**
   * 关闭所有窗口
   */
  const closeAllWindows = async () => {
    if (!initialized.value) {
      console.warn('窗体管理器未初始化')
      return
    }

    const windowIds = Array.from(openWindows.keys())
    const promises = windowIds.map(windowId => 
      closeWindow(windowId).catch(error => 
        console.error(`关闭窗口失败: ${windowId}`, error)
      )
    )

    try {
      await Promise.all(promises)
      console.log('所有窗口已关闭')
    } catch (error) {
      console.error('关闭部分窗口失败:', error)
    }
  }

  /**
   * 获取窗口信息
   */
  const getWindowInfo = (windowId) => {
    return openWindows.get(windowId) || null
  }

  /**
   * 检查窗口是否打开
   */
  const isWindowOpen = (windowId) => {
    return openWindows.has(windowId)
  }

  /**
   * 按类型获取窗口
   */
  const getWindowsByType = (type) => {
    return Array.from(openWindows.entries())
      .filter(([, info]) => info.type === type)
      .map(([id, info]) => ({ id, ...info }))
  }

  /**
   * 检查某类型窗口是否已打开
   */
  const isWindowTypeOpen = (type) => {
    return getWindowsByType(type).length > 0
  }

  // 便捷方法
  const quick = {
    // 工作台相关窗口
    estimateDemo: (options) => openWindow('estimate-demo', options),
    tableTest: (options) => openWindow('table-test', options),
    dataViewer: (options) => openWindow('data-viewer', options),
    formPage: (options) => openWindow('form-page', options),
    settings: (options) => openWindow('settings', options),
    windowDemo: (options) => openWindow('window-manager-demo', options),

    // 子应用窗口
    roughEstimate: (options) => openWindow('rough-estimate-main', options),
    mainShell: (options) => openWindow('main-shell-dashboard', options),

    // 常用操作
    closeAll: () => closeAllWindows(),
    getOpenCount: () => windowCount.value,
    getOpenList: () => openWindowList.value
  }

  /**
   * 添加窗口模板
   */
  const addWindowTemplate = (type, template) => {
    windowTemplates[type] = { ...template }
    console.log(`窗口模板已添加: ${type}`)
  }

  /**
   * 批量添加窗口模板
   */
  const addWindowTemplates = (templates) => {
    Object.entries(templates).forEach(([type, template]) => {
      addWindowTemplate(type, template)
    })
  }

  // 生命周期管理
  if (autoInitialize) {
    onMounted(async () => {
      await init()
    })
  }

  onUnmounted(() => {
    if (windowManager.value?.cleanup) {
      windowManager.value.cleanup()
    }
  })

  // 返回 API
  return {
    // 状态
    initialized,
    windowCount,
    hasOpenWindows,
    openWindowList,

    // 核心方法
    init,
    openWindow,
    openCustomWindow,
    closeWindow,
    closeAllWindows,

    // 查询方法
    getWindowInfo,
    isWindowOpen,
    getWindowsByType,
    isWindowTypeOpen,

    // 模板管理
    addWindowTemplate,
    addWindowTemplates,

    // 便捷方法
    quick,

    // 底层管理器访问（高级用法）
    windowManager,

    // 窗口信息访问
    getOpenWindows: () => openWindows,
    getWindowTemplates: () => windowTemplates
  }
}

/**
 * 创建全局单例实例
 * 用于在应用中共享窗体管理器状态
 */
let globalWindowManager = null

export function useGlobalWindowManager(options = {}) {
  if (!globalWindowManager) {
    globalWindowManager = useUniversalWindowManager({
      autoInitialize: false, // 全局实例手动初始化
      ...options
    })
  }
  return globalWindowManager
}

/**
 * 专门用于子应用的窗体管理器
 */
export function useChildAppWindowManager(options = {}) {
  return useUniversalWindowManager({
    isChildWindow: true,
    autoDetect: false,
    ...options
  })
}

/**
 * 专门用于主应用的窗体管理器
 */
export function useMainAppWindowManager(options = {}) {
  return useUniversalWindowManager({
    isChildWindow: false,
    autoDetect: false,
    ...options
  })
}