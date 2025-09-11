# 子窗体放大缩小功能修复

## 问题描述

子窗体的放大缩小按钮没有样式，且功能不可用。

## 问题分析

经过代码分析，发现了以下几个问题：

### 1. SimpleHeader 组件缺少 useChildWindow 属性

**问题：** `SimpleHeader` 组件没有 `useChildWindow` 属性，但子窗体在使用时传递了这个属性。

**影响：** 子窗体的窗口控制逻辑无法正确区分主窗口和子窗口模式。

### 2. 窗口控制方法逻辑不完整

**问题：** `SimpleHeader` 组件的窗口控制方法没有根据窗口类型（主窗口/子窗口）使用不同的处理逻辑。

**影响：** 子窗体的最小化、最大化、关闭功能无法正常工作。

### 3. 事件处理方法为空实现

**问题：** 子窗体应用中的 `onMinimize`、`onMaximize`、`onClose` 方法只有 console.log，没有实际功能。

**影响：** 虽然头部组件会自动处理，但缺少必要的事件响应。

### 4. 窗口控制按钮样式不够明显

**问题：** `SimpleHeader` 组件的窗口控制按钮样式较为简陋，不够明显。

**影响：** 用户体验不佳，按钮不够突出。

## 修复方案

### 1. 为 SimpleHeader 组件添加 useChildWindow 属性

```javascript
// 在 SimpleHeader.vue 中添加
useChildWindow: {
  type: Boolean,
  default: false
}
```

### 2. 修复窗口控制方法逻辑

```javascript
// 最小化方法
const handleMinimize = async () => {
  if (loading.value) return
  
  loading.value = true
  try {
    emit('minimize')
    
    if (isTauri.value && currentWindow.value) {
      if (props.useChildWindow) {
        // 子窗口直接最小化
        await currentWindow.value.minimize()
      } else {
        // 主窗口使用自定义逻辑
        const { invoke } = await import('@tauri-apps/api/core')
        const windowLabel = currentWindow.value.label || 'main'
        await invoke('minimize_window_with_children', { windowId: windowLabel })
      }
    }
  } catch (error) {
    console.error('最小化窗口失败:', error)
  } finally {
    loading.value = false
  }
}
```

### 3. 改进窗口控制按钮样式

```css
.control-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: rgba(0, 0, 0, 0.04);
  color: #595959;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 14px;
}

.control-btn:hover {
  background: rgba(0, 0, 0, 0.08);
  color: #262626;
  transform: translateY(-1px);
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-left: 12px;
  padding: 4px;
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.02);
}
```

### 4. 更新事件处理方法注释

在子窗体应用中添加注释说明头部组件会自动处理窗口控制逻辑。

## 修复文件列表

1. `demo/packages/shared-components/src/layouts/SimpleHeader.vue`
   - 添加 `useChildWindow` 属性
   - 修复窗口控制方法逻辑
   - 改进按钮样式

2. `demo/packages/rough-estimate/src/App.vue`
   - 更新事件处理方法注释

3. `demo/packages/main-shell/src/App.vue`
   - 更新事件处理方法注释

## 验证方法

1. 启动开发服务器：`npm run dev`
2. 打开主应用
3. 创建子窗口（如概算管理）
4. 测试子窗口的最小化、最大化、关闭功能
5. 检查按钮样式是否正常显示

## 技术要点

### 子窗口配置

子窗口在 Rust 端配置为：
```rust
.decorations(false) // 不显示原生标题栏
.skip_taskbar(true) // 不在状态栏显示
```

这意味着子窗口完全依赖自定义头部组件提供窗口控制功能。

### 窗口控制逻辑区别

- **主窗口：** 使用 `invoke('minimize_window_with_children')` 等自定义命令
- **子窗口：** 直接使用 `currentWindow.minimize()` 等 Tauri API

### 样式设计原则

- 按钮大小适中（32x32px）
- 悬停效果明显
- 关闭按钮有特殊的红色悬停效果
- 整体风格与应用保持一致

## 后续优化建议

1. 考虑添加键盘快捷键支持
2. 添加窗口状态变化的动画效果
3. 支持自定义按钮图标
4. 添加更多窗口控制选项（如置顶、透明度等）
