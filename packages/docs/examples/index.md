# ModuForge-RS 示例项目

本节包含完整的真实项目案例，展示如何使用 ModuForge-RS 框架构建复杂的文档编辑应用。

## Price-RS 工程造价系统

Price-RS 是一个基于 ModuForge-RS 框架构建的工程造价计算系统，展示了框架在实际业务场景中的应用。该项目包含：

### 📚 [项目概述与架构](./price-rs-complete-project.md)
- 完整的项目结构分析
- 多扩展系统架构
- 业务领域建模
- 技术栈选择

### 🔧 [扩展开发实战](./price-rs-extension-development.md)
- 扩展系统设计
- 节点定义与实现
- 业务逻辑集成
- 测试策略

### 🚀 [运行时启动与引导](./price-rs-runtime-bootstrap.md)
- Bootstrap 架构详解
- Provider 模式实现
- 启动流程分析
- 进度跟踪机制

## 为什么选择 Price-RS 作为案例？

1. **真实业务场景**：工程造价计算是复杂的业务领域，涉及大量数据处理和计算逻辑
2. **完整架构展示**：展示了从扩展系统到运行时引导的完整技术栈
3. **最佳实践示范**：包含了错误处理、异步编程、测试等最佳实践
4. **可扩展设计**：展示了如何构建可扩展的插件化系统

## 学习路径

1. 首先阅读[项目概述](./price-rs-complete-project.md)了解整体架构
2. 然后学习[扩展开发](./price-rs-extension-development.md)理解如何构建业务功能
3. 最后研究[运行时引导](./price-rs-runtime-bootstrap.md)掌握系统启动机制

## 源代码

完整的 Price-RS 项目源代码位于：`E:\workespace\2025\rust\price-rs`

您可以克隆该项目并运行测试来深入了解实现细节：

```bash
# 克隆项目
git clone <repository-url>

# 运行测试
cargo test

# 运行特定的导出测试
cargo test export_with_progress --lib
```

## 相关资源

- [ModuForge-RS 核心概念](/guide/core-concepts)
- [插件系统设计](/guide/plugins)
- [核心库文档](/crates/core)