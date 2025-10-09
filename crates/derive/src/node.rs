//! Node 派生宏实现模块
//!
//! 提供 #[derive(Node)] 派生宏的完整实现，包括属性解析、验证和代码生成。
//! 严格遵循单一职责原则，专门处理 Node 相关的派生宏逻辑。

pub mod derive_impl;
