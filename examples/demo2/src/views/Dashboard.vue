<template>
  <div class="dashboard">
    <a-layout class="layout">
      <!-- 使用共享头部组件 -->
      <AppHeader
        title="造价管理系统"
        :show-window-controls="true"
        :is-maximized="isMaximized"
        @minimize="minimizeWindow"
        @maximize="toggleMaximize"
        @close="closeWindow"
      >
        <template #right>
          <div class="user-info">
            <a-space>
              <a-avatar :size="32" style="background-color: rgba(255, 255, 255, 0.2); color: white;">
                <template #icon><UserOutlined /></template>
              </a-avatar>
              <span style="color: white; font-weight: 500;">管理员</span>
            </a-space>
          </div>
        </template>
      </AppHeader>

      <!-- 主内容区 -->
      <a-layout-content class="content">
        <div class="content-wrapper">
          <!-- 欢迎区域 -->
          <a-card class="welcome-card" :bordered="false">
            <div class="welcome-content">
              <h1>欢迎使用造价管理系统</h1>
              <p>基于微前端架构的现代化造价管理解决方案</p>
            </div>
          </a-card>

          <!-- 模块网格 -->
          <div class="modules-grid">
            <a-row :gutter="[24, 24]">
              <a-col :span="8" v-for="module in modules" :key="module.key">
                <a-card 
                  class="module-card" 
                  hoverable
                  @click="openModule(module)"
                >
                  <div class="module-content">
                    <div class="module-icon">
                      <component :is="module.icon" :style="{ fontSize: '48px', color: module.color }" />
                    </div>
                    <h3>{{ module.title }}</h3>
                    <p>{{ module.description }}</p>
                    <a-tag :color="module.status === 'ready' ? 'green' : 'orange'">
                      {{ module.status === 'ready' ? '可用' : '开发中' }}
                    </a-tag>
                  </div>
                </a-card>
              </a-col>
            </a-row>
          </div>

          <!-- 子窗口演示 -->
          <!-- 演示区域可以根据需要添加内容 -->

          <!-- 系统状态 -->
          <a-card title="系统状态" class="status-card">
            <a-row :gutter="16">
              <a-col :span="6">
                <a-statistic title="微前端模块" :value="6" suffix="个" />
              </a-col>
              <a-col :span="6">
                <a-statistic title="已开发模块" :value="1" suffix="个" />
              </a-col>
              <a-col :span="6">
                <a-statistic title="系统状态" value="正常" />
              </a-col>
              <a-col :span="6">
                <a-statistic title="在线用户" :value="1" suffix="人" />
              </a-col>
            </a-row>
          </a-card>
        </div>
      </a-layout-content>
    </a-layout>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { AppHeader, useMainWindowManagement } from '@cost-app/shared-components'
import {
  UserOutlined,
  CalculatorOutlined,
  DollarOutlined,
  CheckCircleOutlined,
  FileTextOutlined,
  AuditOutlined
} from '@ant-design/icons-vue'
// 使用新的简化窗口管理
const {
  isMaximized,
  minimizeWindow: minimize,
  toggleMaximize: toggle,
  closeWindow: close
} = useMainWindowManagement()

// 包装窗口操作函数以添加用户反馈
const minimizeWindow = async () => {
  try {
    await minimize()
    message.success('窗口已最小化')
  } catch (error) {
    message.error('窗口操作失败')
  }
}

const toggleMaximize = async () => {
  try {
    await toggle()
  } catch (error) {
    message.error('窗口操作失败')
  }
}

const closeWindow = async () => {
  try {
    await close()
  } catch (error) {
    message.error('窗口操作失败')
  }
}

const modules = ref([
  {
    key: 'rough-estimate',
    title: '概算',
    description: '项目概算管理和计算',
    icon: CalculatorOutlined,
    color: '#1890ff',
    status: 'ready',
    port: 5174
  },
  {
    key: 'budget',
    title: '预算',
    description: '项目预算编制和管理',
    icon: DollarOutlined,
    color: '#52c41a',
    status: 'development',
    port: 5175
  },
  {
    key: 'budget-review',
    title: '预算审核',
    description: '预算审核流程管理',
    icon: CheckCircleOutlined,
    color: '#faad14',
    status: 'development',
    port: 5176
  },
  {
    key: 'settlement',
    title: '结算',
    description: '项目结算管理',
    icon: FileTextOutlined,
    color: '#722ed1',
    status: 'development',
    port: 5177
  },
  {
    key: 'settlement-review',
    title: '结算审核',
    description: '结算审核流程管理',
    icon: AuditOutlined,
    color: '#eb2f96',
    status: 'development',
    port: 5178
  },
])

const openModule = async (module) => {
  try {
    message.loading(`正在打开${module.title}模块...`, 2)

    // 外部模块：使用开发服务器或静态文件
    const isDev = import.meta.env.DEV
    let url

    if (isDev) {
      // 开发环境：使用开发服务器端口
      url = `http://localhost:${module.port}`
    } else {
      // 生产环境：使用相对路径访问打包后的静态文件
      url = `${module.key}/index.html`
    }

    // 调用 Tauri 命令创建新窗口
    await invoke('create_module_window', {
      moduleKey: module.key,
      title: module.title,
      url: url
    })

    message.success(`${module.title}模块已在新窗口中打开`)
  } catch (error) {
    console.error('打开模块失败:', error)
    message.error(`打开${module.title}模块失败: ${error}`)
  }
}

// 窗口管理通过 useMainWindowManagement 自动初始化
</script>

<style scoped>
.dashboard {
  height: 100vh;
}

.layout {
  height: 100%;
}

.header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.15);
  padding: 0;
  border-bottom: none;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 100%;
  width: 100%;
  padding: 0 24px;
}

.header-center {
  flex: 1;
  height: 100%;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.logo h2 {
  margin: 0;
  color: white;
  font-weight: 600;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.content {
  background: #f0f2f5;
  padding: 24px;
}

.content-wrapper {
  max-width: 1200px;
  margin: 0 auto;
}

.welcome-card {
  margin-bottom: 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
}

.welcome-card :deep(.ant-card-body) {
  padding: 48px 24px;
}

.welcome-content {
  text-align: center;
  color: white;
}

.welcome-content h1 {
  color: white;
  margin-bottom: 8px;
  font-size: 32px;
}

.welcome-content p {
  color: rgba(255, 255, 255, 0.8);
  font-size: 16px;
  margin: 0;
}

.modules-grid {
  margin-bottom: 24px;
}

.module-card {
  height: 200px;
  transition: all 0.3s ease;
  cursor: pointer;
}

.module-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
}

.module-content {
  text-align: center;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
}

.module-icon {
  margin-bottom: 16px;
}

.module-content h3 {
  margin: 8px 0;
  color: #262626;
  font-size: 18px;
}

.module-content p {
  color: #8c8c8c;
  margin-bottom: 12px;
  font-size: 14px;
}

.status-card {
  background: white;
}

/* 窗口控制按钮样式 */
.window-controls {
  display: flex;
  align-items: center;
  gap: 4px;
}

.window-control-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.window-control-btn:hover {
  background-color: rgba(255, 255, 255, 0.2);
}

.window-control-btn.close-btn:hover {
  background-color: #ff4d4f;
  color: white;
}

.window-control-btn {
  color: white;
}

/* 拖动区域样式 */
[data-tauri-drag-region] {
  -webkit-app-region: drag;
}

/* 确保按钮不被拖动 */
.window-controls,
.user-info,
.logo {
  -webkit-app-region: no-drag;
}
</style>
