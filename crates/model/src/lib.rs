//! ModuForge-RS 数据模型模块
//!
//! 该模块定义了框架中使用的核心数据模型，包括：
//! - 节点系统
//! - 标记系统
//! - 属性系统
//! - 模式定义
//! - 内容匹配
//! - ID 生成
//! - 错误处理
//!
//! 主要组件：
//! - `node`: 节点定义，表示文档中的基本元素
//! - `mark`: 标记定义，用于文档的格式化
//! - `attrs`: 属性定义，存储节点和标记的属性
//! - `mark_type`: 标记类型定义，定义不同类型的标记
//! - `node_type`: 节点类型定义，定义不同类型的节点
//! - `schema`: 模式定义，定义文档结构规则
//! - `content`: 内容匹配定义，处理内容验证和匹配
//! - `error`: 错误类型和处理
//! - `id_generator`: ID 生成器，生成唯一标识符
//! - `node_pool`: 节点池，管理节点实例
//! - `types`: 通用类型定义

//节点定义
pub mod node;
//标记定义
pub mod mark;
//属性定义
pub mod attrs;
//标记类型定义
pub mod mark_type;
//节点类型定义
pub mod node_type;
//模式定义
pub mod schema;
//内容匹配定义
pub mod content;
//id生成器定义
pub mod error;
pub mod id_generator;
pub mod node_pool;
pub mod ops;
pub mod tree;
pub mod types;

pub mod imbl {
    pub use imbl::*;
}

pub use node::Node;
pub use mark::Mark;
pub use attrs::Attrs;
pub use error::*;
pub use id_generator::IdGenerator;
pub use node_pool::NodePool;
pub use ops::*;
pub use tree::Tree;
pub use types::*;
pub use mark_type::MarkType;
pub use node_type::NodeType;
pub use schema::Schema;
