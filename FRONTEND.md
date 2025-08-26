# ModuForge-RS 前端开发完整指南

该文档为 ModuForge-RS 演示应用的前端开发提供完整技术指导，供 Claude Code 在进行前端编码时参考。

## 项目概览

这是一个基于 Vue 3 + Tauri 的造价管理系统演示应用，主应用集成控制台功能，支持多窗口管理和部分微前端模块化开发。

**重要说明**：工作台 = 控制台，控制台功能已在主应用的 `Dashboard.vue` 中完整实现。

## 技术栈

### 核心技术
- **Vue 3.5.13**: 使用 Composition API 和 `<script setup>` 语法
- **Vue Router 4.5.1**: 客户端路由管理
- **Pinia 3.0.3**: 状态管理
- **Ant Design Vue 4.2.6**: UI 组件库
- **Tauri 2.5.0**: 桌面应用框架
- **Vite 6.2.4**: 构建工具和开发服务器

### 开发工具
- **unplugin-auto-import**: 自动导入 Vue API 和第三方库
- **unplugin-vue-components**: 自动导入 Ant Design Vue 组件
- **Less 4.4.0**: CSS 预处理器，支持嵌套和变量
- **vue-devtools**: Vue 3 调试工具

## 项目目录结构

```
examples/demo/                    # 主演示应用
├── src/                         # 前端源码目录
│   ├── App.vue                  # 根组件
│   ├── main.js                  # 应用入口点
│   ├── router/                  # 路由配置
│   │   └── index.js            # 路由定义和配置
│   ├── views/                   # 页面级组件
│   │   ├── Dashboard.vue       # 工作台主页（控制台）
│   │   ├── DataPage.vue        # 数据查看器
│   │   ├── EstimateDemo.vue    # 概算演示页面
│   │   ├── FormPage.vue        # 表单页面
│   │   ├── SettingsPage.vue    # 系统设置
│   │   └── TableTest.vue       # 表格测试页面
│   ├── components/              # 通用组件
│   │   ├── WindowManagerDemo.vue # 窗口管理演示
│   │   └── pricing/            # 计价相关组件目录
│   ├── composables/             # 组合式函数
│   │   └── useWindowModal.js   # 窗口模态框管理
│   ├── utils/                   # 工具函数
│   │   ├── index.js            # 通用工具函数
│   │   └── README.md           # 工具函数说明
│   └── assets/                  # 静态资源
│       ├── base.css            # 基础样式
│       ├── main.css            # 主样式文件
│       └── logo.svg            # Logo 图标
├── packages/                    # 微前端子应用（部分模块）
│   ├── rough-estimate/         # 概算模块
│   ├── shared-components/      # 共享组件库
│   └── budget/                 # 预算模块
├── src-tauri/                  # Tauri 后端源码
│   ├── src/                    # Rust 后端代码
│   │   ├── commands/           # Tauri 命令定义
│   │   ├── controller/         # 业务逻辑控制器
│   │   ├── core/               # 核心功能模块
│   │   ├── plugins/            # 插件系统
│   │   └── lib.rs              # 库入口
│   └── tauri.conf.json         # Tauri 配置文件
└── public/                     # 公共静态文件
    └── favicon.ico             # 网站图标
```

## 核心组件分析

### 1. 根组件 (App.vue)
```vue
<template>
  <div id="app">
    <!-- 全局配置提供器 -->
    <a-config-provider :theme="theme">
      <!-- 路由视图 -->
      <router-view />
    </a-config-provider>
  </div>
</template>
```

**职责**：
- 提供全局主题配置
- 作为应用根容器
- 路由视图挂载点

### 2. 控制台组件 (Dashboard.vue)
```vue
<script setup>
import { ref } from 'vue'
import AppHeader from 'shared-components/AppHeader.vue'
import { UserOutlined } from '@ant-design/icons-vue'

// 窗口状态管理
const isMaximized = ref(false)

// 模块配置
const modules = [
  { key: 'budget', title: '预算管理', icon: 'Calculator' },
  { key: 'estimate', title: '概算', icon: 'LineChart' },
  // ...
]
</script>
```

**特点**：
- 作为应用主控制台（工作台）
- 使用共享头部组件 `AppHeader`
- 模块化的网格布局
- 支持窗口控制操作
- 集成所有主要功能模块入口

### 3. 其他页面组件职责分工
- **DataPage.vue**: 数据展示和查看器
- **FormPage.vue**: 表单操作页面
- **EstimateDemo.vue**: 概算功能演示
- **SettingsPage.vue**: 系统配置管理
- **TableTest.vue**: 表格功能测试

