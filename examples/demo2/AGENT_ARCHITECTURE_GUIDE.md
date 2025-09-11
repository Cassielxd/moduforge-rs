# 🏗️ ModuForge Demo2 前端架构指南

## 📋 项目概览

ModuForge Demo2 是一个基于 **Tauri + Vue 3 + 微前端架构** 的造价管理系统，采用现代化的前端开发技术栈，支持多窗口协作和模块化开发。

## 🎯 核心架构原则

### 1. 微前端架构 (Micro-Frontend)
- **主应用**: 控制台和文件管理中心 (端口: 5173)
  - 文件创建、打开、历史记录
  - 概算、预算、结算、审核文件管理
  - 系统导航和用户管理
- **子模块**: 专业业务编制应用，可独立开发、测试和部署
  - 项目单项编制和计算
  - 单位工程增删改查
  - 复制粘贴、批量操作
  - 专业表格编辑和数据处理
- **共享组件库**: 提供统一的 UI 组件和状态管理

### 2. 多窗口桌面应用体验
- **Tauri 框架**: 轻量级桌面应用框架，替代 Electron
- **窗口管理**: 支持父子窗口关系、最小化跟随、模态对话框
- **启动屏幕**: 3 秒启动屏幕，优化用户体验

### 3. 组件化和状态管理
- **Vue 3 Composition API**: 现代化响应式编程模式
- **Ant Design Vue 4.x**: 企业级 UI 组件库
- **Pinia**: 轻量级状态管理
- **共享状态**: 跨模块数据同步和持久化

## 📁 项目结构详解

```
demo2/
├── src/                          # 主应用源码（控制台）
│   ├── App.vue                   # 应用根组件
│   ├── main.js                   # 应用入口
│   ├── views/
│   │   ├── Dashboard.vue         # 主控制台界面
│   │   ├── FileManager.vue       # 文件管理器 🔄
│   │   └── HistoryList.vue       # 历史文件列表 🔄
│   ├── router/
│   │   └── index.js              # 路由配置
│   └── assets/                   # 静态资源
├── packages/                     # 微前端业务模块
│   ├── rough-estimate/           # 概算编制模块 ✅
│   │   ├── src/views/
│   │   │   ├── EstimateMain.vue  # 概算主界面
│   │   │   ├── ProjectEdit.vue   # 项目编制 🔄
│   │   │   ├── ItemManager.vue   # 单项管理 🔄
│   │   │   └── UnitManager.vue   # 单位工程管理 🔄
│   ├── budget/                   # 预算编制模块 🔄
│   │   ├── src/views/
│   │   │   ├── BudgetMain.vue    # 预算主界面
│   │   │   └── BudgetEdit.vue    # 预算编制
│   ├── settlement/               # 结算编制模块 🔄
│   └── shared-components/        # 共享组件库
│       ├── src/
│       │   ├── components/       # 通用编制组件
│       │   │   ├── CostTable.vue # 成本表格（s-table）
│       │   │   ├── ItemForm.vue  # 单项编辑表单 🔄
│       │   │   └── UnitTree.vue  # 单位工程树 🔄
│       │   ├── layouts/          # 布局组件
│       │   ├── composables/      # 编制业务逻辑
│       │   ├── store/            # 状态管理
│       │   └── utils/            # 工具函数
│       └── dist/                 # 构建产物
├── src-tauri/                    # Tauri 后端
│   ├── src/
│   │   ├── main.rs               # 主程序入口
│   │   ├── commands/             # Tauri 命令
│   │   │   ├── file_manager.rs   # 文件管理命令 🔄
│   │   │   ├── djgc.rs           # 概算相关命令
│   │   │   ├── rcj.rs            # 预算相关命令
│   │   │   └── gcxm.rs           # 结算相关命令
│   │   ├── controller/           # 控制器
│   │   ├── core/                 # 核心模块
│   │   ├── nodes/                # 节点类型
│   │   └── plugins/              # 插件系统
│   └── tauri.conf.json           # Tauri 配置
├── dist/                         # 构建产物
├── build-config/                 # 构建配置
└── dev-setup.js                  # 开发环境启动脚本
```

## 🔧 技术栈详情

