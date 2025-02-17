#![deny(clippy::unwrap_used)]
#![allow(clippy::module_inception)]

mod config;
mod decision;
mod engine;
mod error;
pub mod handler;
pub mod loader;
#[path = "model/mod.rs"]
pub mod model;
mod util;

pub use config::ZEN_CONFIG;
pub use decision::Decision;
pub use engine::{DecisionEngine, EvaluationOptions};
pub use error::EvaluationError;
pub use handler::graph::DecisionGraphResponse;
pub use handler::graph::DecisionGraphTrace;
pub use handler::graph::DecisionGraphValidationError;
pub use handler::node::NodeError;
pub use zen_expression::*;
