<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { House, Document, View, Minus, FullScreen, Close, Setting, Clock, User, SwitchButton, ArrowDown, Moon, Sunny } from "@element-plus/icons-vue";
import { Window } from '@tauri-apps/api/window';
import { ElMessage, ElMessageBox } from 'element-plus';
import { invoke } from '@tauri-apps/api/core';
// @ts-ignore
import SettingsDialog from './SettingsDialog.vue';
import { useHistoryStore } from '@/stores/history';
import { useRootStore } from '@/stores/root';
import { useUserStore } from '@/stores/user';

const appWindow = new Window('main');
const router = useRouter();
const route = useRoute();

const activeMenu = ref((route.name as string) || "home");
const showSettings = ref(false);

const historyStore = useHistoryStore();
const rootStore = useRootStore();
const userStore = useUserStore();

// 主题管理
const theme = ref<'light' | 'dark' | 'system'>('system');

// 应用主题
const applyTheme = (newTheme: 'light' | 'dark' | 'system') => {
  theme.value = newTheme;

  if (newTheme === 'system') {
    document.documentElement.removeAttribute('data-theme');
    localStorage.removeItem('theme');
  } else {
    document.documentElement.setAttribute('data-theme', newTheme);
    localStorage.setItem('theme', newTheme);
  }
};

// 切换主题
const toggleTheme = () => {
  const themes: ('light' | 'dark' | 'system')[] = ['system', 'light', 'dark'];
  const currentIndex = themes.indexOf(theme.value);
  const nextTheme = themes[(currentIndex + 1) % themes.length];
  applyTheme(nextTheme);
};

// 获取主题显示文本
const getThemeText = () => {
  switch (theme.value) {
    case 'light': return '浅色主题';
    case 'dark': return '深色主题';
    case 'system': return '跟随系统';
    default: return '跟随系统';
  }
};

// 获取主题图标
const getThemeIcon = () => {
  switch (theme.value) {
    case 'light': return Sunny;
    case 'dark': return Moon;
    case 'system': return Setting;
    default: return Setting;
  }
};

onMounted(() => {
  // 读取保存的主题设置
  const savedTheme = localStorage.getItem('theme') as 'light' | 'dark' | null;
  if (savedTheme) {
    applyTheme(savedTheme);
  }
});

watch(() => historyStore.historyList, (newVal) => {
  menuItems.value[1].children = newVal;
});
const menuItems = ref([
  {
    name: "home",
    label: "首页",
    icon: House,
    path: "/home/dashboard"
  },
  {
    name: "history",
    label: "历史记录",
    icon: Clock,
    children: historyStore.historyList
  }
]);

const handleMenuSelect = (key: string) => {
  const item = menuItems.value.find((item) => item.name === key);
  if (item && item.path) {
    router.push(item.path);
    activeMenu.value = key;
  }
};

const minimizeWindow = () => appWindow.minimize();
const maximizeWindow = () => appWindow.toggleMaximize();
const closeWindow = () => appWindow.close();

const clearHistory = () => {
  //historyStore.clearHistory();
};

// 退出登录
const handleLogout = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要退出登录吗？',
      '确认退出',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );

    // 清除用户状态
    userStore.logout();
    
    // 显示退出成功消息
    ElMessage.success('已退出登录');
    
    // 不需要路由跳转，登录弹窗会自动显示
  } catch (error) {
    // 用户取消退出，什么都不做
    console.log('用户取消退出登录');
  }
};
</script>

