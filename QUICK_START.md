# 🚀 ModuForge-RS 快速开始指南

## 🎯 项目简介

ModuForge-RS 包含两个主要部分：

1. **Demo 应用** - 基于 Tauri + Vue 3 的造价管理系统演示
2. **核心框架** - 基于 Rust 的状态管理和数据转换框架

## ⚡ 快速体验 Demo 应用

### 1. 环境准备

```bash
# 确保已安装 Node.js 16+ 和 Rust
node --version
cargo --version
```

### 2. 启动应用

```bash
# 克隆项目
git clone <repository-url>
cd moduforge-rs

# 进入 demo 目录
cd demo

# 安装依赖
npm install

# 启动应用
npm run tauri:dev
```

### 3. 体验功能

- 🏠 **主应用**: 工作台界面，点击模块卡片
- 📊 **概算模块**: 在新窗口中打开，体验完整功能
- 🪟 **窗口控制**: 最小化、最大化、关闭等操作
- 🧩 **共享组件**: 统一的头部组件和窗口管理

## 🛠️ 开发指南

### 添加新模块

```bash
# 复制现有模块
cp -r demo/packages/rough-estimate demo/packages/new-module

# 修改配置
# 1. 更新 package.json 中的名称和端口
# 2. 修改 vite.config.js 中的端口号
# 3. 在主应用 Dashboard.vue 中注册新模块
```

### 使用共享组件

```javascript
// 导入组件
import { AppHeader, SimpleHeader } from 'shared-components'

// 在模板中使用
<AppHeader title="我的应用" @close="handleClose" />
```

### 构建和打包

```bash
# 构建所有模块
npm run build

# 打包 Tauri 应用
npm run tauri:build
```

## 📚 详细文档

- [完整 README](./README.md) - 项目完整介绍
- [Demo 应用文档](./demo/README.md) - Demo 应用详细说明
- [共享组件使用](./demo/packages/shared-components/src/layouts/README.md) - 组件库使用指南
- [窗口管理指南](./demo/WINDOW_MANAGEMENT_GUIDE.md) - Tauri 窗口管理
- [打包指南](./demo/PACKAGING_GUIDE.md) - 构建和打包说明

## 🎯 核心特性

### Demo 应用特性
- ✅ 微前端架构
- ✅ 多窗口支持
- ✅ 通用头部组件
- ✅ 窗口控制功能
- ✅ 自动化构建

### ModuForge 框架特性
- ✅ 不可变数据结构
- ✅ 事件驱动架构
- ✅ 插件系统
- ✅ 状态管理
- ✅ 事务支持

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 License

计价软件内部团队使用请勿泄露。
