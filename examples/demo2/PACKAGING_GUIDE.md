# 📦 微前端模块打包指南

## 🎯 问题描述

原本的项目在 Tauri 打包时，只打包了主应用，而 `packages/` 目录中的子模块（如 `rough-estimate`、`main-shell` 等）没有被包含在最终的打包产物中，导致打包后的程序无法找到这些子模块页面。

## ✅ 解决方案

### 1. 修改构建脚本

在 `demo/package.json` 中修改了构建脚本，确保在构建主应用之前先构建所有子模块：

```json
{
  "scripts": {
    "build": "npm run build:packages && vite build",
    "build:packages": "npm run build:rough-estimate && npm run build:main-shell && npm run build:shared-components",
    "build:rough-estimate": "cd packages/rough-estimate && npm run build && npm run copy-dist",
    "build:main-shell": "cd packages/main-shell && npm run build && npm run copy-dist",
    "build:shared-components": "cd packages/shared-components && npm run build"
  }
}
```

### 2. 添加复制脚本

为每个子模块添加了复制脚本，将构建产物复制到主应用的 `dist` 目录：

- `packages/rough-estimate/copy-dist.js`
- `packages/main-shell/copy-dist.js`

### 3. 修改 Vite 配置

在主应用的 `vite.config.js` 中添加了 `copyPackagesPlugin` 插件，确保在构建时自动复制子模块的构建产物。

### 4. 修改窗口创建逻辑

在 `src/views/Dashboard.vue` 中修改了 `openModule` 函数，使其能够在开发环境和生产环境中正确处理 URL：

- **开发环境**: 使用 `http://localhost:端口` 访问开发服务器
- **生产环境**: 使用相对路径 `模块名/index.html` 访问打包后的静态文件

在 `src-tauri/src/main.rs` 中修改了 `create_module_window` 函数，使其能够正确处理相对路径。

## 📁 构建产物结构

构建完成后，`dist` 目录结构如下：

```
dist/
├── index.html              # 主应用入口
├── splashscreen.html       # 启动屏幕
├── favicon.ico             # 图标
├── assets/                 # 主应用资源
│   ├── Dashboard-xxx.css
│   ├── Dashboard-xxx.js
│   ├── index-xxx.css
│   └── index-xxx.js
├── main-shell/             # 主应用模块
│   ├── index.html
│   └── assets/
└── rough-estimate/         # 概算模块
    ├── index.html
    └── assets/
```

## 🚀 使用方法

### 开发环境

1. 启动子模块（如需要）：
```bash
cd packages/rough-estimate
npm run dev
```

2. 启动主应用：
```bash
npm run tauri:dev
```

### 生产环境构建

1. 构建所有模块：
```bash
npm run build
```

2. 打包 Tauri 应用：
```bash
npm run tauri:build
```

## ✨ 特性

- ✅ 自动构建所有子模块
- ✅ 自动复制构建产物到主应用
- ✅ 开发环境和生产环境自动切换 URL 处理
- ✅ 支持在新窗口中打开子模块
- ✅ 完整的微前端架构支持

## 🔧 故障排除

### 构建失败

如果构建失败，请检查：

1. 所有子模块的依赖是否已安装
2. 子模块的 `vite.config.js` 配置是否正确
3. 复制脚本是否有权限访问目标目录

### 子模块无法打开

如果打包后的应用中子模块无法打开，请检查：

1. `dist` 目录中是否包含子模块文件夹
2. 子模块的 `index.html` 文件是否存在
3. Tauri 的安全配置是否允许访问本地文件

## 📝 注意事项

- 每次修改子模块后，需要重新运行 `npm run build` 来更新构建产物
- 子模块的路由配置应该使用相对路径
- 确保所有子模块都有正确的 `index.html` 入口文件