<template>
  <div class="main-layout">
    <header class="main-header" data-tauri-drag-region>
      <div class="header-left">
        <h1 class="app-title">ModuForge Demo</h1>
        <div v-if="rootStore.getRootId" class="project-id">
          <el-tag size="small" type="info">项目ID: {{ rootStore.getRootId }}</el-tag>
        </div>
      </div>
      <div class="header-menu">
        <el-menu :default-active="activeMenu" mode="horizontal" @select="handleMenuSelect" background-color="#ffffff"
          text-color="#303133" active-text-color="#409EFF" :ellipsis="false" class="header-nav-menu">
          <template v-for="item in menuItems" :key="item.name">
            <el-sub-menu v-if="item.children" :index="item.name">
              <template #title>
                <el-icon>
                  <component :is="item.icon" />
                </el-icon>
                <span>{{ item.label }}</span>
              </template>
              <div class="history-submenu">
                <div class="history-header">
                  <span>历史记录</span>
                  <el-button type="danger" size="small" @click.stop="clearHistory">清空</el-button>
                </div>
                <el-scrollbar max-height="300px">
                  <div class="history-list">

                    <div v-for="history in item.children" :key="history.state_version" class="history-item">
                      <div class="history-item-header">
                        <span class="history-title">{{ history.description }}</span>
                        <el-tag size="small"
                          :type="history.type === '创建' ? 'success' : history.type === '修改' ? 'warning' : 'danger'">
                          {{ history.type || "操作" }}
                        </el-tag>
                      </div>
                      <div class="history-time">{{ history.timestamp }}</div>
                    </div>
                  </div>
                </el-scrollbar>
              </div>
            </el-sub-menu>
            <el-menu-item v-else :index="item.name">
              <el-icon>
                <component :is="item.icon" />
              </el-icon>
              <span>{{ item.label }}</span>
            </el-menu-item>
          </template>
        </el-menu>
      </div>
      <!-- 用户信息区域 -->
      <div class="user-section">
        <el-dropdown trigger="click" placement="bottom-end">
          <div class="user-info">
            <el-avatar :size="32" :icon="User" />
            <span class="user-name">{{ userStore.userName }}</span>
            <el-icon class="dropdown-icon">
              <ArrowDown />
            </el-icon>
          </div>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item>
                <el-icon>
                  <User />
                </el-icon>
                个人信息
              </el-dropdown-item>
              <el-dropdown-item @click="toggleTheme">
                <el-icon>
                  <component :is="getThemeIcon()" />
                </el-icon>
                {{ getThemeText() }}
              </el-dropdown-item>
              <el-dropdown-item @click="showSettings = true">
                <el-icon>
                  <Setting />
                </el-icon>
                系统设置
              </el-dropdown-item>
              <el-dropdown-item divided @click="handleLogout">
                <el-icon>
                  <SwitchButton />
                </el-icon>
                退出登录
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>

      <div class="window-controls">
        <button class="window-control-button" @click="minimizeWindow">
          <el-icon>
            <Minus />
          </el-icon>
        </button>
        <button class="window-control-button" @click="maximizeWindow">
          <el-icon>
            <FullScreen />
          </el-icon>
        </button>
        <button class="window-control-button close" @click="closeWindow">
          <el-icon>
            <Close />
          </el-icon>
        </button>
      </div>
    </header>
    <div class="main-content">
      <router-view />
    </div>
    <SettingsDialog v-model="showSettings" />
  </div>
</template>

<style scoped>
.main-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background: var(--el-bg-color);
  position: relative;
  overflow: hidden;
}

.main-layout::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background:
    radial-gradient(circle at 20% 80%, rgba(120, 119, 198, 0.3) 0%, transparent 50%),
    radial-gradient(circle at 80% 20%, rgba(255, 255, 255, 0.15) 0%, transparent 50%),
    radial-gradient(circle at 40% 40%, rgba(120, 119, 198, 0.2) 0%, transparent 50%);
  pointer-events: none;
  z-index: 0;
}

