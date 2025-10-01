//! ModuForge-RS 声明式宏
//!
//! 该模块提供了ModuForge项目的声明式宏，包括：
//! - impl_command!: 快速实现Command trait
//! - impl_extension!: 创建Extension实例 (legacy)
//! - mf_extension!: 声明式扩展定义宏 (新版本，类似Deno的extension!宏)
//! - mf_extension_with_config!: 带配置支持的扩展宏
//! - mf_ops!: 声明操作函数块
//! - mf_op!: 创建操作函数
//! - mf_global_attr!: 创建全局属性项
//! - impl_plugin!: 快速实现Plugin trait (legacy)
//! - mf_plugin!: 声明式插件定义宏 (新版本，类似extension宏)
//! - mf_plugin_with_config!: 带配置支持的插件宏
//! - mf_plugin_metadata!: 创建插件元数据
//! - mf_plugin_config!: 创建插件配置
//! - impl_state_field!: 快速实现StateField trait
//! - derive_plugin_state!: 为类型实现Resource trait
//! - mark!: 创建Mark实例
//! - node!: 创建Node实例
//!
//! ## 注意
//!
//! 此crate现在是普通的库crate，不是proc-macro crate，
//! 所以可以正常导出声明式宏。
//!
//! ## 使用方法
//!
//! ```toml
//! [dependencies]
//! mf-macro = { path = "../macro" }
//! ```
//!
//! ```rust
//! use mf_macro::{
//!     impl_command, mark, node, impl_plugin,
//!     mf_extension, mf_extension_with_config, mf_ops, mf_op, mf_global_attr,
//!     mf_plugin, mf_plugin_with_config, mf_plugin_metadata, mf_plugin_config,
//!     impl_state_field, derive_plugin_state
//! };
//! ```

pub mod command;
pub mod extension;
pub mod mark;
pub mod node;
pub mod plugin;

// 重新导出所有宏
