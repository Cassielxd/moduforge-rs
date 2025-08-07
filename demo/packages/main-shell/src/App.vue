<template>
  <div id="app">
    <el-container class="app-container">
      <!-- 顶部导航 -->
      <el-header class="app-header">
        <div class="header-content">
          <div class="logo">
            <h2>造价管理系统</h2>
          </div>
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
        </div>
      </el-header>

      <!-- 主内容区 -->
      <el-main class="app-main">
        <router-view />
      </el-main>
    </el-container>
  </div>
</template>

<script setup>
import { onMounted } from 'vue'

onMounted(() => {
  console.log('🏠 主应用已启动')
})
</script>

<style scoped>
.app-container {
  height: 100vh;
}

.app-header {
  background: #fff;
  border-bottom: 1px solid #e4e7ed;
  padding: 0;
}

.header-content {
  display: flex;
  align-items: center;
  height: 100%;
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 20px;
}

.logo {
  margin-right: 40px;
}

.logo h2 {
  margin: 0;
  color: #409eff;
  font-weight: 600;
}

.main-nav {
  flex: 1;
  border-bottom: none;
}

.app-main {
  background: #f5f7fa;
  padding: 20px;
}
</style>
