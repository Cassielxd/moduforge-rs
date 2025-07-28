//! ModuForge-RS 声明式宏
//!
//! 该模块提供了ModuForge项目的声明式宏，包括：
//! - impl_command!: 快速实现Command trait
//! - impl_extension!: 创建Extension实例
//! - impl_plugin!: 快速实现Plugin trait
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
//! use mf_macro::{impl_command, mark, node, impl_plugin};
//! ```

pub mod command;
pub mod extension;
pub mod mark;
pub mod node;
pub mod plugin;

// 重新导出所有宏
pub use command::*;
pub use extension::*;
pub use mark::*;
pub use node::*;
pub use plugin::*;
