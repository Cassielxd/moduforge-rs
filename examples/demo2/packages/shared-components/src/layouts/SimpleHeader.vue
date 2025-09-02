<template>
  <div class="project-header" :class="{ 'draggable': draggable, 'web-environment': !isTauri }" :data-tauri-drag-region="draggable">
    <!-- 第一层：顶部操作区 -->
    <div class="header-top">
      <!-- 左侧内容 -->
      <div class="left-content">
        <!-- Logo区域 -->
        <div class="logo-link" :data-tauri-drag-region="draggable">
          <img :src="logoUrl" alt="logo" class="logo" />
        </div>

        <!-- 操作工具 -->
        <div class="tools-section">
          <!-- 保存按钮 -->
          <div class="tooltip-wrapper" v-if="showSaveButton">
            <a-tooltip 
              placement="bottom"
              title="保存(Ctrl+S)"
            >
              <div 
                class="icon-font save-btn" 
                :class="{ disabled: saving || saveStatus }"
                @click="!saving && !saveStatus && handleSave(true)"
              >
                <icon-font 
                  type="icon-kuaisubaocun" 
                  :style="{ 
                    color: '#ffffff', 
                    fontSize: '14px',
                    cursor: (saving || saveStatus) ? 'not-allowed' : 'pointer' 
                  }"
                />
              </div>
            </a-tooltip>
          </div>

          <!-- 远程协助按钮 -->
          <div class="tooltip-wrapper" v-if="showRemoteAssist">
            <a-tooltip 
              placement="bottom"
              title="申请远程协助"
            >
              <div 
                class="icon-font remote-assist-btn"
                @click="handleRemoteAssist"
              >
                <icon-font 
                  type="icon-shenqingyuanchengxiezhu" 
                  :style="{ 
                    color: '#ffffff', 
                    fontSize: '14px',
                    cursor: 'pointer' 
                  }"
                />
              </div>
            </a-tooltip>
          </div>
        </div>
      </div>

      <!-- 中间标题区域 -->
      <div class="title-section" :data-tauri-drag-region="draggable">
        <span class="project-title">{{ projectPath || title || "建设工程计价管理软件" }}</span>
        <span v-if="!saveStatus" class="unsaved-indicator"> *</span>
        
        <!-- 副标题反馈信息 -->
        <div class="sub-title" v-if="showFeedback">
          <div class="feedback-item">
            <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA8AAAARBAMAAAD3+MHvAAAAG1BMVEUAAAD///8AAADAwMAzMzP///8AAAD///8AAADrKC3qAAAAB3RSTlMA////////pX+jAAAALUlEQVQI12NgAAJBBgYGLgYGJgYGFgYGNgYGdgYGDgYGTgYGLgYGfgYGAQYGAB4FAnPUBpqLAAAAAElFTkSuQmCC" alt="icon" class="feedback-icon" />
            <span>公测快速反馈：</span>
            <span class="feedback-link" @click="openExternal('https://www.yunsuanfang.com/feedback')">https://www.yunsuanfang.com/feedback</span>
          </div>
          <div class="feedback-item">
            <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAPBAMAAAAv0UM9AAAAG1BMVEUAAAD///8AAADAwMAzMzP///8AAAD///8AAADrKC3qAAAAB3RSTlMA////////pX+jAAAALUlEQVQI12NgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgAAC2AAMVA" alt="icon" class="feedback-icon" style="width: 10px; height: 15px" />
            <span>手机快速反馈：鼠标移入扫一扫</span>
            <div class="qr-code-trigger" @mouseenter="showQRCode = true" @mouseleave="showQRCode = false">
              <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA8AAAAPBAMAAADJ+Ih5AAAAG1BMVEUAAAD///8AAADAwMAzMzP///8AAAD///8AAADrKC3qAAAAB3RSTlMA////////pX+jAAAALUlEQVQI12NgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgAAC2AAMVA" alt="qr-code" class="qr-icon" />
              <div v-if="showQRCode" class="qr-code-popup">
                <img :src="qrCodeUrl" alt="qr-code" class="qr-code-img" />
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 右侧菜单 -->
      <div class="right-menu">
        <!-- 用户信息 -->
        <div class="user-info" v-if="showUserInfo">
          <slot name="user-info"></slot>
        </div>

        <!-- 窗口控制按钮 -->
        <div class="window-controls" v-if="showWindowControls">
          <a-tooltip 
            placement="bottom"
            :title="minimizeTitle"
          >
            <button
              class="control-btn minimize-btn"
              @click="handleMinimize"
              :disabled="loading"
            >
              <svg width="12" height="12" viewBox="0 0 12 12">
                <path d="M2 6h8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
            </button>
          </a-tooltip>

          <a-tooltip 
            placement="bottom"
            :title="maximizeTitle"
          >
            <button
              class="control-btn maximize-btn"
              @click="handleMaximize"
              :disabled="loading"
            >
              <svg width="12" height="12" viewBox="0 0 12 12">
                <path v-if="!isMaximized" d="M2 2h8v8H2z" stroke="currentColor" stroke-width="1.5" fill="none"/>
                <path v-else d="M2 3h6v6H2z M4 1h6v6" stroke="currentColor" stroke-width="1.5" fill="none"/>
              </svg>
            </button>
          </a-tooltip>

          <a-tooltip 
            placement="bottom"
            :title="closeTitle"
          >
            <button
              class="control-btn close-btn"
              @click="handleClose"
              :disabled="loading"
            >
              <svg width="12" height="12" viewBox="0 0 12 12">
                <path d="M2 2l8 8M10 2l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
            </button>
          </a-tooltip>
        </div>
      </div>
    </div>

    <!-- 第二层：菜单导航区 -->
    <div class="header-menu">
      <!-- 菜单导航项 -->
      <div class="menu-navigation">
        <div
          class="menu-item"
          :class="{ selected: currentTab === item.key }"
          v-for="item in menuItems"
          :key="item.key"
        >
          <!-- 文件菜单（下拉） -->
          <div v-if="item.key === 'file'" class="dropdown-menu">
            <div class="menu-label" @click="toggleFileMenu">
              {{ item.name }}
              <svg class="dropdown-icon" :class="{ open: showFileMenu }" width="12" height="8" viewBox="0 0 12 8">
                <path d="M1 1l5 5 5-5" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
            <div v-if="showFileMenu" class="dropdown-panel">
              <div
                v-for="child in getFileMenuItems(item.children)"
                :key="child.key"
                class="dropdown-item"
                :class="{ disabled: isMenuItemDisabled(child.key) }"
                @click="handleFileMenuClick(child.key)"
              >
                <svg v-if="child.icon" class="menu-icon" width="14" height="14">
                  <!-- 这里可以根据icon类型渲染对应的SVG图标 -->
                  <use :href="`#${child.icon}`"></use>
                </svg>
                <span>{{ child.name }}</span>
              </div>
              <!-- 帮助中心子菜单 -->
              <div class="dropdown-item has-submenu" @click="toggleHelpMenu" v-if="showHelpCenter">
                <svg class="menu-icon" width="14" height="14">
                  <use href="#icon-bangzhuzhongxin1"></use>
                </svg>
                <span>帮助中心</span>
                <div v-if="showHelpSubmenu" class="submenu-panel">
                  <div class="submenu-item" @click.stop="handleHelpClick(1)">政策文件</div>
                  <div class="submenu-item" @click.stop="handleHelpClick(2)">勘误说明</div>
                </div>
              </div>
            </div>
          </div>

          <!-- 普通菜单项 -->
          <div v-else class="menu-label" @click="handleMenuClick(item.key)">
            {{ item.name }}
          </div>
        </div>
      </div>

      <!-- 右侧反馈信息（在菜单层显示） -->
      <div class="menu-feedback" v-if="showMenuFeedback">
        <div class="feedback-item">
          <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA8AAAARBAMAAAD3+MHvAAAAG1BMVEUAAAD///8AAADAwMAzMzP///8AAAD///8AAADrKC3qAAAAB3RSTlMA////////pX+jAAAALUlEQVQI12NgAAJBBgYGLgYGJgYGFgYGNgYGdgYGDgYGTgYGLgYGfgYGAQYGAB4FAnPUBpqLAAAAAElFTkSuQmCC" alt="icon" class="feedback-icon" />
          <span>公测快速反馈：</span>
          <span class="feedback-link" @click="openExternal('https://www.yunsuanfang.com/feedback')">https://www.yunsuanfang.com/feedback</span>
        </div>
        <div class="feedback-item">
          <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAPBAMAAAAv0UM9AAAAG1BMVEUAAAD///8AAADAwMAzMzP///8AAAD///8AAADrKC3qAAAAB3RSTlMA////////pX+jAAAALUlEQVQI12NgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgAAC2AAMVA" alt="icon" class="feedback-icon" style="width: 10px; height: 15px" />
          <span>手机快速反馈：鼠标移入扫一扫</span>
          <div class="qr-code-trigger" @mouseenter="showQRCode = true" @mouseleave="showQRCode = false">
            <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA8AAAAPBAMAAADJ+Ih5AAAAG1BMVEUAAAD///8AAADAwMAzMzP///8AAAD///8AAADrKC3qAAAAB3RSTlMA////////pX+jAAAALUlEQVQI12NgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgYGBgAAC2AAMVA" alt="qr-code" class="qr-icon" />
            <div v-if="showQRCode" class="qr-code-popup">
              <img :src="qrCodeUrl" alt="qr-code" class="qr-code-img" />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { webWindowController, isTauriEnvironment } from '../utils/webEnvironment.js'
