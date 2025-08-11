# 🏗️ ModuForge Demo - 造价管理系统

基于 Tauri + Vue 3 + 微前端架构的现代化造价管理系统。

## 🎯 项目特性

- ✅ **微前端架构**: 模块化设计，支持独立开发和部署
- ✅ **Tauri 桌面应用**: 高性能的桌面应用框架
- ✅ **Vue 3 + Vite**: 现代化的前端技术栈
- ✅ **多窗口支持**: 每个模块可在独立窗口中运行
- ✅ **自动化构建**: 一键构建所有子模块

## 📁 项目结构

```
demo/
├── src/                        # 主应用源码
│   ├── views/Dashboard.vue     # 工作台主界面
│   ├── App.vue                 # 应用根组件
│   └── main.js                 # 应用入口
├── packages/                   # 微前端模块
│   ├── rough-estimate/         # 概算模块 ✅
│   ├── main-shell/             # 主应用模块 ✅
│   ├── budget/                 # 预算模块 (待开发)
│   └── shared-components/      # 共享组件库
├── src-tauri/                  # Tauri 后端
└── dist/                       # 构建产物
```

## 🚀 快速开始

### 环境要求

- Node.js 16+
- Rust (用于 Tauri)
- npm 或 yarn

### 安装依赖

```bash
npm install
```

### 开发环境

1. **启动子模块**（可选）:
```bash
cd packages/rough-estimate
npm run dev
```

2. **启动主应用**:
```bash
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

| 模块 | 状态 | 端口 | 描述 |
|------|------|------|------|
| 概算 | ✅ 可用 | 5174 | 项目概算管理和计算 |
| 主应用 | ✅ 可用 | 5173 | 主应用界面和导航 |
| 预算 | 🔄 开发中 | 5175 | 项目预算编制和管理 |
| 预算审核 | 🔄 开发中 | 5176 | 预算审核流程管理 |
| 结算 | 🔄 开发中 | 5177 | 项目结算管理 |
| 结算审核 | 🔄 开发中 | 5178 | 结算审核流程管理 |

## 🔧 开发指南

### 添加新模块

1. 在 `packages/` 目录下创建新模块
2. 配置模块的 `vite.config.js`，确保设置 `base: './'`
3. 在主应用的构建脚本中添加新模块
4. 在 Dashboard 中添加模块卡片

### 构建配置

项目使用自动化构建流程：
- 先构建所有子模块
- 将子模块构建产物复制到主应用 `dist` 目录
- 最后构建主应用

详细信息请参考 [PACKAGING_GUIDE.md](./PACKAGING_GUIDE.md)

## 📚 相关文档

- [📦 打包指南](./PACKAGING_GUIDE.md) - 详细的构建和打包说明
- [🏗️ 微前端架构](./README_MICROFRONTEND.md) - 微前端架构详细说明

## 🛠️ 技术栈

- **前端**: Vue 3, Vite, Ant Design Vue
- **桌面**: Tauri
- **状态管理**: Pinia
- **路由**: Vue Router
- **构建**: Vite + 自定义构建脚本

## 📝 开发说明

### IDE 推荐

- [VSCode](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar)

### 全局状态管理

项目使用 Pinia 进行状态管理，支持跨模块的状态共享。

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 推送到分支
5. 创建 Pull Request