### 前端技术栈
- **框架**: Vue 3.5.13 + Composition API
- **构建工具**: Vite 6.2.4
- **UI 框架**: Ant Design Vue 4.2.6
- **高级表格**: @surely-vue/table 5.0.4（s-table）
- **路由**: Vue Router 4.5.1
- **状态管理**: Pinia 3.0.3
- **CSS 预处理器**: Less 4.4.0
- **自动导入**: unplugin-auto-import + unplugin-vue-components

### 桌面应用技术栈
- **桌面框架**: Tauri 2.5.0
- **后端语言**: Rust
- **进程通信**: Tauri Command API
- **安全策略**: CSP + Capabilities

### 开发工具
- **热重载**: Vite HMR + Vue DevTools
- **代码格式化**: 统一配置的 ESLint + Prettier
- **包管理**: npm workspaces

## 🎨 设计系统和 UI 规范

### 主题配置
```javascript
const theme = ref({
  token: {
    colorPrimary: '#1890ff',  // 主色调
    borderRadius: 6,          // 圆角
  },
})
```

### 色彩规范
- **概算模块**: `#1890ff` (蓝色)
- **预算模块**: `#52c41a` (绿色)
- **预算审核**: `#faad14` (橙色)
- **结算模块**: `#722ed1` (紫色)
- **结算审核**: `#eb2f96` (粉色)

### 布局规范
- **头部高度**: 64px
- **内容边距**: 24px
- **卡片间距**: 24px
- **网格间距**: [24, 24]

## 🚀 微前端模块规范

### 模块端口分配
```javascript
const PORT_ALLOCATION = {
  'main-app': 5173,        // 主应用
  'rough-estimate': 5174,  // 概算模块
  'shared-components': 5175, // 共享组件库
  'budget': 5176,          // 预算模块
  'budget-review': 5177,   // 预算审核模块
  'settlement': 5178,      // 结算模块
  'settlement-review': 5179 // 结算审核模块
}
```

### 模块标准配置
每个微前端模块必须包含：

1. **package.json** - 模块配置
```json
{
  "name": "@cost-app/module-name",
  "scripts": {
    "dev": "vite --port 5174",
    "build": "vite build",
    "copy-dist": "node copy-dist.js"
  }
}
```

2. **vite.config.js** - 构建配置
```javascript
export default defineConfig({
  plugins: [vue()],
  server: { port: 5174 },
  base: './',  // 重要：生产环境相对路径
})
```

3. **copy-dist.js** - 构建产物复制脚本

### 模块注册流程
1. 在主应用 `Dashboard.vue` 中注册模块信息
2. 配置模块端口和路径
3. 添加构建脚本到根 package.json
4. 实现模块间状态同步

## 🔄 状态管理架构

### 全局状态结构
```javascript
const globalState = {
  user: {
    profile: {},
    permissions: [],
    isLoggedIn: false
  },
  estimate: {
    projects: [],
    selectedItems: [],
    currentProject: null
  },
  ui: {
    loading: {},
    notifications: [],
    activeWindows: {}
  },
  system: {
    modules: [],
    windowStates: {}
  }
}
```

### 组合式函数模式
```javascript
// 使用示例
import { useEstimate, useUser, useWindowManager } from '@cost-app/shared-components'

const { projects, addProject } = useEstimate()
const { user, isLoggedIn } = useUser()  
const { createWindow, closeWindow } = useWindowManager()
```

## 🪟 窗口管理系统

### 窗口类型
1. **主窗口** - 工作台界面，无装饰，自定义标题栏
2. **子窗口** - 业务模块窗口，跟随父窗口状态
3. **模态窗口** - 对话框窗口，阻止父窗口交互
4. **启动屏幕** - 应用启动时的加载界面

### 窗口生命周期
```rust
// Tauri 后端窗口管理
#[tauri::command]
async fn create_module_window(
    app: tauri::AppHandle,
    module_key: String,
    title: String,
    url: String
) -> Result<(), String>
```

### 窗口状态同步
- 最小化跟随：子窗口跟随主窗口最小化
- 关闭级联：主窗口关闭时自动关闭所有子窗口
- 状态持久化：记住窗口位置和大小

## 🔧 开发工作流

