# Tauri 窗体管理指南

本指南介绍如何在 Tauri 中实现类似 Electron 的父子窗体关系管理功能。

## 功能特性

### 1. 父子窗体关系
- ✅ 子窗体跟随父窗体最小化/恢复
- ✅ 父窗体关闭时自动关闭所有子窗体
- ✅ 子窗体不在状态栏显示 (`skip_taskbar: true`)
- ✅ 完整的窗体生命周期管理

### 2. 窗体类型支持
- **普通子窗体**: 独立的子窗口，可以自由操作
- **模态窗体**: 阻止父窗体交互，总是在顶部显示
- **工具窗体**: 轻量级的辅助窗口

### 3. 窗体状态同步
- 窗体最小化状态跟踪
- 父子窗体关系维护
- 窗体事件监听和处理

## 核心实现

### 后端 (Rust)

#### 1. 窗体关系管理器
```rust
// 窗体关系数据结构
#[derive(Debug, Clone)]
struct WindowRelation {
    parent: Option<String>,
    children: Vec<String>,
    is_minimized: bool,
}

// 全局窗体管理器
lazy_static::lazy_static! {
    static ref WINDOW_MANAGER: WindowManager = Arc::new(Mutex::new(HashMap::new()));
}
```

#### 2. 核心命令
- `create_child_window`: 创建子窗口
- `close_child_window`: 关闭子窗口
- `minimize_window_with_children`: 最小化窗口及其子窗口
- `restore_window_with_children`: 恢复窗口及其子窗口
- `close_window_with_children`: 关闭窗口及其所有子窗口

#### 3. 窗体事件监听
```rust
// 主窗口事件监听
window.on_window_event(move |event| {
    match event {
        WindowEvent::CloseRequested { .. } => {
            // 关闭所有子窗口
            let child_windows = get_child_windows(&window_id);
            for child_id in child_windows {
                // 关闭子窗口逻辑
            }
        }
        _ => {}
    }
});
```

### 前端 (Vue.js)

#### 1. 窗体管理组合式函数
```javascript
// useWindowManager.js
export function useWindowManager() {
  const {
    createChildWindow,
    closeChildWindow,
    minimizeWindow,
    restoreWindow,
    closeWindow
  } = useWindowManager()
}
```

#### 2. 使用示例
```vue
<script setup>
import { useWindowManager } from '@/composables/useWindowManager.js'

const { createChildWindow, createModalDialog } = useWindowManager()

// 创建普通子窗口
const openChildWindow = async () => {
  await createChildWindow({
    windowId: 'child-1',
    title: '子窗口',
    url: '/child-page',
    width: 800,
    height: 600
  })
}

// 创建模态对话框
const openModal = async () => {
  await createModalDialog({
    windowId: 'modal-1',
    title: '模态对话框',
    url: '/modal-page',
    width: 500,
    height: 300
  })
}
</script>
```

## 配置说明

### 1. 子窗口配置
```rust
let child_window = tauri::WebviewWindowBuilder::new(&app, &window_id, webview_url)
    .title(&title)
    .inner_size(width, height)
    .resizable(true)
    .decorations(true)
    .visible(true)
    .focused(true)
    .center()
    .skip_taskbar(true)  // 关键：不在状态栏显示
    .build()?;
```

### 2. 模态窗口配置
```rust
if is_modal {
    window_builder = window_builder.always_on_top(true);
}
```

## 使用方法

### 1. 启动演示
```bash
cd demo
npm run tauri dev
```

### 2. 访问演示页面
- 在主界面点击"窗体管理演示"模块
- 或直接访问 `/#/window-manager` 路由

### 3. 测试功能
1. **创建子窗口**: 使用不同参数创建各种类型的子窗口
2. **最小化测试**: 最小化主窗口，观察子窗口是否同时隐藏
3. **恢复测试**: 恢复主窗口，观察子窗口是否同时显示
4. **关闭测试**: 关闭主窗口，观察所有子窗口是否自动关闭
5. **模态测试**: 创建模态窗口，测试父窗口是否被禁用

## 最佳实践

### 1. 窗口标识符
- 使用有意义的窗口 ID，如 `module-${moduleKey}` 或 `dialog-${type}`
- 避免重复的窗口 ID

### 2. 窗口大小
- 为不同类型的窗口设置合适的默认大小
- 考虑屏幕分辨率和用户体验

### 3. 错误处理
- 始终包装窗口操作在 try-catch 中
- 提供用户友好的错误消息

### 4. 内存管理
- 及时清理窗口关系
- 避免内存泄漏

## 扩展功能

### 1. 窗口位置管理
可以扩展实现窗口位置的相对定位：
```rust
// 子窗口相对于父窗口的位置
let parent_position = parent_window.outer_position()?;
let child_position = (parent_position.x + 50, parent_position.y + 50);
```

### 2. 窗口状态持久化
可以将窗口状态保存到本地存储：
```rust
// 保存窗口状态到配置文件
let window_state = WindowState {
    position: window.outer_position()?,
    size: window.outer_size()?,
    is_maximized: window.is_maximized()?,
};
```

### 3. 窗口通信
可以实现父子窗口间的数据通信：
```rust
// 使用事件系统进行窗口间通信
parent_window.emit("data-update", payload)?;
child_window.listen("data-update", |event| {
    // 处理数据更新
})?;
```

## 注意事项

1. **平台差异**: 不同操作系统的窗口行为可能有差异
2. **性能考虑**: 大量子窗口可能影响性能
3. **用户体验**: 避免创建过多的子窗口
4. **测试覆盖**: 在不同平台上测试窗口管理功能

## 故障排除

### 1. 子窗口不跟随父窗口最小化
- 检查窗口关系是否正确注册
- 确认事件监听器是否正常工作

### 2. 窗口无法创建
- 检查窗口 ID 是否重复
- 验证 URL 是否有效

### 3. 模态窗口无法禁用父窗口
- 确认模态参数设置正确
- 检查事件发送是否成功

通过以上实现，你可以在 Tauri 中获得类似 Electron 的完整窗体管理体验。
