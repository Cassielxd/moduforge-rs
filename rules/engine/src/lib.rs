//! # moduforge-rules-engine
//!
//! moduforge-rules-engine 是一个对业务友好的开源业务规则引擎（BRE），用于根据 GoRules JSON 决策模型（JDM）标准执行决策模型。
//!
//! # 使用方法
//!
//! 要使用 Noop（默认）加载器执行简单决策，您可以使用以下代码：
//!
//! ```rust
//! use serde_json::json;
//! use moduforge_rules_engine::DecisionEngine;
//! use moduforge_rules_engine::model::DecisionContent;
//!
//! async fn evaluate() {
//!     let decision_content: DecisionContent = serde_json::from_str(include_str!("jdm_graph.json")).unwrap();
//!     let engine = DecisionEngine::default();
//!     let decision = engine.create_decision(decision_content.into());
//!
//!     let result = decision.evaluate(&json!({ "input": 12 })).await;
//! }
//! ```
//!
//! 另外，您也可以使用 `Decision::from` 函数间接创建决策，而无需构建引擎。
//!
//! # 加载器
//!
//! 对于更高级的用例，当您需要加载多个决策并使用图时，您可以使用以下预制的加载器之一：
//! - FilesystemLoader - 使用给定路径作为根目录，尝试基于相对路径加载决策
//! - MemoryLoader - 作为 HashMap（键值存储）工作
//! - ClosureLoader - 允许定义简单的异步回调函数，该函数接收键作为参数并返回 `Arc<DecisionContent>` 实例
//! - NoopLoader - （默认）无法加载决策，允许使用 create_decision（主要用于跨语言统一 API）
//!
//! ## 文件系统加载器
//!
//! 假设您有一个位于 /app/decisions 下的决策模型文件夹（.json 文件），您可以按以下方式使用 FilesystemLoader：
//!
//! ```rust
//! use serde_json::json;
//! use moduforge_rules_engine::DecisionEngine;
//! use moduforge_rules_engine::loader::{FilesystemLoader, FilesystemLoaderOptions};
//!
//! async fn evaluate() {
//!     let engine = DecisionEngine::new(FilesystemLoader::new(FilesystemLoaderOptions {
//!         keep_in_memory: true, // 可选，保持在内存中以提高性能
//!         root: "/app/decisions"
//!     }));
//!     
//!     let context = json!({ "customer": { "joinedAt": "2022-01-01" } });
//!     // 如果您计划多次使用它，可以缓存 JDM 以获得轻微的性能提升
//!     // 在绑定（其他语言）的情况下，这种提升会更大
//!     {
//!         let promotion_decision = engine.get_decision("commercial/promotion.json").await.unwrap();
//!         let result = promotion_decision.evaluate(&context).await.unwrap();
//!     }
//!     
//!     // 或者按需加载
//!     {
//!         let result = engine.evaluate("commercial/promotion.json", &context).await.unwrap();
//!     }
//! }
//!
//!
//! ```
//!
//! ## 自定义加载器
//! 您可以通过实现 `DecisionLoader` trait 为 zen 引擎创建自定义加载器。
//! 以下是 MemoryLoader 的实现示例：
//! ```rust
//! use std::collections::HashMap;
//! use std::sync::{Arc, RwLock};
//! use zen_engine::loader::{DecisionLoader, LoaderError, LoaderResponse};
//! use zen_engine::model::DecisionContent;
//!
//! #[derive(Debug, Default)]
//! pub struct MemoryLoader {
//!     memory_refs: RwLock<HashMap<String, Arc<DecisionContent>>>,
//! }
//!
//! impl MemoryLoader {
//!     pub fn add<K, D>(&self, key: K, content: D)
//!         where
//!             K: Into<String>,
//!             D: Into<DecisionContent>,
//!     {
//!         let mut mref = self.memory_refs.write().unwrap();
//!         mref.insert(key.into(), Arc::new(content.into()));
//!     }
//!
//!     pub fn get<K>(&self, key: K) -> Option<Arc<DecisionContent>>
//!         where
//!             K: AsRef<str>,
//!     {
//!         let mref = self.memory_refs.read().unwrap();
//!         mref.get(key.as_ref()).map(|r| r.clone())
//!     }
//!
//!     pub fn remove<K>(&self, key: K) -> bool
//!         where
//!             K: AsRef<str>,
//!     {
//!         let mut mref = self.memory_refs.write().unwrap();
//!         mref.remove(key.as_ref()).is_some()
//!     }
//! }
//!
//! impl DecisionLoader for MemoryLoader {
//! fn load<'a>(&'a self, key: &'a str) -> impl Future<Output = LoaderResponse> + 'a {
//!     async move {
//!         self.get(&key)
//!             .ok_or_else(|| LoaderError::NotFound(key.to_string()).into())
//!     }
//! }
//! ```

#![deny(clippy::unwrap_used)]
#![allow(clippy::module_inception)]

pub mod config;
pub mod decision;
pub mod engine;
pub mod error;
pub mod handler;
pub mod loader;
#[path = "model/mod.rs"]
pub mod model;
pub mod util;

pub use config::ZEN_CONFIG;
pub use decision::Decision;
pub use engine::{DecisionEngine, EvaluationOptions};
pub use error::EvaluationError;
pub use handler::graph::DecisionGraphResponse;
pub use handler::graph::DecisionGraphTrace;
pub use handler::graph::DecisionGraphValidationError;
pub use handler::node::NodeError;
pub use moduforge_rules_expression::Variable;