## 路由系统

### 路由配置
```javascript
const routes = [
  {
    path: '/',
    name: 'Dashboard',
    component: () => import('../views/Dashboard.vue'),
    meta: { title: '工作台' }  // 工作台 = 控制台
  },
  {
    path: '/form-page',
    name: 'FormPage', 
    component: () => import('../views/FormPage.vue'),
    meta: { title: '表单页面' }
  },
  // 其他路由...
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// 路由守卫 - 设置页面标题
router.beforeEach((to, from, next) => {
  if (to.meta?.title) {
    document.title = `${to.meta.title} - 造价管理系统`
  }
  next()
})
```

**特性**：
- 使用 `createWebHistory` 模式
- 懒加载所有页面组件
- 自动设置页面标题
- Meta 信息管理

## 核心配置

### Vite 配置特性
- **自动导入**: Ant Design Vue 组件和 API 自动导入
- **别名配置**: `@` 指向 `src` 目录
- **Less 支持**: 预处理器配置和变量覆盖
- **环境变量**: 
  - `VITE_APP_TITLE`: "造价管理系统"
  - `VITE_API_BASE_URL`: "http://localhost:20008/api"
- **构建插件**: 自定义插件复制 splashscreen.html 和概算模块构建产物

### 开发服务器
- **端口**: 5173
- **代理**: API 请求代理到后端服务

## 开发规范和最佳实践

### 1. Vue 3 编码规范

#### 组件定义
```vue
<script setup>
// 1. 导入依赖
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'

// 2. Props 定义
const props = defineProps({
  title: {
    type: String,
    required: true
  },
  visible: {
    type: Boolean,
    default: false
  }
})

// 3. Emits 定义  
const emit = defineEmits(['update:visible', 'confirm'])

// 4. 响应式数据
const loading = ref(false)
const formData = reactive({
  name: '',
  email: ''
})

// 5. 计算属性
const isValid = computed(() => {
  return formData.name && formData.email
})

// 6. 方法定义
const handleSubmit = async () => {
  loading.value = true
  try {
    // 业务逻辑
    emit('confirm', formData)
  } finally {
    loading.value = false
  }
}

// 7. 生命周期钩子
onMounted(() => {
  // 初始化逻辑
})
</script>
```

#### 模板规范
```vue
<template>
  <div class="component-wrapper">
    <!-- 条件渲染 -->
    <div v-if="loading" class="loading">加载中...</div>
    
    <!-- 列表渲染 -->
    <a-list
      :data-source="items"
      :loading="loading"
    >
      <template #renderItem="{ item }">
        <a-list-item :key="item.id">
          {{ item.name }}
        </a-list-item>
      </template>
    </a-list>
    
    <!-- 事件处理 -->
    <a-button 
      type="primary" 
      :loading="loading"
      @click="handleSubmit"
    >
      提交
    </a-button>
  </div>
</template>
```

### 2. 样式规范

#### 使用 Scoped 样式
```vue
<style scoped lang="less">
.component-wrapper {
  padding: 16px;
  background: #fff;
  border-radius: 6px;
  
  .loading {
    text-align: center;
    color: @primary-color; // Less 变量
  }
  
  // 嵌套选择器
  .ant-btn {
    margin-top: 16px;
  }
}
</style>
```

#### 全局样式覆盖
```less
// 在 assets/main.css 中
:deep(.ant-table-thead > tr > th) {
  background: #fafafa;
  font-weight: 600;
}
```

### 3. 组合式函数 (Composables)

#### 示例：窗口管理
```javascript
// composables/useWindowModal.js
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export function useWindowModal() {
  const isMaximized = ref(false)
  const isMinimized = ref(false)
  
  const minimizeWindow = async () => {
    await invoke('minimize_window')
    isMinimized.value = true
  }
  
  const maximizeWindow = async () => {
    await invoke('maximize_window')
    isMaximized.value = !isMaximized.value
  }
  
  const closeWindow = async () => {
    await invoke('close_window')
  }
  
  return {
    isMaximized,
    isMinimized,
    minimizeWindow,
    maximizeWindow,
    closeWindow
  }
}
```

#### 使用组合式函数
```vue
<script setup>
import { useWindowModal } from '@/composables/useWindowModal'

const { 
  isMaximized, 
  minimizeWindow, 
  maximizeWindow, 
  closeWindow 
} = useWindowModal()
</script>
```

## Tauri 集成开发

### 1. 后端命令调用

