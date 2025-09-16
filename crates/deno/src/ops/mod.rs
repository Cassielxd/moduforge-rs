//! ModuForge Deno Op 函数
//!
//! 提供 JavaScript 访问 ModuForge 核心功能的 Op 函数
//! 避免序列化，直接操作 Rust 数据结构

pub mod state_ops;
pub mod transaction_ops;
pub mod node_ops;

pub use state_ops::*;
pub use transaction_ops::*;
pub use node_ops::*;

use deno_core::Extension;

/// 创建 ModuForge Deno 扩展
pub fn create_moduforge_extension() -> Extension {
    Extension::builder("moduforge")
        .ops(vec![
            // 状态相关 Ops
            state_ops::op_state_get_version::DECL,
            state_ops::op_state_get_field::DECL,
            state_ops::op_state_has_field::DECL,
            state_ops::op_state_get_doc::DECL,
            state_ops::op_state_get_schema::DECL,

            // 事务相关 Ops
            transaction_ops::op_transaction_new::DECL,
            transaction_ops::op_transaction_set_node_attribute::DECL,
            transaction_ops::op_transaction_add_node::DECL,
            transaction_ops::op_transaction_remove_node::DECL,
            transaction_ops::op_transaction_add_mark::DECL,
            transaction_ops::op_transaction_remove_mark::DECL,
            transaction_ops::op_transaction_set_meta::DECL,
            transaction_ops::op_transaction_get_meta::DECL,

            // 节点相关 Ops
            node_ops::op_node_get_attribute::DECL,
            node_ops::op_node_get_children::DECL,
            node_ops::op_node_get_parent::DECL,
            node_ops::op_node_find_by_id::DECL,
        ])
        .build()
}