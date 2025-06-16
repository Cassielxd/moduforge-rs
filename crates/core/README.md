# ModuForge Runtime

ModuForge Runtime 是一个强大的 Rust 编辑器运行时框架，提供了插件加载、热更新、依赖管理等核心功能。该框架设计用于构建可扩展的编辑器系统，支持异步操作和事件驱动架构。

## 主要特性

- **插件系统**
  - 支持动态插件加载和卸载
  - 插件生命周期管理
  - 插件间通信机制

- **异步运行时**
  - 基于 Tokio 的异步操作支持
  - 事件驱动的架构设计
  - 高性能的并发处理

- **状态管理**
  - 事务性状态更新
  - 撤销/重做支持
  - 历史记录管理

- **事件系统**
  - 事件总线设计
  - 发布-订阅模式
  - 异步事件处理

- **扩展性**
  - 模块化设计
  - 可自定义的扩展点
  - 灵活的配置选项

## 运行时实现

ModuForge Runtime 提供了两种运行时实现，以满足不同的使用场景：

### Runtime
同步运行时实现，适用于：
- 简单的编辑器场景
- 不需要复杂异步操作的应用
- 对性能要求不是特别高的场景

特点：
- 同步操作处理
- 简单的状态管理
- 基础的事件处理
- 适合快速开发和原型验证

### AsyncRuntime
异步运行时实现，适用于：
- 复杂的编辑器场景
- 需要处理大量并发操作
- 高性能要求的应用

特点：
- 基于 Tokio 的异步操作
- 流式处理引擎（FlowEngine）
- 高级状态管理
- 异步事件处理
- 更好的性能和扩展性

### 使用场景对比

| 特性 | Runtime | AsyncRuntime |
|------|---------|--------------|
| 并发处理 | 基础 | 高级 |
| 性能 | 一般 | 优秀 |
| 复杂度 | 低 | 高 |
| 适用场景 | 简单应用 | 复杂应用 |
| 资源消耗 | 较低 | 较高 |
| 开发难度 | 简单 | 较复杂 |

## 核心组件

### Runtime
编辑器核心实现，负责：
- 文档状态管理
- 事件处理
- 插件系统集成
- 存储管理

### ExtensionManager
扩展管理器，处理：
- 插件加载和卸载
- 插件生命周期管理
- 插件配置管理

### HistoryManager
历史记录管理器，提供：
- 操作历史追踪
- 撤销/重做功能
- 状态快照管理

### EventBus
事件总线，实现：
- 事件分发
- 事件订阅
- 异步事件处理

## 依赖项

主要依赖包括：
- tokio: 异步运行时
- metrics: 性能指标收集
- serde: 序列化/反序列化
- async-trait: 异步特征支持
- moduforge-core: 核心功能库

## 使用示例

### 同步运行时示例
```rust
use moduforge_runtime::{
    Runtime,
    RuntimeOptions,
    EditorResult,
};

fn main() -> EditorResult<()> {
    // 创建编辑器配置
    let options = EditorOptions::new()
        .with_extensions(vec![])
        .with_content("初始内容");

    // 创建编辑器实例
    let mut editor = Editor::create(options)?;

    // 初始化编辑器
    editor.init()?;

    // 使用编辑器...
    
    Ok(())
}
```

### 异步运行时示例
```rust
use moduforge_runtime::{
    Editor,
    EditorOptions,
    EditorResult,
};

#[tokio::main]
async fn main() -> EditorResult<()> {
    // 创建编辑器配置
    let options = EditorOptions::new()
        .with_extensions(vec![])
        .with_content("初始内容");

    // 创建异步编辑器实例
    let mut editor = Editor::create(options).await?;

    // 初始化编辑器
    editor.init().await?;

    // 使用异步编辑器...
    
    Ok(())
}
```

## 项目结构

```
src/
├── async_processor.rs    // 异步处理器实现
├── async_runtime.rs      // 异步运行时实现
├── error.rs             // 错误处理
├── event.rs             // 事件系统
├── extension.rs         // 扩展接口
├── extension_manager.rs // 扩展管理器
├── flow.rs             // 流程控制
├── history_manager.rs   // 历史记录管理
├── macros.rs           // 宏定义
├── node.rs             // 节点实现
├── runtime.rs          // 运行时核心
├── traits.rs           // 特征定义
└── types.rs            // 类型定义
```

## 开发指南

1. 克隆仓库
2. 安装 Rust 工具链
3. 运行测试：`cargo test`
4. 构建项目：`cargo build`

## 贡献指南

欢迎提交 Pull Request 和 Issue。在提交代码前，请确保：
1. 代码符合 Rust 代码规范
2. 添加了必要的测试
3. 更新了相关文档

## 许可证

本项目采用 MIT 许可证 