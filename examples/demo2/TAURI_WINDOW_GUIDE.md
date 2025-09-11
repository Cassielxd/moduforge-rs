# Tauri 窗口管理系统使用指南

## 概述

基于 Tauri 的窗口管理系统专门为 OperateBar 操作设计，提供了创建独立 Tauri 窗口的能力。与应用内弹窗不同，这些窗口是真正的系统级窗口，具有独立的进程和更好的性能。

## ✅ 已修复的问题

### 1. 操作窗口组件移至公共组件库
- **问题**: 操作窗口组件在各子应用中，无法跨应用复用
- **解决**: 将所有操作窗口组件移至 `@cost-app/shared-components`
- **位置**: `packages/shared-components/src/components/operate/`
- **导出**: 在 `shared-components/src/index.js` 中统一导出

### 2. 添加公共窗口头部组件
- **问题**: 缺少统一的窗口头部样式
- **解决**: 使用 `ModalWindowHeader` 组件提供统一的头部
- **功能**: 标题、副标题、关闭按钮

### 3. 实现骨架屏加载效果
- **问题**: 窗口打开时没有加载状态
- **解决**: 添加 `loading` 状态和 `a-skeleton` 组件
- **效果**: 500ms 模拟加载时间，提供友好的用户体验

### 4. 移除不用的应用内弹窗代码
- **问题**: 保留了旧的 DOM 弹窗相关代码
- **解决**: 移除 `UniversalModal`、`useGlobalModal` 等相关代码
- **清理**: 删除模板中的弹窗组件和相关方法

### 5. 修复组件引用错误
- **问题**: OperatePage 中组件路径错误
- **解决**: 使用正确的公共组件库导入方式
- **方式**: `import('@cost-app/shared-components').then(m => m.ComponentName)`

### 6. 修复窗口控制按钮问题
- **问题**: 除了新建窗口，其他窗口的最小化、最大化、关闭按钮不能用
- **原因**: 窗口配置为模态窗口 (`modal: true`)
- **解决**: 将所有操作窗口改为非模态 (`modal: false`)
- **配置**: 启用 `minimizable: true`, `maximizable: true`, `closable: true`

### 7. 优化加载体验
- **问题**: 先显示 loading 再显示骨架屏，体验不佳
- **解决**: 直接显示骨架屏，移除外层的 `a-spin` 组件
- **时间**: 减少加载延迟到 200-300ms
- **效果**: 窗口打开即显示骨架屏，然后快速切换到实际内容

### 8. 集成新建功能
- **问题**: 新建窗口使用独立的 formWindowManager，与操作窗口系统分离
- **发现**: 新建窗口控制按钮正常工作的原因是 `modal: !isModal` 配置
- **解决**: 将新建、编辑、查看功能集成到操作窗口系统
- **配置**: 添加 `create-record`, `edit-record`, `view-record` 预设
- **方法**: 提供 `createRecord()`, `editRecord()`, `viewRecord()` 便捷方法

### 9. 修复窗口控制按钮根本问题
- **根本原因**: Tauri 后端所有窗口都使用 `decorations(false)` 移除原生标题栏
- **问题**: `ModalWindowHeader` 组件只发出事件，没有实际的窗口控制逻辑
- **解决**: 在操作窗口组件中监听头部事件并调用 Tauri API
- **实现**: 使用 `getCurrentWebviewWindow()` 获取窗口引用，调用 `minimize()`, `maximize()`, `unmaximize()` 方法
- **效果**: 所有操作窗口的控制按钮现在都能正常工作

### 10. 修复窗口管理逻辑问题
- **问题1**: 关闭窗口后留下白屏页面
- **解决1**: 在 `handleCancel` 中调用 `currentWindow.value.close()` 真正关闭窗口
- **问题2**: 重复打开窗口，缺少窗口存在性检查
- **解决2**: 使用固定窗口ID `operate-${operateName}` 替代时间戳ID，确保窗口复用逻辑正常工作
- **改进**: 移除时间戳，使用 `windowId` 参数传递窗口标识

### 11. 窗口控制方法统一化
- **问题**: 每个操作窗口组件都重复实现窗口控制逻辑
- **解决**: 创建 `useWindowControls` composable 统一管理窗口控制
- **位置**: `packages/shared-components/src/composables/useWindowControls.js`
- **功能**: 提供 `handleMinimize`, `handleMaximize`, `handleClose` 等统一方法
- **效果**: 减少代码重复，提高维护性

### 12. 修复新建窗体组件加载问题
- **问题**: FormWindow 组件未在 OperatePage 组件映射中注册
- **解决**: 在 `componentMap` 中添加 FormWindow 组件映射
- **配置**: `'FormWindow': () => import('@cost-app/shared-components').then(m => m.FormWindow)`