### 本地开发启动流程
```bash
# 1. 启动所有服务
npm run dev:all

# 或分别启动
npm run dev          # 主应用 (5173)
cd packages/rough-estimate && npm run dev  # 概算模块 (5174)
cd packages/shared-components && npm run dev  # 共享组件 (5175)
```

### 生产构建流程
```bash
# 1. 构建所有子模块
npm run build:packages

# 2. 构建主应用
npm run build

# 3. Tauri 打包
npm run tauri:build
```

### 构建产物处理
1. 子模块先构建到各自的 `dist` 目录
2. 复制脚本将子模块产物复制到主应用 `dist` 目录
3. Tauri 打包时包含所有构建产物

## 📦 共享组件库规范

### 组件导出结构
```javascript
// packages/shared-components/src/index.js
export { default as AppHeader } from './layouts/AppHeader.vue'
export { default as SimpleHeader } from './layouts/SimpleHeader.vue'
export { default as CostForm } from './components/CostForm.vue'
export { default as CostTable } from './components/CostTable.vue'

// S-Table 高级表格组件（强制使用）
export { STable, setupSTable } from './plugins/stable.js'

// 组合式函数
export { useGlobalStore } from './composables/useGlobalStore.js'
export { useMainWindowManagement } from './composables/useMainWindowManagement.js'
export { useFormWindowManager } from './composables/useFormWindowManager.js'
export { useTableOperations } from './composables/useTableOperations.js'
export { useCostCalculation } from './composables/useCostCalculation.js'
```

### 头部组件使用规范
```vue
<template>
  <!-- 主应用头部 -->
  <AppHeader
    title="造价管理系统"
    :show-window-controls="true"
    :is-maximized="isMaximized"
    @minimize="minimizeWindow"
    @maximize="toggleMaximize"
    @close="closeWindow"
  >
    <template #right>
      <div class="user-info">管理员</div>
    </template>
  </AppHeader>

  <!-- 子窗口头部 -->
  <SimpleHeader 
    title="概算模块"
    :show-window-controls="true"
  />
</template>
```

### S-Table 表格组件使用规范

#### 强制使用 S-Table
所有涉及复杂表格操作的场景必须使用 `s-table` 而不是 Ant Design Vue 的 `a-table`：

```vue
<template>
  <div class="table-container">
    <!-- 工具栏 -->
    <div class="table-toolbar">
      <a-space>
        <a-button type="primary" @click="addRow">
          <template #icon><PlusOutlined /></template>
          新增
        </a-button>
        <a-button danger @click="deleteSelected">
          <template #icon><DeleteOutlined /></template>
          删除
        </a-button>
      </a-space>
    </div>

    <!-- S-Table 核心组件 -->
    <s-table
      :columns="tableColumns"
      :data-source="filteredData"
      :delay="200"
      :animateRows="false"
      :pagination="false"
      :loading="loading"
      :scroll="{ x: 1200 }"
      size="middle"
      bordered
      row-key="id"
      @change="handleTableChange"
    >
    </s-table>

    <!-- 汇总信息 -->
    <div class="table-footer">
      <div class="summary-info">
        <a-space>
          <span>总计: ¥{{ formatAmount(summary.total) }}</span>
          <span>已选: {{ selectedRowKeys.length }} 项</span>
        </a-space>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { STable, useTableOperations, useCostCalculation } from '@cost-app/shared-components'

const { calculateTotal, formatAmount } = useCostCalculation()
const { handleTableChange, selectedRowKeys } = useTableOperations()
</script>
```

#### S-Table 配置要求
1. **许可证破解**: 使用内置的许可证破解功能（已集成在 `stable.js` 中）
2. **性能优化**: 设置 `delay: 200`、`animateRows: false` 提升性能
3. **滚动配置**: 复杂表格必须设置 `scroll: { x: 1200 }` 支持水平滚动
4. **行选择**: 使用 `row-key="id"` 确保行唯一标识

#### S-Table 适用场景
- **成本表格**: 概算、预算、结算等金额计算表格
- **数据列表**: 超过 5 列的复杂数据展示
- **可编辑表格**: 支持行内编辑的表格
- **大数据量**: 超过 100 行数据的表格

