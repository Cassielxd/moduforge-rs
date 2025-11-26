//! 通用抽象层
//!
//! 定义数据容器、数据单元和 Schema 的通用接口。
//!
//! # 模块说明
//!
//! * `item` - 定义 `DataItem` trait，所有数据单元的基础接口
//! * `container` - 定义 `DataContainer` trait，数据容器的通用接口
//! * `schema` - 定义 `SchemaDefinition` trait，Schema 的通用接口
//!
//! # 默认实现
//!
//! * `Node` 实现了 `DataItem`
//! * `NodePool` 实现了 `DataContainer`
//! * `Schema` 实现了 `SchemaDefinition`
//!
//! # 示例
//!
//! ```ignore
//! use mf_model::traits::{DataItem, DataContainer, SchemaDefinition};
//! use mf_model::{Node, NodePool, Schema};
//!
//! // 使用默认实现
//! let node: Node = /* ... */;
//! let pool: NodePool = /* ... */;
//! let schema: Schema = /* ... */;
//!
//! // 通过 trait 方法访问
//! println!("Node type: {}", node.type_name());
//! println!("Pool size: {}", pool.size());
//! println!("Schema name: {}", schema.name());
//! ```

pub mod container;
pub mod item;
pub mod schema;

// 重新导出，方便使用
pub use item::DataItem;
pub use container::DataContainer;
pub use schema::SchemaDefinition;
