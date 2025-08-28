# 布局组件使用指南

## 组件介绍

### AppHeader - 主应用头部组件
功能丰富的头部组件，适用于主应用窗口，包含完整的窗口控制功能和自定义内容区域。

### SimpleHeader - 简化头部组件
轻量级头部组件，适用于子窗口或简单页面，提供基本的窗口控制功能。

## 使用方法

### 1. 导入组件

```javascript
import { AppHeader, SimpleHeader } from 'shared-components'
```

### 2. 在主应用中使用 AppHeader

```vue
<template>
  <div class="app">
    <AppHeader 
      title="造价管理系统"
      :show-logo="true"
      :show-window-controls="true"
      :draggable="true"
      @minimize="onMinimize"
      @maximize="onMaximize"
      @close="onClose"
    >
      <!-- 左侧自定义内容 -->
      <template #left>
        <div class="custom-logo">
          <img src="/logo.png" alt="Logo" />
          <h2>我的应用</h2>
        </div>
      </template>
      
      <!-- 中间导航菜单 -->
      <template #center>
        <nav class="main-nav">
          <a href="#" class="nav-item">首页</a>
          <a href="#" class="nav-item">概算</a>
          <a href="#" class="nav-item">预算</a>
        </nav>
      </template>
      
      <!-- 右侧用户信息 -->
      <template #right>
        <div class="user-info">
          <span>欢迎，管理员</span>
          <img src="/avatar.png" alt="Avatar" class="avatar" />
        </div>
      </template>
    </AppHeader>
    
    <main class="app-content">
      <!-- 应用主内容 -->
    </main>
  </div>
</template>

<script setup>
const onMinimize = () => {
  console.log('窗口最小化')
}

const onMaximize = () => {
  console.log('窗口最大化/还原')
}

const onClose = () => {
  console.log('窗口关闭')
}
</script>
```

### 3. 在子窗口中使用 SimpleHeader

```vue
<template>
  <div class="child-window">
    <SimpleHeader 
      title="概算管理"
      :show-window-controls="true"
      :draggable="true"
      @minimize="onMinimize"
      @maximize="onMaximize"
      @close="onClose"
    >
      <!-- 右侧工具栏 -->
      <template #right>
        <div class="toolbar">
          <button class="btn">保存</button>
          <button class="btn">导出</button>
        </div>
      </template>
    </SimpleHeader>
    
    <main class="window-content">
      <!-- 子窗口内容 -->
    </main>
  </div>
</template>

<script setup>
const onMinimize = () => {
  console.log('子窗口最小化')
}

const onMaximize = () => {
  console.log('子窗口最大化/还原')
}

const onClose = () => {
  console.log('子窗口关闭')
}
</script>
```

## Props 说明

### AppHeader Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| title | String | '应用标题' | 应用标题 |
| showLogo | Boolean | true | 是否显示默认Logo |
| showWindowControls | Boolean | true | 是否显示窗口控制按钮 |
| draggable | Boolean | true | 是否启用拖拽功能 |
| minimizeTitle | String | '最小化' | 最小化按钮提示文字 |
| maximizeTitle | String | '最大化/还原' | 最大化按钮提示文字 |
| closeTitle | String | '关闭' | 关闭按钮提示文字 |
| useChildWindow | Boolean | false | 是否为子窗口模式 |

### SimpleHeader Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| title | String | '应用' | 窗口标题 |
| showWindowControls | Boolean | true | 是否显示窗口控制按钮 |
| draggable | Boolean | true | 是否启用拖拽功能 |
| minimizeTitle | String | '最小化' | 最小化按钮提示文字 |
| maximizeTitle | String | '最大化/还原' | 最大化按钮提示文字 |
| closeTitle | String | '关闭' | 关闭按钮提示文字 |

## 事件说明

| 事件名 | 参数 | 说明 |
|--------|------|------|
| minimize | - | 最小化按钮点击事件 |
| maximize | - | 最大化按钮点击事件 |
| close | - | 关闭按钮点击事件 |
| window-state-change | { maximized: boolean } | 窗口状态变化事件 |

## 插槽说明

### AppHeader 插槽

- `left`: 左侧内容区域
- `center`: 中间内容区域（可拖拽）
- `right`: 右侧内容区域

### SimpleHeader 插槽

- `left`: 左侧内容区域
- `center`: 中间内容区域（可拖拽）
- `right`: 右侧内容区域

## 样式自定义

组件提供了基础样式，你可以通过CSS变量或覆盖样式来自定义外观：

```css
/* 自定义主题色 */
.app-header {
  --header-bg: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  --header-text-color: white;
  --control-btn-hover: rgba(255, 255, 255, 0.2);
}

.simple-header {
  --header-bg: #fff;
  --header-border: #e8e8e8;
  --header-text-color: #262626;
}
```

## 注意事项

1. 组件会自动检测是否在 Tauri 环境中运行，非 Tauri 环境下窗口控制功能将不可用
2. 主窗口和子窗口的窗口控制逻辑不同，请根据实际情况设置 `useChildWindow` 属性
3. 拖拽功能需要 Tauri 支持，确保正确设置 `data-tauri-drag-region` 属性
4. 组件支持深色主题，会根据系统设置自动切换
