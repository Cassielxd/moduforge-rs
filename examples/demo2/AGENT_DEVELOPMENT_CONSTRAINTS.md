# 🤖 Agent 开发约束指南

## 📋 概述

本文档为 ModuForge Demo2 项目的 Agent 开发提供严格的约束和指导原则，确保所有 Agent（steering-architect、strategic-planner、task-executor）都按照既定的架构模式和开发规范进行工作。

## 🏗️ 架构约束

### 1. 微前端架构强制约束

#### 主应用约束（控制台）
- **端口固定**: 主应用必须使用端口 5173
- **技术栈锁定**: Vue 3.5.13 + Ant Design Vue 4.2.6 + Tauri 2.5.0
- **布局结构**: 必须使用 `<a-layout>` 作为主布局容器
- **头部组件**: 强制使用 `AppHeader` 组件，禁止自定义标题栏
- **职责限制**: 只能包含文件管理、历史记录、模块导航功能
- **业务约束**: **严禁**在主应用中编写具体业务编制逻辑

#### 微前端模块约束（业务编制）
- **端口分配表**（严格遵循，不得修改）:
  ```
  主应用（控制台）: 5173
  概算编制模块: 5174  
  共享组件: 5175
  预算编制模块: 5176
  预算审核模块: 5177
  结算编制模块: 5178
  结算审核模块: 5179
  ```
- **模块命名**: 必须使用 kebab-case 格式
- **目录结构**: 所有模块必须遵循标准目录结构
- **独立性**: 每个模块必须能独立开发、构建和部署
- **业务职责**: 每个子模块专注于具体的业务编制功能
- **核心功能**: 必须包含项目/单项/单位工程的增删改查复制粘贴等操作

### 2. 技术栈固化约束

#### 前端技术栈（不得变更）
```json
{
  "vue": "^3.5.13",
  "ant-design-vue": "^4.2.6",
  "@surely-vue/table": "^5.0.4",
  "vue-router": "^4.5.1",
  "pinia": "^3.0.3",
  "vite": "^6.2.4",
  "@tauri-apps/api": "2.5.0"
}
```

#### 构建工具约束
- **主构建工具**: Vite 6.2.4（禁止使用 webpack）
- **CSS 预处理器**: Less 4.4.0（禁止使用 Sass）
- **自动导入**: 必须使用 unplugin-auto-import + unplugin-vue-components

### 3. 组件库和设计系统约束

#### UI 组件库约束
- **强制使用**: Ant Design Vue 4.2.6
- **禁止引入**: Element Plus、Vuetify 或其他 UI 库
- **图标库**: 必须使用 @ant-design/icons-vue
- **主题配置**: 使用统一的主题色 `#1890ff`
- **表格组件**: 复杂表格必须使用 @surely-vue/table（s-table）

#### 表格组件使用约束（重要）
**强制规则**: 所有复杂表格操作必须使用 `s-table`，禁止使用 `a-table`

```javascript
// ✅ 正确 - 必须使用 s-table
<s-table
  :columns="columns"
  :data-source="dataSource"
  :delay="200"
  :animateRows="false"
  :scroll="{ x: 1200 }"
  row-key="id"
/>

// ❌ 错误 - 禁止在复杂场景使用 a-table
<a-table :columns="columns" :data-source="dataSource" />
```

**s-table 适用场景（强制）**:
- 成本计算表格（概算、预算、结算）
- 超过 5 列的数据表格
- 需要行内编辑的表格
- 超过 100 行数据的表格
- 需要复杂操作（批量操作、导出等）的表格

**a-table 仅限使用场景**:
- 简单的展示型表格（≤ 5 列，≤ 50 行）
- 纯静态数据展示
- 不涉及金额计算的表格

#### 共享组件强制使用
```javascript
// 强制使用的共享组件
import { 
  AppHeader,      // 主应用头部（强制）
  SimpleHeader,   // 子模块头部（强制）
  CostForm,       // 成本表单组件
  CostTable,      // 成本表格组件（内置s-table）
  STable,         // S-Table高级表格（强制）
  FormWindow,     // 表单窗口组件
  // 表格相关组合式函数
  useTableOperations,  // 表格操作（强制）
  useCostCalculation   // 成本计算（强制）
} from '@cost-app/shared-components'
```

## 🎯 开发流程约束

### 1. Agent 工作流约束

#### Steering Architect 约束
- **职责范围**: 只能定义项目蓝图，不得进行具体代码实现
- **输出文件**: 只能生成 product.md、tech.md、structure.md
- **技术选型**: 必须基于现有技术栈，不得引入新技术
- **架构模式**: 必须遵循既定的微前端架构

