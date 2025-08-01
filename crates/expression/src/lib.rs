//! A lightweight expression language designed for the evaluation of expressions in various contexts.
//!
//! Zen Expression is a versatile single-threaded expression language designed for simplicity
//! high-performance. It's primarily used for evaluating and processing JSON data offers key components that empower developers in creating responsive and
//! non-blocking I/O applications
//! Out of the box, it comes with amazing benefits:
//! - 🚀 Blazingly fast - Perform millions of evaluations per second
//! - 🧠 Intuitive syntax - Minimalistic and expressive syntax
//! - 💼 Portable - Can be compiled for all standard architectures including WASM
//!
//! For a full list of language references, visit [documentation](https://gorules.io/docs/rules-engine/expression-language/).
//!
//! # Example
//! Evaluate expression using isolate:
//! ```
//! use zen_expression::evaluate_expression;
//! use zen_expression::variable::Variable;
//! use rust_decimal_macros::dec;
//! use serde_json::json;
//!
//! fn main() {
//!     let context = json!({ "tax": { "percentage": 10 } });
//!     let tax_amount = evaluate_expression("50 * tax.percentage / 100", context.into()).unwrap();
//!
//!     assert_eq!(tax_amount, Variable::Number(dec!(5)));
//! }
//! ```
//!
//! ## High Performance
//! When evaluating a lot of expressions at once, you can use Isolate directly. Under the hood, Isolate
//! will re-use allocated memory from previous evaluations, drastically improving performance.
//!
//! ```
//! use zen_expression::Isolate;
//! use zen_expression::variable::Variable;
//! use rust_decimal_macros::dec;
//! use serde_json::json;
//!
//! fn main() {
//!     let context = json!({ "tax": { "percentage": 10 } });
//!     let mut isolate = Isolate::with_environment(context.into());
//!
//!     // Fast 🚀
//!     for _ in 0..1_000 {
//!         let tax_amount = isolate.run_standard("50 * tax.percentage / 100").unwrap();
//!         assert_eq!(tax_amount, Variable::Number(dec!(5)));
//!     }
//! }
//! ```
//!
//! # Feature flags
//!
//! Name | Description | Default?
//! ---|---|---
//! `regex-deprecated` | Uses standard `regex` crate | Yes
//! `regex-lite` | Opts for usage of lightweight `regex-lite` crate. Useful for reducing build size, especially in WASM. | No

mod isolate;

mod arena;
pub mod compiler;
mod exports;
pub mod expression;
pub mod functions;
pub mod intellisense;
pub mod lexer;
pub mod parser;
pub mod validate;
pub mod variable;
pub mod vm;

pub use exports::{
    compile_expression, compile_unary_expression, evaluate_expression,
    evaluate_unary_expression,
};
pub use expression::{Expression, ExpressionKind};
pub use isolate::{Isolate, IsolateError};
pub use variable::Variable;

// 导出自定义函数相关
pub use functions::mf_function::{MfFunction, MfFunctionRegistry};
pub use functions::defs::FunctionSignature;
pub use functions::{StateGuard, with_state_async};
