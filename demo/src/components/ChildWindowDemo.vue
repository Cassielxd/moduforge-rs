<template>
  <div class="child-window-demo">
    <div class="demo-header">
      <h2>🪟 子窗口演示</h2>
      <p>演示 Tauri 中的模态和非模态子窗口功能</p>
    </div>

    <div class="demo-section">
      <h3>📋 表单子窗口</h3>
      <p>在概算操作中打开复杂表单窗口</p>
      
      <div class="button-group">
        <a-button 
          type="primary" 
          @click="openFormWindow(false)"
          :loading="loading"
        >
          <template #icon><WindowsOutlined /></template>
          打开非模态表单
        </a-button>
        
        <a-button 
          type="primary" 
          danger
          @click="openFormWindow(true)"
          :loading="loading"
        >
          <template #icon><LockOutlined /></template>
          打开模态表单
        </a-button>
      </div>
      
      <div class="info-box">
        <p><strong>非模态窗口：</strong>可以同时操作主窗口和子窗口</p>
        <p><strong>模态窗口：</strong>必须先关闭子窗口才能操作主窗口</p>
      </div>
    </div>

    <div class="demo-section">
      <h3>⚙️ 设置子窗口</h3>
      <p>打开系统设置或配置窗口</p>
      
      <div class="button-group">
        <a-button 
          @click="openSettingsWindow(false)"
          :loading="loading"
        >
          <template #icon><SettingOutlined /></template>
          普通设置窗口
        </a-button>
        
        <a-button 
          @click="openSettingsWindow(true)"
          :loading="loading"
        >
          <template #icon><ExclamationCircleOutlined /></template>
          重要设置（模态）
        </a-button>
      </div>
    </div>

    <div class="demo-section">
      <h3>📊 数据查看窗口</h3>
      <p>查看详细数据或报表</p>
      
      <div class="button-group">
        <a-button 
          @click="openDataWindow"
          :loading="loading"
        >
          <template #icon><BarChartOutlined /></template>
          打开数据窗口
        </a-button>
      </div>
    </div>

    <div class="demo-section">
      <h3>📊 概算表单演示</h3>
      <p>完整的概算表单窗口演示，包含数据表格和表单操作</p>

      <div class="button-group">
        <a-button
          type="primary"
          @click="openEstimateDemo"
          :loading="loading"
        >
          <template #icon><CalculatorOutlined /></template>
          打开概算演示
        </a-button>
      </div>
    </div>

    <div class="demo-section">
      <h3>🎛️ 窗口控制</h3>
      <div class="button-group">
        <a-button
          @click="closeAllChildWindows"
          :loading="loading"
        >
          <template #icon><CloseOutlined /></template>
          关闭所有子窗口
        </a-button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import {
  WindowsOutlined,
  LockOutlined,
  SettingOutlined,
  ExclamationCircleOutlined,
  BarChartOutlined,
  CloseOutlined,
  CalculatorOutlined
} from '@ant-design/icons-vue'

const loading = ref(false)

// 打开表单子窗口
const openFormWindow = async (modal = false) => {
  try {
    loading.value = true
    
    const windowId = modal ? 'modal-form' : 'form-window'
    const title = modal ? '模态表单窗口' : '表单窗口'
    
    // 这里可以是你的表单页面URL
    const url = '/form-page' // 或者外部URL如 'http://localhost:5174/form'
    
    await invoke('create_child_window', {
      windowId,
      title,
      url,
      modal,
      width: 900,
      height: 700,
      parentWindow: 'main'
    })
    
    message.success(`${title}已打开`)
  } catch (error) {
    console.error('打开表单窗口失败:', error)
    message.error(`打开窗口失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 打开设置子窗口
const openSettingsWindow = async (modal = false) => {
  try {
    loading.value = true
    
    const windowId = modal ? 'modal-settings' : 'settings-window'
    const title = modal ? '重要设置（模态）' : '系统设置'
    
    const url = '/settings-page'
    
    await invoke('create_child_window', {
      windowId,
      title,
      url,
      modal,
      width: 800,
      height: 600,
      parentWindow: 'main'
    })
    
    message.success(`${title}已打开`)
  } catch (error) {
    console.error('打开设置窗口失败:', error)
    message.error(`打开窗口失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 打开数据查看窗口
const openDataWindow = async () => {
  try {
    loading.value = true
    
    await invoke('create_child_window', {
      windowId: 'data-viewer',
      title: '数据查看器',
      url: '/data-page',
      modal: false,
      width: 1200,
      height: 800,
      parentWindow: 'main'
    })
    
    message.success('数据查看器已打开')
  } catch (error) {
    console.error('打开数据窗口失败:', error)
    message.error(`打开窗口失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 打开概算演示
const openEstimateDemo = async () => {
  try {
    loading.value = true

    await invoke('create_child_window', {
      windowId: 'estimate-demo',
      title: '概算表单演示',
      url: '/estimate-demo',
      modal: false,
      width: 1400,
      height: 900,
      parentWindow: 'main'
    })

    message.success('概算演示窗口已打开')
  } catch (error) {
    console.error('打开概算演示失败:', error)
    message.error(`打开窗口失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 关闭所有子窗口
const closeAllChildWindows = async () => {
  try {
    loading.value = true

    const childWindows = ['modal-form', 'form-window', 'modal-settings', 'settings-window', 'data-viewer', 'estimate-demo']

    for (const windowId of childWindows) {
      try {
        await invoke('close_child_window', {
          windowId,
          parentWindow: 'main'
        })
      } catch (error) {
        // 忽略不存在的窗口错误
        console.log(`窗口 ${windowId} 不存在或已关闭`)
      }
    }

    message.success('所有子窗口已关闭')
  } catch (error) {
    console.error('关闭窗口失败:', error)
    message.error(`关闭窗口失败: ${error}`)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.child-window-demo {
  padding: 24px;
  max-width: 800px;
  margin: 0 auto;
}

.demo-header {
  text-align: center;
  margin-bottom: 32px;
  padding-bottom: 16px;
  border-bottom: 1px solid #f0f0f0;
}

.demo-header h2 {
  margin-bottom: 8px;
  color: #1890ff;
}

.demo-section {
  margin-bottom: 32px;
  padding: 20px;
  background: #fafafa;
  border-radius: 8px;
  border: 1px solid #f0f0f0;
}

.demo-section h3 {
  margin-bottom: 8px;
  color: #262626;
}

.demo-section p {
  margin-bottom: 16px;
  color: #595959;
}

.button-group {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}

.info-box {
  background: #e6f7ff;
  border: 1px solid #91d5ff;
  border-radius: 6px;
  padding: 12px;
  margin-top: 16px;
}

.info-box p {
  margin-bottom: 4px;
  font-size: 13px;
  color: #0050b3;
}

.info-box p:last-child {
  margin-bottom: 0;
}
</style>