#### Strategic Planner 约束  
- **输入依赖**: 必须基于 Steering Architect 的输出文件
- **任务粒度**: 任务必须细化到具体文件和函数级别
- **优先级**: 必须按照模块依赖关系排序任务
- **输出格式**: requirements.md、design.md、tasks.md 格式固定

#### Task Executor 约束
- **严格执行**: 只能按照 tasks.md 执行，不得自主决策
- **代码规范**: 必须遵循项目代码规范和格式
- **测试要求**: 每个功能必须包含对应测试
- **完成确认**: 任务完成必须通过测试验证

### 2. 文件操作约束

#### 禁止操作的文件
```
- package.json（根目录，版本锁定）
- vite.config.js（主应用，配置锁定）
- tauri.conf.json（Tauri 配置，安全相关）
- components.d.ts（自动生成文件）
- auto-imports.d.ts（自动生成文件）
```

#### 必须操作的文件
```
- 业务组件文件（.vue）
- 路由配置（router/index.js）
- 状态管理（store相关）
- 样式文件（.css/.less）
- 工具函数（utils目录）
```

### 3. 代码规范约束

#### Vue 组件约束
```vue
<!-- 强制模板结构 -->
<template>
  <div class="component-name">
    <!-- 内容 -->
  </div>
</template>

<script setup>
// 强制使用 Composition API
import { ref, computed, onMounted } from 'vue'
// 强制使用共享组合式函数
import { useGlobalStore } from '@cost-app/shared-components'
</script>

<style scoped>
/* 强制使用 scoped 样式 */
</style>
```

#### 命名约束
- **组件名**: PascalCase（如 `EstimateForm`）
- **文件名**: PascalCase.vue（如 `EstimateForm.vue`）
- **路由名**: kebab-case（如 `/estimate-detail`）
- **CSS 类名**: kebab-case（如 `.estimate-form`）

## 🔧 功能开发约束

### 1. 窗口管理约束

#### 窗口创建约束
```javascript
// 强制使用的窗口创建方法
import { invoke } from '@tauri-apps/api/core'

const openModule = async (module) => {
  const url = isDev ? `http://localhost:${module.port}` : `${module.key}/index.html`
  await invoke('create_module_window', {
    moduleKey: module.key,
    title: module.title, 
    url: url
  })
}
```

#### 窗口控制约束
- **主窗口**: 必须无装饰，使用自定义标题栏
- **子窗口**: 必须跟随主窗口状态
- **模态窗口**: 必须阻止父窗口交互
- **窗口大小**: 遵循既定尺寸规范

### 2. 状态管理约束

#### 全局状态结构（不得修改）
```javascript
const state = {
  user: { profile: {}, permissions: [], isLoggedIn: false },
  estimate: { projects: [], selectedItems: [], currentProject: null },
  ui: { loading: {}, notifications: [], activeWindows: {} },
  system: { modules: [], windowStates: {} }
}
```

#### 状态操作约束
- **禁止直接修改**: 不得直接修改 state
- **强制使用 actions**: 所有状态修改必须通过 actions
- **数据同步**: 状态变更必须同步到所有窗口
- **持久化**: 关键数据必须自动持久化

### 3. 路由约束

#### 路由配置约束
```javascript
// 强制的路由结构
const routes = [
  {
    path: '/',
    name: 'Dashboard', 
    component: () => import('../views/Dashboard.vue')
  },
  // 其他路由必须遵循相同模式
]
```

#### 路由跳转约束
- **模块间跳转**: 必须通过窗口管理实现
- **模块内跳转**: 使用 Vue Router
- **路径格式**: 使用 kebab-case

## 📦 构建和部署约束

### 1. 构建流程约束

#### 构建顺序（严格遵循）
```bash
# 1. 构建共享组件库
cd packages/shared-components && npm run build

# 2. 构建各子模块
cd packages/rough-estimate && npm run build && npm run copy-dist

# 3. 构建主应用
npm run build

