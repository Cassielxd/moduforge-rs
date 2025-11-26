//! ModuForge-RS 数据转换模块
//!
//! 该模块负责处理文档的转换操作，包括：
//! - 节点操作（添加、移动、删除、替换）
//! - 标记操作
//! - 属性更新
//! - 批量操作
//! - 补丁应用
//! - 增量更新和内存优化
//!
//! 主要组件：
//! - `attr_step`: 属性步骤，处理属性更新操作
//! - `draft`: 草稿系统，管理文档的临时状态
//! - `mark_step`: 标记步骤，处理标记的添加和删除
//! - `node_step`: 节点步骤，处理节点的各种操作
//! - `patch`: 补丁系统，用于增量更新
//! - `step`: 步骤定义，定义转换操作的基本接口
//! - `transform`: 转换系统，协调各种转换操作
//!
//! # 泛型框架
//!
//! Transform 层现在支持泛型容器和 Schema：
//! - `TransformGeneric<C, S>`: 泛型 Transform，支持任意 DataContainer 和 SchemaDefinition
//! - `Transform`: NodePool + Schema 的具体实现
//! - `StepGeneric<C, S>`: 泛型 Step trait
//! - `Step`: NodePool + Tree 的具体 Step trait
//!
//! 使用者可以实现自己的 StepGeneric 来支持不同的存储类型。

pub mod attr_step;
pub mod batch_step;
pub mod mark_step;
pub mod node_step;
pub mod step;
pub mod transform;
use anyhow::Result;

pub type TransformResult<T> = Result<T>;

pub fn transform_error(msg: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("事务应用失败: {}", msg.into())
}

// 导出泛型类型
pub use step::{StepGeneric, StepResult};
pub use transform::{TransformGeneric, Transform};

// 导出具体 NodePool Step 实现
pub use node_step::{
    AddNodeStep, RemoveNodeStep, MoveNodeStep,
};
