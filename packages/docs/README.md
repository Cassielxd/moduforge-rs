# ModuForge-RS 文档网站

本目录包含 ModuForge-RS 框架的完整文档，使用 VitePress 构建静态网站。

## 🚀 快速开始

### 安装依赖

```bash
cd packages/docs
npm install
```

### 本地开发

```bash
npm run dev
```

访问 http://localhost:3000 查看文档网站。

### 构建生产版本

```bash
npm run build
```

构建输出将位于 `dist/` 目录。

### 预览生产版本

```bash
npm run preview
```

## 📁 目录结构

```
packages/docs/
├── .vitepress/           # VitePress 配置
│   ├── config.ts         # 主配置文件
│   └── theme/            # 自定义主题
├── en/                   # 英文文档
├── public/               # 静态资源
├── *.md                  # 中文文档
└── package.json          # 项目配置
```

## 📝 文档说明

### 中文文档
- 位于根目录的 `.md` 文件
- 使用中文作为主要语言
- 包含完整的框架介绍和使用指南

### 英文文档
- 位于 `en/` 目录
- 提供英文版本的文档
- 与中文版本保持内容同步

## 🎨 主题定制

自定义主题位于 `.vitepress/theme/` 目录：
- `index.ts` - 主题入口文件
- `custom.css` - 自定义样式

## 🌐 多语言支持

文档支持中英文双语：
- 中文：根路径 `/`
- 英文：`/en/` 路径

## 📚 文档内容

### 核心文档
- `index.md` - 项目概述
- `plugin-development-guide.md` - 插件开发指南
- `setup-external-project.md` - 外部项目集成
- `example-integration-project.md` - 集成示例

### 架构文档
- `architecture_use_cases.md` - 架构应用场景
- `architecture_limitations_analysis.md` - 架构限制分析
- `business_dependency_design.md` - 业务依赖设计
- `meta_based_dependency_design.md` - 元数据依赖设计

### 技术文档
- `node-budget-mapping.md` - 节点预算映射
- `CUSTOM_FUNCTIONS.md` - 自定义函数
- `simple_enhanced_history.md` - 历史增强功能
- `websocket-error-troubleshooting.md` - WebSocket 故障排查

### 演示文档
- `demo-showcase.md` - 功能演示
- `ANALYSIS.md` - 项目分析

## 🔧 维护

### 添加新文档
1. 在根目录创建中文版 `.md` 文件
2. 在 `en/` 目录创建对应的英文版
3. 更新 `.vitepress/config.ts` 中的导航配置

### 更新导航
在 `.vitepress/config.ts` 的 `nav` 和 `sidebar` 配置中添加新页面。

### 样式修改
修改 `.vitepress/theme/custom.css` 文件来自定义样式。

## 📄 许可

本文档基于 MIT 许可发布。 