## 🎯 业务架构设计指南

### 主应用职责范围（控制台）
主应用严格限制在文件管理和系统导航功能：

```vue
<!-- Dashboard.vue - 控制台主界面 -->
<template>
  <div class="console-dashboard">
    <!-- 文件管理区域 -->
    <a-card title="文件管理" class="file-manager-card">
      <div class="file-actions">
        <a-button type="primary" @click="createNewFile">
          <template #icon><FileAddOutlined /></template>
          新建文件
        </a-button>
        <a-button @click="openFile">
          <template #icon><FolderOpenOutlined /></template>
          打开文件
        </a-button>
      </div>
      
      <!-- 最近文件列表 -->
      <div class="recent-files">
        <a-list :data-source="recentFiles" @click="openFileInModule">
          <template #renderItem="{ item }">
            <a-list-item class="file-item">
              <a-list-item-meta
                :title="item.name"
                :description="`${item.type} | ${item.updateTime}`"
              >
                <template #avatar>
                  <a-avatar :style="{ backgroundColor: getFileTypeColor(item.type) }">
                    {{ getFileTypeIcon(item.type) }}
                  </a-avatar>
                </template>
              </a-list-item-meta>
            </a-list-item>
          </template>
        </a-list>
      </div>
    </a-card>

    <!-- 模块入口区域 -->
    <div class="modules-grid">
      <a-row :gutter="[24, 24]">
        <a-col :span="6" v-for="module in businessModules" :key="module.key">
          <a-card @click="openModuleWithFile(module)" hoverable>
            <div class="module-content">
              <component :is="module.icon" />
              <h3>{{ module.title }}</h3>
              <p>{{ module.description }}</p>
            </div>
          </a-card>
        </a-col>
      </a-row>
    </div>
  </div>
</template>

<script setup>
// 主应用只负责文件管理，不包含业务编制逻辑
const recentFiles = ref([
  { id: 1, name: '项目A概算.dgc', type: '概算', updateTime: '2024-01-15' },
  { id: 2, name: '项目B预算.rcj', type: '预算', updateTime: '2024-01-14' },
  { id: 3, name: '项目C结算.gcxm', type: '结算', updateTime: '2024-01-13' }
])

const openFileInModule = (file) => {
  // 根据文件类型打开对应的业务模块
  const moduleMap = {
    '概算': 'rough-estimate',
    '预算': 'budget', 
    '结算': 'settlement',
    '审核': 'review'
  }
  
  const moduleKey = moduleMap[file.type]
  openModuleWindow(moduleKey, file)
}
</script>
```

### 子模块职责范围（业务编制）
子模块专注于具体的业务编制工作：

```vue
<!-- packages/rough-estimate/src/views/EstimateMain.vue -->
<template>
  <div class="estimate-workspace">
    <!-- 项目基本信息 -->
    <ProjectInfoPanel :project="currentProject" />
    
    <!-- 单位工程树状结构 -->
    <div class="workspace-content">
      <div class="unit-tree-panel">
        <UnitTree 
          :units="projectUnits"
          :selected="selectedUnit"
          @select="handleUnitSelect"
          @add="handleAddUnit"
          @delete="handleDeleteUnit"
          @copy="handleCopyUnit"
        />
      </div>
      
      <!-- 单项编制表格 -->
      <div class="items-panel">
        <CostTable
          ref="costTableRef"
          :data="currentUnitItems"
          :columns="estimateColumns"
          table-type="estimate"
          @add-row="handleAddItem"
          @edit-row="handleEditItem"
          @delete-row="handleDeleteItem"
          @copy-rows="handleCopyItems"
          @paste-rows="handlePasteItems"
        />
      </div>
    </div>
    
    <!-- 汇总统计 -->
    <SummaryPanel :summary="projectSummary" />
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { CostTable, UnitTree, ProjectInfoPanel, SummaryPanel } from '@cost-app/shared-components'
import { useEstimateEditor, useClipboard, useCalculation } from '@cost-app/shared-components'

// 业务编制逻辑
const { 
  currentProject,
  projectUnits,
  selectedUnit,
  currentUnitItems,
  handleAddItem,
  handleEditItem,
  handleDeleteItem
} = useEstimateEditor()

const { 
  copyItems,
  pasteItems,
  canPaste
} = useClipboard()

const { 
  projectSummary,
  calculateUnit,
  calculateProject 
} = useCalculation()

// 核心业务操作
const handleAddUnit = (parentId) => {
  // 新增单位工程逻辑
}

const handleCopyItems = (items) => {
  copyItems(items)
  message.success(`已复制 ${items.length} 个单项`)
}

const handlePasteItems = () => {
  const pastedItems = pasteItems()
  message.success(`已粘贴 ${pastedItems.length} 个单项`)
}
</script>
```

