# ModuForge-RS 文档站点地图

## 📚 核心文档

### 入门指南
- [项目概述](/) - 项目简介和主要特性
- [快速入门](/quick-start) - 快速开始使用框架
- [架构概览](/architecture-overview) - 框架整体架构
- [API 参考](/api-reference) - 完整 API 文档

### 开发指南
- [插件开发指南](/plugin-development-guide) - 插件开发详细教程
- [宏系统开发指南](/macro-system-guide) - Node和Mark派生宏使用指南
- [宏展开示例](/macro-expansion-example) - 完整的宏展开结果展示
- [性能优化指南](/performance-guide) - 性能优化最佳实践
- [外部项目集成](/setup-external-project) - 集成到现有项目
- [集成示例项目](/example-integration-project) - 完整集成示例

## 🏗️ 架构设计

### 设计文档
- [架构设计总览](/architecture-design) - 完整架构设计
- [应用场景分析](/architecture_use_cases) - 适用场景分析
- [架构限制分析](/architecture_limitations_analysis) - 设计限制说明
- [业务依赖设计](/business_dependency_design) - 业务层依赖关系
- [元数据依赖设计](/meta_based_dependency_design) - 元数据驱动设计

## 💼 业务应用

### 应用实例
- [节点预算映射](/node-budget-mapping) - 预算管理应用
- [自定义函数开发](/CUSTOM_FUNCTIONS) - 扩展表达式语言
- [历史增强功能](/simple_enhanced_history) - 历史记录管理

## 🤝 协作与部署

### 协作系统
- [协作系统](/collaboration-system) - 实时协作功能
- [WebSocket 故障排查](/websocket-error-troubleshooting) - 连接问题解决

### 部署指南
- [部署指南](/DEPLOYMENT) - 生产环境部署

## 🎯 演示与分析

### 项目展示
- [功能演示](/demo-showcase) - 功能展示和演示
- [项目分析](/ANALYSIS) - 项目技术分析

## 🌐 多语言版本

### English Documentation
- [English Version](/en/) - Complete English documentation
- All Chinese documents have corresponding English versions in `/en/` directory

## 📁 文档结构

```
packages/docs/
├── 📄 核心文档 (中文)
│   ├── index.md (首页)
│   ├── quick-start.md (快速入门)
│   ├── api-reference.md (API 参考)
│   ├── architecture-overview.md (架构概览)
│   └── performance-guide.md (性能指南)
├── 🔧 开发指南
│   ├── plugin-development-guide.md
│   ├── setup-external-project.md
│   └── example-integration-project.md
├── 🏗️ 架构设计
│   ├── architecture-design.md
│   ├── architecture_use_cases.md
│   ├── architecture_limitations_analysis.md
│   ├── business_dependency_design.md
│   └── meta_based_dependency_design.md
├── 💼 业务应用
│   ├── node-budget-mapping.md
│   ├── CUSTOM_FUNCTIONS.md
│   └── simple_enhanced_history.md
├── 🤝 协作部署
│   ├── collaboration-system.md
│   ├── DEPLOYMENT.md
│   └── websocket-error-troubleshooting.md
├── 🎯 演示分析
│   ├── demo-showcase.md
│   └── ANALYSIS.md
└── 🌐 英文文档
    └── en/ (所有文档的英文版本)
```

## 🔍 快速查找

### 按角色查找
- **新用户**: [项目概述](/) → [快速入门](/quick-start) → [集成示例](/example-integration-project)
- **开发者**: [API 参考](/api-reference) → [插件开发](/plugin-development-guide) → [性能优化](/performance-guide)
- **架构师**: [架构概览](/architecture-overview) → [架构设计](/architecture-design) → [场景分析](/architecture_use_cases)
- **运维人员**: [部署指南](/DEPLOYMENT) → [故障排查](/websocket-error-troubleshooting)

### 按技术领域查找
- **状态管理**: [API 参考 - mf-state](/api-reference#mf-state-api)
- **规则引擎**: [API 参考 - mf-engine](/api-reference#mf-engine-api)
- **协作功能**: [协作系统](/collaboration-system) → [API 参考 - mf-collaboration](/api-reference#mf-collaboration-api)
- **性能优化**: [性能指南](/performance-guide) → [架构限制分析](/architecture_limitations_analysis)

## 🛠️ 文档工具

### 本地开发
```bash
# 启动文档开发服务器
npm run dev

# 构建文档
npm run build

# 预览构建结果
npm run preview
```

### 文档配置
- VitePress 配置: `.vitepress/config.ts`
- 支持 Mermaid 图表渲染
- 支持多语言切换
- 支持本地搜索

---

> 💡 **提示**: 使用文档站点的搜索功能可以快速找到相关内容，支持中英文搜索。