### 13. 清理冗余代码
- **删除文件**:
  - `UniversalModal.vue` - 应用内弹窗组件
  - `DataImportModal.vue` - 旧的导入弹窗
  - `DataExportModal.vue` - 旧的导出弹窗
  - `useUniversalModal.js` - 弹窗管理 composable
  - `UNIVERSAL_MODAL_GUIDE.md` - 弹窗系统文档
- **更新导出**: 移除 shared-components 中对已删除文件的引用
- **系统简化**: 统一使用 Tauri 窗口系统，移除应用内弹窗

### 14. 修复 FormWindow 组件问题
- **问题1**: `windowInfo.value` 未定义导致的运行时错误
- **解决1**: 使用可选链操作符 `windowInfo.value?.label` 进行安全访问
- **问题2**: FormWindow 使用旧的窗口控制方法
- **解决2**: 替换为统一的 `useWindowControls` composable
- **改进**: 添加骨架屏加载效果，提升用户体验

### 15. 窗口打开时的白屏优化
- **问题**: 窗口打开时会短暂显示白屏
- **解决**: 为所有窗口组件添加骨架屏加载状态
- **实现**:
  - 添加 `loading` 状态变量
  - 300ms 加载延迟模拟
  - 使用 `a-skeleton` 组件显示骨架屏
- **效果**: 窗口打开时立即显示骨架屏，然后平滑过渡到实际内容

### 16. 重构表单窗口系统
- **问题**: 原有的 FormWindow 组件过于复杂，依赖多个已删除的 composable
- **解决**: 完全重写表单窗口，采用操作窗口的统一模式
- **删除文件**:
  - `FormWindow.vue` - 旧的表单窗口组件
  - `SimpleFormWindow.vue` - 简化表单窗口
  - `useFormWindowManager.js` - 表单窗口管理器
  - `useSimpleWindowManagement.js` - 简化窗口管理
  - `useWindowDataExchange.js` - 窗口数据交换
  - `useUniversalWindowManager.js` - 通用窗口管理器
- **新建文件**: `packages/shared-components/src/components/operate/FormWindow.vue`
- **特性**:
  - 统一的窗口控制（使用 `useWindowControls`）
  - 骨架屏加载效果
  - 支持 create/edit/view 三种模式
  - 完整的表单验证和提交逻辑
  - 响应式设计

## 系统架构

### 核心组件

1. **useOperateWindowManager.js** - Tauri 窗口管理器
2. **OperatePage.vue** - 操作页面路由组件
3. **操作窗口组件** - 具体的操作功能组件

### 工作流程

```
OperateBar 点击 → useOperateWindowManager → 创建 Tauri 窗口 → 加载 OperatePage → 动态加载操作组件
```

## 使用方式

### 1. 在主应用中集成

```javascript
// EstimateMain.vue
import { useGlobalOperateWindow } from '@cost-app/shared-components'

// 初始化 Tauri 窗口管理器
const operateWindowManager = useGlobalOperateWindow({
  appId: 'rough-estimate',
  defaultPort: '5174',
  routePath: 'operate-page'
})

// 操作处理
const handleOperateClick = (item) => {
  const tauriWindowOperations = [
    'import-data',
    'export-table', 
    'batch-operation',
    'system-settings'
  ]
  
  if (tauriWindowOperations.includes(item.name)) {
    operateWindowManager.openOperateWindow(item, {
      windowInfo: windowInfo.value,
      data: selectedData,
      tableData: estimateData.value,
      customParams: {
        selectedCount: selectedData.length,
        totalCount: estimateData.value.length
      }
    })
  }
}
```

### 2. 表单操作集成

现在新建、编辑、查看功能已集成到操作窗口系统：

```javascript
// 在 handleOperateClick 中处理表单操作
const tauriWindowOperations = [
  'create', 'edit', 'view',  // 表单操作
  'import-data', 'export-table', 'batch-operation'
]

// 表单操作使用专门的便捷方法
if (item.name === 'create') {
  operateWindowManager.createRecord({
    windowInfo: windowInfo.value
  })
} else if (item.name === 'edit') {
  operateWindowManager.editRecord(selectedRecord, {
    windowInfo: windowInfo.value
  })
} else if (item.name === 'view') {
  operateWindowManager.viewRecord(selectedRecord, {
    windowInfo: windowInfo.value
  })
}
```

### 3. 操作配置

在操作配置中标记需要使用 Tauri 窗口的操作：