### 创建新业务模块步骤
1. **复制现有模块结构**
```bash
cp -r packages/rough-estimate packages/new-module
```

2. **修改配置文件**
- 更新 `package.json` 中的名称和端口
- 修改 `vite.config.js` 中的端口
- 更新 `copy-dist.js` 中的路径

3. **在主控制台注册**
```javascript
// src/views/Dashboard.vue
const businessModules = ref([
  {
    key: 'new-module',
    title: '新业务模块',
    description: '专业编制功能',
    icon: NewModuleIcon,
    color: '#1890ff',
    fileTypes: ['new'], // 支持的文件类型
    port: 5180
  }
])
```

### 模块间通信模式
```javascript
// 使用共享状态进行通信
import { useGlobalStore } from '@cost-app/shared-components'

// 发送数据
const { updateEstimateData } = useGlobalStore()
updateEstimateData(newData)

// 接收数据
const { estimateData } = useGlobalStore()
watch(estimateData, (newData) => {
  // 响应数据变化
})
```

## 🔐 安全策略

### Tauri 安全配置
```json
{
  "security": {
    "csp": null,
    "capabilities": ["default", "remote-capability"]
  }
}
```

### 数据安全规范
- 敏感数据加密存储
- 窗口间通信验证
- 文件访问权限控制

## 📊 性能优化策略

### 前端性能优化
- **代码分割**: 按模块延迟加载
- **资源压缩**: Vite 自动压缩和优化
- **缓存策略**: 浏览器缓存 + 服务端缓存
- **组件懒加载**: 大组件按需加载

### 桌面应用优化
- **启动优化**: 启动屏幕 + 渐进式加载
- **内存管理**: 及时清理无用资源
- **窗口优化**: 窗口虚拟化和缓存

## 🐛 调试和测试

### 调试工具
- **Vue DevTools**: 组件状态调试
- **Tauri DevTools**: 桌面应用调试
- **网络面板**: API 请求监控
- **控制台**: 日志和错误追踪

### 测试策略
- **单元测试**: 组件和函数测试
- **集成测试**: 模块间交互测试
- **端到端测试**: 用户流程测试
- **性能测试**: 加载和响应时间测试

## 🚨 注意事项和最佳实践

### 开发约束
1. **端口冲突**: 确保每个模块使用不同端口
2. **路径规范**: 生产环境使用相对路径
3. **状态管理**: 避免直接修改 state，使用 actions
4. **组件命名**: 使用 PascalCase 命名组件

### 部署要求
1. **构建顺序**: 必须先构建子模块再构建主应用
2. **文件权限**: 确保复制脚本有执行权限
3. **依赖同步**: 所有模块依赖版本保持一致

### 错误处理
```javascript
// 统一错误处理模式
try {
  await moduleOperation()
  message.success('操作成功')
} catch (error) {
  console.error('操作失败:', error)
  message.error(`操作失败: ${error.message}`)
}
```

## 📈 扩展路线图

### 短期目标 (1-2 周)
- [x] 完成概算模块开发
- [ ] 开发预算模块
- [ ] 完善共享组件库
- [ ] 实现模块间数据同步

### 中期目标 (1-2 月)
- [ ] 开发其余业务模块
- [ ] 实现用户权限系统
- [ ] 添加数据持久化
- [ ] 性能优化和测试

### 长期目标 (3-6 月)
- [ ] 插件系统扩展
- [ ] 多语言支持
- [ ] 云端数据同步
- [ ] 移动端适配

这份架构指南为 agent 开发提供了全面的约束和指导，确保所有后续开发都严格遵循既定的架构模式和开发规范。