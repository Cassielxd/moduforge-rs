# 🏗️ ModuForge Demo - 造价管理系统

基于 Tauri + Vue 3 + 微前端架构的现代化造价管理系统演示应用。

## 🎯 项目特性

- ✅ **微前端架构**: 模块化设计，支持独立开发和部署
- ✅ **Tauri 桌面应用**: 高性能的桌面应用框架，替代 Electron
- ✅ **Vue 3 + Vite**: 现代化的前端技术栈
- ✅ **多窗口支持**: 每个模块可在独立 Tauri 窗口中运行
- ✅ **共享组件库**: 统一的 UI 组件和窗口控制功能
- ✅ **自动化构建**: 一键构建所有子模块和打包

## 📁 项目结构

```
examples/demo2/
├── src/                        # 主应用源码
│   ├── views/Dashboard.vue     # 工作台主界面
│   ├── App.vue                 # 应用根组件
│   └── main.js                 # 应用入口
├── packages/                   # 微前端模块
│   ├── rough-estimate/         # 概算模块 ✅
│   ├── budget/                 # 预算模块 (待开发)
│   └── shared-components/      # 共享组件库 ✅
│       ├── src/components/     # 通用组件
│       ├── src/layouts/        # 头部组件
│       └── src/composables/    # 组合式函数
├── src-tauri/                  # Tauri 后端
│   ├── src/main.rs            # 主程序入口
│   └── tauri.conf.json        # Tauri 配置
└── dist/                       # 构建产物
```

## 🚀 快速开始

### 环境要求

- **Node.js** 16+
- **Rust** 1.70+ (用于 Tauri)
- **npm** 或 **yarn**

### 安装依赖

```bash
# 安装所有依赖（包括子模块）
npm install
```

### 开发环境

1. **启动概算模块**（必须先启动）:
```bash
cd packages/rough-estimate
npm run dev  # 启动在 http://localhost:5174
```

2. **启动主应用**:
```bash
# 在项目根目录
npm run tauri:dev
```

### 生产构建

```bash
# 构建所有模块
npm run build

# 打包 Tauri 应用
npm run tauri:build
```

## 📋 可用模块

| 模块 | 状态 | 端口 | 描述 | 功能 |
|------|------|------|------|------|
| **主应用** | ✅ 完成 | 5173 | 工作台和导航中心 | 模块导航、系统设置 |
| **概算模块** | ✅ 完成 | 5174 | 项目概算管理 | 概算编制、计算、审核 |
| **共享组件** | ✅ 完成 | - | 通用组件库 | 头部、表格、窗口控制 |
| **预算模块** | 🔄 开发中 | 5176 | 项目预算编制 | 预算编制、管理 |
| **预算审核** | 🔄 开发中 | 5177 | 预算审核流程 | 审核流程、意见反馈 |
| **结算模块** | 🔄 开发中 | 5178 | 项目结算管理 | 结算编制、计算 |
| **结算审核** | 🔄 开发中 | 5179 | 结算审核流程 | 审核流程、签批 |

## 🎮 核心功能

### 1. 微前端架构
- **独立开发**: 每个模块可独立开发、测试、部署
- **技术栈自由**: 不同模块可使用不同的技术栈
- **团队协作**: 支持多团队并行开发

### 2. Tauri 多窗口系统
- **独立窗口**: 每个模块在独立的 Tauri 窗口中运行
- **窗口管理**: 支持最小化、最大化、关闭等操作
- **父子关系**: 主窗口关闭时自动关闭所有子窗口
- **状态同步**: 窗口状态实时同步

### 3. 共享组件库
- **统一 UI**: 提供一致的用户界面组件
- **窗口控制**: 统一的窗口控制逻辑
- **操作窗口**: 支持数据导入、导出、批量操作等
- **表单系统**: 支持新建、编辑、查看等表单操作

### 4. 操作窗口系统
- **数据导入**: Excel、CSV 文件导入功能
- **数据导出**: 多格式数据导出（Excel、PDF、CSV）
- **批量操作**: 批量编辑、删除、审核等操作
- **系统设置**: 系统配置和参数管理

## 🔧 开发指南

### 添加新模块

1. **创建模块目录**:
```bash
mkdir packages/new-module
cd packages/new-module
npm init -y
```

2. **配置 Vite**:
```javascript
// vite.config.js
export default {
  base: './',  // 重要：确保相对路径
  build: {
    outDir: 'dist'
  }
}
```

3. **注册到主应用**:
```javascript
// src/views/Dashboard.vue
const modules = [
  {
    key: 'new-module',
    title: '新模块',
    description: '新模块描述',
    port: 5180,
    status: 'available'
  }
]
```

