import { ref, reactive } from 'vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'

/**
 * 通用的操作窗口管理器
 * 专门用于 OperateBar 操作打开 Tauri 窗口
 */
export function useOperateWindowManager(config = {}) {
  const defaultConfig = {
    appId: 'default-app',           // 应用标识
    defaultPort: '5176',            // 默认端口
    routePath: 'operate-page',      // 路由路径
    windowSizes: {                  // 窗口大小配置
      small: { width: 600, height: 400 },
      medium: { width: 800, height: 600 },
      large: { width: 1000, height: 700 },
      xlarge: { width: 1200, height: 800 }
    }
  }

  // 合并配置
  const finalConfig = { ...defaultConfig, ...config }

  // 窗口注册表
  const windowRegistry = reactive(new Map())

  /**
   * 检测当前环境是否为开发环境
   */
  const isDevEnvironment = () => {
    return window.location.port && (
      window.location.port.startsWith('51') || 
      window.location.hostname === 'localhost' ||
      window.location.hostname === '127.0.0.1'
    )
  }

  /**
   * 构建操作窗口URL
   * @param {string} operateName - 操作名称
   * @param {URLSearchParams} params - URL参数
   * @returns {string} 完整的URL
   */
  const buildOperateUrl = (operateName, params) => {
    const currentOrigin = window.location.origin
    const isDev = isDevEnvironment()
    
    console.log('构建操作窗口URL:', {
      operateName,
      origin: currentOrigin,
      isDev,
      params: params.toString()
    })

    if (isDev) {
      // 开发环境：使用当前应用的完整URL
      return `${currentOrigin}/#/${finalConfig.routePath}/${operateName}?${params.toString()}`
    } else {
      // 生产环境：使用相对路径指向子应用
      return `/${finalConfig.appId}/index.html#/${finalConfig.routePath}/${operateName}?${params.toString()}`
    }
  }

  /**
   * 注册操作窗口配置
   * @param {string} operateName - 操作名称
   * @param {Object} windowConfig - 窗口配置
   */
  const registerOperateWindow = (operateName, windowConfig) => {
    const defaultWindowConfig = {
      title: operateName,
      size: 'medium',
      modal: false,  // 改为非模态，这样窗口控制按钮才能正常工作
      resizable: true,
      minimizable: true,  // 启用最小化
      maximizable: true,
      closable: true,
      alwaysOnTop: false,
      skipTaskbar: false,
      routePath: operateName,
      componentName: operateName,
      loadingDelay: 300
    }

    const finalWindowConfig = { ...defaultWindowConfig, ...windowConfig }
    windowRegistry.set(operateName, finalWindowConfig)
    
    console.log('注册操作窗口:', operateName, finalWindowConfig)
  }

  /**
   * 通用的操作窗口创建逻辑
   * @param {Object} operateItem - 操作项配置
   * @param {Object} options - 选项
   */
  const createOperateWindow = async (operateItem, options = {}) => {
    const { 
      windowInfo = {}, 
      customParams = {}, 
      windowOptions = {},
      data = null,
      tableData = null
    } = options

    try {
      const operateName = operateItem.name
      const windowConfig = windowRegistry.get(operateName)
      
      if (!windowConfig) {
        throw new Error(`未找到操作窗口配置: ${operateName}`)
      }

      // 生成窗口ID - 使用固定ID以便窗口复用
      const windowId = `operate-${operateName}`

      // 生成窗口标题
      const title = windowConfig.title || operateItem.label || operateName

      console.log('创建操作窗口:', {
        windowId,
        title,
        operateName,
        config: windowConfig
      })

      // 构建URL参数
      const parentWindowLabel = windowInfo.label || `module-${finalConfig.appId}`
      const currentPort = window.location.port || finalConfig.defaultPort
      
      // 检查是否为表单模式操作
      const isFormMode = windowConfig.formMode

      const params = new URLSearchParams({
        operate: operateName,
        operateLabel: operateItem.label || operateName,
        component: windowConfig.componentName,
        modal: windowConfig.modal.toString(),
        parentWindow: parentWindowLabel,
        appId: finalConfig.appId,
        appPort: currentPort,
        windowId: windowId, // 添加窗口ID用于识别
        ...customParams // 合并自定义参数
      })

      // 如果是表单模式，添加表单相关参数
      if (isFormMode) {
        params.append('mode', windowConfig.formMode)
        params.append('formType', finalConfig.formType || 'estimate')
      }

      // 传递数据
      if (data) {
        params.append('data', JSON.stringify(data))
      }
      if (tableData) {
        params.append('tableData', JSON.stringify(tableData))
      }

      // 先尝试显示已存在的相同类型窗口
      try {
        await invoke('show_existing_window', { windowId })
        console.log('已显示现有操作窗口:', windowId)
        message.success(`${title}窗口已显示`)
        return
      } catch (error) {
        console.log('操作窗口不存在，创建新窗口:', windowId)
      }

      // 构建完整的URL
      const fullUrl = buildOperateUrl(windowConfig.routePath || operateName, params)
      console.log('操作窗口完整URL:', fullUrl)

      // 窗口大小配置
      const sizeConfig = finalConfig.windowSizes[windowConfig.size] || finalConfig.windowSizes.medium
      const finalWindowOptions = {
        windowId,
        title,
        url: fullUrl,
        modal: windowConfig.modal,
        width: sizeConfig.width,
        height: sizeConfig.height,
        resizable: windowConfig.resizable,
        minimizable: windowConfig.minimizable,
        maximizable: windowConfig.maximizable,
        closable: windowConfig.closable,
        alwaysOnTop: windowConfig.alwaysOnTop,
        skipTaskbar: windowConfig.skipTaskbar,
        parentWindow: windowInfo.label,
        ...windowOptions
      }

      // 使用invoke直接调用后端创建窗口
      await invoke('create_child_window', finalWindowOptions)
      
      console.log('创建操作窗口成功:', {
        parentWindow: windowInfo.label,
        config: finalWindowOptions
      })

      message.success(`${title}窗口已打开`)
    } catch (error) {
      console.error('打开操作窗口失败:', error)
      message.error(`打开${operateItem.label || operateItem.name}窗口失败`)
      throw error
    }
  }

  /**
   * 根据操作项打开窗口
   * @param {Object} operateItem - 操作项
   * @param {Object} options - 选项
   */
  const openOperateWindow = async (operateItem, options = {}) => {
    return createOperateWindow(operateItem, options)
  }

  /**
   * 批量注册操作窗口
   * @param {Object} windowConfigs - 窗口配置映射
   */
  const registerOperateWindows = (windowConfigs) => {
    Object.entries(windowConfigs).forEach(([operateName, config]) => {
      registerOperateWindow(operateName, config)
    })
  }

  /**
   * 便捷方法：创建表单窗口
   * @param {string} mode - 表单模式：create, edit, view
   * @param {Object} data - 表单数据（编辑和查看时需要）
   * @param {Object} options - 选项
   */
  const createFormWindow = async (mode, data = null, options = {}) => {
    const operateItem = {
      name: `${mode}-record`,
      label: mode === 'create' ? '新建' : mode === 'edit' ? '编辑' : '查看'
    }

    return createOperateWindow(operateItem, {
      ...options,
      data: data ? [data] : null // 包装为数组格式
    })
  }

  /**
   * 便捷方法：新建记录
   */
  const createRecord = async (options = {}) => {
    return createFormWindow('create', null, options)
  }

  /**
   * 便捷方法：编辑记录
   */
  const editRecord = async (data, options = {}) => {
    return createFormWindow('edit', data, options)
  }

  /**
   * 便捷方法：查看记录
   */
  const viewRecord = async (data, options = {}) => {
    return createFormWindow('view', data, options)
  }

  return {
    openOperateWindow,
    registerOperateWindow,
    registerOperateWindows,
    createFormWindow,
    createRecord,
    editRecord,
    viewRecord,
    isDevEnvironment,
    buildOperateUrl,
    windowRegistry,
    config: finalConfig
  }
}