# 4. Tauri 打包
npm run tauri:build
```

#### 构建配置约束
- **base 路径**: 子模块必须设置 `base: './'`
- **输出目录**: 子模块构建到各自 dist 目录
- **复制脚本**: 必须使用 copy-dist.js 复制构建产物

### 2. 依赖管理约束

#### 版本锁定约束
- **主要依赖**: 版本号严格锁定，不得升级
- **开发依赖**: 可在补丁版本范围内更新
- **新增依赖**: 必须先评估影响再添加

#### 工作空间约束
```json
{
  "workspaces": ["packages/*"],
  "private": true,
  "type": "module"
}
```

## 🎨 UI/UX 约束

### 1. 设计系统约束

#### 色彩规范（不得修改）
```javascript
const COLORS = {
  primary: '#1890ff',
  'rough-estimate': '#1890ff',
  'budget': '#52c41a', 
  'budget-review': '#faad14',
  'settlement': '#722ed1',
  'settlement-review': '#eb2f96'
}
```

#### 间距规范（不得修改）
```javascript
const SPACING = {
  headerHeight: '64px',
  contentPadding: '24px',
  cardGutter: '24px',
  gridGutter: '[24, 24]'
}
```

### 2. 交互规范约束

#### 用户操作反馈
- **Loading 状态**: 所有异步操作必须显示 loading
- **成功提示**: 使用 `message.success()`
- **错误处理**: 使用 `message.error()` 并记录日志
- **确认对话框**: 删除操作必须二次确认

#### 响应式约束
- **断点定义**: 使用 Ant Design Vue 标准断点
- **网格系统**: 使用 `<a-row>` 和 `<a-col>`
- **移动端**: 暂不支持，专注桌面端体验

## 🔐 安全和性能约束

### 1. 安全约束

#### 数据安全
- **敏感数据**: 必须加密存储
- **API 调用**: 必须验证响应数据
- **文件访问**: 遵循 Tauri 安全策略
- **跨窗口通信**: 验证消息来源

#### Tauri 安全配置（不得修改）
```json
{
  "security": {
    "csp": null,
    "capabilities": ["default", "remote-capability"]
  }
}
```

### 2. 性能约束

#### 加载性能
- **初始加载**: 主应用 < 2 秒
- **模块加载**: 子模块 < 1 秒  
- **资源大小**: 单个模块 < 5MB
- **内存使用**: 主应用 < 200MB

#### 运行时性能
- **响应时间**: UI 操作 < 100ms
- **数据更新**: 状态同步 < 50ms
- **窗口操作**: 窗口切换 < 200ms

## ⚠️ 禁止事项

### 1. 技术选型禁止
- 不得引入 React、Angular 或其他框架
- 不得使用 jQuery 或其他传统库
- 不得使用 Webpack 替代 Vite
- 不得使用 Element Plus 替代 Ant Design Vue
- **严禁在复杂表格场景使用 a-table 替代 s-table**

### 2. 架构职责禁止
- **主应用禁止事项**：
  - 不得在主应用中实现具体业务编制功能
  - 不得在主应用中编写项目单项管理逻辑
  - 不得在主应用中实现单位工程增删改查
  - 不得在主应用中处理复制粘贴等编制操作
- **子模块禁止事项**：
  - 不得在子模块中实现文件管理功能
  - 不得在子模块中维护历史文件列表
  - 不得越权访问其他模块的业务数据

### 3. 架构变更禁止
- 不得修改微前端架构模式
- 不得改变端口分配方案
- 不得合并独立模块
- 不得破坏模块独立性

### 4. 配置修改禁止
- 不得修改 Tauri 安全配置
- 不得更改主题色彩定义
- 不得调整标准间距规范
- 不得破坏构建流程顺序

### 5. 开发实践禁止
- 不得直接修改 node_modules
- 不得绕过状态管理直接操作 DOM
- 不得在组件间直接传递大量数据
- 不得忽略错误处理和日志记录
- **不得在复杂表格中绕过 s-table 直接使用原生 HTML table**
- **不得移除或修改 s-table 的许可证破解代码**

## 📋 检查清单

### Agent 开发前检查
- [ ] 确认当前架构文档版本
- [ ] 验证依赖版本兼容性
- [ ] 检查端口分配冲突
- [ ] 确认模块依赖关系
- [ ] **验证表格场景是否需要使用 s-table**
- [ ] **确认功能归属（主应用 vs 子模块）**
- [ ] **验证是否违反职责边界约束**

### 代码提交前检查  
- [ ] 代码格式符合规范
- [ ] 所有测试通过
- [ ] 构建流程正常
- [ ] 性能指标达标
- [ ] **复杂表格确认使用 s-table 而非 a-table**
- [ ] **s-table 配置参数正确（delay: 200, animateRows: false 等）**

### 部署前检查
- [ ] 所有模块构建成功
- [ ] 静态资源复制完整
- [ ] Tauri 配置正确
- [ ] 安全策略生效
- [ ] **s-table 许可证破解功能正常**

## 🚨 违规处理

### 违规等级
- **轻微违规**: 命名不规范、注释缺失等
- **中等违规**: 技术选型错误、性能不达标等  
- **严重违规**: 破坏架构、安全漏洞等

### 处理流程
1. **发现违规**: 立即停止当前开发
2. **分析影响**: 评估违规影响范围
3. **制定方案**: 确定修复和回退策略
4. **执行修复**: 严格按照约束进行修复
5. **验证结果**: 确保修复后符合所有约束

本约束文档是 ModuForge Demo2 项目 Agent 开发的最高准则，所有 Agent 必须严格遵循，确保项目的一致性、可维护性和可扩展性。