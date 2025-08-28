/**
 * Web环境工具函数
 * 为浏览器环境提供窗口控制的替代方案
 */

/**
 * 检测是否在Tauri环境中
 */
export const isTauriEnvironment = () => {
  return typeof window !== 'undefined' && window.__TAURI__
}

/**
 * 检测是否支持全屏API
 */
export const supportsFullscreen = () => {
  return !!(
    document.fullscreenEnabled ||
    document.webkitFullscreenEnabled ||
    document.mozFullScreenEnabled ||
    document.msFullscreenEnabled
  )
}

/**
 * 检测是否处于全屏状态
 */
export const isFullscreen = () => {
  return !!(
    document.fullscreenElement ||
    document.webkitFullscreenElement ||
    document.mozFullScreenElement ||
    document.msFullscreenElement
  )
}

/**
 * 进入全屏模式
 */
export const enterFullscreen = async () => {
  const element = document.documentElement
  
  if (element.requestFullscreen) {
    await element.requestFullscreen()
  } else if (element.webkitRequestFullscreen) {
    await element.webkitRequestFullscreen()
  } else if (element.mozRequestFullScreen) {
    await element.mozRequestFullScreen()
  } else if (element.msRequestFullscreen) {
    await element.msRequestFullscreen()
  }
}

/**
 * 退出全屏模式
 */
export const exitFullscreen = async () => {
  if (document.exitFullscreen) {
    await document.exitFullscreen()
  } else if (document.webkitExitFullscreen) {
    await document.webkitExitFullscreen()
  } else if (document.mozCancelFullScreen) {
    await document.mozCancelFullScreen()
  } else if (document.msExitFullscreen) {
    await document.msExitFullscreen()
  }
}

/**
 * 切换全屏状态
 */
export const toggleFullscreen = async () => {
  if (isFullscreen()) {
    await exitFullscreen()
  } else {
    await enterFullscreen()
  }
}

/**
 * 模拟窗口最小化效果
 */
export const simulateMinimize = () => {
  const body = document.body
  const html = document.documentElement
  const minimizeOverlay = document.createElement('div')
  
  // 保存原始样式
  const originalBodyStyle = body.style.cssText
  const originalHtmlStyle = html.style.cssText
  
  // 创建最小化状态指示器
  minimizeOverlay.style.cssText = `
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 10000;
    cursor: pointer;
  `
  
  minimizeOverlay.innerHTML = `
    <div style="
      background: white;
      padding: 20px 40px;
      border-radius: 8px;
      text-align: center;
      box-shadow: 0 4px 20px rgba(0,0,0,0.3);
    ">
      <div style="font-size: 16px; margin-bottom: 10px;">窗口已最小化</div>
      <div style="font-size: 12px; color: #666;">点击任意位置恢复窗口</div>
    </div>
  `
  
  // 最小化动画
  body.style.transition = 'transform 0.3s ease, opacity 0.3s ease'
  body.style.transform = 'scale(0.1)'
  body.style.opacity = '0'
  
  setTimeout(() => {
    body.style.display = 'none'
    document.body.appendChild(minimizeOverlay)
    
    // 点击恢复
    minimizeOverlay.addEventListener('click', () => {
      document.body.removeChild(minimizeOverlay)
      
      // 完全恢复原始样式
      body.style.cssText = originalBodyStyle
      html.style.cssText = originalHtmlStyle
      
      // 确保显示和透明度正确
      body.style.display = 'block'
      body.style.opacity = '1'
      body.style.transform = 'scale(1)'
      body.style.transition = 'transform 0.3s ease, opacity 0.3s ease'
      
      // 延迟清除过渡效果
      setTimeout(() => {
        body.style.transition = originalBodyStyle.includes('transition') ? 
          originalBodyStyle.match(/transition[^;]*/)?.[0] || '' : ''
      }, 300)
    })
  }, 300)
  
  return Promise.resolve()
}

/**
 * Web环境下的窗口控制器
 */
export class WebWindowController {
  constructor() {
    this.isMaximized = false
    this.setupFullscreenListeners()
  }

  setupFullscreenListeners() {
    // 不再使用全屏API，状态由 toggleMaximize 方法直接管理
  }

  /**
   * 最小化处理
   */
  async minimize() {
    try {
      // await simulateMinimize()
      return { success: true, action: 'minimize' }
    } catch (error) {
      console.error('最小化失败:', error)
      return { success: false, error: error.message }
    }
  }

