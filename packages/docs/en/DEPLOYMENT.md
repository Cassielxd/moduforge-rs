# ModuForge-RS 文档网站部署指南

## 📋 项目概述

本项目是 ModuForge-RS 框架的官方文档网站，基于 VitePress 构建。提供中英文双语支持，包含完整的框架文档、插件开发指南、架构分析等内容。

## 🚀 快速开始

### 方式一：使用批处理脚本（推荐）

**Windows 用户**：
```bash
# 启动开发服务器
./start-docs.bat

# 构建生产版本
./build-docs.bat
```

### 方式二：使用 npm 命令

```bash
# 进入文档目录
cd packages/docs

# 安装依赖
npm install

# 启动开发服务器
npm run dev

# 构建生产版本
npm run build

# 预览生产版本
npm run preview
```

## 🌐 访问地址

- **开发环境**: `http://localhost:3000` (仅在本地开发时可用)
- **中文版**: `http://localhost:3000/` (本地开发环境)
- **英文版**: `http://localhost:3000/en/` (本地开发环境)

> **注意**: 上述链接仅在本地运行 `npm run dev` 时可用。生产环境的访问地址将根据你的部署方式而定。

## 📁 项目结构

```
packages/docs/
├── .vitepress/           # VitePress 配置
│   ├── config.mjs        # 主配置文件
│   └── theme/            # 自定义主题
│       ├── index.js      # 主题入口
│       └── custom.css    # 自定义样式
├── en/                   # 英文文档目录
│   ├── index.md          # 英文首页
│   └── *.md              # 其他英文文档
├── public/               # 静态资源
│   └── logo.svg          # 项目 Logo
├── dist/                 # 构建输出（自动生成）
├── *.md                  # 中文文档
├── package.json          # 项目配置
├── start-docs.bat        # 开发启动脚本
├── build-docs.bat        # 构建脚本
└── README.md             # 项目说明
```

## 📚 文档内容结构

### 🏠 首页特性
- **响应式设计**: 支持桌面和移动设备
- **多语言切换**: 中英文无缝切换
- **特性展示**: 6大核心特性高亮展示
- **快速导航**: 直达核心功能页面

### 📖 文档分类

#### 1. 开始使用
- 项目概述 (`index.md`)
- 外部项目集成 (`setup-external-project.md`)
- 集成示例 (`example-integration-project.md`)

#### 2. 开发指南
- 插件开发指南 (`plugin-development-guide.md`)
- 自定义函数 (`CUSTOM_FUNCTIONS.md`)
- 节点预算映射 (`node-budget-mapping.md`)

#### 3. 架构设计
- 应用场景分析 (`architecture_use_cases.md`)
- 架构限制分析 (`architecture_limitations_analysis.md`)
- 业务依赖设计 (`business_dependency_design.md`)
- 元数据依赖设计 (`meta_based_dependency_design.md`)

#### 4. 示例和演示
- 功能演示 (`demo-showcase.md`)
- 历史增强 (`simple_enhanced_history.md`)

#### 5. 故障排查
- WebSocket 错误排查 (`websocket-error-troubleshooting.md`)
- 项目分析 (`ANALYSIS.md`)

## 🎨 主题定制

### 颜色主题
- **主色调**: `#FF6B35` (橙色)
- **辅助色**: `#F7931E` (金色)
- **支持暗黑模式**: 自动切换

### 自定义样式
位置：`.vitepress/theme/custom.css`

- 品牌色彩配置
- 响应式布局优化
- 代码高亮主题
- 特色区块样式

## 🔧 配置说明

### VitePress 配置
文件：`.vitepress/config.mjs`

主要配置项：
- **多语言支持**: 中文（默认）+ 英文
- **导航菜单**: 顶部导航和侧边栏
- **搜索功能**: 本地搜索
- **Git 集成**: 编辑链接和最后更新时间

### 导航结构
```javascript
// 中文导航
nav: [
  { text: '首页', link: '/' },
  { text: '指南', link: '/plugin-development-guide' },
  { text: '架构', link: '/architecture_use_cases' },
  { text: '示例', link: '/demo-showcase' }
]

// 英文导航
nav: [
  { text: 'Home', link: '/en/' },
  { text: 'Guide', link: '/en/plugin-development-guide' },
  { text: 'Architecture', link: '/en/architecture_use_cases' },
  { text: 'Examples', link: '/en/demo-showcase' }
]
```

## 🚀 部署方案

### 1. 静态网站托管

**Vercel 部署**:
```bash
# 安装 Vercel CLI
npm i -g vercel

# 部署
vercel --prod
```

**Netlify 部署**:
```bash
# 构建命令
npm run build

# 输出目录
dist
```

**GitHub Pages 部署**:
```yaml
# .github/workflows/deploy.yml
name: Deploy Docs
on:
  push:
    branches: [main]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: 18
      - run: cd packages/docs && npm install
      - run: cd packages/docs && npm run build
      - uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: packages/docs/dist
```

### 2. Docker 部署

创建 `Dockerfile`:
```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY packages/docs .
RUN npm install
RUN npm run build
FROM nginx:alpine
COPY --from=0 /app/dist /usr/share/nginx/html
EXPOSE 80
```

## 📝 内容维护

### 添加新文档
1. **中文文档**: 在根目录创建 `.md` 文件
2. **英文文档**: 在 `en/` 目录创建对应文件
3. **更新导航**: 修改 `.vitepress/config.mjs` 中的 `nav` 和 `sidebar`

### 文档格式规范
```markdown
---
title: 页面标题
description: 页面描述
---

# 页面标题

## 章节标题

### 子章节

- 使用 Markdown 标准语法
- 支持代码高亮
- 支持数学公式
- 支持自定义容器
```

### 多语言同步
- 保持中英文文档内容同步
- 文件名保持一致
- 导航结构对应

## 🔍 功能特性

### ✅ 已实现功能
- [x] 中英文双语支持
- [x] 响应式设计
- [x] 本地搜索
- [x] 暗黑模式
- [x] 代码高亮
- [x] 自定义主题
- [x] Git 集成
- [x] 快速导航
- [x] 特性展示页

### 🚧 计划功能
- [ ] 全文搜索集成
- [ ] 评论系统
- [ ] 版本切换
- [ ] PDF 导出
- [ ] 离线支持

## 🐛 故障排查

### 常见问题

**1. 端口被占用**
```bash
# 查看端口占用
netstat -ano | findstr :3000
# 杀死进程
taskkill /PID <PID> /F
```

**2. 依赖安装失败**
```bash
# 清除缓存
npm cache clean --force
# 删除 node_modules
rm -rf node_modules
# 重新安装
npm install
```

**3. 构建失败**
```bash
# 检查 Node.js 版本
node --version  # 需要 16+
# 清除构建缓存
rm -rf .vitepress/cache
rm -rf dist
```

## 📄 许可证

本文档基于 MIT 许可证发布。详见项目根目录的 LICENSE 文件。

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 发起 Pull Request

欢迎贡献文档内容、修复错误或提出改进建议！ 