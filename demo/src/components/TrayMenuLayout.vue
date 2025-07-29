<script setup>
import { onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Monitor, Close } from '@element-plus/icons-vue';

// 处理显示主窗口
const handleShowMainWindow = async () => {
    try {
        await invoke('show_main_window');
        await hideTrayMenu();
    } catch (error) {
        console.error('显示主窗口失败:', error);
    }
};

// 处理退出应用
const handleQuitApp = async () => {
    try {
        await invoke('quit_app');
    } catch (error) {
        console.error('退出应用失败:', error);
    }
};

// 隐藏托盘菜单
const hideTrayMenu = async () => {
    try {
        await invoke('hide_tray_menu');
    } catch (error) {
        console.error('隐藏托盘菜单失败:', error);
    }
};

onMounted(() => {
    // 监听窗口失去焦点事件
    const handleBlur = () => {
        setTimeout(() => {
            hideTrayMenu();
        }, 200);
    };

    // 监听键盘 ESC 事件
    const handleKeyDown = (event: KeyboardEvent) => {
        if (event.key === 'Escape') {
            hideTrayMenu();
        }
    };

    window.addEventListener('blur', handleBlur);
    document.addEventListener('keydown', handleKeyDown);

    onUnmounted(() => {
        window.removeEventListener('blur', handleBlur);
        document.removeEventListener('keydown', handleKeyDown);
    });
});
</script>

<template>
    <div class="tray-menu-layout">
        <!-- 直接内联菜单内容 -->
        <div class="tray-menu" @click.stop>
            <div class="menu-items">
                <div class="menu-item" @click="handleShowMainWindow">
                    <el-icon>
                        <Monitor />
                    </el-icon>
                    <span>显示窗口</span>
                </div>
                <div class="menu-divider"></div>
                <div class="menu-item danger" @click="handleQuitApp">
                    <el-icon>
                        <Close />
                    </el-icon>
                    <span>退出应用</span>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.tray-menu-layout {
    width: 100vw;
    height: 100vh;
    margin: 0;
    padding: 0;
    background: transparent;
    overflow: hidden;
}

.tray-menu {
    width: 100%;
    height: 100%;
    background: var(--el-bg-color);
    border: 1px solid var(--el-border-color);
    border-radius: 6px;
    box-shadow: var(--el-box-shadow);
    overflow: hidden;
    backdrop-filter: blur(10px);
}

.menu-items {
    padding: 4px 0;
}

.menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    cursor: pointer;
    font-size: 14px;
    color: var(--el-text-color-primary);
    transition: background-color 0.2s;
}

.menu-item:hover {
    background-color: var(--el-fill-color-light);
}

.menu-item.danger {
    color: var(--el-color-danger);
}

.menu-item.danger:hover {
    background-color: var(--el-color-danger-light-9);
}

.menu-divider {
    height: 1px;
    background-color: var(--el-border-color-lighter);
    margin: 4px 0;
}

.el-icon {
    font-size: 16px;
}

/* 暗色主题适配 */
@media (prefers-color-scheme: dark) {
    .tray-menu {
        background: #1a1a1a;
        border: 1px solid #2d2d2d;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    }

    .menu-divider {
        background-color: #2d2d2d;
    }
}
</style>