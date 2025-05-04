//! ModuForge-RS 宏定义模块
//! 
//! 该模块提供了用于简化代码编写的自定义宏，包括：
//! - 命令宏
//! - 扩展宏
//! - 标记宏
//! - 节点宏
//! - 插件宏
//! 
//! 主要组件：
//! - `command`: 命令宏，用于定义命令
//! - `extension`: 扩展宏，用于定义扩展
//! - `mark`: 标记宏，用于定义标记
//! - `node`: 节点宏，用于定义节点
//! - `plugin`: 插件宏，用于定义插件
//! 
//! 这些宏可以帮助开发者更简洁地定义各种组件，减少样板代码。

pub mod command;
pub mod extension;
pub mod mark;
pub mod node;
pub mod plugin;