.main-header {
  height: 60px;
  display: flex;
  align-items: center;
  padding: 0 20px;
  border-bottom: 1px solid var(--el-border-color);
  flex-shrink: 0;
  background: var(--el-bg-color);
  gap: 40px;
  position: relative;
  z-index: 1;
  box-shadow: 0 1px 4px var(--el-box-shadow-light);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.app-title {
  font-size: 20px;
  color: var(--el-text-color-primary);
  margin: 0;
  font-weight: 600;
}

.project-id {
  display: flex;
  align-items: center;
}

.header-menu {
  display: flex;
}

.header-nav-menu {
  border-bottom: none !important;
  min-width: auto !important;
  background: transparent !important;
}

.header-nav-menu .el-menu-item {
  border-bottom: none !important;
  white-space: nowrap;
  overflow: visible;
  text-overflow: unset;
  color: var(--el-text-color-primary) !important;
  background: transparent !important;
}

.header-nav-menu .el-menu-item:hover {
  background: var(--el-fill-color-light) !important;
  color: var(--el-text-color-primary) !important;
}

.header-nav-menu .el-menu-item.is-active {
  background: var(--el-color-primary-light-9) !important;
  color: var(--el-color-primary) !important;
  border-bottom: 2px solid var(--el-color-primary) !important;
}

.main-content {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
  z-index: 1;
  margin: 8px;
  border-radius: 8px;
  background: var(--el-bg-color-page);
  box-shadow: var(--el-box-shadow);
  border: 1px solid var(--el-border-color-lighter);
}

.user-section {
  margin-left: auto;
  margin-right: 16px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  border-radius: 6px;
  transition: background-color 0.2s;
  background: transparent;
  border: 1px solid var(--el-border-color-light);
}

.user-info:hover {
  background: var(--el-fill-color-light);
}

.user-name {
  font-size: 14px;
  color: var(--el-text-color-primary);
  font-weight: 500;
  white-space: nowrap;
}

.dropdown-icon {
  font-size: 12px;
  color: var(--el-text-color-regular);
  transition: transform 0.2s;
}

.window-controls {
  display: flex;
  gap: 8px;
}

.window-control-button {
  width: 30px;
  height: 30px;
  border: none;
  background: transparent;
  color: var(--el-text-color-regular);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
  border: 1px solid var(--el-border-color-light);
}

.window-control-button:hover {
  background: var(--el-fill-color-light);
  color: var(--el-text-color-primary);
}

.window-control-button.close:hover {
  background: var(--el-color-danger);
  color: #ffffff;
}

/* 暗色主题适配 */
@media (prefers-color-scheme: dark) {
  .main-layout {
    background: #1a1a1a;
  }

  .main-header {
    background: #1a1a1a;
    border-bottom: 1px solid #2d2d2d;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
  }

  .main-content {
    background: #212121;
    border: 1px solid #2d2d2d;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  }

  .user-info {
    border: 1px solid #2d2d2d;
  }

  .user-info:hover {
    background: #2d2d2d;
  }

  .window-control-button {
    border: 1px solid #2d2d2d;
  }

  .window-control-button:hover {
    background: #2d2d2d;
  }
}

/* 历史记录子菜单样式 */
.history-submenu {
  min-width: 300px;
  padding: 8px 0;
}

.history-header {
  padding: 8px 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #e4e7ed;
  margin-bottom: 8px;
}

.history-header span {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
}

.history-list {
  padding: 0 12px;
}

.history-item {
  padding: 8px 0;
  border-bottom: 1px solid #f0f0f0;
}

.history-item:last-child {
  border-bottom: none;
}

.history-item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.history-title {
  font-size: 13px;
  color: #303133;
  flex: 1;
  margin-right: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-time {
  font-size: 12px;
  color: #909399;
}

:deep(.el-sub-menu__title) {
  height: 60px;
  line-height: 60px;
}

:deep(.el-menu--popup) {
  min-width: 300px;
  padding: 0;
}

:deep(.el-menu--horizontal > .el-sub-menu .el-sub-menu__icon-arrow) {
  right: 0;
}

:deep(.el-menu--horizontal > .el-sub-menu .el-sub-menu__title) {
  padding-right: 20px;
}

:deep(.el-menu--popup-container) {
  right: 0 !important;
  left: auto !important;
}

/* 添加历史记录菜单靠右的样式 */
.header-menu :deep(.el-menu-item:last-child) {
  margin-left: auto;
}
</style>
