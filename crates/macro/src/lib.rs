//! ModuForge-RS 声明式宏
//!
//! 该模块提供了 ModuForge 项目的声明式宏，包括：
//! - `impl_extension!`：创建 Extension 实例（legacy）
//! - `mf_extension!`：声明式扩展定义宏（类似 Deno 的 extension! 宏）
//! - `mf_extension_with_config!`：带配置支持的扩展宏
//! - `mf_ops!`：声明操作函数宏
//! - `mf_op!`：创建操作函数
//! - `mf_global_attr!`：创建全局属性项
//! - `impl_plugin!`：快速实现 Plugin trait（legacy）
//! - `mf_plugin!`：声明式插件定义宏
//! - `mf_plugin_with_config!`：带配置支持的插件宏
//! - `mf_plugin_metadata!`：创建插件元数据
//! - `mf_plugin_config!`：创建插件配置
//! - `impl_state_field!`：快速实现 StateField trait
//! - `derive_plugin_state!`：为类型实现 Resource trait
//! - `mark!`：创建 Mark 实例
//! - `node!`：创建 Node 实例
//!
//! > 注意：异步命令属性宏 `#[impl_command]` 现由 `moduforge-macros-derive`
//! > 提供，请直接从该 crate 导入。
//!
//! ## 使用方法
//!
//! ```toml
//! [dependencies]
//! mf-macro = { path = "../macro" }
//! mf-derive = { path = "../derive" } # 如果需要使用 #[impl_command]
//! ```
//!
//! ```rust
//! use mf_macro::{
//!     mark, node, impl_plugin,
//!     mf_extension, mf_extension_with_config, mf_ops, mf_op, mf_global_attr,
//!     mf_plugin, mf_plugin_with_config, mf_plugin_metadata, mf_plugin_config,
//!     impl_state_field, derive_plugin_state
//! };
//! use mf_derive::impl_command; // #[impl_command] 请从 `mf_derive` 导入
//! ```

pub mod extension;
pub mod mark;
pub mod node;
pub mod plugin;
