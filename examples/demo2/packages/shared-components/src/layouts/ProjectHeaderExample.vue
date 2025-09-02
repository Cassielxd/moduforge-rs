<template>
  <div class="project-header-example">
    <!-- 使用迁移后的 ProjectHeader 组件 -->
    <SimpleHeader
      :title="projectTitle"
      :project-path="projectPath"
      :project-type="projectType"
      :logo-url="logoUrl"
      :save-status="saveStatus"
      :current-tab="currentTab"
      :menu-items="menuItems"
      :show-save-button="true"
      :show-remote-assist="true"
      :show-user-info="true"
      :show-feedback="showFeedback"
      :show-menu-feedback="false"
      :show-help-center="true"
      :show-window-controls="true"
      @save="handleSave"
      @remote-assist="handleRemoteAssist"
      @external-link="handleExternalLink"
      @menu-click="handleMenuClick"
      @file-menu-click="handleFileMenuClick"
      @help-click="handleHelpClick"
      @minimize="handleMinimize"
      @maximize="handleMaximize"
      @close="handleClose"
      @window-state-change="handleWindowStateChange"
      @tab-change="handleTabChange"
    >
      <!-- 用户信息插槽 -->
      <template #user-info>
        <div class="user-info-content">
          <div class="avatar"></div>
          <span class="username">{{ userInfo.name || '用户' }}</span>
        </div>
      </template>
    </SimpleHeader>

    <!-- 页面内容区域 -->
    <div class="page-content">
      <h2>ProjectHeader 迁移示例</h2>
      <div class="status-panel">
        <h3>组件状态</h3>
        <ul>
          <li>项目路径: {{ projectPath }}</li>
          <li>项目类型: {{ projectType }}</li>
          <li>保存状态: {{ saveStatus ? '已保存' : '未保存' }}</li>
          <li>当前选项卡: {{ currentTab }}</li>
          <li>用户信息: {{ userInfo.name }}</li>
        </ul>
      </div>

      <div class="action-panel">
        <h3>测试功能</h3>
        <div class="button-group">
          <button @click="toggleSaveStatus" class="test-btn">
            切换保存状态
          </button>
          <button @click="changeProjectType" class="test-btn">
            切换项目类型
          </button>
          <button @click="updateProjectPath" class="test-btn">
            更新项目路径
          </button>
          <button @click="toggleFeedback" class="test-btn">
            切换反馈显示
          </button>
        </div>
      </div>

      <div class="event-log">
        <h3>事件日志</h3>
        <div class="log-container">
          <div
            v-for="(log, index) in eventLogs"
            :key="index"
            class="log-item"
            :class="log.type"
          >
            <span class="timestamp">{{ log.timestamp }}</span>
            <span class="event">{{ log.event }}</span>
            <span class="data">{{ log.data }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import SimpleHeader from './SimpleHeader.vue'

// 组件状态
const projectTitle = ref('建设工程计价管理软件')
const projectPath = ref('C:\\Projects\\MyProject\\project.ysf')
const projectType = ref('ys') // 'ys' | 'yssh' | 'jieSuan' | 'DW'
const saveStatus = ref(true)
const currentTab = ref('customize')
const showFeedback = ref(true)

// Logo URL (使用base64占位图片)
const logoUrl = ref('data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAIsAAAATCAMAAAClPb9HAAAAVFBMVEUAAAD///8AAAD///8AAAD///8AAAD///8AAAD///8AAAD///8AAAD///8AAAD///8AAAD///8AAAD///8AAAD///8AAAD///8AAAD///8AAAD///8AAAD///9FTzD9AAAAG3RSTlMAECAwQFBgcICPn6+/z9/v7+/v7+/v7+/v7+/vGXjVXgAAAHBJREFUSMft0rEJACAQQ9Gg4P7rxU2s7BfPx0teV0TEd/QfEREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREREXlRByCcAOXVAT9MAAAAASUVORK5CYII=')

// 用户信息
const userInfo = ref({
  name: '测试用户',
  avatar: '',
  loginType: 'normal'
})

// 菜单配置
const menuItems = computed(() => {
  const baseMenus = [
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

  // 根据项目类型添加不同菜单
  if (projectType.value === 'yssh') {
    baseMenus.push({ name: '分析与报告', key: 'analysisAndReporting' })
  }

  if (projectType.value !== 'jieSuan') {
    baseMenus.push({ name: '电子标', key: 'electronicLabel' })
  }

  if (projectType.value === 'jieSuan') {
    baseMenus.push({ name: '指标', key: 'quota' })
  }

  return baseMenus
})

// 事件日志
const eventLogs = ref([])

// 添加事件日志
const addEventLog = (event, data = '', type = 'info') => {
  const log = {
    timestamp: new Date().toLocaleTimeString(),
    event,
    data: typeof data === 'object' ? JSON.stringify(data) : String(data),
    type
  }
  eventLogs.value.unshift(log)
  
  // 限制日志数量
  if (eventLogs.value.length > 50) {
    eventLogs.value = eventLogs.value.slice(0, 50)
  }
}

// 事件处理方法
const handleSave = (data) => {
  addEventLog('保存文件', data, 'success')
  // 模拟保存过程
  setTimeout(() => {
    saveStatus.value = true
    addEventLog('保存完成', '', 'success')
  }, 1000)
}

const handleRemoteAssist = () => {
  addEventLog('远程协助请求', '', 'info')
}

const handleExternalLink = (url) => {
  addEventLog('外部链接点击', url, 'info')
}

const handleMenuClick = (key) => {
  addEventLog('菜单点击', key, 'info')
  currentTab.value = key
}

const handleFileMenuClick = (key) => {
  addEventLog('文件菜单点击', key, 'info')
  
  switch (key) {
    case 'open':
      addEventLog('打开文件对话框', '', 'info')
      break
    case 'new':
      addEventLog('新建项目', '', 'info')
      break
    case 'save':
      handleSave({ showLoading: true })
      break
    case 'asideSave':
      addEventLog('另存为对话框', '', 'info')
      break
    case 'setup':
      addEventLog('打开设置面板', '', 'info')
      break
    case 'checkUpdate':
      addEventLog('检查更新', '', 'info')
      break
  }
}

const handleHelpClick = (type) => {
  const helpTypes = { 1: '政策文件', 2: '勘误说明' }
  addEventLog('帮助中心点击', helpTypes[type] || type, 'info')
}

const handleMinimize = () => {
  addEventLog('窗口最小化', '', 'warning')
}

const handleMaximize = () => {
  addEventLog('窗口最大化/还原', '', 'warning')
}

const handleClose = () => {
  addEventLog('窗口关闭', '', 'error')
}

const handleWindowStateChange = (state) => {
  addEventLog('窗口状态变化', state, 'info')
}

const handleTabChange = (tab) => {
  addEventLog('选项卡变化', tab, 'info')
}

// 测试方法
const toggleSaveStatus = () => {
  saveStatus.value = !saveStatus.value
  addEventLog('保存状态切换', saveStatus.value ? '已保存' : '未保存', 'info')
}

const changeProjectType = () => {
  const types = ['ys', 'yssh', 'jieSuan', 'DW']
  const currentIndex = types.indexOf(projectType.value)
  const nextIndex = (currentIndex + 1) % types.length
  projectType.value = types[nextIndex]
  addEventLog('项目类型切换', projectType.value, 'info')
}

const updateProjectPath = () => {
  const paths = [
    'C:\\Projects\\MyProject\\project.ysf',
    'D:\\Work\\Budget2023\\main.ysf',
    'E:\\Documents\\Construction\\building.ysf'
  ]
  const currentIndex = paths.indexOf(projectPath.value)
  const nextIndex = (currentIndex + 1) % paths.length
  projectPath.value = paths[nextIndex]
  addEventLog('项目路径更新', projectPath.value, 'info')
}

const toggleFeedback = () => {
  showFeedback.value = !showFeedback.value
  addEventLog('反馈显示切换', showFeedback.value ? '显示' : '隐藏', 'info')
}
</script>

<style scoped>
.project-header-example {
  min-height: 100vh;
  background: #f5f5f5;
}

.page-content {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.page-content h2 {
  color: #333;
  margin-bottom: 20px;
}

.page-content h3 {
  color: #555;
  margin: 16px 0 12px 0;
  font-size: 16px;
}

.status-panel,
.action-panel,
.event-log {
  background: white;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.status-panel ul {
  list-style: none;
  padding: 0;
  margin: 0;
}

.status-panel li {
  padding: 8px 0;
  border-bottom: 1px solid #eee;
  color: #666;
}

.status-panel li:last-child {
  border-bottom: none;
}

.button-group {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.test-btn {
  padding: 8px 16px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  color: #333;
  cursor: pointer;
  transition: all 0.2s ease;
}

.test-btn:hover {
  background: #f0f0f0;
  border-color: #bbb;
}

.test-btn:active {
  transform: translateY(1px);
}

.log-container {
  max-height: 300px;
  overflow-y: auto;
  border: 1px solid #eee;
  border-radius: 4px;
}

.log-item {
  display: flex;
  gap: 12px;
  padding: 8px 12px;
  border-bottom: 1px solid #f0f0f0;
  font-size: 12px;
  font-family: monospace;
}

.log-item:last-child {
  border-bottom: none;
}

.log-item.info {
  background: #f8f9fa;
}

.log-item.success {
  background: #d4edda;
  color: #155724;
}

.log-item.warning {
  background: #fff3cd;
  color: #856404;
}

.log-item.error {
  background: #f8d7da;
  color: #721c24;
}

.timestamp {
  color: #666;
  flex-shrink: 0;
  width: 80px;
}

.event {
  font-weight: bold;
  flex-shrink: 0;
  width: 120px;
}

.data {
  color: #888;
  word-break: break-all;
}

.user-info-content {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  line-height: 32px;
}

.avatar {
  display: inline-block;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background-color: #dfdfdf;
}

.username {
  display: inline-block;
  max-width: 62px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  color: #fff;
  font-size: 12px;
}
</style>