<template>
  <div id="app">
    <div class="app-container">
      <!-- 使用共享头部组件 -->
      <AppHeader
        title="造价管理系统"
        :show-window-controls="true"
        :use-child-window="true"
        @minimize="onMinimize"
        @maximize="onMaximize"
        @close="onClose"
      >
        <template #center>
          <el-menu
            :default-active="$route.path"
            mode="horizontal"
            router
            class="main-nav"
          >
            <el-menu-item index="/">工作台</el-menu-item>
            <el-menu-item index="/rough-estimate">概算</el-menu-item>
            <el-menu-item index="/budget">预算</el-menu-item>
            <el-menu-item index="/budget-review">预算审核</el-menu-item>
            <el-menu-item index="/settlement">结算</el-menu-item>
            <el-menu-item index="/settlement-review">结算审核</el-menu-item>
          </el-menu>
        </template>
      </AppHeader>

      <!-- 主内容区 -->
      <div class="app-main">
        <router-view />
      </div>
    </div>
  </div>
</template>

<script setup>
import { onMounted } from 'vue'
import { AppHeader } from '@cost-app/shared-components'

// 窗口控制方法
const onMinimize = () => {
  console.log('主应用窗口最小化')
  // AppHeader 组件会自动处理窗口控制逻辑
}

const onMaximize = () => {
  console.log('主应用窗口最大化/还原')
  // AppHeader 组件会自动处理窗口控制逻辑
}

const onClose = () => {
  console.log('主应用窗口关闭')
  // AppHeader 组件会自动处理窗口控制逻辑
}

onMounted(() => {
  console.log('🏠 主应用已启动')
})
</script>

<style scoped>
.app-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.main-nav {
  flex: 1;
  border-bottom: none;
  background: transparent;
}

.main-nav :deep(.el-menu-item) {
  color: white;
  border-bottom: 2px solid transparent;
}

.main-nav :deep(.el-menu-item:hover) {
  background-color: rgba(255, 255, 255, 0.1);
  color: white;
}

.main-nav :deep(.el-menu-item.is-active) {
  background-color: rgba(255, 255, 255, 0.2);
  border-bottom-color: white;
  color: white;
}

.app-main {
  flex: 1;
  background: #f5f7fa;
  padding: 20px;
  overflow-y: auto;
}
</style>