#### 定义 Rust 命令
```rust
// src-tauri/src/commands/mod.rs
#[tauri::command]
pub async fn calculate_cost(items: Vec<CostItem>) -> Result<f64, String> {
    let total = items.iter().map(|item| item.amount * item.price).sum();
    Ok(total)
}
```

#### 前端调用
```javascript
import { invoke } from '@tauri-apps/api/tauri'

const calculateTotal = async (items) => {
  try {
    const total = await invoke('calculate_cost', { items })
    return total
  } catch (error) {
    console.error('计算失败:', error)
    throw error
  }
}
```

### 2. 窗口管理

#### 创建子窗口
```javascript
import { WebviewWindow } from '@tauri-apps/api/window'

const openPricingWindow = async () => {
  const pricingWindow = new WebviewWindow('pricing', {
    url: '/pricing-workbench',
    title: '计价工作台',
    width: 1200,
    height: 800,
    resizable: true
  })
  
  // 窗口事件监听
  pricingWindow.once('tauri://created', () => {
    console.log('计价窗口创建成功')
  })
}
```

### 3. 文件系统操作

```javascript
import { open, save } from '@tauri-apps/api/dialog'
import { readTextFile, writeTextFile } from '@tauri-apps/api/fs'

// 打开文件
const openFile = async () => {
  const file = await open({
    filters: [{
      name: '项目文件',
      extensions: ['json', 'xml']
    }]
  })
  
  if (file) {
    const content = await readTextFile(file)
    return JSON.parse(content)
  }
}

// 保存文件
const saveProject = async (data) => {
  const path = await save({
    defaultPath: 'project.json'
  })
  
  if (path) {
    await writeTextFile(path, JSON.stringify(data, null, 2))
  }
}
```

## 微前端架构（部分模块）

### 1. Workspace 配置

```json
{
  "name": "cost-estimation-workspace",
  "workspaces": [
    "packages/*"
  ]
}
```

### 2. 现有微前端模块

#### rough-estimate（概算模块）
- 独立的概算计算功能
- 支持多窗口操作
- 与主应用集成

#### shared-components（共享组件库）
```javascript
// packages/shared-components/index.js
export { default as AppHeader } from './src/AppHeader.vue'
export { default as CostForm } from './src/CostForm.vue'
export { default as CostTable } from './src/CostTable.vue'
```

#### 使用共享组件
```vue
<script setup>
import { AppHeader, CostForm } from 'shared-components'
</script>
```

### 3. 构建流程
```bash
# 开发模式
npm run dev:all          # 启动所有子应用开发服务器

# 构建流程
npm run build:packages   # 构建概算模块和共享组件
npm run build           # 构建主应用并复制子应用构建产物
```

## 状态管理 (Pinia)

### 1. Store 定义

```javascript
// stores/project.js
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useProjectStore = defineStore('project', () => {
  // 状态
  const currentProject = ref(null)
  const projects = ref([])
  const loading = ref(false)
  
  // 计算属性
  const hasActiveProject = computed(() => !!currentProject.value)
  const projectCount = computed(() => projects.value.length)
  
  // 动作
  const loadProject = async (id) => {
    loading.value = true
    try {
      const project = await invoke('get_project', { id })
      currentProject.value = project
    } finally {
      loading.value = false
    }
  }
  
  const saveProject = async (project) => {
    await invoke('save_project', { project })
    // 更新本地状态
    if (currentProject.value?.id === project.id) {
      currentProject.value = project
    }
  }
  
  return {
    currentProject,
    projects,
    loading,
    hasActiveProject,
    projectCount,
    loadProject,
    saveProject
  }
})
```

### 2. 组件中使用 Store

```vue
<script setup>
import { useProjectStore } from '@/stores/project'
import { storeToRefs } from 'pinia'

const projectStore = useProjectStore()
const { currentProject, loading, hasActiveProject } = storeToRefs(projectStore)
const { loadProject, saveProject } = projectStore

// 使用状态和方法
onMounted(async () => {
  await loadProject(1)
})
</script>
```

## 开发工作流

### 1. 本地开发环境

```bash
# 安装依赖
npm install

# 启动前端开发服务器 (仅前端，端口 5173)
npm run dev

# 启动 Tauri 开发环境 (前端 + 后端)
npm run tauri:dev

# 启动所有微前端子应用
npm run dev:all
```

### 2. 构建流程

```bash
# 构建微前端子应用
npm run build:packages

# 构建主应用
npm run build

# 构建 Tauri 桌面应用
npm run tauri:build
```