  /**
   * 最大化/还原处理
   */
  async toggleMaximize() {
    try {
      this.isMaximized = !this.isMaximized
      const html = document.documentElement
      const body = document.body
      
      if (this.isMaximized) {
        // 保存当前样式
        html.setAttribute('data-original-style', html.style.cssText || '')
        body.setAttribute('data-original-style', body.style.cssText || '')
        
        // 最大化效果
        html.style.cssText = `
          width: 100% !important;
          height: 100% !important;
          margin: 0 !important;
          padding: 0 !important;
          overflow: hidden !important;
        `
        body.style.cssText = `
          width: 100vw !important;
          height: 100vh !important;
          margin: 0 !important;
          padding: 0 !important;
          overflow: hidden !important;
          position: fixed !important;
          top: 0 !important;
          left: 0 !important;
          z-index: 9999 !important;
        `
        
        // 隐藏滚动条
        document.body.style.overflow = 'hidden'
      } else {
        // 还原样式
        const originalHtmlStyle = html.getAttribute('data-original-style') || ''
        const originalBodyStyle = body.getAttribute('data-original-style') || ''
        
        html.style.cssText = originalHtmlStyle
        body.style.cssText = originalBodyStyle
        
        html.removeAttribute('data-original-style')
        body.removeAttribute('data-original-style')
        
        // 恢复滚动条
        document.body.style.overflow = ''
      }
      
      return { 
        success: true, 
        action: this.isMaximized ? 'maximize' : 'restore',
        isMaximized: this.isMaximized 
      }
    } catch (error) {
      console.error('切换最大化状态失败:', error)
      return { success: false, error: error.message }
    }
  }

  /**
   * 关闭处理
   */
  async close() {
    try {
      // 显示确认对话框
      const confirmed = confirm('确定要关闭窗口吗？')
      if (!confirmed) {
        return { success: false, action: 'close_cancelled' }
      }
      
      // 创建关闭动画效果
      const body = document.body
      const closeOverlay = document.createElement('div')
      
      closeOverlay.style.cssText = `
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.9);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 10001;
        transition: opacity 0.3s ease;
      `
      
      closeOverlay.innerHTML = `
        <div style="
          background: white;
          padding: 30px 50px;
          border-radius: 8px;
          text-align: center;
          box-shadow: 0 4px 20px rgba(0,0,0,0.5);
        ">
          <div style="font-size: 18px; margin-bottom: 15px;">窗口正在关闭...</div>
          <div style="font-size: 12px; color: #666;">如果浏览器阻止关闭，请手动关闭标签页</div>
        </div>
      `
      
      document.body.appendChild(closeOverlay)
      
      // 渐变关闭效果
      body.style.transition = 'opacity 0.5s ease'
      body.style.opacity = '0.3'
      
      setTimeout(() => {
        // 尝试关闭窗口
        window.close()
        
        // 如果2秒后还没关闭，提供手动关闭指引
        setTimeout(() => {
          if (!window.closed) {
            closeOverlay.innerHTML = `
              <div style="
                background: white;
                padding: 30px 50px;
                border-radius: 8px;
                text-align: center;
                box-shadow: 0 4px 20px rgba(0,0,0,0.5);
              ">
                <div style="font-size: 18px; margin-bottom: 15px; color: #f56565;">无法自动关闭窗口</div>
                <div style="font-size: 14px; margin-bottom: 10px;">请使用以下方式手动关闭：</div>
                <div style="font-size: 12px; color: #666; line-height: 1.5;">
                  • 按 Ctrl+W (Windows) 或 Cmd+W (Mac)<br>
                  • 点击浏览器标签页的 × 按钮<br>
                  • 关闭整个浏览器窗口
                </div>
                <button onclick="location.reload()" style="
                  margin-top: 15px;
                  padding: 8px 16px;
                  background: #4299e1;
                  color: white;
                  border: none;
                  border-radius: 4px;
                  cursor: pointer;
                ">刷新页面</button>
              </div>
            `
          }
        }, 2000)
      }, 500)
      
      return { success: true, action: 'close' }
    } catch (error) {
      console.error('关闭窗口失败:', error)
      return { success: false, error: error.message }
    }
  }

  /**
   * 获取当前最大化状态
   */
  getMaximizedState() {
    return this.isMaximized
  }
}

// 创建全局实例
export const webWindowController = new WebWindowController()