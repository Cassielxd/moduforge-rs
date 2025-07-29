<script setup>
import { ref, reactive } from 'vue';
import { ElMessage } from 'element-plus';
import { User, Lock, Document, Close } from '@element-plus/icons-vue';
import { useUserStore } from '@/stores/user';
import { invoke } from '@tauri-apps/api/core';

const userStore = useUserStore();
const loading = ref(false);

// 控制弹窗显示
const props = defineProps({
    modelValue: {
        type: Boolean,
        default: false,
    },
});

const emit = defineEmits('update:modelValue');

// 登录表单数据
const loginForm = reactive({
    username: '',
    password: '',
    remember: false
});

// 表单验证规则
const rules = {
    username: [
        { required: true, message: '请输入用户名', trigger: 'blur' },
        { min: 3, max: 20, message: '用户名长度在 3 到 20 个字符', trigger: 'blur' }
    ],
    password: [
        { required: true, message: '请输入密码', trigger: 'blur' },
        { min: 6, max: 20, message: '密码长度在 6 到 20 个字符', trigger: 'blur' }
    ]
};

const loginFormRef = ref();

// 退出应用
const handleQuit = async () => {
    try {
        await invoke('quit_app');
    } catch (error) {
        console.error('退出应用失败:', error);
        // 备用方案
        window.close();
    }
};