// 预设的操作窗口配置
export const operateWindowPresets = {
  // 新建记录 - 集成 formWindowManager 的新建功能
  'create-record': {
    title: '新建',
    size: 'medium',
    modal: false,  // 确保窗口控制按钮可用
    resizable: true,
    minimizable: true,
    maximizable: true,
    closable: true,
    componentName: 'FormWindow',
    routePath: 'form-page',
    formMode: 'create'  // 标记为表单模式
  },

  // 编辑记录
  'edit-record': {
    title: '编辑',
    size: 'medium',
    modal: false,
    resizable: true,
    minimizable: true,
    maximizable: true,
    closable: true,
    componentName: 'FormWindow',
    routePath: 'form-page',
    formMode: 'edit'
  },

  // 查看记录
  'view-record': {
    title: '查看',
    size: 'medium',
    modal: false,
    resizable: true,
    minimizable: true,
    maximizable: true,
    closable: true,
    componentName: 'FormWindow',
    routePath: 'form-page',
    formMode: 'view'
  },

  // 数据导入
  'import-data': {
    title: '数据导入',
    size: 'large',
    modal: false,  // 改为非模态，确保窗口控制按钮可用
    resizable: true,
    minimizable: true,
    maximizable: true,
    closable: true,
    componentName: 'DataImportWindow',
    routePath: 'import-data'
  },

  // 数据导出
  'export-table': {
    title: '数据导出',
    size: 'medium',
    modal: false,  // 改为非模态
    resizable: true,
    minimizable: true,
    maximizable: true,
    closable: true,
    componentName: 'DataExportWindow',
    routePath: 'export-data'
  },

  // 批量操作
  'batch-operation': {
    title: '批量操作',
    size: 'large',
    modal: false,  // 改为非模态
    resizable: true,
    minimizable: true,
    maximizable: true,
    closable: true,
    componentName: 'BatchOperationWindow',
    routePath: 'batch-operation'
  },

  // 系统设置
  'system-settings': {
    title: '系统设置',
    size: 'large',
    modal: false,
    resizable: true,
    minimizable: true,
    maximizable: true,
    closable: true,
    componentName: 'SystemSettingsWindow',
    routePath: 'system-settings'
  },

  // 数据分析
  'data-analysis': {
    title: '数据分析',
    size: 'xlarge',
    modal: false,
    resizable: true,
    minimizable: true,
    maximizable: true,
    closable: true,
    componentName: 'DataAnalysisWindow',
    routePath: 'data-analysis'
  },

  // 模板管理
  'template-manage': {
    title: '模板管理',
    size: 'large',
    modal: false,
    resizable: true,
    minimizable: true,
    maximizable: true,
    closable: true,
    componentName: 'TemplateManageWindow',
    routePath: 'template-manage'
  },

  // 计税设置
  'tax-settings': {
    title: '计税设置',
    size: 'medium',
    modal: false,  // 改为非模态
    resizable: true,
    minimizable: true,
    maximizable: true,
    closable: true,
    componentName: 'TaxSettingsWindow',
    routePath: 'tax-settings'
  }
}

// 全局操作窗口管理器实例
let globalOperateWindowManager = null

/**
 * 获取全局操作窗口管理器
 * @param {Object} config - 配置
 */
export function useGlobalOperateWindow(config = {}) {
  if (!globalOperateWindowManager) {
    globalOperateWindowManager = useOperateWindowManager(config)
    
    // 注册预设的操作窗口
    globalOperateWindowManager.registerOperateWindows(operateWindowPresets)
  }
  
  return globalOperateWindowManager
}

/**
 * 注册操作窗口的便捷方法
 * @param {string} operateName - 操作名称
 * @param {Object} windowConfig - 窗口配置
 */
export function registerOperateWindow(operateName, windowConfig) {
  const manager = useGlobalOperateWindow()
  manager.registerOperateWindow(operateName, windowConfig)
}