```javascript
// operateConfig.js
const operateList = ref([
  {
    label: '新建',
    name: 'create',
    iconType: 'icon-cs-xinzeng',
    useTauriWindow: true, // 使用 Tauri 窗口
  },
  {
    label: '数据导入',
    name: 'import-data',
    iconType: 'icon-cs-daoru',
    useTauriWindow: true, // 标记使用 Tauri 窗口
  },
  {
    label: '数据导出',
    name: 'export-table',
    iconType: 'icon-cs-daochubaobiao',
    useTauriWindow: true,
  }
])
```

### 3. 创建操作窗口组件

```vue
<!-- DataImportWindow.vue -->
<template>
  <div class="data-import-window">
    <!-- 组件内容 -->
  </div>
</template>

<script setup>
// Props - 从 OperatePage 传递
const props = defineProps({
  operateType: String,      // 操作类型
  operateLabel: String,     // 操作标签
  data: Array,             // 选中的数据
  tableData: Array,        // 完整表格数据
  parentWindow: String,    // 父窗口标识
  appId: String,          // 应用ID
  selectedCount: Number,   // 选中数量
  totalCount: Number,      // 总数量
  hasSelection: Boolean    // 是否有选中
})

// Emits - 向 OperatePage 发送事件
const emit = defineEmits(['submit', 'cancel', 'close', 'update'])

// 事件处理
const handleSubmit = (data) => {
  emit('submit', data) // 提交数据
}

const handleCancel = () => {
  emit('cancel') // 取消操作
}
</script>
```

### 4. 注册操作窗口

```javascript
// 使用预设配置
import { operateWindowPresets } from '@cost-app/shared-components'

// 或自定义注册
operateWindowManager.registerOperateWindow('custom-operation', {
  title: '自定义操作',
  size: 'large',           // small, medium, large, xlarge
  modal: true,             // 是否模态
  resizable: true,         // 是否可调整大小
  componentName: 'CustomOperationWindow',
  routePath: 'custom-operation'
})
```

## 预设操作窗口

系统预设了以下常用操作窗口：

### 数据导入 (import-data)
- **功能**: Excel、CSV 文件导入
- **窗口大小**: large (1000x700)
- **模态**: true
- **组件**: DataImportWindow

### 数据导出 (export-table)
- **功能**: 多格式数据导出
- **窗口大小**: medium (800x600)
- **模态**: true
- **组件**: DataExportWindow

### 批量操作 (batch-operation)
- **功能**: 批量处理数据
- **窗口大小**: large (1000x700)
- **模态**: true

### 系统设置 (system-settings)
- **功能**: 系统配置管理
- **窗口大小**: large (1000x700)
- **模态**: false

### 数据分析 (data-analysis)
- **功能**: 数据统计分析
- **窗口大小**: xlarge (1200x800)
- **模态**: false

### 模板管理 (template-manage)
- **功能**: 模板增删改查
- **窗口大小**: large (1000x700)
- **模态**: false

## 窗口配置选项

```javascript
const windowConfig = {
  title: '窗口标题',
  size: 'medium',          // 窗口大小: small, medium, large, xlarge
  modal: false,            // 是否模态窗口 (建议设为 false，确保窗口控制按钮可用)
  resizable: true,         // 是否可调整大小
  minimizable: true,       // 是否可最小化 (建议设为 true)
  maximizable: true,       // 是否可最大化
  closable: true,          // 是否可关闭
  alwaysOnTop: false,      // 是否置顶
  skipTaskbar: false,      // 是否跳过任务栏
  routePath: 'operation',  // 路由路径
  componentName: 'OperationWindow', // 组件名称
  loadingDelay: 300        // 加载延迟
}
```

## 窗口通信

### 父窗口 → 子窗口
通过 URL 参数传递数据：

```javascript
operateWindowManager.openOperateWindow(item, {
  data: selectedData,
  tableData: allData,
  customParams: {
    mode: 'edit',
    category: 'estimate'
  }
})
```

### 子窗口 → 父窗口
通过 Tauri API 发送消息：

```javascript
// 在操作窗口组件中
import { invoke } from '@tauri-apps/api/core'

const handleSubmit = async (data) => {
  // 向父窗口发送消息
  await invoke('send_window_message', {
    targetWindow: parentWindow,
    message: {
      type: 'operate-submit',
      operate: operateType,
      data
    }
  })
  
  // 关闭当前窗口
  await invoke('close_current_window')
}
```

## 路由配置

在应用的路由中添加操作页面：

```javascript
// router/index.js
const routes = [
  {
    path: '/operate-page/:operate?',
    name: 'OperatePage',
    component: () => import('../views/OperatePage.vue'),
    meta: { title: '操作页面' }
  }
]
```

