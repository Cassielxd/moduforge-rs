# ModuForge-RS 数据转换包

[![Crates.io](https://img.shields.io/crates/v/moduforge-transform)](https://crates.io/crates/moduforge-transform)
[![Documentation](https://docs.rs/moduforge-transform/badge.svg)](https://docs.rs/moduforge-transform)
[![License](https://img.shields.io/crates/l/moduforge-transform)](LICENSE)

ModuForge-RS 数据转换包提供了基于不可变数据结构的文档转换系统，支持节点操作、标记管理、属性更新和批量处理。该包是 ModuForge-RS 框架的核心组件，为文档编辑和状态管理提供高效、可靠的转换能力。

## 🏗️ 架构概述

ModuForge-RS 数据转换包采用基于步骤的转换架构，确保文档变更的可预测性和可追溯性。系统基于以下核心设计原则：

- **步骤驱动**: 所有转换操作通过步骤（Step）进行，支持序列化和反序列化
- **延迟计算**: 使用延迟计算优化性能，只在需要时重新计算文档状态
- **Copy-on-Write**: 采用写时复制策略，减少不必要的内存分配
- **事务支持**: 完整的提交和回滚机制，支持历史记录管理
- **批量操作**: 高效的批量步骤应用，减少中间状态创建

### 核心架构组件

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Transform     │    │   Step          │    │   Patch         │
│   (转换系统)     │◄──►│   (步骤接口)     │◄──►│   (补丁系统)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   NodeStep      │    │   AttrStep      │    │   MarkStep      │
│   (节点操作)     │    │   (属性操作)     │    │   (标记操作)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🚀 核心功能

### 1. 转换系统 (Transform)
- **延迟计算**: 使用 `LazyDoc` 枚举实现智能的文档状态计算
- **草稿系统**: 基于 `Tree` 的草稿状态管理，支持临时修改
- **历史管理**: 完整的步骤历史和反向步骤记录
- **批量操作**: 高效的批量步骤应用，减少中间状态创建
- **提交回滚**: 支持事务提交和回滚操作

### 2. 步骤系统 (Step)
- **统一接口**: 所有转换操作都实现 `Step` 特征
- **序列化支持**: 支持步骤的序列化和反序列化
- **反向操作**: 自动生成反向步骤，支持撤销操作
- **错误处理**: 完善的错误处理和结果反馈机制

### 3. 节点操作 (NodeStep)
- **添加节点**: `AddNodeStep` 支持在指定父节点下添加新节点
- **删除节点**: `RemoveNodeStep` 支持删除指定节点及其子树
- **移动节点**: `MoveNodeStep` 支持节点在不同父节点间移动
- **递归处理**: 自动处理节点的递归结构和子节点关系

### 4. 属性操作 (AttrStep)
- **属性更新**: 支持批量更新节点属性
- **模式验证**: 基于 Schema 的属性验证和过滤
- **类型安全**: 使用 `serde_json::Value` 确保类型安全
- **增量更新**: 支持属性的增量更新操作

### 5. 标记操作 (MarkStep)
- **添加标记**: `AddMarkStep` 支持为节点添加标记
- **删除标记**: `RemoveMarkStep` 支持删除指定类型的标记
- **标记验证**: 基于 Schema 的标记类型验证
- **批量操作**: 支持批量标记操作

### 6. 补丁系统 (Patch)
- **增量更新**: 支持文档的增量更新操作
- **路径定位**: 使用路径数组精确定位节点位置
- **操作类型**: 支持属性更新、节点操作、标记操作等多种类型
- **序列化**: 完整的补丁序列化和反序列化支持

## 📦 技术栈

### 核心依赖
```toml
[dependencies]
# 不可变数据结构
im = { version = "15.1", features = ["serde"] }

# 序列化
serde = { version = "1.0", features = ["derive", "rc"] }
serde_json = "1.0"

# 异步运行时
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"

# 错误处理
anyhow = "1"
thiserror = "2.0.12"

# 日志系统
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"

# 时间处理
time = "0.3"
```

### ModuForge-RS 内部依赖
```toml
# 数据模型
moduforge-model = "0.4.10"
```

## 🚀 快速开始

### 基本使用

```rust
use mf_transform::{
    Transform, TransformResult,
    node_step::{AddNodeStep, RemoveNodeStep},
    attr_step::AttrStep,
    mark_step::{AddMarkStep, RemoveMarkStep},
    step::Step
};
use mf_model::{node_type::NodeEnum, schema::Schema, node_pool::NodePool, mark::Mark};
use std::sync::Arc;
use im::HashMap as ImHashMap;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建文档和 Schema
    let schema = Arc::new(Schema::default());
    let doc = Arc::new(NodePool::default());
    
    // 创建转换器
    let mut transform = Transform::new(doc, schema);
    
    // 添加节点
    let node_enum = NodeEnum::new("test_node", "paragraph");
    let add_step = Arc::new(AddNodeStep::new(
        "parent_id".to_string(),
        vec![node_enum]
    ));
    transform.step(add_step)?;
    
    // 更新属性
    let mut attrs = ImHashMap::new();
    attrs.insert("class".to_string(), json!("highlight"));
    let attr_step = Arc::new(AttrStep::new(
        "test_node".to_string(),
        attrs
    ));
    transform.step(attr_step)?;
    
    // 添加标记
    let mark = Mark::new("bold".to_string(), ImHashMap::new());
    let mark_step = Arc::new(AddMarkStep::new(
        "test_node".to_string(),
        vec![mark]
    ));
    transform.step(mark_step)?;
    
    // 提交更改
    transform.commit();
    
    println!("转换完成，文档已更新");
    Ok(())
}
```

### 批量操作

```rust
use mf_transform::{Transform, TransformResult};
use mf_model::{node_type::NodeEnum, schema::Schema, node_pool::NodePool};
use std::sync::Arc;

async fn batch_operations() -> TransformResult<()> {
    let schema = Arc::new(Schema::default());
    let doc = Arc::new(NodePool::default());
    let mut transform = Transform::new(doc, schema);
    
    // 准备批量步骤
    let mut steps = Vec::new();
    
    // 添加多个节点
    for i in 0..5 {
        let node_enum = NodeEnum::new(&format!("node_{}", i), "paragraph");
        let step = Arc::new(AddNodeStep::new(
            "parent_id".to_string(),
            vec![node_enum]
        ));
        steps.push(step);
    }
    
    // 批量应用步骤
    transform.apply_steps_batch(steps)?;
    
    // 提交更改
    transform.commit();
    
    println!("批量操作完成，添加了 {} 个节点", transform.history_size());
    Ok(())
}
```

### 事务管理

```rust
use mf_transform::Transform;
use mf_model::{node_type::NodeEnum, schema::Schema, node_pool::NodePool};
use std::sync::Arc;

async fn transaction_management() -> anyhow::Result<()> {
    let schema = Arc::new(Schema::default());
    let doc = Arc::new(NodePool::default());
    let mut transform = Transform::new(doc, schema);
    
    // 执行一些操作
    let node_enum = NodeEnum::new("test_node", "paragraph");
    let step = Arc::new(AddNodeStep::new(
        "parent_id".to_string(),
        vec![node_enum]
    ));
    transform.step(step)?;
    
    // 检查是否有未提交的更改
    if transform.doc_changed() {
        println!("有未提交的更改，历史大小: {}", transform.history_size());
        
        // 可以选择提交或回滚
        // transform.commit();  // 提交更改
        // transform.rollback(); // 回滚更改
    }
    
    Ok(())
}
```

### 自定义步骤

```rust
use mf_transform::{step::{Step, StepResult}, TransformResult};
use mf_model::{schema::Schema, tree::Tree};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CustomStep {
    node_id: String,
    operation: String,
}

impl CustomStep {
    pub fn new(node_id: String, operation: String) -> Self {
        Self { node_id, operation }
    }
}

impl Step for CustomStep {
    fn name(&self) -> String {
        "custom_step".to_string()
    }
    
    fn apply(
        &self,
        tree: &mut Tree,
        _schema: Arc<Schema>,
    ) -> TransformResult<StepResult> {
        // 执行自定义操作
        match self.operation.as_str() {
            "highlight" => {
                // 高亮节点逻辑
                println!("高亮节点: {}", self.node_id);
                Ok(StepResult::ok())
            }
            "hide" => {
                // 隐藏节点逻辑
                println!("隐藏节点: {}", self.node_id);
                Ok(StepResult::ok())
            }
            _ => Ok(StepResult::fail("未知操作".to_string())),
        }
    }
    
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }
    
    fn invert(&self, _tree: &Arc<Tree>) -> Option<Arc<dyn Step>> {
        // 生成反向操作
        let reverse_operation = match self.operation.as_str() {
            "highlight" => "unhighlight",
            "hide" => "show",
            _ => return None,
        };
        
        Some(Arc::new(CustomStep::new(
            self.node_id.clone(),
            reverse_operation.to_string(),
        )))
    }
}
```

## 🔧 配置选项

### 转换器配置

```rust
use mf_transform::Transform;
use mf_model::{schema::Schema, node_pool::NodePool};
use std::sync::Arc;

// 创建转换器
let schema = Arc::new(Schema::default());
let doc = Arc::new(NodePool::default());
let mut transform = Transform::new(doc, schema);

// 配置选项
transform.set_auto_commit(false);  // 禁用自动提交
transform.set_batch_size(100);     // 设置批量大小
```

### 步骤配置

```rust
use mf_transform::node_step::AddNodeStep;
use mf_model::node_type::NodeEnum;

// 创建节点步骤
let step = AddNodeStep::new(
    "parent_id".to_string(),
    vec![NodeEnum::new("child_node", "paragraph")]
);

// 步骤配置
step.set_validate(true);      // 启用验证
step.set_optimize(true);      // 启用优化
```

## 📊 性能特性

### 延迟计算优化
- **智能计算**: 只在需要时重新计算文档状态
- **状态缓存**: 缓存已计算的状态，避免重复计算
- **增量更新**: 支持增量更新，减少计算开销

### 内存管理
- **Copy-on-Write**: 采用写时复制策略，减少内存分配
- **结构共享**: 利用不可变数据结构的结构共享特性
- **批量操作**: 批量处理减少中间状态创建

### 并发性能
- **无锁设计**: 使用不可变数据结构避免锁竞争
- **原子操作**: 基于原子操作的状态管理
- **并发安全**: 线程安全的转换操作

## 🛠️ 错误处理

ModuForge-RS 数据转换包提供了完善的错误处理机制：

```rust
use mf_transform::{TransformResult, transform_error};

// 自定义错误处理
fn handle_transform_error(result: TransformResult<()>) -> anyhow::Result<()> {
    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            // 记录错误
            tracing::error!("转换操作失败: {}", e);
            
            // 根据错误类型进行不同处理
            if e.to_string().contains("node not found") {
                return Err(transform_error("节点不存在").into());
            }
            
            Err(e)
        }
    }
}
```

### 常见错误类型
- **节点错误**: 节点不存在或操作无效
- **属性错误**: 属性验证失败或类型不匹配
- **标记错误**: 标记操作失败或类型无效
- **序列化错误**: 步骤序列化或反序列化失败
- **验证错误**: Schema 验证失败

## 🧪 测试

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_node_step() {
        let schema = Arc::new(Schema::default());
        let doc = Arc::new(NodePool::default());
        let mut transform = Transform::new(doc, schema);
        
        let node_enum = NodeEnum::new("test_node", "paragraph");
        let step = Arc::new(AddNodeStep::new(
            "parent_id".to_string(),
            vec![node_enum]
        ));
        
        let result = transform.step(step);
        assert!(result.is_ok());
        assert!(transform.doc_changed());
    }
    
    #[test]
    fn test_attr_step() {
        let schema = Arc::new(Schema::default());
        let doc = Arc::new(NodePool::default());
        let mut transform = Transform::new(doc, schema);
        
        let mut attrs = ImHashMap::new();
        attrs.insert("class".to_string(), json!("test"));
        let step = Arc::new(AttrStep::new(
            "test_node".to_string(),
            attrs
        ));
        
        let result = transform.step(step);
        assert!(result.is_ok());
    }
}
```

### 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_complex_transformation() {
        let schema = Arc::new(Schema::default());
        let doc = Arc::new(NodePool::default());
        let mut transform = Transform::new(doc, schema);
        
        // 执行复杂的转换序列
        let steps = create_complex_steps();
        
        for step in steps {
            let result = transform.step(step);
            assert!(result.is_ok());
        }
        
        // 验证最终状态
        transform.commit();
        assert_eq!(transform.history_size(), 5);
    }
}
```

## 🔍 监控和调试

### 性能监控

```rust
use mf_transform::Transform;
use std::time::Instant;

async fn monitor_transform_performance(mut transform: Transform) {
    let start = Instant::now();
    
    // 执行转换操作
    let steps = create_test_steps();
    for step in steps {
        transform.step(step).unwrap();
    }
    
    let duration = start.elapsed();
    tracing::info!(
        "转换完成 - 步骤数: {}, 耗时: {:?}",
        transform.history_size(),
        duration
    );
}
```

### 状态调试

```rust
use mf_transform::Transform;

fn debug_transform(transform: &Transform) {
    tracing::debug!("转换器状态:");
    tracing::debug!("  历史大小: {}", transform.history_size());
    tracing::debug!("  文档已更改: {}", transform.doc_changed());
    tracing::debug!("  基础文档: {:?}", transform.base_doc);
}
```

## 📚 API 参考

### 核心类型

- **`Transform`**: 主转换器结构体
- **`Step`**: 步骤特征，所有转换操作的基础接口
- **`StepResult`**: 步骤执行结果
- **`Patch`**: 补丁枚举，描述文档修改操作

### 步骤类型

- **`AddNodeStep`**: 添加节点步骤
- **`RemoveNodeStep`**: 删除节点步骤
- **`MoveNodeStep`**: 移动节点步骤
- **`AttrStep`**: 属性更新步骤
- **`AddMarkStep`**: 添加标记步骤
- **`RemoveMarkStep`**: 删除标记步骤

### 主要方法

#### Transform
- `new(doc, schema)`: 创建新转换器
- `step(step)`: 应用单个步骤
- `apply_steps_batch(steps)`: 批量应用步骤
- `commit()`: 提交更改
- `rollback()`: 回滚更改
- `doc()`: 获取当前文档状态
- `doc_changed()`: 检查文档是否已更改
- `history_size()`: 获取历史大小

#### Step
- `name()`: 获取步骤名称
- `apply(tree, schema)`: 应用步骤
- `serialize()`: 序列化步骤
- `invert(tree)`: 生成反向步骤

## 🤝 贡献指南

我们欢迎社区贡献！请查看以下指南：

1. **代码风格**: 遵循 Rust 标准编码规范
2. **测试覆盖**: 为新功能添加相应的测试
3. **文档更新**: 更新相关文档和示例
4. **性能考虑**: 考虑性能影响和优化

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🔗 相关链接

- [ModuForge-RS 主页](https://github.com/moduforge/moduforge-rs)
- [API 文档](https://docs.rs/moduforge-transform)
- [示例项目](https://github.com/moduforge/moduforge-rs/tree/main/demo)
- [问题反馈](https://github.com/moduforge/moduforge-rs/issues) 