import { createFromIconfontCN } from '@ant-design/icons-vue'

// 创建 IconFont 组件
const IconFont = createFromIconfontCN({
  scriptUrl: '//at.alicdn.com/t/c/font_4199803_ualaqaqnkv.js',
})

// Props
const props = defineProps({
  // 基础配置
  title: {
    type: String,
    default: '建设工程计价管理软件'
  },
  projectPath: {
    type: String,
    default: ''
  },
  logoUrl: {
    type: String,
    default: new URL('../assets/img/logo.png', import.meta.url).href
  },
  projectType: {
    type: String,
    default: 'ys'
  },
  
  // 显示控制
  showWindowControls: {
    type: Boolean,
    default: true
  },
  showSaveButton: {
    type: Boolean,
    default: true
  },
  showRemoteAssist: {
    type: Boolean,
    default: true
  },
  showUserInfo: {
    type: Boolean,
    default: true
  },
  showFeedback: {
    type: Boolean,
    default: true
  },
  showMenuFeedback: {
    type: Boolean,
    default: false
  },
  showHelpCenter: {
    type: Boolean,
    default: true
  },
  
  // 窗口配置
  draggable: {
    type: Boolean,
    default: true
  },
  minimizeTitle: {
    type: String,
    default: '最小化'
  },
  maximizeTitle: {
    type: String,
    default: '最大化/还原'
  },
  closeTitle: {
    type: String,
    default: '关闭'
  },
  useChildWindow: {
    type: Boolean,
    default: false
  },
  
  // 状态数据
  saveStatus: {
    type: Boolean,
    default: true
  },
  currentTab: {
    type: String,
    default: 'customize'
  },
  
  // 菜单配置
  menuItems: {
    type: Array,
    default: () => [
      {
        name: '文件',
        key: 'file',
        dropdown: true,
        children: [
          { name: '打开', key: 'open', icon: 'icon-dakai' },
          { name: '新建', key: 'new', icon: 'icon-xinjian' },
          { name: '另存为', key: 'asideSave', icon: 'icon-lingcunwei' },
          { name: '保存', key: 'save', icon: 'icon-baocun' },
          { name: '设置', key: 'setup', icon: 'icon-shezhi' },
          { name: '帮助中心', key: 'helpCenter', icon: 'icon-bangzhuzhongxin1' },
          { name: '检查更新', key: 'checkUpdate', icon: 'icon-jianchagengxin' }
        ]
      },
      { name: '编制', key: 'customize' },
      { name: '报表', key: 'reportForm' }
    ]
  }
})

