import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'

/**
 * 通用的表单窗口管理器
 * 用于创建和管理子应用的表单窗口
 */
export function useFormWindowManager(config = {}) {
  const defaultConfig = {
    appId: 'default-app',           // 应用标识
    formType: 'form',               // 表单类型
    defaultPort: '5176',            // 默认端口
    routePath: 'form-page',         // 路由路径
    titleMap: {                     // 标题映射
      create: '新建',
      edit: '编辑', 
      view: '查看'
    },
    windowSizes: {                  // 窗口大小配置
      modal: { width: 700, height: 500 },
      nonModal: { width: 800, height: 600 }
    }
  }

  // 合并配置
  const finalConfig = { ...defaultConfig, ...config }

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
   * 构建表单窗口URL
   * @param {URLSearchParams} params - URL参数
   * @returns {string} 完整的URL
   */
  const buildFormUrl = (params) => {
    const currentOrigin = window.location.origin
    const isDev = isDevEnvironment()
    
    console.log('环境检测:', {
      origin: currentOrigin,
      port: window.location.port,
      hostname: window.location.hostname,
      pathname: window.location.pathname,
      isDev
    })

    if (isDev) {
      // 开发环境：使用当前应用的完整URL
      return `${currentOrigin}/#/${finalConfig.routePath}?${params.toString()}`
    } else {
      // 生产环境：使用相对路径指向子应用
      return `/${finalConfig.appId}/index.html#/${finalConfig.routePath}?${params.toString()}`
    }
  }

  /**
   * 通用的窗口创建逻辑
   */
  const createWindow = async (mode, data, options = {}) => {
    const { windowInfo = {}, customParams = {}, windowOptions = {} } = options

    try {
      // 生成窗口ID
      const windowId = data?.id 
        ? `${finalConfig.formType}-form-${mode}-${data.id}` 
        : `${finalConfig.formType}-form-${mode}`
      
      // 生成窗口标题
      const title = finalConfig.titleMap[mode] || `${mode} ${finalConfig.formType}`
      const isModal = mode !== 'view' // 编辑和创建使用模态，查看使用非模态

      console.log('打开表单窗口:', {
        windowId,
        title,
        mode,
        modal: isModal,
        dataId: data?.id,
        appId: finalConfig.appId,
        formType: finalConfig.formType
      })

      // 构建URL参数
      const parentWindowLabel = windowInfo.label || `module-${finalConfig.appId}`
      const currentPort = window.location.port || finalConfig.defaultPort
      
      const params = new URLSearchParams({
        mode,
        formType: finalConfig.formType,
        modal: isModal.toString(),
        parentWindow: parentWindowLabel,
        appId: finalConfig.appId,
        appPort: currentPort,
        ...customParams // 合并自定义参数
      })

      if (data) {
        params.append('data', JSON.stringify(data))
      }

      // 先尝试显示已存在的窗口
      try {
        await invoke('show_existing_window', { windowId })
        console.log('已显示现有窗口:', windowId)
        message.success(`${title}窗口已显示`)
        return
      } catch (error) {
        console.log('窗口不存在，创建新窗口:', windowId)
      }

      // 构建完整的URL
      const fullUrl = buildFormUrl(params)
      console.log('子窗口完整URL:', fullUrl)

      // 窗口大小配置
      const sizeConfig = isModal ? finalConfig.windowSizes.modal : finalConfig.windowSizes.nonModal
      const finalWindowOptions = {
        windowId,
        title,
        url: fullUrl,
        modal: !isModal,
        width: sizeConfig.width,
        height: sizeConfig.height,
        parentWindow: windowInfo.label,
        ...windowOptions
      }

      // 使用invoke直接调用后端创建窗口
      await invoke('create_child_window', finalWindowOptions)
      
      console.log('创建子窗口成功:', {
        parentWindow: windowInfo.label,
        modal: isModal,
        config: finalWindowOptions
      })

      message.success(`${title}窗口已打开`)
    } catch (error) {
      console.error('打开表单窗口失败:', error)
      message.error('打开窗口失败')
      throw error
    }
  }

  /**
   * 便捷方法：创建新记录
   */
  const createForm = async (options = {}) => {
    return createWindow('create', null, options)
  }

  /**
   * 便捷方法：编辑记录
   */
  const editForm = async (data, options = {}) => {
    return createWindow('edit', data, options)
  }

  /**
   * 便捷方法：查看记录
   */
  const viewForm = async (data, options = {}) => {
    return createWindow('view', data, options)
  }

  return {
    createForm,
    editForm,
    viewForm,
    isDevEnvironment,
    buildFormUrl,
    config: finalConfig
  }
}

// 预设配置
export const formWindowPresets = {
  estimate: {
    appId: 'rough-estimate',
    formType: 'estimate',
    defaultPort: '5176',
    routePath: 'form-page',
    titleMap: {
      create: '新建概算',
      edit: '编辑概算',
      view: '查看概算'
    }
  },
  budget: {
    appId: 'budget',
    formType: 'budget',
    defaultPort: '5177',
    routePath: 'form-page',
    titleMap: {
      create: '新建预算',
      edit: '编辑预算',
      view: '查看预算'
    }
  }
}

/**
 * 预设的概算表单窗口管理器
 */
export function useEstimateFormWindow() {
  return useFormWindowManager(formWindowPresets.estimate)
}

/**
 * 预设的预算表单窗口管理器
 */
export function useBudgetFormWindow() {
  return useFormWindowManager(formWindowPresets.budget)
}