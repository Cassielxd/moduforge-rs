//! ModuForge-RS 宏扩展模块
//!
//! 提供 #[derive(Node)] 和 #[derive(Mark)] 派生宏，
//! 支持声明式的节点和标记定义，简化 ModuForge-RS 框架的使用。
//!
//! # 功能特性
//! 
//! - **Node 派生宏**: 自动生成 `to_node()` 方法，将结构体转换为 mf_core::node::Node
//! - **Mark 派生宏**: 自动生成 `to_mark()` 方法，将结构体转换为 mf_core::mark::Mark
//! - **编译时验证**: 在编译期检查属性配置和类型兼容性
//! - **友好错误消息**: 提供详细的编译错误信息和修复建议
//!
//! # 设计原则
//!
//! 此模块严格遵循 SOLID 设计原则：
//! - **单一职责原则 (SRP)**: 每个模块只负责一个明确的功能
//! - **接口隔离原则 (ISP)**: 提供精简、专用的接口
//! - **开闭原则 (OCP)**: 通过插件系统支持扩展
//! - **里氏替换原则 (LSP)**: 确保实现类型的可替换性

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

// 内部模块声明 - proc-macro crate 不能导出这些模块，只能内部使用
mod common;
mod parser;
mod converter;
mod generator;
mod node;
mod mark;

/// 插件状态派生宏
/// 
/// 为结构体实现 Resource trait，用于依赖注入系统。
/// 这是现有功能，保持不变以确保向后兼容性。
#[proc_macro_derive(PState)]
pub fn derive_plugin_state(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl Resource for #name {}
    };

    TokenStream::from(expanded)
}

/// Node 派生宏
///
/// 为结构体生成 `to_node()` 方法，将结构体实例转换为 `mf_core::node::Node`。
/// 
/// # 支持的属性
/// 
/// - `#[node_type = "类型名"]` - 必需，指定节点类型标识符
/// - `#[marks = "mark1 mark2"]` - 可选，指定支持的标记类型列表
/// - `#[content = "内容表达式"]` - 可选，指定内容约束表达式
/// - `#[attr]` - 字段级属性，标记字段作为节点属性
/// 
/// # 示例
/// 
/// ```rust
/// use mf_derive::Node;
/// 
/// #[derive(Node)]
/// #[node_type = "project"]
/// #[marks = "color bold"]
/// #[content = "text*"]
/// pub struct ProjectNode {
///     #[attr]
///     name: String,
///     
///     #[attr] 
///     description: Option<String>,
/// }
/// 
/// // 使用生成的方法
/// let project = ProjectNode {
///     name: "示例项目".to_string(),
///     description: Some("这是一个示例项目".to_string()),
/// };
/// let node = project.to_node();
/// ```
/// 
/// # 设计原则体现
/// 
/// - **单一职责**: 只负责 Node 相关的派生宏功能
/// - **开闭原则**: 通过属性配置支持扩展，无需修改代码
/// - **接口隔离**: 提供专门的 Node 转换接口
#[proc_macro_derive(Node, attributes(node_type, marks, content, attr, id))]
pub fn derive_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // 使用带错误恢复的处理函数，确保友好的编译错误消息
    let result = node::derive_impl::process_derive_node_with_recovery(input);
    TokenStream::from(result)
}

/// Mark 派生宏
/// 
/// 为结构体生成 `to_mark()` 方法，将结构体实例转换为 `mf_core::mark::Mark`。
/// 
/// # 支持的属性
/// 
/// - `#[mark_type = "类型名"]` - 必需，指定标记类型标识符
/// - `#[attr]` - 字段级属性，标记字段作为标记属性
/// 
/// # 示例
/// 
/// ```rust
/// use mf_derive::Mark;
/// 
/// #[derive(Mark)]
/// #[mark_type = "emphasis"]
/// pub struct EmphasisMark {
///     #[attr]
///     level: i32,
///     
///     #[attr]
///     color: Option<String>,
/// }
/// 
/// // 使用生成的方法
/// let emphasis = EmphasisMark {
///     level: 2,
///     color: Some("red".to_string()),
/// };
/// let mark = emphasis.to_mark();
/// ```
/// 
/// # 设计原则体现
/// 
/// - **单一职责**: 只负责 Mark 相关的派生宏功能
/// - **里氏替换原则**: 生成的 Mark 实例可完全替换手动创建的实例
/// - **接口隔离**: 提供专门的 Mark 转换接口
#[proc_macro_derive(Mark, attributes(mark_type, attr))]
pub fn derive_mark(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // 使用带错误恢复的处理函数，确保友好的编译错误消息
    let result = mark::derive_impl::process_derive_mark_with_recovery(input);
    TokenStream::from(result)
}