// Emits
const emit = defineEmits([
  // 窗口控制事件
  'minimize', 'maximize', 'close', 'window-state-change',
  // 功能操作事件
  'save', 'remote-assist', 'external-link',
  // 菜单导航事件
  'menu-click', 'file-menu-click', 'help-click',
  // 状态更新事件
  'tab-change', 'save-status-change'
])

// State
const loading = ref(false)
const saving = ref(false)
const isMaximized = ref(false)
const currentWindow = ref(null)
const showQRCode = ref(false)
const showFileMenu = ref(false)
const showHelpSubmenu = ref(false)
const qrCodeUrl = ref('')

// 检测是否在 Tauri 环境中
const isTauri = computed(() => {
  return isTauriEnvironment()
})


// 生成二维码URL
const generateQRCode = async () => {
  try {
    // 这里可以使用QRCode库生成二维码
    // 暂时使用占位图片
    const text = 'https://h5.yunsuanfang.com/userFeedback'
    // 实际项目中应该导入QRCode库：import QRCode from 'qrcode'
    // qrCodeUrl.value = await QRCode.toDataURL(text)
    qrCodeUrl.value = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA8AAAAPAQMAAACXP7ikAAAABlBMVEX///8AAABVwtN+AAAAAXRSTLMA/YUAGwAAAAxJREFUCNdjYBgFAAABlAAB3wCbQgAAAABJRU5ErkJggg=='
  } catch (error) {
    console.warn('生成二维码失败:', error)
  }
}

