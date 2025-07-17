<script setup lang="ts">
import { computed, onMounted, ref, defineAsyncComponent } from 'vue';
import { useUserStore } from '@/stores/user';

const LoginDialog = defineAsyncComponent(() => import('@/components/LoginDialog.vue'));

const userStore = useUserStore();
const isInitialized = ref(false);

// 计算是否需要显示登录弹窗 - 只有在初始化完成后才显示
const showLoginDialog = computed({
  get: () => isInitialized.value && !userStore.isLoggedIn,
  set: (value: boolean) => {
    // 如果试图关闭登录弹窗但用户未登录，则不允许关闭
    // 弹窗只能通过成功登录来关闭
  }
});

// 应用初始化时尝试恢复登录状态
onMounted(async () => {
  console.log('App 初始化开始');
  
  // 先尝试从存储恢复用户状态
  userStore.initUserFromStorage();
  
  // 标记为已初始化
  isInitialized.value = true;
  
  console.log('App 初始化完成，登录状态:', userStore.isLoggedIn);
});
</script>

<template>
  <div id="app">
    <!-- 主要内容区域 -->
    <router-view v-if="isInitialized" />
    
    <!-- 加载状态 -->
    <div v-else class="app-loading">
      <div class="loading-content">
        <div class="loading-spinner"></div>
        <p>应用初始化中...</p>
      </div>
    </div>
    
    <!-- 全局登录弹窗 -->
    <LoginDialog v-model="showLoginDialog" />
  </div>
</template>

<style>
/* Resetting default margin and padding */
body,
html {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

#app {
  padding: 0;
  margin: 0;
  height: 100vh;
  width: 100vw;
}

.app-loading {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.loading-content {
  text-align: center;
  color: white;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  border-top-color: white;
  animation: spin 1s ease-in-out infinite;
  margin: 0 auto 15px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