// 登录处理
const handleLogin = async () => {
    try {
        const valid = await loginFormRef.value.validate();
        if (!valid) return;

        loading.value = true;

        // 模拟登录请求
        await new Promise(resolve => setTimeout(resolve, 1500));

        // 模拟用户信息（实际项目中从API获取）
        const mockUser = {
            id: '1001',
            username: loginForm.username,
            nickname: loginForm.username === 'admin' ? '系统管理员' : '用户',
            email: `${loginForm.username}@moduforge.com`,
            role: loginForm.username === 'admin' ? 'admin' : 'user',
            permissions: loginForm.username === 'admin'
                ? ['read', 'write', 'delete', 'admin']
                : ['read', 'write']
        };

        const mockToken = `token_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

        // 保存用户登录状态
        userStore.login(mockUser, mockToken);

        // 登录成功后的处理
        ElMessage.success('登录成功！');

        // 关闭弹窗
        emit('update:modelValue', false);

        // 重置表单
        loginForm.username = '';
        loginForm.password = '';
        loginForm.remember = false;

    } catch (error) {
        console.error('登录失败:', error);
        ElMessage.error('登录失败，请检查用户名和密码');
    } finally {
        loading.value = false;
    }
};

// 重置表单
const handleReset = () => {
    loginFormRef.value.resetFields();
};

// 快速登录（演示用）
const quickLogin = () => {
    loginForm.username = 'admin';
    loginForm.password = 'admin123';
    handleLogin();
};
</script>

<template>
    <el-dialog :model-value="modelValue" @update:model-value="emit('update:modelValue', $event)" width="420px"
        :close-on-click-modal="false" :close-on-press-escape="false" :show-close="false" :modal="true"
        :append-to-body="true" class="minimal-login-dialog" :show-header="false">
        <div class="login-container">
            <!-- 关闭按钮 -->
            <div class="close-button" @click="handleQuit">
                <el-icon :size="16">
                    <Close />
                </el-icon>
            </div>

            <!-- Logo 和标题 -->
            <div class="header-section">
                <div class="logo-container">
                    <el-icon :size="40" color="#007AFF">
                        <Document />
                    </el-icon>
                </div>
                <h1 class="app-name">ModuForge Demo</h1>
                <p class="app-description">登录您的账户</p>
            </div>

            <!-- 登录表单 -->
            <div class="form-section">
                <el-form ref="loginFormRef" :model="loginForm" :rules="rules" class="login-form"
                    @submit.prevent="handleLogin">
                    <el-form-item prop="username">
                        <el-input v-model="loginForm.username" placeholder="用户名" size="large" class="clean-input"
                            clearable>
                            <template #prefix>
                                <el-icon color="#8E8E93">
                                    <User />
                                </el-icon>
                            </template>
                        </el-input>
                    </el-form-item>

                    <el-form-item prop="password">
                        <el-input v-model="loginForm.password" type="password" placeholder="密码" size="large"
                            class="clean-input" show-password clearable @keyup.enter="handleLogin">
                            <template #prefix>
                                <el-icon color="#8E8E93">
                                    <Lock />
                                </el-icon>
                            </template>
                        </el-input>
                    </el-form-item>

                    <div class="form-options">
                        <el-checkbox v-model="loginForm.remember" class="remember-checkbox">
                            记住我
                        </el-checkbox>
                        <el-link class="forgot-password" :underline="false">
                            忘记密码？
                        </el-link>
                    </div>

                    <el-button type="primary" size="large" :loading="loading" @click="handleLogin" class="login-button">
                        {{ loading ? '登录中...' : '登录' }}
                    </el-button>
                </el-form>

                <!-- 分割线 -->
                <div class="divider">
                    <span>或</span>
                </div>

                <!-- 快速登录 -->
                <el-button @click="quickLogin" class="demo-button" size="large" plain>
                    演示登录 (admin/admin123)
                </el-button>
            </div>
        </div>
    </el-dialog>
</template>

<style scoped>
/* 弹窗基础样式 */
.minimal-login-dialog {
    --el-dialog-margin-top: 10vh;
}

.minimal-login-dialog :deep(.el-dialog) {
    background: #ffffff;
    border-radius: 16px;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
    border: 1px solid rgba(0, 0, 0, 0.06);
    margin: 10vh auto 0;
}

.minimal-login-dialog :deep(.el-dialog__body) {
    padding: 0;
}

.minimal-login-dialog :deep(.el-overlay) {
    background: rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(10px);
}

/* 主容器 */
.login-container {
    position: relative;
    padding: 32px;
    width: 100%;
}

/* 关闭按钮 */
.close-button {
    position: absolute;
    top: 20px;
    right: 20px;
    width: 32px;
    height: 32px;
    border-radius: 8px;
    background: #F2F2F7;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s ease;
    color: #8E8E93;
}

.close-button:hover {
    background: #E5E5EA;
    color: #48484A;
}

/* 头部区域 */
.header-section {
    text-align: center;
    margin-bottom: 32px;
}

.logo-container {
    margin-bottom: 16px;
}

.app-name {
    margin: 0 0 8px 0;
    font-size: 28px;
    font-weight: 600;
    color: #1D1D1F;
    letter-spacing: -0.5px;
}

.app-description {
    margin: 0;
    font-size: 16px;
    color: #86868B;
    font-weight: 400;
}

/* 表单区域 */
.form-section {
    width: 100%;
}

.login-form {
    margin-bottom: 24px;
}

.login-form .el-form-item {
    margin-bottom: 16px;
}

/* 输入框样式 */
.clean-input :deep(.el-input__wrapper) {
    background: #F2F2F7;
    border: 1px solid transparent;
    border-radius: 12px;
    box-shadow: none;
    height: 50px;
    padding: 0 16px;
    transition: all 0.2s ease;
}

.clean-input :deep(.el-input__wrapper:hover) {
    background: #E8E8ED;
}

.clean-input :deep(.el-input__wrapper.is-focus) {
    background: #ffffff;
    border-color: #007AFF;
    box-shadow: 0 0 0 1px #007AFF;
}

.clean-input :deep(.el-input__inner) {
    font-size: 16px;
    color: #1D1D1F;
    font-weight: 400;
}

.clean-input :deep(.el-input__inner::placeholder) {
    color: #8E8E93;
    font-weight: 400;
}

/* 表单选项 */
.form-options {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
}

.remember-checkbox :deep(.el-checkbox__label) {
    font-size: 14px;
    color: #1D1D1F;
    font-weight: 400;
}

.remember-checkbox :deep(.el-checkbox__input.is-checked .el-checkbox__inner) {
    background-color: #007AFF;
    border-color: #007AFF;
}

.forgot-password {
    font-size: 14px;
    color: #007AFF;
    font-weight: 500;
}

/* 登录按钮 */
.login-button {
    width: 100%;
    height: 50px;
    background: #007AFF;
    border: none;
    border-radius: 12px;
    font-size: 16px;
    font-weight: 600;
    color: #ffffff;
    transition: all 0.2s ease;
    margin-bottom: 20px;
}

.login-button:hover {
    background: #0056CC;
    transform: translateY(-1px);
}

.login-button:active {
    transform: translateY(0);
}

.login-button.is-loading {
    background: #007AFF;
}

/* 分割线 */
.divider {
    position: relative;
    text-align: center;
    margin: 20px 0;
    font-size: 14px;
    color: #8E8E93;
}

.divider::before {
    content: '';
    position: absolute;
    top: 50%;
    left: 0;
    right: 0;
    height: 1px;
    background: #E5E5EA;
}

.divider span {
    background: #ffffff;
    padding: 0 16px;
    font-weight: 500;
}

/* 演示按钮 */
.demo-button {
    width: 100%;
    height: 46px;
    background: #ffffff;
    border: 1px solid #E5E5EA;
    border-radius: 12px;
    font-size: 15px;
    font-weight: 500;
    color: #007AFF;
    transition: all 0.2s ease;
}

.demo-button:hover {
    background: #F2F2F7;
    border-color: #D1D1D6;
}
</style>