### 3. 调试技巧

#### Vue DevTools
- 安装浏览器扩展
- 查看组件树和状态
- 监控路由变化
- 检查 Pinia Store 状态

#### Tauri 调试
```javascript
// 开发环境日志输出
if (import.meta.env.DEV) {
  console.log('开发模式调试信息')
}

// 错误处理
window.addEventListener('error', (e) => {
  console.error('全局错误:', e.error)
})
```

## 性能优化建议

### 1. 代码分割
```javascript
// 路由级别懒加载
const Dashboard = () => import('../views/Dashboard.vue')

// 组件级别懒加载
const HeavyComponent = defineAsyncComponent(() =>
  import('../components/HeavyComponent.vue')
)
```

### 2. 组件优化
```vue
<script setup>
import { shallowRef, markRaw } from 'vue'

// 使用 shallowRef 优化大对象
const tableData = shallowRef([])

// 使用 markRaw 标记不需要响应式的对象
const chart = markRaw(new Chart())
</script>
```

### 3. 构建优化
```javascript
// vite.config.js
export default {
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'ant-design': ['ant-design-vue'],
          'vue-vendor': ['vue', 'vue-router', 'pinia']
        }
      }
    }
  }
}
```

## 常见开发任务

### 添加新页面
1. 在 `src/views/` 创建 Vue 组件
2. 在 `src/router/index.js` 添加路由配置
3. 设置合适的 meta 信息 (title)
4. 在 `Dashboard.vue` 中添加模块入口（如需要）

### 添加新组件
1. 在 `src/components/` 创建组件文件
2. 使用 `<script setup>` 语法
3. 添加必要的 props 和 emits 定义

### 集成 Tauri 命令
1. 在 Rust 后端定义命令
2. 在前端使用 `invoke()` 调用命令
3. 处理异步结果和错误

### 样式自定义
1. 在组件中使用 Less 语法
2. 通过 ConfigProvider 自定义主题变量
3. 使用 scoped 样式避免冲突

## 故障排除

### 常见问题
1. **热重载问题**: 参考 `HOT_RELOAD_GUIDE.md`
2. **窗口管理**: 参考 `WINDOW_MANAGEMENT_GUIDE.md`
3. **子窗口修复**: 参考 `CHILD_WINDOW_FIX.md`
4. **打包问题**: 参考 `PACKAGING_GUIDE.md`

### 调试策略
1. 检查浏览器控制台错误
2. 使用 Vue DevTools 检查组件状态
3. 查看 Tauri 日志输出
4. 检查网络请求状态

## 测试策略

### 1. 单元测试
```javascript
// 使用 Vitest
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import CostForm from '@/components/CostForm.vue'

describe('CostForm', () => {
  it('应该正确计算总成本', async () => {
    const wrapper = mount(CostForm)
    await wrapper.setData({ 
      items: [{ price: 100, quantity: 2 }] 
    })
    expect(wrapper.vm.totalCost).toBe(200)
  })
})
```

### 2. E2E 测试
```javascript
// 使用 Playwright
import { test, expect } from '@playwright/test'

test('应该能够创建新项目', async ({ page }) => {
  await page.goto('/')
  await page.click('button:has-text("新建项目")')
  await page.fill('input[name="projectName"]', '测试项目')
  await page.click('button:has-text("保存")')
  await expect(page.locator('.project-list')).toContainText('测试项目')
})
```

## 最佳实践总结

1. **组件设计**: 保持组件单一职责，提高复用性
2. **状态管理**: 合理使用 Pinia 管理全局状态
3. **错误处理**: 统一错误处理和用户反馈
4. **性能监控**: 使用 Vue DevTools 监控组件性能
5. **代码规范**: 保持一致的代码风格和注释规范
6. **控制台集中**: 所有主要功能入口集中在 Dashboard.vue 控制台
7. **模块化开发**: 合理使用微前端架构进行功能模块分离
8. **测试**: 为关键组件编写单元测试

## 架构注意事项

1. **控制台定位**: `Dashboard.vue` 是应用的核心控制台，集成所有主要功能模块入口
2. **微前端选择性使用**: 仅对需要独立开发和部署的复杂模块使用微前端架构
3. **共享组件**: 通过 `shared-components` 包复用通用组件和逻辑
4. **窗口管理**: 充分利用 Tauri 的多窗口能力提供更好的用户体验

这份完整指南涵盖了 ModuForge-RS 前端开发的所有核心方面，确保开发过程的一致性、高效性和可维护性。