## 开发环境 vs 生产环境

### 开发环境
- URL: `http://localhost:5174/#/operate-page/import-data?params...`
- 使用当前应用的完整 URL

### 生产环境
- URL: `/rough-estimate/index.html#/operate-page/import-data?params...`
- 使用相对路径指向子应用

## 最佳实践

### 1. 组件设计

- **轻量化**: 操作窗口组件应该专注于单一功能
- **响应式**: 支持不同窗口大小的自适应布局
- **错误处理**: 完善的错误处理和用户反馈

### 2. 数据传递

- **最小化**: 只传递必要的数据
- **序列化**: 确保数据可以 JSON 序列化
- **验证**: 在组件中验证接收到的数据

### 3. 性能优化

- **懒加载**: 使用动态导入减少初始包大小
- **缓存**: 合理使用组件缓存
- **内存管理**: 及时清理不需要的数据

### 4. 窗口控制配置

**重要**: 为确保窗口控制按钮正常工作，请遵循以下配置：

```javascript
// ✅ 推荐配置 - 窗口控制按钮可用
const windowConfig = {
  modal: false,        // 必须设为 false，模态窗口会禁用控制按钮
  minimizable: true,   // 启用最小化按钮
  maximizable: true,   // 启用最大化按钮
  closable: true,      // 启用关闭按钮
  resizable: true      // 启用窗口大小调整
}

// ❌ 避免的配置 - 窗口控制按钮不可用
const windowConfig = {
  modal: true,         // 模态窗口会禁用所有控制按钮
  minimizable: false,  // 禁用最小化
  maximizable: false   // 禁用最大化
}
```

### 5. 窗口控制按钮实现

**技术背景**: Tauri 后端使用 `decorations(false)` 移除原生窗口装饰，必须使用自定义组件提供窗口控制。

**实现方式**:
```javascript
// 使用统一的窗口控制 composable
import { useWindowControls } from '@cost-app/shared-components'

// 获取窗口控制方法
const { isMaximized, handleMinimize, handleMaximize, handleClose } = useWindowControls()

// 在模板中监听头部事件
<ModalWindowHeader
  :is-maximized="isMaximized"
  @minimize="handleMinimize"
  @maximize="handleMaximize"
  @close="handleCancel"
/>

// 关闭窗口处理
const handleCancel = async () => {
  await handleClose(emit) // 统一的关闭逻辑
}
```

**composable 内部实现**:
```javascript
// useWindowControls.js
export function useWindowControls() {
  const currentWindow = ref(null)
  const isMaximized = ref(false)

  // 自动获取窗口引用
  onMounted(async () => {
    currentWindow.value = getCurrentWebviewWindow()
  })

  // 统一的窗口控制方法
  const handleMinimize = async () => {
    await currentWindow.value.minimize()
  }

  const handleMaximize = async () => {
    if (isMaximized.value) {
      await currentWindow.value.unmaximize()
      isMaximized.value = false
    } else {
      await currentWindow.value.maximize()
      isMaximized.value = true
    }
  }

  const handleClose = async (emit) => {
    if (emit) emit('cancel')
    await currentWindow.value.close()
  }

  return { isMaximized, handleMinimize, handleMaximize, handleClose }
}
```

### 6. 用户体验

- **加载状态**: 直接显示骨架屏，避免多层加载提示
- **进度反馈**: 长时间操作显示进度条
- **错误恢复**: 提供重试和恢复机制
- **窗口控制**: 确保最小化、最大化、关闭按钮可用

## 故障排除

### 常见问题

1. **窗口创建失败**
   - 检查 Tauri API 是否可用
   - 确认窗口配置参数正确
   - 查看控制台错误信息

2. **组件加载失败**
   - 检查组件路径和导入
   - 确认路由配置正确
   - 验证组件导出格式

3. **数据传递问题**
   - 检查 URL 参数长度限制
   - 确认数据序列化正确
   - 验证组件 props 定义

### 调试技巧

```javascript
// 开启调试模式
const operateWindowManager = useGlobalOperateWindow({
  debug: true // 开启调试日志
})

// 监听窗口事件
operateWindowManager.on('window-created', (windowId) => {
  console.log('窗口创建:', windowId)
})

operateWindowManager.on('window-closed', (windowId) => {
  console.log('窗口关闭:', windowId)
})
```

## 总结

Tauri 窗口管理系统提供了一个强大而灵活的解决方案，用于处理 OperateBar 中需要独立窗口的操作。通过统一的接口和配置机制，可以轻松地添加新的操作窗口，同时保持良好的用户体验和系统性能。
