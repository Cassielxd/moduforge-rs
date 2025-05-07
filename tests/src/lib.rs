//! ModuForge-RS 测试模块
//!
//! 该模块包含了框架的测试用例，包括：
//! - 基础测试
//! - 命令测试
//! - 内容测试
//! - 扩展测试
//! - 中间件测试
//! - 插件测试
//! - 快照测试
//! - ZIP 测试
//!
//! 主要组件：
//! - `base`: 基础测试工具和辅助函数
//! - `commands`: 命令系统测试
//! - `content_test`: 内容处理测试
//! - `ext`: 扩展系统测试
//! - `middleware`: 中间件测试
//! - `plugins`: 插件系统测试
//! - `snapshot_test`: 快照测试
//! - `zip_test`: ZIP 处理测试
//!
//! 这些测试用例用于确保框架的各个组件正常工作，并保持向后兼容性。

pub mod base;
pub mod commands;
pub mod content_test;
pub mod ext;
pub mod middleware;
pub mod plugins;
pub mod snapshot_test;
pub mod zip_test;