4. **添加构建脚本**:
```json
// package.json
{
  "scripts": {
    "build:new-module": "cd packages/new-module && npm run build && npm run copy-dist"
  }
}
```

### 使用共享组件

```vue
<template>
  <div>
    <!-- 使用头部组件 -->
    <AppHeader
      title="模块标题"
      :show-window-controls="true"
    />

    <!-- 使用操作栏 -->
    <OperateBar
      :operate-list="operateList"
      @operate-click="handleOperate"
    />

    <!-- 使用表格组件 -->
    <CostTable
      :data="tableData"
      :columns="columns"
      row-key="id"
    />
  </div>
</template>

<script setup>
import { AppHeader, OperateBar, CostTable } from '@cost-app/shared-components'
</script>
```

### 窗口控制

```javascript
// 使用窗口控制 composable
import { useWindowControls } from '@cost-app/shared-components'

const { isMaximized, handleMinimize, handleMaximize, handleClose } = useWindowControls()

// 在模板中使用
<ModalWindowHeader
  :is-maximized="isMaximized"
  @minimize="handleMinimize"
  @maximize="handleMaximize"
  @close="handleClose"
/>
```

## 🛠️ 技术栈

### 前端技术
- **Vue 3.5+**: 组合式 API、响应式系统
- **Vite 6.x**: 快速构建工具
- **Ant Design Vue 4.x**: UI 组件库
- **Pinia**: 状态管理
- **Vue Router**: 路由管理

### 桌面技术
- **Tauri 2.x**: 桌面应用框架
- **Rust**: 后端系统语言
- **WebView**: 前端渲染引擎

### 构建工具
- **Vite**: 前端构建
- **Cargo**: Rust 构建
- **npm workspaces**: 多包管理

## 📚 文档说明

本项目包含以下文档，除开发约束文档外已整合到本 README：

### 已整合的文档
- ✅ **微前端架构** (`README_MICROFRONTEND.md`) - 架构设计和实现
- ✅ **打包构建** (`PACKAGING_GUIDE.md`) - 构建流程和配置
- ✅ **窗口管理** (`WINDOW_MANAGEMENT_GUIDE.md`) - Tauri 窗口系统
- ✅ **操作窗口** (`TAURI_WINDOW_GUIDE.md`) - 操作窗口系统
- ✅ **子窗口修复** (`CHILD_WINDOW_FIX.md`) - 窗口控制功能
- ✅ **自定义表单** (`CUSTOM_FORM_GUIDE.md`) - 表单组件系统

### 独立保留的文档
- 📋 **开发架构指南** (`AGENT_ARCHITECTURE_GUIDE.md`) - 开发规范和架构
- 📋 **开发约束文档** (`AGENT_DEVELOPMENT_CONSTRAINTS.md`) - 开发限制和约束

## 🎯 使用方法

### 1. 启动系统
```bash
# 1. 启动概算模块
cd packages/rough-estimate && npm run dev

# 2. 启动主应用（新终端）
npm run tauri:dev
```

### 2. 使用功能
1. **主界面**: 查看 6 个业务模块卡片
2. **打开模块**: 点击"概算"卡片，在新窗口打开概算系统
3. **窗口操作**: 使用最小化、最大化、关闭按钮
4. **数据操作**: 使用导入、导出、新建、编辑等功能

### 3. 开发新模块
1. 复制概算模块结构
2. 修改配置和业务逻辑
3. 注册到主应用
4. 测试窗口功能

## 🔍 故障排除

### 常见问题

1. **模块无法打开**
   - 检查模块是否已启动（端口是否占用）
   - 确认 Tauri 窗口创建权限
   - 查看控制台错误信息

2. **窗口控制按钮无效**
   - 确认窗口配置 `modal: false`
   - 检查 `useWindowControls` 是否正确导入
   - 验证 Tauri API 权限

3. **构建失败**
   - 检查所有子模块依赖是否安装
   - 确认 Rust 环境配置正确
   - 查看构建日志错误信息

### 调试技巧

```bash
# 查看详细日志
RUST_LOG=debug npm run tauri:dev

# 检查端口占用
netstat -ano | findstr :5174

# 清理构建缓存
npm run clean && npm install
```

## 🎊 总结

这是一个完整的现代化造价管理系统，采用微前端架构和 Tauri 桌面技术：

- ✅ **架构先进**: 微前端 + Tauri 多窗口
- ✅ **功能完整**: 概算模块已完成，其他模块框架就绪
- ✅ **开发友好**: 统一的组件库和开发规范
- ✅ **用户体验**: 现代化界面和流畅的窗口操作
- ✅ **可扩展性**: 易于添加新模块和功能

现在可以基于这个框架继续开发其他业务模块，享受微前端架构带来的开发效率提升！
