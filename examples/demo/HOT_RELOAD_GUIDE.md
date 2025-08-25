# 热更新和共享状态管理指南

## 概述

本指南介绍如何解决 shared-components 的热更新问题，以及如何在各子模块间共享状态管理。

## 🔥 热更新解决方案

### 问题分析
- shared-components 每次修改都需要重新打包
- 子模块无法实时获取 shared-components 的更新
- 开发效率低下

### 解决方案

#### 1. 开发模式配置
shared-components 现在支持开发模式的监听构建：

```bash
# 启动 shared-components 热更新
cd packages/shared-components
npm run dev

# 或者使用监听模式
npm run build:watch
```

#### 2. 自动化开发环境
使用提供的开发脚本一键启动所有服务：

```bash
# 启动所有服务（推荐）
npm run dev:all

# 或者分别启动
npm run dev:shared    # 启动 shared-components 热更新
npm run dev           # 启动主应用
```

#### 3. Vite 配置优化
shared-components 的 vite.config.js 已优化：
- 开发模式下不压缩代码，加快构建速度
- 启用 sourcemap 便于调试
- 支持文件监听自动重新构建

### 使用方法

1. **启动开发环境**：
   ```bash
   npm run dev:all
   ```

2. **服务端口**：
   - 主应用: http://localhost:5173
   - 概算模块: http://localhost:5174  
   - Shared Components: http://localhost:5175

3. **开发流程**：
   - 修改 shared-components 中的组件
   - 自动重新构建
   - 子模块自动获取最新版本

## 🔄 共享状态管理

### 状态管理架构

使用 Vue 3 的响应式系统实现轻量级状态管理，支持：
- 跨模块数据共享
- 自动持久化
- 跨窗口同步
- 类型安全

### 核心功能

#### 1. 全局状态
```javascript
import { useGlobalStore } from '@cost-app/shared-components'

const { state, actions, getters } = useGlobalStore()
```

#### 2. 分类状态管理
```javascript
// 用户状态
import { useUser } from '@cost-app/shared-components'
const { user, isLoggedIn, setUser } = useUser()

// 概算数据
import { useEstimate } from '@cost-app/shared-components'
const { projects, addProject, updateProject } = useEstimate()

// 表单窗口
import { useFormWindows } from '@cost-app/shared-components'
const { activeWindows, registerWindow } = useFormWindows()
```

#### 3. 数据持久化
```javascript
import { usePersistence } from '@cost-app/shared-components'
const { save, load, startAutoSave } = usePersistence()

// 自动保存（30秒间隔）
startAutoSave()
```

#### 4. 跨窗口同步
```javascript
import { useDataSync } from '@cost-app/shared-components'
const { syncAcrossWindows, broadcastChange } = useDataSync()

// 启用跨窗口同步
syncAcrossWindows()
```

### 在概算中的使用示例

```vue
<script setup>
import { useEstimate } from '@cost-app/shared-components'

// 获取共享状态
const {
  projects,
  selectedItems,
  addProject,
  updateProject,
  deleteProject,
  selectItems
} = useEstimate()

// 添加新项目
const handleAddProject = (projectData) => {
  addProject(projectData)
  // 数据自动同步到其他模块
}

// 删除项目
const handleDeleteProject = (id) => {
  deleteProject(id)
  // 自动清除相关选择状态
}
</script>
```

## 📁 文件结构

```
demo/
├── packages/
│   ├── shared-components/
│   │   ├── src/
│   │   │   ├── store/
│   │   │   │   └── index.js          # 状态管理核心
│   │   │   ├── composables/
│   │   │   │   └── useGlobalStore.js # 组合式函数
│   │   │   └── components/           # 共享组件
│   │   ├── vite.config.js           # 优化的构建配置
│   │   └── package.json             # 开发脚本
│   ├── rough-estimate/              # 概算模块
│   └── main-shell/                  # 主壳模块
├── dev-setup.js                    # 开发环境启动脚本
└── package.json                    # 工作区配置
```

## 🛠️ 开发工具

### 1. 状态调试
```javascript
import { useStoreDebug } from '@cost-app/shared-components'
const { logState, exportState, importState } = useStoreDebug()

// 查看当前状态
logState()

// 导出状态（用于调试）
const stateJson = exportState()
console.log(stateJson)
```

### 2. 性能监控
```javascript
import { useLoading } from '@cost-app/shared-components'
const { setLoading, loading } = useLoading()

// 设置加载状态
setLoading('estimate', true)
// 操作完成后
setLoading('estimate', false)
```

### 3. 通知系统
```javascript
import { useNotifications } from '@cost-app/shared-components'
const { addNotification, notifications } = useNotifications()

// 添加通知
addNotification({
  type: 'success',
  title: '操作成功',
  message: '数据已保存'
})
```

## 🚀 最佳实践

### 1. 开发流程
1. 启动开发环境：`npm run dev:all`
2. 修改 shared-components 组件
3. 观察自动重新构建
4. 在子模块中测试更新

### 2. 状态管理
1. 优先使用分类的组合式函数（如 `useEstimate`）
2. 避免直接修改 state，使用提供的 actions
3. 利用计算属性获取派生状态
4. 合理使用持久化功能

### 3. 性能优化
1. 开发模式下关闭代码压缩
2. 使用 sourcemap 便于调试
3. 合理设置自动保存间隔
4. 避免不必要的状态监听

### 4. 调试技巧
1. 使用 Vue DevTools 查看响应式状态
2. 利用状态调试工具导出/导入状态
3. 查看浏览器控制台的构建日志
4. 使用网络面板检查模块加载

## 🔧 故障排除

### 常见问题

1. **热更新不生效**
   - 检查 shared-components 是否正在运行 `npm run dev`
   - 确认文件保存后是否有构建日志
   - 重启开发服务器

2. **状态不同步**
   - 检查是否正确导入组合式函数
   - 确认是否启用了跨窗口同步
   - 查看浏览器控制台错误信息

3. **构建失败**
   - 检查依赖是否正确安装
   - 确认 Node.js 版本兼容性
   - 清除 node_modules 重新安装

### 调试命令

```bash
# 检查依赖
npm ls @cost-app/shared-components

# 清除缓存
npm run clean
npm install

# 单独构建 shared-components
cd packages/shared-components
npm run build

# 查看构建输出
npm run build -- --mode development
```

## 📈 扩展功能

### 1. 添加新的状态模块
1. 在 `store/index.js` 中添加新的状态分支
2. 在 `useGlobalStore.js` 中创建对应的组合式函数
3. 导出新的功能函数

### 2. 自定义持久化策略
1. 扩展 `persistence` 对象
2. 添加不同的存储后端（IndexedDB、SessionStorage）
3. 实现数据加密/解密

### 3. 增强跨窗口通信
1. 使用 BroadcastChannel API
2. 实现更复杂的消息路由
3. 添加消息确认机制

这个解决方案提供了完整的热更新和状态管理功能，大大提升了开发效率和用户体验。
