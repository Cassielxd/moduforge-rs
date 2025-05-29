//! ModuForge-RS 数据转换模块
//!
//! 该模块负责处理文档的转换操作，包括：
//! - 节点操作（添加、移动、删除、替换）
//! - 标记操作
//! - 属性更新
//! - 批量操作
//! - 补丁应用
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
//! 核心类型：
//! - `ConcreteStep`: 具体步骤枚举，表示所有可能的转换操作
//! - `PatchStep`: 补丁步骤，用于应用补丁
//! - `BatchStep`: 批量步骤，用于执行多个转换操作

pub mod attr_step;
pub mod mark_step;
pub mod node_step;
pub mod patch;
pub mod step;
pub mod transform;
use anyhow::Result;

pub type TransformResult<T> = Result<T>;

pub fn transform_error(msg: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("事务应用失败: {}", msg.into())
}