// 获取文件菜单项（根据项目类型过滤）
const getFileMenuItems = (children = []) => {
  if (props.projectType === 'yssh') {
    return children.filter(x => x.key === 'asideSave' || x.key === 'open')
  } else if (props.projectType === 'jieSuan') {
    return children.filter(x => x.key !== 'helpCenter')
  }
  return children.filter(x => x.key !== 'helpCenter')
}

// 检查菜单项是否禁用
const isMenuItemDisabled = (key) => {
  return key === 'save' && props.saveStatus
}

// 打开外部链接
const openExternal = (url) => {
  emit('external-link', url)
  // 在实际应用中，可能需要使用electron的shell模块或window.open
  if (typeof window !== 'undefined') {
    window.open(url, '_blank')
  }
}

// 检测是否为子窗口
const isChildWindow = computed(() => {
  if (!isTauri.value) return false
  try {
    // 通过URL参数或窗口标识符来判断是否为子窗口
    const urlParams = new URLSearchParams(window.location.search)
    const isChild = urlParams.has('mode') || urlParams.has('formType') || 
                    window.location.pathname.includes('form-page')
    console.log('检测子窗口状态:', isChild, 'URL:', window.location.href)
    return isChild
  } catch (error) {
    console.warn('检测子窗口状态失败:', error)
    return false
  }
})

// 初始化窗口管理
const initWindowManager = async () => {
  if (!isTauri.value) {
    console.log('非Tauri环境，跳过窗口管理器初始化')
    return
  }

  try {
    const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow')
    currentWindow.value = getCurrentWebviewWindow()

    if (currentWindow.value) {
      console.log('窗口管理器初始化成功', {
        label: currentWindow.value.label,
        isChild: isChildWindow.value
      })
      
      // 检查当前窗口状态
      try {
        isMaximized.value = await currentWindow.value.isMaximized()
        console.log('当前窗口最大化状态:', isMaximized.value)
      } catch (stateError) {
        console.warn('获取窗口状态失败:', stateError)
        isMaximized.value = false
      }

      // 监听窗口状态变化事件
      try {
        const unlisten = await currentWindow.value.listen('tauri://resize', async () => {
          try {
            const newMaximizedState = await currentWindow.value.isMaximized()
            isMaximized.value = newMaximizedState
            emit('window-state-change', { maximized: newMaximizedState })
            console.log('窗口状态变化:', newMaximizedState)
          } catch (listenerError) {
            console.warn('监听器中获取窗口状态失败:', listenerError)
          }
        })

        return unlisten
      } catch (listenError) {
        console.warn('添加窗口状态监听器失败:', listenError)
      }
    } else {
      console.warn('无法获取当前窗口实例')
    }
  } catch (error) {
    console.error('初始化窗口管理器失败:', error)
    // 在非Tauri环境或出错时，设置默认状态
    isMaximized.value = false
  }
}

// 功能操作方法
const handleSave = async (showLoading = false) => {
  if (saving.value || props.saveStatus) {
    console.log('保存操作进行中或文件已保存，跳过保存')
    return
  }

  saving.value = true

  try {
    emit('save', { showLoading })
    console.log('保存操作已触发')
  } catch (error) {
    console.error('保存操作失败:', error)
  } finally {
    saving.value = false
  }
}

const handleRemoteAssist = () => {
  emit('remote-assist')
  console.log('远程协助请求已发送')
}

// 菜单操作方法
const toggleFileMenu = () => {
  showFileMenu.value = !showFileMenu.value
  if (showFileMenu.value) {
    showHelpSubmenu.value = false
  }
}

const toggleHelpMenu = () => {
  showHelpSubmenu.value = !showHelpSubmenu.value
}

const handleMenuClick = (key) => {
  if (props.currentTab === key) return
  
  emit('menu-click', key)
  emit('tab-change', key)
  
  // 关闭下拉菜单
  showFileMenu.value = false
  showHelpSubmenu.value = false
  
  console.log('菜单点击:', key)
}

const handleFileMenuClick = (key) => {
  if (isMenuItemDisabled(key)) {
    return
  }
  
  emit('file-menu-click', key)
  showFileMenu.value = false
  showHelpSubmenu.value = false
  
  console.log('文件菜单点击:', key)
}

const handleHelpClick = (type) => {
  emit('help-click', type)
  showHelpSubmenu.value = false
  showFileMenu.value = false
  
  console.log('帮助菜单点击:', type)
}

