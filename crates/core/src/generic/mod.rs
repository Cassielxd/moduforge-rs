//! 泛型运行时系统
//!
//! 此模块包含所有泛型化的核心类型，支持任意 DataContainer 和 SchemaDefinition 组合。
//!
//! # 模块组织
//!
//! - `event` - EventGeneric<C, S> 和事件总线
//! - `middleware` - MiddlewareGeneric<C, S> 中间件系统
//! - `messages` - Actor 消息系统的泛型类型
//! - `runtime` - RuntimeTraitGeneric<C, S> 统一运行时接口
//!
//! # 设计原则
//!
//! 1. **完全泛型**: 所有类型都支持任意容器和模式组合
//! 2. **类型安全**: 编译期保证容器和模式的匹配
//! 3. **向后兼容**: 通过类型别名保持与现有代码的兼容性
//!
//! # 使用示例
//!
//! ```rust
//! use mf_core::generic::runtime::RuntimeTraitGeneric;
//! use mf_model::{node_pool::NodePool, schema::Schema};
//!
//! // 使用默认类型
//! async fn use_default_runtime(
//!     runtime: &mut dyn RuntimeTraitGeneric<NodePool, Schema>
//! ) {
//!     // ...
//! }
//!
//! // 使用自定义类型
//! async fn use_custom_runtime<C, S>(
//!     runtime: &mut dyn RuntimeTraitGeneric<C, S>
//! )
//! where
//!     C: DataContainer + 'static,
//!     S: SchemaDefinition<Container = C> + 'static,
//! {
//!     // ...
//! }
//! ```

pub mod event;
pub mod extension;
pub mod extension_manager;
pub mod flow_engine;
pub mod middleware;
pub mod messages;
pub mod runtime;
pub mod types;

// Re-export commonly used types
pub use event::EventGeneric;
pub use extension::{ExtensionGeneric, OpFnGeneric, OpFnItemGeneric, NodeTransformFnGeneric};
pub use extension_manager::ExtensionManagerGeneric;
pub use flow_engine::{AsyncFlowEngineGeneric, SyncFlowEngineGeneric, TransactionProcessorGeneric};
pub use middleware::{MiddlewareGeneric, MiddlewareStackGeneric};
pub use messages::{EventBusMessageGeneric, StateMessageGeneric, StateSnapshotGeneric, TransactionMessageGeneric};
pub use runtime::RuntimeTraitGeneric;
pub use types::{HistoryEntryWithMetaGeneric, ProcessorResultGeneric, TaskParamsGeneric, TransactionStatus};