// 窗口控制方法
const handleMinimize = async () => {
  if (loading.value) {
    console.log('操作进行中，跳过最小化')
    return
  }

  loading.value = true
  
  try {
    // 先发出事件，让外部组件处理
    emit('minimize')
    
    // 如果是非 Tauri 环境，执行 Web 模拟
    if (!isTauri.value) {
      await webWindowController.minimize()
      return
    }

    // 在 Tauri 环境下，执行实际的最小化操作
    if (currentWindow.value) {
      await currentWindow.value.minimize()
      console.log('窗口已最小化')
    } else {
      console.warn('无法获取当前窗口实例，最小化失败')
    }
  } catch (error) {
    console.error('最小化窗口失败:', error)
  } finally {
    loading.value = false
  }
}

const handleMaximize = async () => {
  if (loading.value) {
    console.log('操作进行中，跳过最大化切换')
    return
  }

  loading.value = true
  
  try {
    // 先发出事件，让外部组件处理
    emit('maximize')

    // 如果是非 Tauri 环境，执行 Web 模拟
    if (!isTauri.value) {
      const result = await webWindowController.toggleMaximize()
      if (result.success) {
        isMaximized.value = result.isMaximized
      }
      return
    }

    // 在 Tauri 环境下，执行实际的最大化操作
    if (currentWindow.value) {
      if (isMaximized.value) {
        await currentWindow.value.unmaximize()
        console.log('窗口已还原')
      } else {
        await currentWindow.value.maximize()
        console.log('窗口已最大化')
      }
      
      // 更新状态
      isMaximized.value = await currentWindow.value.isMaximized()
      emit('window-state-change', { maximized: isMaximized.value })
    } else {
      console.warn('无法获取当前窗口实例，最大化操作失败')
    }
  } catch (error) {
    console.error('最大化切换失败:', error)
  } finally {
    loading.value = false
  }
}

const handleClose = async () => {
  if (loading.value) {
    console.log('操作进行中，跳过关闭')
    return
  }

  loading.value = true
  
  try {
    // 先发出事件，让外部组件处理
    emit('close')

    // 如果是非 Tauri 环境，执行 Web 模拟
    if (!isTauri.value) {
      await webWindowController.close()
      return
    }

    // 在 Tauri 环境下，执行实际的关闭操作
    if (currentWindow.value) {
      // 对于子窗口，直接关闭
      if (isChildWindow.value) {
        await currentWindow.value.close()
        console.log('子窗口已关闭')
      } else {
        // 对于主窗口，使用后端命令处理子窗口
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('close_window_with_children', {
          windowId: currentWindow.value.label
        })
        console.log('主窗口及所有子窗口已关闭')
      }
    } else {
      console.warn('无法获取当前窗口实例，关闭失败')
    }
  } catch (error) {
    console.error('关闭窗口失败:', error)
  } finally {
    loading.value = false
  }
}

// 键盘快捷键处理
const handleKeyboard = (event) => {
  // Ctrl+S 保存
  if ((event.ctrlKey || event.metaKey) && event.key === 's') {
    event.preventDefault()
    handleSave(true)
  }
}

// 点击外部关闭下拉菜单
const handleClickOutside = (event) => {
  if (!event.target.closest('.dropdown-menu')) {
    showFileMenu.value = false
    showHelpSubmenu.value = false
  }
}

// 生命周期
let unlistenResize = null

onMounted(async () => {
  console.log('ProjectHeader 组件挂载', {
    title: props.title,
    projectPath: props.projectPath,
    projectType: props.projectType,
    showWindowControls: props.showWindowControls,
    isTauri: isTauri.value
  })
  
  // 生成二维码
  await generateQRCode()
  
  // 初始化窗口管理
  if (isTauri.value) {
    unlistenResize = await initWindowManager()
  } else {
    // 非Tauri环境，同步Web控制器状态
    isMaximized.value = webWindowController.getMaximizedState()
    console.log('Web环境初始化，最大化状态:', isMaximized.value)
  }
  
  // 添加全局事件监听
  document.addEventListener('keydown', handleKeyboard)
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  if (unlistenResize) {
    unlistenResize()
  }
  
  // 移除全局事件监听
  document.removeEventListener('keydown', handleKeyboard)
  document.removeEventListener('click', handleClickOutside)
})

// 暴露方法给父组件使用
defineExpose({
  handleSave,
  handleRemoteAssist,
  handleMenuClick,
  toggleFileMenu
})
</script>

<style scoped>
@import '../styles/project-header.css';
</style>
